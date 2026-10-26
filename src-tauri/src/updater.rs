//! Resident-owned update checks.
//!
//! The updater runs in the windowless resident so the panel, Settings and
//! extension sessions can be stopped before an install. Updates are never
//! installed silently: startup checks and the tray item both ask first.
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::{Update, UpdaterExt};

const STARTUP_DELAY: Duration = Duration::from_secs(15);
static CHECKING: AtomicBool = AtomicBool::new(false);

pub fn schedule_startup_check(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(STARTUP_DELAY);
        check(&app, false);
    });
}

pub fn check(app: &AppHandle, manual: bool) {
    if CHECKING.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let updater = match app.updater() {
            Ok(updater) => updater,
            Err(error) => {
                CHECKING.store(false, Ordering::SeqCst);
                if manual {
                    message(
                        &app,
                        MessageDialogKind::Error,
                        "Updates unavailable",
                        format!("The updater is not configured: {error}"),
                    );
                }
                return;
            }
        };
        let result = updater.check().await;
        CHECKING.store(false, Ordering::SeqCst);
        match result {
            Ok(Some(update)) => prompt(app, update),
            Ok(None) => {
                if manual {
                    message(
                        &app,
                        MessageDialogKind::Info,
                        "No updates",
                        "Asaaan Launcher is up to date.",
                    );
                }
            }
            Err(error) => {
                eprintln!("ASAAAN_UPDATE check failed: {error}");
                if manual {
                    message(
                        &app,
                        MessageDialogKind::Error,
                        "Update check failed",
                        format!("Could not check for updates. {error}"),
                    );
                }
            }
        }
    });
}

fn prompt(app: AppHandle, update: Update) {
    let version = update.version.clone();
    let current = update.current_version.clone();
    app.dialog()
        .message(format!(
            "Asaaan Launcher {version} is available (current version {current}). Install it now?"
        ))
        .title("Update available")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Install".into(),
            "Later".into(),
        ))
        .show(move |install| {
            if install {
                install_update(app, update);
            }
        });
}

fn install_update(app: AppHandle, update: Update) {
    #[cfg(target_os = "linux")]
    if std::env::var_os("APPIMAGE").is_none() {
        message(
            &app,
            MessageDialogKind::Info,
            "Manual update required",
            "Automatic updates are available in the AppImage build. Download the latest .deb from the release page and install it manually.",
        );
        return;
    }
    tauri::async_runtime::spawn(async move {
        let bytes = match update.download(|_, _| {}, || {}).await {
            Ok(bytes) => bytes,
            Err(error) => {
                message(
                    &app,
                    MessageDialogKind::Error,
                    "Update failed",
                    format!("The update could not be downloaded: {error}"),
                );
                return;
            }
        };
        // Stop owned children before replacing the running executable.
        if let Some(extensions) = app.try_state::<crate::extension_service::Service>() {
            extensions.stop();
        }
        if let Some(settings) = app.try_state::<crate::settings::SettingsProcess>() {
            settings.stop();
        }
        if let Some(resident) = app.try_state::<crate::resident::Resident>() {
            resident.stop();
        }
        if let Err(error) = update.install(&bytes) {
            message(
                &app,
                MessageDialogKind::Error,
                "Update failed",
                format!("The update could not be installed: {error}"),
            );
            return;
        }
        app.restart();
    });
}

fn message(
    app: &AppHandle,
    kind: MessageDialogKind,
    title: impl Into<String>,
    body: impl Into<String>,
) {
    app.dialog()
        .message(body)
        .title(title)
        .kind(kind)
        .show(|_| {});
}
