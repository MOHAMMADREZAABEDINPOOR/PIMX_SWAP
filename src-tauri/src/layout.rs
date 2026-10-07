use crate::config::{wide, Config};
use serde::Serialize;
use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;
use windows::Win32::{
    Globalization::{GetLocaleInfoW, LOCALE_SENGLISHDISPLAYNAME, LOCALE_SISO639LANGNAME},
    UI::{
        Input::KeyboardAndMouse::*,
        WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
    },
};
use windows::{
    core::{w, PCWSTR, PWSTR},
    Win32::System::Registry::*,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Layout {
    pub id: String,
    pub name: String,
    pub language: String,
    pub active: bool,
}

pub fn foreground_layout() -> String {
    // SAFETY: these query-only APIs accept null process-id pointers and live foreground handles.
    unsafe {
        let thread = GetWindowThreadProcessId(GetForegroundWindow(), None);
        format!("{:016X}", GetKeyboardLayout(thread).0 as usize)
    }
}

pub fn installed() -> Vec<Layout> {
    // SAFETY: Windows fills only the supplied slice; locale buffer length is explicit.
    unsafe {
        let count = GetKeyboardLayoutList(None);
        if count <= 0 {
            return vec![];
        }
        let mut handles = vec![HKL::default(); count as usize];
        let count = GetKeyboardLayoutList(Some(&mut handles));
        handles.truncate(count.max(0) as usize);
        let active = foreground_layout();
        handles
            .into_iter()
            .map(|h| {
                let locale = h.0 as usize as u32 & 0xffff;
                let mut buf = [0u16; 128];
                let n = GetLocaleInfoW(locale, LOCALE_SENGLISHDISPLAYNAME, Some(&mut buf));
                let id = format!("{:016X}", h.0 as usize);
                let name = if n > 1 {
                    String::from_utf16_lossy(&buf[..n as usize - 1])
                } else {
                    format!("Layout {id}")
                };
                let language = language(locale);
                Layout {
                    active: id == active,
                    id,
                    name,
                    language,
                }
            })
            .collect()
    }
}

fn language(locale: u32) -> String {
    let mut buf = [0u16; 32];
    let n = unsafe { GetLocaleInfoW(locale, LOCALE_SISO639LANGNAME, Some(&mut buf)) };
    if n > 1 {
        String::from_utf16_lossy(&buf[..n as usize - 1])
    } else {
        "und".into()
    }
}

/// Enumerate the Windows keyboard layout catalog without activating every layout.
pub fn available() -> Vec<Layout> {
    let loaded = installed();
    let mut result = loaded.clone();
    unsafe {
        let mut key = HKEY::default();
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            w!("SYSTEM\\CurrentControlSet\\Control\\Keyboard Layouts"),
            None,
            KEY_READ,
            &mut key,
        )
        .is_err()
        {
            return result;
        }
        for index in 0..1024 {
            let mut name = [0u16; 256];
            let mut len = name.len() as u32;
            if RegEnumKeyExW(
                key,
                index,
                Some(PWSTR(name.as_mut_ptr())),
                &mut len,
                None,
                None,
                None,
                None,
            )
            .is_err()
            {
                break;
            }
            let id = String::from_utf16_lossy(&name[..len as usize]);
            let Ok(code) = u32::from_str_radix(&id, 16) else {
                continue;
            };
            // IMEs do not provide a reversible physical-key character table.
            if id.len() != 8 || code & 0xf0000000 == 0xe0000000 {
                continue;
            }
            let locale = code & 0xffff;
            let mut text = [0u16; 256];
            let mut bytes = (text.len() * 2) as u32;
            if RegGetValueW(
                key,
                PCWSTR(name.as_ptr()),
                w!("Layout Text"),
                RRF_RT_REG_SZ,
                None,
                Some(text.as_mut_ptr().cast()),
                Some(&mut bytes),
            )
            .is_err()
            {
                continue;
            }
            let len = text.iter().position(|c| *c == 0).unwrap_or(text.len());
            result.push(Layout {
                id,
                name: String::from_utf16_lossy(&text[..len]),
                language: language(locale),
                active: false,
            });
        }
        let _ = RegCloseKey(key);
    }
    result.sort_by_key(|l| l.name.to_lowercase());
    result
}

fn resolve(id: &str) -> Result<HKL, String> {
    if let Some(l) = installed().iter().find(|l| l.id == id) {
        return Ok(HKL(
            usize::from_str_radix(&l.id, 16).map_err(|_| "invalid_layout")? as *mut _,
        ));
    }
    if id.len() != 8 || !available().iter().any(|l| l.id == id) {
        return Err("layout_unavailable".into());
    }
    unsafe { LoadKeyboardLayoutW(PCWSTR(wide(id).as_ptr()), KLF_NOTELLSHELL) }
        .map_err(|_| "layout_unavailable".into())
}

pub fn detection_layouts(config: &Config) -> Vec<Layout> {
    let mut layouts = installed();
    for l in available() {
        if (config.preferred_layouts.contains(&l.id)
            || l.id == config.source
            || l.id == config.target)
            && !layouts.iter().any(|loaded| loaded.id == l.id)
        {
            layouts.push(l);
        }
    }
    layouts
}

#[derive(Debug, Clone, Copy)]
struct Stroke {
    scan: u32,
    shift: bool,
    caps: bool,
    altgr: bool,
}
#[derive(Debug, Clone)]
struct Glyph {
    text: String,
    dead: bool,
}
#[derive(Default)]
struct Table {
    reverse: HashMap<String, Vec<Stroke>>,
    glyphs: HashMap<(u32, u8), Glyph>,
}
impl Stroke {
    fn flags(self) -> u8 {
        self.shift as u8 | ((self.caps as u8) << 1) | ((self.altgr as u8) << 2)
    }
}

fn combining(c: char) -> Option<char> {
    match c {
        '\u{00b4}' => Some('\u{0301}'),
        '`' => Some('\u{0300}'),
        '^' => Some('\u{0302}'),
        '~' => Some('\u{0303}'),
        '\u{00a8}' => Some('\u{0308}'),
        '\u{02c7}' => Some('\u{030c}'),
        '\u{02da}' => Some('\u{030a}'),
        '\u{00b8}' => Some('\u{0327}'),
        _ => None,
    }
}
fn compose(dead: &str, base: &str) -> String {
    if base == " " {
        return dead.to_string();
    }
    if let Some(mark) = dead.chars().next().and_then(combining) {
        let s: String = format!("{base}{mark}").nfc().collect();
        if s.chars().count() < base.chars().count() + 1 {
            return s;
        }
    }
    format!("{dead}{base}")
}

fn table(id: &str) -> Result<Table, String> {
    let hkl = resolve(id)?;
    let mut result = Table::default();
    // SAFETY: HKL was verified loaded; key-state and UTF-16 output arrays have documented sizes.
    // Flag 4 prevents ToUnicodeEx from changing the user's dead-key state (Windows 10 1607+).
    unsafe {
        for flags in [0u8, 1, 4, 5, 2, 3, 6, 7] {
            for scan in 1..=0x7f {
                let vk = MapVirtualKeyExW(scan, MAPVK_VSC_TO_VK_EX, Some(hkl));
                if vk == 0 || vk > 255 {
                    continue;
                }
                let stroke = Stroke {
                    scan,
                    shift: flags & 1 != 0,
                    caps: flags & 2 != 0,
                    altgr: flags & 4 != 0,
                };
                let mut state = [0u8; 256];
                if stroke.shift {
                    state[VK_SHIFT.0 as usize] = 0x80;
                }
                if stroke.caps {
                    state[VK_CAPITAL.0 as usize] = 1;
                }
                if stroke.altgr {
                    for key in [VK_CONTROL, VK_MENU, VK_RMENU] {
                        state[key.0 as usize] = 0x80;
                    }
                }
                let mut buf = [0u16; 16];
                let n = ToUnicodeEx(vk, scan, &state, &mut buf, 4, Some(hkl));
                if n == 0 {
                    continue;
                }
                let text = String::from_utf16_lossy(&buf[..n.unsigned_abs().min(16) as usize]);
                if text.is_empty() || text.chars().any(char::is_control) {
                    continue;
                }
                let glyph = Glyph {
                    text: text.clone(),
                    dead: n < 0,
                };
                result.glyphs.insert((scan, stroke.flags()), glyph);
                result.reverse.entry(text).or_insert_with(|| vec![stroke]);
            }
        }
    }
    let mut dead: Vec<_> = result
        .glyphs
        .iter()
        .filter(|(_, g)| g.dead)
        .map(|(&(scan, flags), g)| {
            (
                Stroke {
                    scan,
                    shift: flags & 1 != 0,
                    caps: flags & 2 != 0,
                    altgr: flags & 4 != 0,
                },
                g.text.clone(),
            )
        })
        .collect();
    dead.sort_by_key(|(s, _)| {
        (
            [0u8, 1, 4, 5, 2, 3, 6, 7]
                .iter()
                .position(|f| *f == s.flags())
                .unwrap_or(8),
            s.scan,
        )
    });
    let bases: Vec<_> = result
        .reverse
        .iter()
        .map(|(text, path)| (text.clone(), path.clone()))
        .collect();
    for (stroke, accent) in dead {
        for (base, path) in &bases {
            if base.chars().count() != 1 {
                continue;
            }
            let composed = compose(&accent, base);
            if composed.chars().count() == 1 {
                let mut strokes = vec![stroke];
                strokes.extend(path);
                result.reverse.entry(composed).or_insert(strokes);
            }
        }
    }
    Ok(result)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversion {
    pub text: String,
    pub unmapped: usize,
    pub mapped: usize,
}
#[derive(Default)]
pub struct Engine {
    tables: HashMap<String, Table>,
}
impl Engine {
    pub fn clear(&mut self) {
        self.tables.clear();
    }
    pub fn convert(
        &mut self,
        text: &str,
        source: &str,
        target: &str,
    ) -> Result<Conversion, String> {
        if text.len() > 1_000_000 {
            return Err("text_too_large".into());
        }
        for id in [source, target] {
            if !self.tables.contains_key(id) {
                self.tables.insert(id.to_string(), table(id)?);
            }
        }
        let from = &self.tables[source];
        let to = &self.tables[target];
        let mut output = String::with_capacity(text.len());
        let mut unmapped = 0;
        let mut mapped = 0;
        let mut offset = 0;
        let mut pending_dead: Option<String> = None;
        while offset < text.len() {
            let tail = &text[offset..];
            let mut found = None;
            for (index, _) in tail
                .char_indices()
                .skip(1)
                .take(4)
                .chain(std::iter::once((tail.len().min(16), '\0')))
            {
                if !tail.is_char_boundary(index) {
                    continue;
                }
                if let Some(strokes) = from.reverse.get(&tail[..index]) {
                    found = Some((index, strokes));
                }
            }
            if let Some((len, strokes)) = found {
                let mut piece = String::new();
                let mut dead = pending_dead.clone();
                let mut valid = true;
                for stroke in strokes {
                    if let Some(g) = to.glyphs.get(&(stroke.scan, stroke.flags())) {
                        if g.dead {
                            if let Some(prev) = dead.replace(g.text.clone()) {
                                piece.push_str(&prev);
                            }
                        } else if let Some(accent) = dead.take() {
                            piece.push_str(&compose(&accent, &g.text));
                        } else {
                            piece.push_str(&g.text);
                        }
                    } else {
                        valid = false;
                        break;
                    }
                }
                if valid {
                    output.push_str(&piece);
                    pending_dead = dead;
                    mapped += tail[..len].chars().count();
                } else {
                    if let Some(accent) = pending_dead.take() {
                        output.push_str(&accent);
                    }
                    output.push_str(&tail[..len]);
                    unmapped += tail[..len].chars().count();
                }
                offset += len;
            } else {
                let c = tail.chars().next().ok_or("invalid_text")?;
                if let Some(accent) = pending_dead.take() {
                    output.push_str(&accent);
                }
                output.push(c);
                offset += c.len_utf8();
                if !c.is_whitespace() {
                    unmapped += 1;
                }
            }
        }
        if let Some(accent) = pending_dead {
            output.push_str(&accent);
        }
        Ok(Conversion {
            text: output,
            unmapped,
            mapped,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dead_key_composition() {
        assert_eq!(compose("´", "e"), "é");
        assert_eq!(compose("^", "a"), "â");
        assert_eq!(compose("´", " "), "´");
    }
    #[test]
    fn installed_layouts_are_unique_and_queryable() {
        let ls = installed();
        assert!(!ls.is_empty());
        for l in ls {
            assert!(table(&l.id).is_ok());
        }
    }
    #[test]
    fn rejects_forged_handle() {
        assert!(table("FFFFFFFFFFFFFFFF").is_err());
    }
}
