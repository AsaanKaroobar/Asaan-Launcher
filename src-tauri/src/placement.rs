//! Place before activation, so the launcher cannot select its own old monitor.
use tauri::Manager;

fn centered(origin: i32, available: u32, extent: u32) -> i32 {
    (i64::from(origin) + (i64::from(available) - i64::from(extent)).max(0) / 2)
        .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn contains(origin: (f64, f64), size: (f64, f64), point: (f64, f64)) -> bool {
    point.0 >= origin.0
        && point.1 >= origin.1
        && point.0 < origin.0 + size.0
        && point.1 < origin.1 + size.1
}
fn monitor_at(
    monitors: &[tauri::Monitor],
    point: (f64, f64),
    logical: bool,
) -> Option<tauri::Monitor> {
    monitors
        .iter()
        .find(|monitor| {
            let scale = if logical { monitor.scale_factor() } else { 1.0 };
            let p = monitor.position();
            let s = monitor.size();
            contains(
                (f64::from(p.x) / scale, f64::from(p.y) / scale),
                (f64::from(s.width) / scale, f64::from(s.height) / scale),
                point,
            )
        })
        .cloned()
}

pub fn center_on_focused_monitor(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    // Wayland has no portable client placement. Hyprland is settled through
    // its own adapter after the map request; other compositors decide.
    if crate::windowing::wayland_session() {
        return Ok(());
    }
    let app = window.app_handle();
    let monitors = app.available_monitors()?;
    let focused =
        foreground_center().and_then(|p| monitor_at(&monitors, p, cfg!(target_os = "macos")));
    let pointer = || {
        let p = app.cursor_position().ok()?;
        // Tao reports the macOS cursor in physical coordinates using the primary scale.
        let scale = if cfg!(target_os = "macos") {
            app.primary_monitor().ok().flatten()?.scale_factor()
        } else {
            1.0
        };
        monitor_at(
            &monitors,
            (p.x / scale, p.y / scale),
            cfg!(target_os = "macos"),
        )
    };
    let monitor = focused
        .or_else(pointer)
        .or_else(|| app.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        return window.center();
    };
    let area = monitor.work_area();
    // Center the content dimensions, matching Tao's normal centering behavior.
    // GTK outer-frame metrics can still describe the initial 640x480 surface
    // before the first map, even when the requested content size is larger.
    let extent = window.inner_size()?;
    let logical = extent.to_logical::<f64>(window.scale_factor()?);
    let target = logical.to_physical::<u32>(monitor.scale_factor());
    let x = centered(area.position.x, area.size.width, target.width);
    let y = centered(area.position.y, area.size.height, target.height);
    if cfg!(target_os = "macos") {
        // AppKit positions are points. Avoid converting through the old screen's scale.
        window.set_position(tauri::LogicalPosition::new(
            f64::from(x) / monitor.scale_factor(),
            f64::from(y) / monitor.scale_factor(),
        ))
    } else {
        window.set_position(tauri::PhysicalPosition::new(x, y))
    }
}

#[cfg(target_os = "linux")]
fn foreground_center() -> Option<(f64, f64)> {
    use x11rb::{
        connection::Connection,
        protocol::xproto::{AtomEnum, ConnectionExt},
    };
    // Wayland does not expose another client's focused surface geometry.
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        return None;
    }
    let (connection, screen) = x11rb::connect(None).ok()?;
    let root = connection.setup().roots[screen].root;
    let atom = connection
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")
        .ok()?
        .reply()
        .ok()?
        .atom;
    let property = connection
        .get_property(false, root, atom, AtomEnum::WINDOW, 0, 1)
        .ok()?
        .reply()
        .ok()?;
    let id = property.value32()?.next()?;
    if id == 0 || id == root {
        return None;
    }
    let geometry = connection.get_geometry(id).ok()?.reply().ok()?;
    let origin = connection
        .translate_coordinates(id, root, 0, 0)
        .ok()?
        .reply()
        .ok()?;
    Some((
        f64::from(origin.dst_x) + f64::from(geometry.width) / 2.0,
        f64::from(origin.dst_y) + f64::from(geometry.height) / 2.0,
    ))
}

#[cfg(target_os = "windows")]
fn foreground_center() -> Option<(f64, f64)> {
    #[repr(C)]
    #[derive(Default)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetForegroundWindow() -> *mut std::ffi::c_void;
        fn GetWindowRect(window: *mut std::ffi::c_void, rect: *mut Rect) -> i32;
    }
    let mut rect = Rect::default();
    // SAFETY: the handle is borrowed from user32; rect is valid writable storage.
    unsafe {
        let handle = GetForegroundWindow();
        if handle.is_null() || GetWindowRect(handle, &mut rect) == 0 {
            return None;
        }
    }
    Some((
        (f64::from(rect.left) + f64::from(rect.right)) / 2.0,
        (f64::from(rect.top) + f64::from(rect.bottom)) / 2.0,
    ))
}

#[cfg(target_os = "macos")]
fn foreground_center() -> Option<(f64, f64)> {
    use std::ffi::c_void;
    type Ref = *const c_void;
    #[repr(C)]
    #[derive(Default)]
    struct Point {
        x: f64,
        y: f64,
    }
    #[repr(C)]
    #[derive(Default)]
    struct Size {
        width: f64,
        height: f64,
    }
    #[repr(C)]
    #[derive(Default)]
    struct Rect {
        origin: Point,
        size: Size,
    }
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGWindowListCopyWindowInfo(options: u32, relative: u32) -> Ref;
        fn CGRectMakeWithDictionaryRepresentation(dict: Ref, rect: *mut Rect) -> bool;
        static kCGWindowLayer: Ref;
        static kCGWindowOwnerPID: Ref;
        static kCGWindowBounds: Ref;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFArrayGetCount(array: Ref) -> isize;
        fn CFArrayGetValueAtIndex(array: Ref, index: isize) -> Ref;
        fn CFDictionaryGetValue(dict: Ref, key: Ref) -> Ref;
        fn CFNumberGetValue(number: Ref, kind: isize, value: *mut c_void) -> bool;
        fn CFRelease(value: Ref);
    }
    // Metadata only, no window titles or screen capture and no permission prompt.
    // CoreGraphics returns front-to-back order in top-left desktop coordinates,
    // the same point space Tao uses for monitor_from_point on macOS.
    unsafe {
        let list = CGWindowListCopyWindowInfo(1 | 16, 0);
        if list.is_null() {
            return None;
        }
        let mut result = None;
        for index in 0..CFArrayGetCount(list) {
            let info = CFArrayGetValueAtIndex(list, index);
            let layer = CFDictionaryGetValue(info, kCGWindowLayer);
            let owner = CFDictionaryGetValue(info, kCGWindowOwnerPID);
            let bounds = CFDictionaryGetValue(info, kCGWindowBounds);
            if layer.is_null() || owner.is_null() || bounds.is_null() {
                continue;
            }
            let mut level = 0i32;
            let mut pid = 0i32;
            let mut rect = Rect::default();
            if CFNumberGetValue(layer, 3, (&mut level as *mut i32).cast())
                && CFNumberGetValue(owner, 3, (&mut pid as *mut i32).cast())
                && level == 0
                && pid as u32 != std::process::id()
                && CGRectMakeWithDictionaryRepresentation(bounds, &mut rect)
                && rect.size.width > 0.0
                && rect.size.height > 0.0
            {
                result = Some((
                    rect.origin.x + rect.size.width / 2.0,
                    rect.origin.y + rect.size.height / 2.0,
                ));
                break;
            }
        }
        CFRelease(list);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{centered, contains};
    #[test]
    fn monitor_bounds_handle_negative_origins_edges_and_scaled_extents() {
        assert!(contains((-1920.0, 0.0), (1920.0, 1080.0), (-100.0, 500.0)));
        assert!(!contains((-1920.0, 0.0), (1920.0, 1080.0), (0.0, 500.0)));
        assert!(contains((1920.0, 0.0), (3840.0, 2160.0), (4000.0, 1500.0)));
        assert!(!contains((1920.0, 0.0), (3840.0, 2160.0), (5760.0, 500.0)));
    }
    #[test]
    fn centers_on_primary_and_negative_origin_monitors() {
        assert_eq!(centered(0, 1920, 750), 585);
        assert_eq!(centered(-1920, 1920, 750), -1335);
        assert_eq!(centered(-1080, 1080, 474), -777);
    }
    #[test]
    fn respects_work_area_and_scaled_window_extent() {
        assert_eq!(centered(40, 2160, 948), 646);
        assert_eq!(centered(3840, 3840, 1500), 5010);
    }
    #[test]
    fn oversized_windows_anchor_to_the_usable_origin_without_underflow() {
        assert_eq!(centered(-800, 640, 750), -800);
        assert_eq!(centered(i32::MAX, 4000, 1), i32::MAX);
    }
}
