// SPDX-License-Identifier: GPL-3.0-or-later
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub version: u32,
    pub hotkey: String,
    pub ui_language: String,
    pub action: String,
    pub favorites: Vec<String>,
    pub recent: Vec<String>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 2,
            hotkey: "Ctrl+Alt+Shift+I".into(),
            ui_language: "en".into(),
            action: "insert".into(),
            favorites: vec![],
            recent: vec![],
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hotkey {
    pub modifiers: u32,
    pub key: u32,
}
impl Hotkey {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut modifiers = 0;
        let mut key = 0;
        for part in text.split('+').map(str::trim) {
            let bit = match part.to_ascii_lowercase().as_str() {
                "alt" => 1,
                "ctrl" => 2,
                "shift" => 4,
                _ => 0,
            };
            if bit != 0 {
                if modifiers & bit != 0 {
                    return Err("Duplicate hotkey modifier".into());
                }
                modifiers |= bit;
            } else if key == 0 && part.len() == 1 && part.as_bytes()[0].is_ascii_alphanumeric() {
                key = u32::from(part.as_bytes()[0].to_ascii_uppercase());
            } else if key == 0 && part.to_ascii_uppercase().starts_with('F') {
                let n: u32 = part[1..].parse().map_err(|_| "Invalid function key")?;
                if !(2..=24).contains(&n) || n == 12 {
                    return Err("Use F2–F24 except reserved F12".into());
                }
                key = 0x70 + n - 1;
            } else {
                return Err("Use Ctrl/Alt/Shift + A–Z, 0–9, or F2–F24 (except F12)".into());
            }
        }
        if modifiers & 3 == 0 || key == 0 {
            return Err("Include Ctrl or Alt and one key".into());
        }
        Ok(Self { modifiers, key })
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 2 {
            return Err("Unsupported settings version; original file preserved".into());
        }
        Hotkey::parse(&self.hotkey)?;
        if !matches!(self.ui_language.as_str(), "en" | "zh-CN")
            || !matches!(self.action.as_str(), "copy" | "insert")
            || self.favorites.len() > 1000
            || self.recent.len() > 30
            || self.favorites.iter().chain(&self.recent).any(|s| {
                s.is_empty()
                    || s.len() > 80
                    || !s
                        .bytes()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            })
        {
            return Err("Invalid palette settings".into());
        }
        Ok(())
    }
    pub fn used(&mut self, id: &str) {
        self.recent.retain(|s| s != id);
        self.recent.insert(0, id.into());
        self.recent.truncate(30);
    }
    pub fn favorite(&mut self, id: &str) {
        if self.favorites.iter().any(|s| s == id) {
            self.favorites.retain(|s| s != id);
        } else if self.favorites.len() < 1000 {
            self.favorites.push(id.into());
        }
    }
}
pub fn default_path() -> Result<PathBuf, String> {
    #[cfg(windows)]
    let root = std::env::var_os("LOCALAPPDATA");
    #[cfg(not(windows))]
    let root = std::env::var_os("XDG_CONFIG_HOME").or_else(|| {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config").into_os_string())
    });
    root.map(|p| PathBuf::from(p).join("IPA-first-TTS/palette.json"))
        .ok_or_else(|| "Local user configuration directory unavailable".into())
}
/// Returns a visible recovery warning. Unknown future versions fail without modification.
pub fn load(path: &Path) -> Result<(Settings, Option<String>), String> {
    if !path.exists() {
        return Ok((Settings::default(), None));
    }
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.len() > 131072 {
        return recover(path, "Settings exceed 128 KiB");
    }
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let parsed: Result<Settings, String> = (|| {
        let mut value: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if value["version"].as_u64().is_some_and(|v| v > 2) {
            return Err("future-version".into());
        }
        if value["version"] == 1 {
            value["version"] = 2.into();
            if value.get("action").is_none() {
                value["action"] = "insert".into();
            }
        }
        let settings: Settings = serde_json::from_value(value).map_err(|e| e.to_string())?;
        settings.validate()?;
        Ok(settings)
    })();
    match parsed {
        Ok(s) => Ok((s, None)),
        Err(e) if e == "future-version" => {
            Err("Settings were written by a newer version; original file preserved".into())
        }
        Err(e) => recover(path, &e),
    }
}
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
fn unique(path: &Path, suffix: &str) -> PathBuf {
    path.with_extension(format!(
        "{suffix}-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ))
}
fn recover(path: &Path, reason: &str) -> Result<(Settings, Option<String>), String> {
    let backup = unique(path, "corrupt");
    fs::rename(path, &backup).map_err(|e| format!("Cannot preserve corrupt settings: {e}"))?;
    Ok((
        Settings::default(),
        Some(format!(
            "Recovered settings: {reason}. Original: {}",
            backup.display()
        )),
    ))
}
pub fn save(path: &Path, settings: &Settings) -> Result<(), String> {
    settings.validate()?;
    // Refuse to replace a newer schema even if an older instance was already running.
    if path.exists() {
        let _ = load(path)?;
    }
    let parent = path.parent().ok_or("No settings parent directory")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temporary = unique(path, "tmp");
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| e.to_string())?;
        file.write_all(&serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        replace(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
#[cfg(not(windows))]
fn replace(from: &Path, to: &Path) -> Result<(), String> {
    fs::rename(from, to).map_err(|e| e.to_string())
}
#[cfg(windows)]
fn replace(from: &Path, to: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };
    let from: Vec<_> = from.as_os_str().encode_wide().chain([0]).collect();
    let to: Vec<_> = to.as_os_str().encode_wide().chain([0]).collect();
    // SAFETY: terminated buffers remain live; both files are in the same config directory.
    if unsafe {
        MoveFileExW(
            from.as_ptr(),
            to.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    } == 0
    {
        Err(std::io::Error::last_os_error().to_string())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hotkey_validation() {
        assert_eq!(
            Hotkey::parse("Ctrl+Alt+Shift+I").unwrap(),
            Hotkey {
                modifiers: 7,
                key: 73
            }
        );
        for bad in [
            "Ctrl+Space",
            "I",
            "Ctrl+Ctrl+I",
            "Alt+F12",
            "Shift+I",
            "Alt+F99",
            "Ctrl+A+B",
        ] {
            assert!(Hotkey::parse(bad).is_err(), "{bad}");
        }
        assert!(Hotkey::parse("Alt+F3").is_ok());
    }
    #[test]
    fn persistence_migration_recovery_and_future_preservation() {
        let directory = unique(&std::env::temp_dir().join("ipa-palette-test"), "dir");
        fs::create_dir(&directory).unwrap();
        let path = directory.join("settings.json");
        let mut settings = load(&path).unwrap().0;
        settings.favorite("vowels-0252");
        settings.used("vowels-0252");
        settings.used("vowels-0252");
        save(&path, &settings).unwrap();
        assert_eq!(load(&path).unwrap().0, settings);
        assert_eq!(settings.recent.len(), 1);
        settings.favorite("vowels-0252");
        save(&path, &settings).unwrap();
        assert!(load(&path).unwrap().0.favorites.is_empty());
        let mut v = serde_json::to_value(&settings).unwrap();
        v["version"] = 1.into();
        v.as_object_mut().unwrap().remove("action");
        fs::write(&path, v.to_string()).unwrap();
        assert_eq!(load(&path).unwrap().0.version, 2);
        fs::write(&path, b"{broken").unwrap();
        let recovered = load(&path).unwrap();
        assert!(recovered.1.is_some());
        assert!(fs::read_dir(&directory).unwrap().any(|e| e
            .unwrap()
            .path()
            .extension()
            .unwrap()
            .to_string_lossy()
            .starts_with("corrupt")));
        v["version"] = 999.into();
        fs::write(&path, v.to_string()).unwrap();
        let original = fs::read(&path).unwrap();
        assert!(load(&path).is_err());
        assert!(save(&path, &settings).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        // This test exclusively owns this freshly-created directory.
        fs::remove_dir_all(directory).unwrap();
    }
}
