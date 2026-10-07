use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use windows::{core::PCWSTR, Win32::System::Registry::*};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    pub ui_language: String,
    pub theme: String,
    pub auto_hotkey: String,
    pub reverse_hotkey: String,
    pub open_hotkey: String,
    pub preferred_layouts: Vec<String>,
    pub source: String,
    pub target: String,
    pub threshold: f64,
    pub launch_at_startup: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
    pub notifications: bool,
    pub reduce_motion: bool,
    pub animation_intensity: u8,
    pub restore_clipboard: bool,
    pub debug_logs: bool,
    pub onboarded: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            ui_language: "en".into(),
            theme: "dark".into(),
            auto_hotkey: "Ctrl+Shift+Space".into(),
            reverse_hotkey: "Ctrl+Alt+Shift+Space".into(),
            open_hotkey: "Ctrl+Alt+P".into(),
            preferred_layouts: vec![],
            source: String::new(),
            target: String::new(),
            threshold: 0.86,
            launch_at_startup: false,
            minimize_to_tray: true,
            start_minimized: false,
            notifications: true,
            reduce_motion: false,
            animation_intensity: 1,
            restore_clipboard: true,
            debug_logs: false,
            onboarded: false,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if !["en", "fa"].contains(&self.ui_language.as_str())
            || !["dark", "light", "system"].contains(&self.theme.as_str())
            || !self.threshold.is_finite()
            || !(0.65..=0.99).contains(&self.threshold)
            || self.animation_intensity > 2
            || self.preferred_layouts.len() > 32
        {
            return Err("invalid_settings".into());
        }
        let keys = [&self.auto_hotkey, &self.reverse_hotkey, &self.open_hotkey];
        let parsed = keys
            .map(|s| crate::hotkeys::parse(s))
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        if parsed[0] == parsed[1] || parsed[0] == parsed[2] || parsed[1] == parsed[2] {
            return Err("duplicate_hotkey".into());
        }
        Ok(())
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let bytes = fs::read(path).map_err(|_| "settings_read_failed")?;
        if bytes.len() > 64 * 1024 {
            return Err("invalid_settings".into());
        }
        let value: Self = serde_json::from_slice(&bytes).map_err(|_| "invalid_settings")?;
        value.validate()?;
        Ok(value)
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let parent = path.parent().ok_or("settings_write_failed")?;
        fs::create_dir_all(parent).map_err(|_| "settings_write_failed")?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|_| "settings_write_failed")?;
        // Windows rename does not replace destinations. ReplaceFileW preserves atomicity on existing files.
        let temp = path.with_extension("tmp");
        {
            use std::io::Write;
            let mut file = fs::File::create(&temp).map_err(|_| "settings_write_failed")?;
            file.write_all(&bytes)
                .map_err(|_| "settings_write_failed")?;
            file.sync_all().map_err(|_| "settings_write_failed")?;
        }
        if path.exists() {
            use windows::Win32::Storage::FileSystem::{ReplaceFileW, REPLACE_FILE_FLAGS};
            let dest = wide(&path.to_string_lossy());
            let src = wide(&temp.to_string_lossy());
            // SAFETY: both paths are nul-terminated owned buffers, all optional pointers null.
            unsafe {
                ReplaceFileW(
                    PCWSTR(dest.as_ptr()),
                    PCWSTR(src.as_ptr()),
                    None,
                    REPLACE_FILE_FLAGS(0),
                    None,
                    None,
                )
                .map_err(|_| "settings_write_failed")?;
            }
        } else {
            fs::rename(&temp, path).map_err(|_| "settings_write_failed")?;
        }
        Ok(())
    }
}
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
pub fn startup(enabled: bool) -> Result<(), String> {
    let key = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let name = wide("PIMXSWAP");
    // SAFETY: current-user registry key only, valid strings and explicit byte lengths; handle closed on every path.
    unsafe {
        let mut handle = HKEY::default();
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key.as_ptr()),
            Some(0),
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut handle,
            None,
        )
        .ok()
        .map_err(|_| "startup_failed")?;
        let result = if enabled {
            let exe = std::env::current_exe().map_err(|_| "startup_failed");
            match exe {
                Ok(exe) => {
                    let cmd = wide(&format!("\"{}\" --background", exe.display()));
                    let bytes =
                        std::slice::from_raw_parts(cmd.as_ptr() as *const u8, cmd.len() * 2);
                    RegSetValueExW(handle, PCWSTR(name.as_ptr()), Some(0), REG_SZ, Some(bytes))
                        .ok()
                        .map_err(|_| "startup_failed".to_string())
                }
                Err(e) => Err(e.to_string()),
            }
        } else {
            let result = RegDeleteValueW(handle, PCWSTR(name.as_ptr()));
            if result.0 == 0 || result.0 == 2 {
                Ok(())
            } else {
                Err("startup_failed".into())
            }
        };
        let _ = RegCloseKey(handle);
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_round_trip() {
        let a = Config::default();
        let b: Config = serde_json::from_str(&serde_json::to_string(&a).expect("serialize"))
            .expect("deserialize");
        assert_eq!(a, b);
        assert!(b.validate().is_ok());
    }
    #[test]
    fn invalid_threshold_and_duplicate_keys() {
        let mut a = Config {
            threshold: 0.1,
            ..Config::default()
        };
        assert!(a.validate().is_err());
        a.threshold = 0.9;
        a.reverse_hotkey = a.auto_hotkey.clone();
        assert_eq!(a.validate().unwrap_err(), "duplicate_hotkey");
    }
    #[test]
    fn partial_old_settings_have_defaults() {
        let c: Config = serde_json::from_str("{\"theme\":\"light\"}").expect("partial config");
        assert_eq!(c.auto_hotkey, "Ctrl+Shift+Space");
    }
    #[test]
    fn durable_save_replaces_existing_configuration() {
        let dir = std::env::temp_dir().join(format!("PIMXSWAP-config-test-{}", std::process::id()));
        let path = dir.join("settings.json");
        let mut config = Config::default();
        config.save(&path).expect("first save");
        config.theme = "light".into();
        config.save(&path).expect("atomic replacement");
        assert_eq!(Config::load(&path).expect("load"), config);
        std::fs::remove_file(&path).expect("remove test file");
        std::fs::remove_dir(&dir).expect("remove empty test dir");
    }
}
