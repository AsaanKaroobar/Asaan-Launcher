//! The resident, rather than its disposable foreground, owns clipboard writes.
use asaaan_lifecycle::clipboard as contract;
pub fn write(app: &tauri::AppHandle, text: String, deadline: u64) -> Result<(), String> {
    let remaining = contract::validate(&text, deadline, contract::now())?;
    let expires = std::time::Instant::now() + remaining;
    #[cfg(target_os = "linux")]
    {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        app.run_on_main_thread(move || {
            let result = if std::time::Instant::now() >= expires {
                Err("Clipboard request expired".into())
            } else {
                gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD).set_text(&text);
                Ok(())
            };
            let _ = tx.send(result);
        })
        .map_err(|e| e.to_string())?;
        rx.recv_timeout(remaining).map_err(|_| {
            "Clipboard acknowledgement timed out; check the clipboard before retrying".to_string()
        })?
    }
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        let mut command = std::process::Command::new("/usr/bin/pbcopy");
        command.env("LC_CTYPE", "UTF-8");
        crate::application_actions::run_bounded(command, Some(text), expires)
    }
    #[cfg(target_os = "windows")]
    {
        use base64::Engine;
        use std::os::windows::process::CommandExt;
        let _ = app;
        let script = "$ErrorActionPreference='Stop'; [Console]::InputEncoding=[System.Text.UTF8Encoding]::new($false); Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.Clipboard]::SetText([Console]::In.ReadToEnd())";
        let encoded = base64::engine::general_purpose::STANDARD.encode(
            script
                .encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>(),
        );
        let mut command = std::process::Command::new("powershell.exe");
        command
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-STA",
                "-EncodedCommand",
                &encoded,
            ])
            .creation_flags(0x08000000);
        crate::application_actions::run_bounded(command, Some(text), expires)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = (app, text, expires);
        Err("Clipboard is unavailable on this platform".into())
    }
}
