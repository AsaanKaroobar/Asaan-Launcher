//! Start-menu and packaged app identities from the Windows StartApps catalog.
use crate::{Application, Target};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Entry {
    name: String,
    #[serde(rename = "AppID")]
    app_id: String,
}
pub fn parse(input: &str) -> Result<Vec<Application>, String> {
    if input.trim().is_empty() {
        return Ok(vec![]);
    }
    let data: serde_json::Value =
        serde_json::from_str(input.trim_start_matches('\u{feff}')).map_err(|e| e.to_string())?;
    let entries: Vec<Entry> = if data.is_null() {
        Ok(vec![])
    } else if data.is_array() {
        serde_json::from_value(data)
    } else {
        serde_json::from_value(data).map(|e| vec![e])
    }
    .map_err(|e| e.to_string())?;
    let mut seen = std::collections::HashSet::new();
    Ok(entries
        .into_iter()
        .filter(|e| {
            !e.name.trim().is_empty()
                && !e.app_id.is_empty()
                && !e.app_id.chars().any(char::is_control)
                && seen.insert(e.app_id.to_lowercase())
        })
        .map(|e| Application {
            id: format!("app:windows:{}", e.app_id.to_lowercase()),
            name: e.name,
            location: e.app_id.clone().into(),
            identity: Some(e.app_id.clone()),
            origin: None,
            keywords: vec![],
            target: Target::Windows(e.app_id),
        })
        .collect())
}
#[cfg(target_os = "windows")]
pub fn scan() -> Result<Vec<Application>, String> {
    use base64::Engine;
    use std::os::windows::process::CommandExt;
    let script = "$ErrorActionPreference='Stop'; [Console]::OutputEncoding=[System.Text.UTF8Encoding]::new(); @(Get-StartApps | Select-Object Name,AppID) | ConvertTo-Json -Compress";
    let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let mut child = std::process::Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-EncodedCommand",
            &base64::engine::general_purpose::STANDARD.encode(bytes),
        ])
        .creation_flags(0x08000000)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    use std::io::Read;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let read_out = std::thread::spawn(move || {
        let mut bytes = vec![];
        stdout
            .take(16 * 1024 * 1024)
            .read_to_end(&mut bytes)
            .map(|_| bytes)
    });
    let read_err = std::thread::spawn(move || {
        let mut bytes = vec![];
        stderr
            .take(64 * 1024)
            .read_to_end(&mut bytes)
            .map(|_| bytes)
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            break status;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Windows application discovery timed out".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    };
    let stdout = read_out
        .join()
        .map_err(|_| "Windows catalog output worker failed")?
        .map_err(|e| e.to_string())?;
    let stderr = read_err
        .join()
        .map_err(|_| "Windows catalog error worker failed")?
        .map_err(|e| e.to_string())?;

    if !status.success() {
        return Err(String::from_utf8_lossy(&stderr).into_owned());
    }
    parse(&String::from_utf8_lossy(&stdout))
}
