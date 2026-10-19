//! GIO owns XDG precedence, localized metadata, visibility and Exec field-code handling.
use crate::{Application, Target};
use gio::prelude::*;
pub fn scan() -> Result<Vec<Application>, String> {
    Ok(gio::AppInfo::all()
        .into_iter()
        .filter(|a| a.should_show())
        .filter_map(|a| {
            let desktop = a.downcast::<gio::DesktopAppInfo>().ok()?;
            let id = desktop.id()?.to_string();
            let path = desktop.filename()?;
            Some(Application {
                id: format!("app:desktop:{id}"),
                name: desktop.display_name().to_string(),
                location: path.clone(),
                identity: Some(id),
                origin: None,
                keywords: desktop.keywords().iter().map(|s| s.to_string()).collect(),
                target: Target::Desktop(path),
            })
        })
        .collect())
}
pub fn desktop(app: &Application) -> Result<gio::DesktopAppInfo, String> {
    let Target::Desktop(path) = &app.target else {
        return Err("Invalid desktop application target".into());
    };
    if !path.is_file() {
        return Err("Application was removed. Refresh the catalog.".into());
    }
    let info =
        gio::DesktopAppInfo::from_filename(path).ok_or("Application desktop entry is invalid")?;
    if !info.should_show() {
        return Err("Application is no longer available on this desktop".into());
    }
    Ok(info)
}
pub fn launch(app: &Application) -> Result<(), String> {
    desktop(app)?
        .launch_uris_as_manager(
            &[],
            None::<&gio::AppLaunchContext>,
            gio::glib::SpawnFlags::SEARCH_PATH
                | gio::glib::SpawnFlags::STDOUT_TO_DEV_NULL
                | gio::glib::SpawnFlags::STDERR_TO_DEV_NULL,
            None,
            None,
        )
        .map_err(|e| e.to_string())
}
