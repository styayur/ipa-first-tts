// SPDX-License-Identifier: MPL-2.0
//! Optional GPL eSpeak executable boundary. No shell, network or speech model.
use crate::G2pProvider;
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
pub struct EspeakG2p {
    pub executable: PathBuf,
}
impl EspeakG2p {
    pub fn discover() -> Self {
        let executable = std::env::var_os("ESPEAK_NG")
            .map(PathBuf::from)
            .or_else(|| {
                let here = PathBuf::from("tools/espeak-ng/espeak-ng.exe");
                if here.is_file() {
                    return Some(here);
                }
                std::env::current_exe()
                    .ok()
                    .and_then(|e| e.parent().map(|p| p.join("tools/espeak-ng/espeak-ng.exe")))
                    .filter(|p| p.is_file())
            })
            .unwrap_or_else(|| {
                PathBuf::from(if cfg!(windows) {
                    "espeak-ng.exe"
                } else {
                    "espeak-ng"
                })
            });
        Self { executable }
    }
    fn resolve(&self) -> Option<PathBuf> {
        if self.executable.is_file() {
            return self.executable.canonicalize().ok();
        }
        if self.executable.components().count() != 1 {
            return None;
        }
        std::env::var_os("PATH").and_then(|v| {
            std::env::split_paths(&v)
                .map(|p| p.join(&self.executable))
                .find(|p| p.is_file())
        })
    }
}
impl G2pProvider for EspeakG2p {
    fn available(&self) -> bool {
        self.resolve().is_some()
    }
    fn provenance_warning(&self) -> String {
        "Adapter and release voice data are pinned to eSpeak NG 1.52.0. Custom ESPEAK_NG/PATH executables are user overrides with unverified runtime version; no scheme fallback occurs.".into()
    }
    fn generate(&self, input: &str, language: &str) -> Result<String, String> {
        let voice = match language {
            "en-US" => "en-us",
            "fr" => "fr-fr",
            "es" => "es",
            _ => return Err("No eSpeak mapping voice registered for this language".into()),
        };
        if input.is_empty() || input.len() > 4096 || input.contains('\0') {
            return Err("Invalid G2P input size/NUL".into());
        }
        if let Some((index, c)) = input.chars().enumerate().find(|(_, c)| {
            !(c.is_ascii_alphanumeric()
                || c.is_ascii_whitespace()
                || "'’\"“”.,!?;:()-".contains(*c)
                || ('\u{00c0}'..='\u{024f}').contains(c)
                || ('\u{0300}'..='\u{036f}').contains(c))
        }) {
            return Err(format!("Non-Latin/uncovered input U+{:04X} at scalar {index}; no character was silently dropped",c as u32));
        }
        let executable = self
            .resolve()
            .ok_or("Optional eSpeak NG executable not found")?;
        let mut command = Command::new(&executable);
        if let Some(parent) = executable
            .parent()
            .filter(|p| p.join("espeak-ng-data").is_dir())
        {
            command.arg(format!(
                "--path={}",
                parent.to_string_lossy().trim_start_matches("\\\\?\\")
            ));
        }
        command
            .args(["-q", "--ipa=2", "-v", voice, "-b", "1", "--stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command
            .spawn()
            .map_err(|e| format!("Cannot launch eSpeak NG: {e}"))?;
        let mut stdin = child.stdin.take().expect("piped stdin");
        let mut bytes = input.as_bytes().to_vec();
        bytes.push(b'\n');
        let writer = thread::spawn(move || stdin.write_all(&bytes));
        let oversized = Arc::new(AtomicBool::new(false));
        let stdout = child.stdout.take().expect("piped stdout");
        let flag = oversized.clone();
        let reader = thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = stdout.take(65537).read_to_end(&mut bytes);
            if bytes.len() > 65536 {
                flag.store(true, Ordering::Relaxed);
            }
            result.map(|_| bytes)
        });
        let stderr = child.stderr.take().expect("piped stderr");
        let flag = oversized.clone();
        let errors = thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = stderr.take(16385).read_to_end(&mut bytes);
            if bytes.len() > 16384 {
                flag.store(true, Ordering::Relaxed);
            }
            result.map(|_| bytes)
        });
        let deadline = Instant::now() + Duration::from_secs(8);
        let status = loop {
            if oversized.load(Ordering::Relaxed) || Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                break Err("eSpeak timeout/output limit; child terminated and reaped".to_owned());
            }
            match child.try_wait() {
                Ok(Some(s)) => break Ok(s),
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(format!("eSpeak wait failed: {e}"));
                }
            }
        };
        let written = writer.join().map_err(|_| "G2P writer panicked")?;
        let output = reader
            .join()
            .map_err(|_| "G2P reader panicked")?
            .map_err(|e| e.to_string())?;
        let error = errors
            .join()
            .map_err(|_| "G2P error reader panicked")?
            .map_err(|e| e.to_string())?;
        let status = status?;
        written.map_err(|e| e.to_string())?;
        if !status.success() || oversized.load(Ordering::Relaxed) {
            return Err(format!(
                "eSpeak failed ({status}): {}",
                String::from_utf8_lossy(&error)
            ));
        }
        let output = String::from_utf8(output)
            .map_err(|e| e.to_string())?
            .trim()
            .to_owned();
        if output.is_empty() || output.contains(['(', ')']) {
            return Err(format!("eSpeak returned empty output or a language-switch annotation; original input retained, output not silently cleaned: {output}"));
        }
        Ok(output)
    }
}
