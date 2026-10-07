use crate::config::Config;
use std::sync::mpsc::{self, Sender};
use windows::core::w;
use windows::Win32::{
    System::Threading::GetCurrentThreadId,
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shortcut {
    pub modifiers: u32,
    pub key: u32,
}
pub fn parse(text: &str) -> Result<Shortcut, String> {
    let mut modifiers = 0u32;
    let mut key = None;
    for token in text.split('+').map(str::trim) {
        let m = match token.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => Some(MOD_CONTROL.0),
            "alt" => Some(MOD_ALT.0),
            "shift" => Some(MOD_SHIFT.0),
            "win" | "super" => Some(MOD_WIN.0),
            _ => None,
        };
        if let Some(m) = m {
            if modifiers & m != 0 {
                return Err("invalid_hotkey".into());
            }
            modifiers |= m;
            continue;
        }
        if key.is_some() {
            return Err("invalid_hotkey".into());
        }
        let upper = token.to_ascii_uppercase();
        key = Some(match upper.as_str() {
            "SPACE" => VK_SPACE.0 as u32,
            "ENTER" => VK_RETURN.0 as u32,
            "TAB" => VK_TAB.0 as u32,
            _ if upper.len() == 1 && upper.as_bytes()[0].is_ascii_alphanumeric() => {
                upper.as_bytes()[0] as u32
            }
            _ if upper.starts_with('F') => {
                let n: u32 = upper[1..].parse().map_err(|_| "invalid_hotkey")?;
                if !(1..=24).contains(&n) {
                    return Err("invalid_hotkey".into());
                }
                VK_F1.0 as u32 + n - 1
            }
            _ => return Err("invalid_hotkey".into()),
        });
    }
    if modifiers == 0 {
        return Err("invalid_hotkey".into());
    }
    Ok(Shortcut {
        modifiers,
        key: key.ok_or("invalid_hotkey")?,
    })
}
fn bindings(c: &Config) -> Result<Vec<Shortcut>, String> {
    [&c.auto_hotkey, &c.reverse_hotkey, &c.open_hotkey]
        .into_iter()
        .map(|s| parse(s))
        .collect()
}
const UPDATE: u32 = WM_APP + 41;
pub const OPEN_INSTANCE: u32 = WM_APP + 42;
struct Update {
    config: Config,
    reply: Sender<Result<(), String>>,
}
pub struct Hotkeys {
    thread: u32,
    send: Sender<Update>,
}
fn register(keys: &[Shortcut]) -> Result<(), String> {
    // SAFETY: thread-owned registrations, no window pointer; Windows reports global collisions.
    unsafe {
        for (i, key) in keys.iter().enumerate() {
            if RegisterHotKey(
                None,
                i as i32 + 1,
                HOT_KEY_MODIFIERS(key.modifiers | MOD_NOREPEAT.0),
                key.key,
            )
            .is_err()
            {
                for j in 0..i {
                    let _ = UnregisterHotKey(None, j as i32 + 1);
                }
                return Err("hotkey_collision".into());
            }
        }
    }
    Ok(())
}
fn unregister() {
    unsafe {
        for i in 1..=3 {
            let _ = UnregisterHotKey(None, i);
        }
    }
}
impl Hotkeys {
    pub fn start(
        config: Config,
        callback: impl Fn(u32) + Send + 'static,
    ) -> Result<(Self, Option<String>), String> {
        let (send, recv) = mpsc::channel::<Update>();
        let (ready_tx, ready_rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("pimxswap-hotkeys".into())
            .spawn(move || {
                // SAFETY: local MSG and PeekMessage creates this thread's message queue before publishing its ID.
                unsafe {
                    let mut msg = MSG::default();
                    let _ = PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE);
                    let window = CreateWindowExW(
                        WINDOW_EX_STYLE(0),
                        w!("STATIC"),
                        w!("PIMXSWAP.Instance"),
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
                    .ok();
                    let mut current = bindings(&config).unwrap_or_default();
                    let initial = register(&current).err();
                    let _ = ready_tx.send((GetCurrentThreadId(), initial));
                    loop {
                        let status = GetMessageW(&mut msg, None, 0, 0).0;
                        if status <= 0 {
                            break;
                        }
                        if msg.message == OPEN_INSTANCE {
                            callback(3);
                        } else if msg.message == WM_HOTKEY {
                            callback(msg.wParam.0 as u32);
                        } else if msg.message == UPDATE {
                            while let Ok(update) = recv.try_recv() {
                                let result = bindings(&update.config).and_then(|new| {
                                    unregister();
                                    if let Err(e) = register(&new) {
                                        let _ = register(&current);
                                        Err(e)
                                    } else {
                                        current = new;
                                        Ok(())
                                    }
                                });
                                let _ = update.reply.send(result);
                            }
                        }
                    }
                    unregister();
                    if let Some(window) = window {
                        let _ = DestroyWindow(window);
                    }
                }
            })
            .map_err(|_| "hotkey_init_failed")?;
        let (thread, error) = ready_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .map_err(|_| "hotkey_init_failed")?;
        Ok((Self { thread, send }, error))
    }
    pub fn update(&self, config: Config) -> Result<(), String> {
        let (tx, rx) = mpsc::channel();
        self.send
            .send(Update { config, reply: tx })
            .map_err(|_| "hotkey_init_failed")?;
        // SAFETY: the thread ID owns a live queue created during start.
        unsafe {
            PostThreadMessageW(self.thread, UPDATE, Default::default(), Default::default())
                .map_err(|_| "hotkey_init_failed")?;
        }
        rx.recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|_| "hotkey_init_failed")?
    }
}
impl Drop for Hotkeys {
    fn drop(&mut self) {
        unsafe {
            let _ =
                PostThreadMessageW(self.thread, WM_QUIT, Default::default(), Default::default());
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_and_normalizes() {
        assert_eq!(
            parse("Ctrl+Shift+Space").unwrap(),
            parse("shift + control + space").unwrap()
        );
        assert_eq!(parse("Ctrl+F12").unwrap().key, 123);
    }
    #[test]
    fn rejects_unsafe_and_invalid() {
        for s in [
            "A",
            "Ctrl+Ctrl+A",
            "Alt+F25",
            "Ctrl+A+B",
            "Ctrl+",
            "Ctrl+Escape",
        ] {
            assert!(parse(s).is_err(), "{s}");
        }
    }
    #[test]
    #[ignore = "Temporarily registers three real global shortcuts; run explicitly, serially"]
    fn windows_collision_keeps_previous_bindings() {
        let config = Config {
            auto_hotkey: "Ctrl+Alt+Shift+F20".into(),
            reverse_hotkey: "Ctrl+Alt+Shift+F21".into(),
            open_hotkey: "Ctrl+Alt+Shift+F22".into(),
            ..Config::default()
        };
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let (keys, error) = Hotkeys::start(config.clone(), move |id| {
            let _ = event_tx.send(id);
        })
        .expect("start");
        assert!(error.is_none());
        let mut inputs = Vec::new();
        for (key, up) in [
            (VK_CONTROL, false),
            (VK_MENU, false),
            (VK_SHIFT, false),
            (VK_F20, false),
            (VK_F20, true),
            (VK_SHIFT, true),
            (VK_MENU, true),
            (VK_CONTROL, true),
        ] {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: key,
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
        // SAFETY: test-owned registered shortcut, balanced key down/up events.
        assert_eq!(
            unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) },
            inputs.len() as u32
        );
        assert_eq!(
            event_rx
                .recv_timeout(std::time::Duration::from_secs(2))
                .expect("WM_HOTKEY callback"),
            1
        );
        let reserved = parse("Ctrl+Alt+Shift+F23").expect("parse");
        unsafe {
            RegisterHotKey(
                None,
                90,
                HOT_KEY_MODIFIERS(reserved.modifiers),
                reserved.key,
            )
            .expect("reserve test key");
        }
        let mut changed = config.clone();
        changed.auto_hotkey = "Ctrl+Alt+Shift+F23".into();
        let result = keys.update(changed);
        unsafe {
            UnregisterHotKey(None, 90).expect("release test key");
        }
        assert_eq!(result.unwrap_err(), "hotkey_collision");
        let old = parse(&config.auto_hotkey).expect("old key");
        assert!(
            unsafe { RegisterHotKey(None, 91, HOT_KEY_MODIFIERS(old.modifiers), old.key) }.is_err(),
            "old binding remains reserved"
        );
        keys.update(config).expect("valid update after collision");
    }
}
