//! Native user confirmation; the public force command cannot supply a confirmation boolean.
#[cfg(target_os = "linux")]
pub fn confirm(app: &tauri::AppHandle, name: &str) -> Result<bool, String> {
    use gtk::prelude::*;
    let (tx, rx) = std::sync::mpsc::channel();
    let title = format!("Force “{name}” to quit?");
    app.run_on_main_thread(move || {
        let dialog = gtk::MessageDialog::builder()
            .title("Asaaan Launcher — Force Quit")
            .message_type(gtk::MessageType::Warning)
            .text(title)
            .secondary_text("You will lose any changes you haven't saved.")
            .modal(true)
            .build();
        dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        dialog.add_button("Force Quit", gtk::ResponseType::Accept);
        dialog.set_default_response(gtk::ResponseType::Cancel);
        let sender = std::cell::RefCell::new(Some(tx));
        dialog.connect_response(move |dialog, response| {
            let sender = sender.borrow_mut().take();
            if let Some(tx) = sender {
                dialog.hide();
                dialog.close();
                let _ = tx.send(response == gtk::ResponseType::Accept);
            }
        });
        dialog.present();
    })
    .map_err(|_| "Native confirmation could not be displayed")?;
    rx.recv()
        .map_err(|_| "Native confirmation was closed without a response".into())
}
#[cfg(target_os = "windows")]
pub fn confirm(_app: &tauri::AppHandle, name: &str) -> Result<bool, String> {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(
            window: *mut std::ffi::c_void,
            text: *const u16,
            title: *const u16,
            flags: u32,
        ) -> i32;
    }
    let text=format!("Force “{name}” to quit?\nYou will lose any changes you haven't saved.\nChoose Yes to force quit.").encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let title = "Asaaan Launcher — Force Quit"
        .encode_utf16()
        .chain(Some(0))
        .collect::<Vec<_>>();
    // Yes/No, warning, default No, foreground. No is the cancel equivalent.
    match unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            title.as_ptr(),
            0x4 | 0x30 | 0x100 | 0x10000 | 0x40000,
        )
    } {
        6 => Ok(true),
        7 => Ok(false),
        _ => Err("Native confirmation could not be displayed".into()),
    }
}
#[cfg(target_os = "macos")]
pub fn confirm(_app: &tauri::AppHandle, name: &str) -> Result<bool, String> {
    // User confirmation has no automatic deadline; waiting is not approval.
    let output = std::process::Command::new("/usr/bin/osascript")
        .args([
            "-l",
            "JavaScript",
            "-e",
            include_str!("force_confirmation_macos.js"),
        ])
        .arg(name)
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|_| "Native confirmation unavailable")?;
    if !output.status.success() {
        return Err("Native confirmation failed".into());
    }
    match output.stdout.as_slice() {
        b"yes\n" => Ok(true),
        b"no\n" => Ok(false),
        _ => Err("Native confirmation response invalid".into()),
    }
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn confirm(_app: &tauri::AppHandle, _name: &str) -> Result<bool, String> {
    Err("Native confirmation unavailable".into())
}
