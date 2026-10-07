//! Bounded selection transactions. No clipboard contents are persisted or logged.
use crate::config::wide;
use std::{
    mem::size_of,
    thread,
    time::{Duration, Instant},
};
use windows::{
    core::w,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{
            DataExchange::*,
            Memory::*,
            Ole::{OleDuplicateData, CLIPBOARD_FORMAT},
        },
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
};
const MAX_BYTES: usize = 16 * 1024 * 1024;
const UNICODE: u32 = 13;

struct Owner(HWND);
impl Owner {
    fn new() -> Result<Self, String> {
        // SAFETY: built-in STATIC class, owned message-only window on the current thread.
        unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("PIMXSWAP Clipboard"),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                None,
                None,
            )
            .map(Self)
            .map_err(|_| "clipboard_busy".into())
        }
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}
struct Open;
impl Open {
    fn acquire(owner: &Owner) -> Result<Self, String> {
        for _ in 0..12 {
            // SAFETY: valid message-only clipboard owner; balanced by Drop.
            if unsafe { OpenClipboard(Some(owner.0)) }.is_ok() {
                return Ok(Self);
            }
            thread::sleep(Duration::from_millis(10));
        }
        Err("clipboard_busy".into())
    }
}
impl Drop for Open {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}
struct NativeFormat {
    format: u32,
    handle: HANDLE,
}
impl NativeFormat {
    unsafe fn duplicate(format: u32, handle: HANDLE) -> Result<Self, String> {
        let copy = if format == 14 {
            HANDLE(CopyEnhMetaFileW(HENHMETAFILE(handle.0), None).0)
        } else {
            OleDuplicateData(handle, CLIPBOARD_FORMAT(format as u16), GMEM_MOVEABLE)
        };
        if copy.is_invalid() {
            Err("clipboard_write_failed".into())
        } else {
            Ok(Self {
                format,
                handle: copy,
            })
        }
    }
}
impl Drop for NativeFormat {
    fn drop(&mut self) {
        unsafe {
            if self.handle.is_invalid() {
                return;
            }
            match self.format {
                2 | 9 => {
                    let _ = DeleteObject(HGDIOBJ(self.handle.0));
                }
                14 => {
                    let _ = DeleteEnhMetaFile(Some(HENHMETAFILE(self.handle.0)));
                }
                3 => {
                    let global = HGLOBAL(self.handle.0);
                    let ptr = GlobalLock(global);
                    if !ptr.is_null() {
                        let _ = DeleteMetaFile((*(ptr as *const METAFILEPICT)).hMF);
                        let _ = GlobalUnlock(global);
                    }
                    let _ = GlobalFree(Some(global));
                }
                _ => {
                    let _ = GlobalFree(Some(HGLOBAL(self.handle.0)));
                }
            }
        }
    }
}
struct Snapshot(Vec<(u32, Vec<u8>)>, Vec<NativeFormat>);
impl Snapshot {
    fn capture(owner: &Owner) -> Result<(Self, u32), String> {
        let _open = Open::acquire(owner)?;
        let mut data = vec![];
        let mut native = vec![];
        let mut format = 0;
        let mut total = 0;
        // SAFETY: clipboard held open, HGLOBAL data copied while locked, no pointers escape.
        unsafe {
            loop {
                format = EnumClipboardFormats(format);
                if format == 0 {
                    break;
                }
                // Owner-display and private GDI entries are owner callbacks, not transferable data.
                if [0x80, 0x81, 0x82, 0x83, 0x8e].contains(&format)
                    || (0x300..=0x3ff).contains(&format)
                {
                    continue;
                }
                let Ok(handle) = GetClipboardData(format) else {
                    continue;
                };
                if [2, 3, 9, 14].contains(&format) {
                    native.push(NativeFormat::duplicate(format, handle)?);
                    continue;
                }
                let global = HGLOBAL(handle.0);
                let size = GlobalSize(global);
                // Some apps publish empty clipboard bookkeeping formats alongside real payloads.
                if size == 0 {
                    continue;
                }
                total += size;
                if total > MAX_BYTES {
                    return Err("clipboard_too_large".into());
                }
                let ptr = GlobalLock(global);
                if ptr.is_null() {
                    continue;
                }
                let bytes = std::slice::from_raw_parts(ptr as *const u8, size).to_vec();
                let _ = GlobalUnlock(global);
                data.push((format, bytes));
            }
        }
        // Closing can synthesize additional formats and advance the sequence. Record
        // the committed sequence, rather than treating Windows' own rendering as a race.
        let previous_owner = unsafe { GetClipboardOwner().ok() };
        drop(_open);
        let _verify = Open::acquire(owner)?;
        if unsafe { GetClipboardOwner().ok() } != previous_owner {
            return Err("clipboard_changed".into());
        }
        Ok((Self(data, native), unsafe { GetClipboardSequenceNumber() }))
    }
    fn put(&self, owner: &Owner, expected: Option<u32>) -> Result<u32, String> {
        let _open = Open::acquire(owner)?;
        // SAFETY: allocate every format before clearing, transfer ownership only after SetClipboardData succeeds.
        unsafe {
            if expected.is_some_and(|s| GetClipboardSequenceNumber() != s) {
                return Err("clipboard_changed".into());
            }
            let mut allocations = vec![];
            for (format, bytes) in &self.0 {
                let h = match GlobalAlloc(GMEM_MOVEABLE, bytes.len()) {
                    Ok(h) => h,
                    Err(_) => {
                        for (_, h) in allocations {
                            let _ = GlobalFree(Some(h));
                        }
                        return Err("clipboard_write_failed".into());
                    }
                };
                let ptr = GlobalLock(h);
                if ptr.is_null() {
                    let _ = GlobalFree(Some(h));
                    for (_, h) in allocations {
                        let _ = GlobalFree(Some(h));
                    }
                    return Err("clipboard_write_failed".into());
                }
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
                let _ = GlobalUnlock(h);
                allocations.push((*format, h));
            }
            let mut native = Vec::new();
            for original in &self.1 {
                match NativeFormat::duplicate(original.format, original.handle) {
                    Ok(copy) => native.push(copy),
                    Err(error) => {
                        for (_, h) in allocations {
                            let _ = GlobalFree(Some(h));
                        }
                        return Err(error);
                    }
                }
            }
            if EmptyClipboard().is_err() {
                for (_, h) in allocations {
                    let _ = GlobalFree(Some(h));
                }
                return Err("clipboard_write_failed".into());
            }
            let mut failed = false;
            for (format, h) in allocations {
                if SetClipboardData(format, Some(HANDLE(h.0))).is_err() {
                    let _ = GlobalFree(Some(h));
                    failed = true;
                }
            }
            for mut copy in native {
                if SetClipboardData(copy.format, Some(copy.handle)).is_ok() {
                    copy.handle = HANDLE::default();
                } else {
                    failed = true;
                }
            }
            if failed {
                return Err("clipboard_write_failed".into());
            }
            // Windows synthesizes CF_TEXT/CF_OEMTEXT/CF_LOCALE when closing a Unicode write.
            // Record the committed sequence, and verify another writer did not intervene.
            drop(_open);
            let _verify = Open::acquire(owner)?;
            if GetClipboardOwner().is_ok_and(|h| h != owner.0) {
                return Err("clipboard_changed".into());
            }
            Ok(GetClipboardSequenceNumber())
        }
    }
}
fn text_snapshot(text: &str) -> Snapshot {
    let data: Vec<u8> = wide(text).iter().flat_map(|c| c.to_le_bytes()).collect();
    Snapshot(vec![(UNICODE, data)], vec![])
}
fn read(owner: &Owner) -> Result<String, String> {
    let _open = Open::acquire(owner)?;
    // SAFETY: validate global memory length before reading, unlock before clipboard is closed.
    unsafe {
        let handle = GetClipboardData(UNICODE).map_err(|_| "selection_unavailable")?;
        let global = HGLOBAL(handle.0);
        let size = GlobalSize(global);
        if !(2..=MAX_BYTES).contains(&size) || !size.is_multiple_of(2) {
            return Err("clipboard_too_large".into());
        }
        let ptr = GlobalLock(global);
        if ptr.is_null() {
            return Err("selection_unavailable".into());
        }
        let units = std::slice::from_raw_parts(ptr as *const u16, size / 2);
        let len = units.iter().position(|c| *c == 0).unwrap_or(units.len());
        let text = String::from_utf16_lossy(&units[..len]);
        let _ = GlobalUnlock(global);
        if text.trim().is_empty() {
            Err("selection_unavailable".into())
        } else {
            Ok(text)
        }
    }
}
fn wait_modifiers() -> Result<(), String> {
    for _ in 0..100 {
        // SAFETY: query-only virtual key state; bounded only while a user-requested transaction is active.
        let held = unsafe {
            [VK_CONTROL, VK_SHIFT, VK_MENU, VK_LWIN, VK_RWIN]
                .iter()
                .any(|k| GetAsyncKeyState(k.0 as i32) < 0)
        };
        if !held {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(10));
    }
    Err("modifiers_held".into())
}
fn keys(key: VIRTUAL_KEY) -> Result<(), String> {
    chord(key, false)
}
fn chord(key: VIRTUAL_KEY, shift: bool) -> Result<(), String> {
    modified_key(key, true, shift)
}
fn modified_key(key: VIRTUAL_KEY, control: bool, shift: bool) -> Result<(), String> {
    let mut inputs = vec![];
    let mut events = vec![];
    if control {
        events.push((VK_CONTROL, false));
    }
    if shift {
        events.push((VK_SHIFT, false));
    }
    events.extend([(key, false), (key, true)]);
    if shift {
        events.push((VK_SHIFT, true));
    }
    if control {
        events.push((VK_CONTROL, true));
    }
    for (vk, up) in events {
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    dwFlags: if up {
                        KEYEVENTF_KEYUP
                    } else {
                        KEYBD_EVENT_FLAGS(0)
                    },
                    ..Default::default()
                },
            },
        });
    }
    // SAFETY: ABI-correct array and size. UIPI failures are detected; no escalation attempted.
    if unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) } != inputs.len() as u32 {
        return Err("input_blocked".into());
    }
    Ok(())
}
fn foreground(hwnd: usize) -> bool {
    unsafe { GetForegroundWindow().0 as usize == hwnd }
}
struct Transaction {
    owner: Owner,
    snapshot: Option<Snapshot>,
    sequence: u32,
}
impl Transaction {
    fn new(restore: bool) -> Result<Self, String> {
        let owner = Owner::new()?;
        let (snapshot, sequence) = if restore {
            let (snapshot, seq) = Snapshot::capture(&owner)?;
            (Some(snapshot), seq)
        } else {
            (None, unsafe { GetClipboardSequenceNumber() })
        };
        Ok(Self {
            owner,
            snapshot,
            sequence,
        })
    }
    fn copy(&mut self, hwnd: usize) -> Result<String, String> {
        wait_modifiers()?;
        if !foreground(hwnd) {
            return Err("focus_changed".into());
        }
        if unsafe { GetClipboardSequenceNumber() } != self.sequence {
            return Err("clipboard_changed".into());
        }
        keys(VK_C)?;
        let until = Instant::now() + Duration::from_millis(800);
        let mut copied_owner = None;
        let mut copy_started = false;
        while Instant::now() < until {
            if !foreground(hwnd) {
                return Err("focus_changed".into());
            }
            let seq = unsafe { GetClipboardSequenceNumber() };
            if seq != self.sequence || copy_started {
                // Search and Office can copy through a helper process. Ownership PID is
                // not a reliable identity check; require unchanged focus and a stable copy sequence.
                let owner = unsafe { GetClipboardOwner().ok() };
                if copy_started && owner != copied_owner {
                    return Err("clipboard_changed".into());
                }
                copied_owner = owner;
                copy_started = true;
                self.sequence = seq;
                let text = match read(&self.owner) {
                    Ok(text) => text,
                    // Office can publish an empty clipboard before its text formats are ready.
                    // Wait within the original deadline; do not start selecting another range yet.
                    Err(error) if error == "selection_unavailable" => {
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => return Err(error),
                };
                if unsafe { GetClipboardSequenceNumber() } != self.sequence {
                    let committed = unsafe { GetClipboardSequenceNumber() };
                    if read(&self.owner)? != text
                        || unsafe { GetClipboardSequenceNumber() } != committed
                    {
                        return Err("clipboard_changed".into());
                    }
                    self.sequence = committed;
                }
                if !foreground(hwnd) {
                    return Err("focus_changed".into());
                }
                return Ok(text);
            }
            if !foreground(hwnd) {
                return Err("focus_changed".into());
            }
            thread::sleep(Duration::from_millis(10));
        }
        Err("selection_unavailable".into())
    }
    fn paste(&mut self, hwnd: usize, text: &str) -> Result<(), String> {
        wait_modifiers()?;
        if !foreground(hwnd) {
            return Err("focus_changed".into());
        }
        self.sequence = text_snapshot(text).put(&self.owner, Some(self.sequence))?;
        if !foreground(hwnd) {
            return Err("focus_changed".into());
        }
        keys(VK_V)?;
        // Only an active paste transaction waits. Clipboard-based apps can read asynchronously.
        thread::sleep(Duration::from_millis(350));
        Ok(())
    }
}
impl Drop for Transaction {
    fn drop(&mut self) {
        if let Some(snapshot) = &self.snapshot {
            let _ = snapshot.put(&self.owner, Some(self.sequence));
        }
    }
}

#[derive(Clone)]
pub struct Selection {
    pub hwnd: usize,
    pub text: String,
    pub created: Instant,
}
pub fn capture(hwnd: usize, restore: bool) -> Result<Selection, String> {
    let mut tx = Transaction::new(restore)?;
    let text = match tx.copy(hwnd) {
        Ok(text) => text,
        Err(error) if error == "selection_unavailable" => {
            // No selection: capture the current line up to the caret, including unfinished words.
            wait_modifiers()?;
            if !foreground(hwnd) {
                return Err("focus_changed".into());
            }
            modified_key(VK_HOME, false, true)?;
            thread::sleep(Duration::from_millis(40));
            match tx.copy(hwnd) {
                Err(error) if error == "selection_unavailable" => {
                    chord(VK_LEFT, true)?;
                    thread::sleep(Duration::from_millis(40));
                    tx.copy(hwnd)?
                }
                result => result?,
            }
        }
        Err(error) => return Err(error),
    };
    Ok(Selection {
        hwnd,
        text,
        created: Instant::now(),
    })
}
pub fn replace(selection: &Selection, text: &str, restore: bool) -> Result<(), String> {
    if selection.created.elapsed() > Duration::from_secs(120) {
        return Err("selection_expired".into());
    }
    let mut tx = Transaction::new(restore)?;
    if tx.copy(selection.hwnd)? != selection.text {
        return Err("selection_changed".into());
    }
    tx.paste(selection.hwnd, text)
}
pub fn read_text() -> Result<String, String> {
    read(&Owner::new()?)
}
pub fn write_text(text: &str) -> Result<(), String> {
    if text.len() > 1_000_000 {
        return Err("text_too_large".into());
    }
    text_snapshot(text).put(&Owner::new()?, None)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::AttachThreadInput;
    #[test]
    fn unicode_snapshot_has_terminator_and_surrogates() {
        let snap = text_snapshot("سلام 🦀");
        let bytes = &snap.0[0].1;
        assert_eq!(&bytes[bytes.len() - 2..], &[0, 0]);
        assert_eq!(bytes.len(), 16);
    }
    #[test]
    #[ignore = "Temporarily writes real clipboard; run explicitly with --ignored --test-threads=1"]
    fn clipboard_round_trip_restores_formats() {
        let owner = Owner::new().unwrap();
        let (snapshot, _) = Snapshot::capture(&owner).unwrap();
        let seq = text_snapshot("PIMXSWAP سلام 🦀").put(&owner, None).unwrap();
        assert_eq!(read(&owner).unwrap(), "PIMXSWAP سلام 🦀");
        snapshot.put(&owner, Some(seq)).unwrap();
    }
    #[test]
    #[ignore = "Temporarily writes real clipboard; run explicitly, serially"]
    fn transaction_never_overwrites_newer_clipboard_data() {
        let owner = Owner::new().expect("owner");
        let (initial, _) = Snapshot::capture(&owner).expect("snapshot");
        let tx = Transaction::new(true).expect("transaction");
        let seq = text_snapshot("PIMXSWAP newer clipboard write")
            .put(&owner, None)
            .expect("external write");
        drop(tx);
        assert_eq!(
            read(&owner).expect("read"),
            "PIMXSWAP newer clipboard write"
        );
        initial
            .put(&owner, Some(seq))
            .expect("restore initial clipboard");
    }
    #[test]
    #[ignore = "Uses the real clipboard and a temporary native edit window; run explicitly, serially"]
    fn native_selection_copy_paste_and_clipboard_restore() {
        native_edit_round_trip(true, "sghl", "سلام", false);
    }
    #[test]
    #[ignore = "Uses a temporary native edit window and the real clipboard; run explicitly, serially"]
    fn native_word_at_caret_copy_paste_and_clipboard_restore() {
        native_edit_round_trip(false, "sghl", "سلام", false);
    }
    #[test]
    #[ignore = "Uses the real clipboard and a temporary native editable field; run serially"]
    fn native_partial_sentence_at_caret_with_bitmap_clipboard() {
        native_edit_round_trip(false, "sghl lk o,", "سلام من خو", false);
    }
    #[test]
    #[ignore = "Requires a running PIMXSWAP with default hotkey and English/Persian pair; uses a temporary native window"]
    fn running_app_global_shortcut_converts_partial_line() {
        native_edit_round_trip(false, "sghl lk o,", "سلام من خو", true);
    }
    #[test]
    #[ignore = "Requires running PIMXSWAP with English/Persian layouts; uses a temporary native window"]
    fn running_app_global_shortcut_converts_persian_to_english() {
        native_edit_round_trip(false, "اثممخ", "hello", true);
    }
    fn native_edit_round_trip(selected: bool, input: &str, expected: &str, global: bool) {
        let previous = unsafe { GetForegroundWindow() };
        let parent = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("PIMXSWAP selection integration test"),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                100,
                100,
                450,
                220,
                None,
                None,
                None,
                None,
            )
        }
        .expect("test window");
        let owner = Owner(parent);
        let edit = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("EDIT"),
                windows::core::PCWSTR(wide(input).as_ptr()),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(4 | 64),
                10,
                10,
                400,
                150,
                Some(parent),
                None,
                None,
                None,
            )
        }
        .expect("edit control");
        unsafe {
            // The test runner is a background console. Join input queues briefly to activate only
            // this temporary test window, then detach before testing production focus guards.
            let current = windows::Win32::System::Threading::GetCurrentThreadId();
            let foreground_thread = GetWindowThreadProcessId(previous, None);
            let attached = foreground_thread != current
                && AttachThreadInput(current, foreground_thread, true).as_bool();
            let _ = BringWindowToTop(parent);
            let _ = SetForegroundWindow(parent);
            let _ = SetFocus(Some(edit));
            SendMessageW(
                edit,
                0x00b1,
                Some(WPARAM(if selected {
                    0
                } else {
                    input.encode_utf16().count()
                })),
                Some(LPARAM(if selected {
                    -1
                } else {
                    input.encode_utf16().count() as isize
                })),
            );
            if attached {
                let _ = AttachThreadInput(current, foreground_thread, false);
            }
        }
        // Dispatch activation messages before the worker starts using the foreground guard.
        // SetForegroundWindow can return before the console's old input queue has settled.
        let activation_deadline = Instant::now() + Duration::from_secs(2);
        let mut stable_since = None;
        loop {
            unsafe {
                let mut message = MSG::default();
                while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
                if GetForegroundWindow() == parent {
                    let since = stable_since.get_or_insert_with(Instant::now);
                    if since.elapsed() >= Duration::from_millis(200) {
                        break;
                    }
                } else {
                    stable_since = None;
                    let current = windows::Win32::System::Threading::GetCurrentThreadId();
                    let foreground = GetWindowThreadProcessId(GetForegroundWindow(), None);
                    let attached = current != foreground
                        && AttachThreadInput(current, foreground, true).as_bool();
                    let _ = SetForegroundWindow(parent);
                    let _ = SetFocus(Some(edit));
                    if attached {
                        let _ = AttachThreadInput(current, foreground, false);
                    }
                }
            }
            assert!(
                Instant::now() < activation_deadline,
                "Test window did not acquire stable foreground focus"
            );
            thread::sleep(Duration::from_millis(10));
        }
        let clipboard_owner = Owner::new().expect("clipboard owner");
        let (initial, _) = Snapshot::capture(&clipboard_owner).expect("snapshot");
        let custom = unsafe { RegisterClipboardFormatW(w!("PIMXSWAP.Integration.Format")) };
        let mut formats = text_snapshot("PIMXSWAP clipboard preservation test");
        formats.0.push((custom, vec![1, 2, 3, 4, 5]));
        let pixels = [0x00112233u32; 4];
        let bitmap = unsafe { CreateBitmap(2, 2, 1, 32, Some(pixels.as_ptr().cast())) };
        assert!(!bitmap.is_invalid());
        formats.1.push(NativeFormat {
            format: 2,
            handle: HANDLE(bitmap.0),
        });
        formats
            .put(&clipboard_owner, None)
            .expect("write original formats");
        let (tx, rx) = std::sync::mpsc::channel();
        let hwnd = parent.0 as usize;
        let expected = expected.to_string();
        let expected_output = expected.clone();
        std::thread::spawn(move || {
            let result = if global {
                let result = chord(VK_SPACE, true);
                thread::sleep(Duration::from_millis(6000));
                result
            } else {
                capture(hwnd, true).and_then(|selection| replace(&selection, &expected, true))
            };
            let _ = tx.send(result);
        });
        let deadline = Instant::now() + Duration::from_secs(8);
        let result = loop {
            unsafe {
                let mut message = MSG::default();
                while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            if let Ok(result) = rx.try_recv() {
                break result;
            }
            if Instant::now() > deadline {
                break Err("test_timeout".into());
            }
            thread::sleep(Duration::from_millis(5));
        };
        let mut text = [0u16; 64];
        let len = unsafe { GetWindowTextW(edit, &mut text) };
        let output = String::from_utf16_lossy(&text[..len as usize]);
        let (after, seq) = Snapshot::capture(&clipboard_owner).expect("after snapshot");
        let restored_text = read(&clipboard_owner).expect("read preserved clipboard");
        let has_custom = after
            .0
            .iter()
            .any(|(format, bytes)| *format == custom && bytes.starts_with(&[1, 2, 3, 4, 5]));
        let has_bitmap = after.1.iter().any(|data| data.format == 2);
        initial
            .put(&clipboard_owner, Some(seq))
            .expect("restore initial clipboard");
        drop(owner);
        unsafe {
            let _ = SetForegroundWindow(previous);
        }
        result.expect("native selection transaction");
        assert_eq!(output, expected_output);
        assert_eq!(restored_text, "PIMXSWAP clipboard preservation test");
        assert!(has_custom);
        assert!(has_bitmap, "Original bitmap clipboard format was preserved");
    }
}
