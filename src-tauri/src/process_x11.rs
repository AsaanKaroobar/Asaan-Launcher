//! X11 equivalent: verified local windows, ICCCM iconify and WM_DELETE_WINDOW only.
//! X11 metadata is cooperative, not a security boundary against hostile clients.
use super::process_actions::{Action, Running};
use asaaan_apps::Application;
use gio::prelude::*;
use std::{collections::HashSet, os::unix::fs::MetadataExt, path::PathBuf};
use x11rb::{
    connection::Connection,
    protocol::xproto::{AtomEnum, ClientMessageEvent, ConnectionExt, EventMask},
    rust_connection::RustConnection,
};
type Error = Box<dyn std::error::Error>;
struct X {
    c: RustConnection,
    root: u32,
}
impl X {
    fn atom(&self, name: &str) -> Result<u32, Error> {
        Ok(self.c.intern_atom(false, name.as_bytes())?.reply()?.atom)
    }
    fn words(&self, w: u32, name: &str, kind: AtomEnum) -> Result<Vec<u32>, Error> {
        let p = self
            .c
            .get_property(false, w, self.atom(name)?, kind, 0, 4096)?
            .reply()?;
        if p.bytes_after != 0 {
            return Err("Oversized native window property".into());
        }
        Ok(p.value32().map(|v| v.collect()).unwrap_or_default())
    }
    fn text(&self, w: u32, name: &str) -> Result<String, Error> {
        let p = self
            .c
            .get_property(false, w, self.atom(name)?, AtomEnum::ANY, 0, 1024)?
            .reply()?;
        if p.bytes_after != 0 || p.format != 8 {
            return Ok(String::new());
        }
        Ok(String::from_utf8(p.value)?
            .trim_end_matches('\0')
            .to_owned())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Identity {
    pid: u32,
    executable: PathBuf,
    start: String,
}
fn process(pid: u32) -> Option<Identity> {
    if protected(pid) {
        return None;
    }
    let base = PathBuf::from(format!("/proc/{pid}"));
    if std::fs::metadata(&base).ok()?.uid() != std::fs::metadata("/proc/self").ok()?.uid() {
        return None;
    }
    let stat = std::fs::read_to_string(base.join("stat")).ok()?;
    let tail = stat.rsplit_once(')')?.1;
    let start = tail.split_whitespace().nth(19)?.to_owned();
    Some(Identity {
        pid,
        executable: std::fs::read_link(base.join("exe")).ok()?,
        start,
    })
}
fn entry_matches(value: &str, id: &str) -> bool {
    !value.is_empty() && (value == id || id.strip_suffix(".desktop") == Some(value))
}
fn executable(info: &gio::DesktopAppInfo) -> Option<PathBuf> {
    let exe = info.executable();
    if exe.is_absolute() {
        return std::fs::canonicalize(exe).ok();
    }
    gio::glib::find_program_in_path(exe).and_then(|p| std::fs::canonicalize(p).ok())
}
pub fn perform(
    app: &Application,
    action: Option<Action>,
    expected_token: Option<&str>,
) -> Result<Option<Running>, String> {
    perform_inner(app, action, expected_token).map_err(|e| {
        if e.downcast_ref::<x11rb::errors::ReplyError>().is_some() {
            "Running-application state could not be verified. Reopen its action menu.".into()
        } else {
            e.to_string()
        }
    })
}
fn perform_inner(
    app: &Application,
    action: Option<Action>,
    expected_token: Option<&str>,
) -> Result<Option<Running>, Error> {
    // XWayland sees only some windows, so cannot provide whole-app controls.
    if std::env::var_os("WAYLAND_DISPLAY").is_some_and(|v| !v.is_empty())
        || std::env::var("XDG_SESSION_TYPE").is_ok_and(|v| v == "wayland")
    {
        return Err("Running-app controls are unavailable in this Wayland session".into());
    }
    let info = asaaan_apps::linux::desktop(app)?;
    let entry = std::fs::read(&app.location)?;
    let id = app
        .identity
        .as_deref()
        .ok_or("Application desktop identity unavailable")?;
    let expected = executable(&info).ok_or("Application executable identity unavailable")?;
    let (c, screen) = x11rb::connect(None).map_err(|_| "X11 application service unavailable")?;
    let x = X {
        root: c.setup().roots[screen].root,
        c,
    };
    let clients = x.words(x.root, "_NET_CLIENT_LIST", AtomEnum::WINDOW)?;
    if clients.len() > 4096 {
        return Err("Native window catalog too large".into());
    }
    let hostname = std::fs::read_to_string("/proc/sys/kernel/hostname")?
        .trim()
        .to_owned();
    let mut windows = vec![];
    let mut seen = HashSet::new();
    let hidden = x.atom("_NET_WM_STATE_HIDDEN")?;
    let delete = x.atom("WM_DELETE_WINDOW")?;
    for w in clients {
        if !seen.insert(w) {
            continue;
        }
        let candidate = (|| -> Result<Option<(u32, Identity, bool, bool)>, Error> {
            let gtk = x.text(w, "_GTK_APPLICATION_ID")?;
            let kde = x.text(w, "_KDE_NET_WM_DESKTOP_FILE")?;
            if !entry_matches(&gtk, id) && !entry_matches(&kde, id) {
                return Ok(None);
            }
            if (!gtk.is_empty() && !entry_matches(&gtk, id))
                || (!kde.is_empty() && !entry_matches(&kde, id))
            {
                return Err("Conflicting native application identity".into());
            }
            if x.text(w, "WM_CLIENT_MACHINE")? != hostname {
                return Err(
                    "Application is remote or its local identity cannot be verified".into(),
                );
            }
            let pids = x.words(w, "_NET_WM_PID", AtomEnum::CARDINAL)?;
            if pids.len() != 1 {
                return Err("Application process identity unavailable".into());
            }
            let identity = process(pids[0]).ok_or("Application process is unavailable")?;
            if identity.executable != expected {
                return Err("Application executable does not match its desktop entry".into());
            }
            let states = x.words(w, "_NET_WM_STATE", AtomEnum::ATOM)?;
            let protocols = x.words(w, "WM_PROTOCOLS", AtomEnum::ATOM)?;
            Ok(Some((
                w,
                identity,
                states.contains(&hidden),
                protocols.contains(&delete),
            )))
        })();
        match candidate {
            Ok(Some(w)) => windows.push(w),
            Ok(None) => {}
            Err(e) if vanished(&e) => continue,
            Err(e) => return Err(e),
        }
    }
    if windows.is_empty() {
        return if action.is_none() {
            Ok(None)
        } else {
            Err("Application is no longer running or its windows cannot be verified".into())
        };
    }
    windows.sort_by_key(|w| w.0);
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    for (w, identity, _, _) in &windows {
        w.hash(&mut hash);
        identity.hash(&mut hash);
    }
    entry.hash(&mut hash);
    let token = format!("x11:{:016x}", hash.finish());
    let mut held = vec![];
    let mut pids = HashSet::new();
    let mut stable = true;
    for (_, identity, _, _) in &windows {
        if pids.insert(identity.pid) {
            match hold(identity) {
                Ok(p) => held.push(p),
                Err(_) => stable = false,
            }
        }
    }
    let running = Running {
        hidden: windows.iter().all(|w| w.2),
        can_quit: windows.iter().all(|w| w.3),
        can_restart: stable && windows.iter().all(|w| w.3),
        can_force: stable,
        token,
    };
    if let Some(action) = action {
        if expected_token != Some(running.token.as_str()) {
            return Err("Application instance changed; reopen its action menu".into());
        }
        if matches!(action, Action::Quit | Action::Restart) && !running.can_quit {
            return Err("Application does not support graceful close requests".into());
        }
        if action == Action::Hide && running.hidden {
            return Err("Application windows are already hidden".into());
        }
        if action == Action::Restart && !running.can_restart {
            return Err("Restart requires stable process handles".into());
        }
        if action == Action::ForceQuit && !running.can_force {
            return Err("Force Quit requires stable process handles".into());
        }
        for (w, identity, _, _) in &windows {
            if process(identity.pid).as_ref() != Some(identity)
                || x.words(*w, "_NET_WM_PID", AtomEnum::CARDINAL)? != vec![identity.pid]
                || (!entry_matches(&x.text(*w, "_GTK_APPLICATION_ID")?, id)
                    && !entry_matches(&x.text(*w, "_KDE_NET_WM_DESKTOP_FILE")?, id))
            {
                return Err("Application changed; reopen its action menu".into());
            }
        }
        if action == Action::ForceQuit {
            use std::os::fd::AsRawFd;
            for p in &held {
                if process(p.identity.pid).as_ref() != Some(&p.identity) {
                    return Err("Application instance changed".into());
                }
            }
            for p in &held {
                if unsafe {
                    libc::syscall(
                        libc::SYS_pidfd_send_signal,
                        p.fd.as_raw_fd(),
                        libc::SIGKILL,
                        std::ptr::null::<libc::siginfo_t>(),
                        0,
                    )
                } != 0
                {
                    return Err("Force Quit was refused; check the app before retrying".into());
                }
            }
            crate::process_wait::wait_for_exit(|| all_exited(&held))
                .map_err(|_| "Could not confirm process exit after Force Quit".to_string())?;
            return Ok(Some(running));
        }
        for (w, _, is_hidden, _) in windows {
            match action {
                Action::Hide if !is_hidden => {
                    let event =
                        ClientMessageEvent::new(32, w, x.atom("WM_CHANGE_STATE")?, [3, 0, 0, 0, 0]);
                    x.c.send_event(
                        false,
                        x.root,
                        EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
                        event,
                    )?
                    .check()?;
                }
                Action::Quit | Action::Restart => {
                    let event = ClientMessageEvent::new(
                        32,
                        w,
                        x.atom("WM_PROTOCOLS")?,
                        [delete, x11rb::CURRENT_TIME, 0, 0, 0],
                    );
                    x.c.send_event(false, w, EventMask::NO_EVENT, event)?
                        .check()?;
                }
                _ => {}
            }
        }
        x.c.flush()?;
        if action == Action::Restart {
            crate::process_wait::wait_for_exit(|| all_exited(&held))?;
            if std::fs::read(&app.location)? != entry {
                return Err("Application desktop entry changed; it was not restarted".into());
            }
            if perform(app, None, None)?.is_some() {
                return Err("Another instance appeared; it was not restarted".into());
            }
            asaaan_apps::launch(app)?;
        }
    }
    Ok(Some(running))
}

fn vanished(error: &Error) -> bool {
    matches!(error.downcast_ref::<x11rb::errors::ReplyError>(),Some(x11rb::errors::ReplyError::X11Error(e)) if e.error_kind==x11rb::protocol::ErrorKind::Window)
}
struct Held {
    identity: Identity,
    fd: std::os::fd::OwnedFd,
}
fn protected(pid: u32) -> bool {
    let mut current = std::process::id();
    let mut seen = HashSet::new();
    while current > 1 && seen.insert(current) {
        if current == pid {
            return true;
        }
        let Some(parent) = std::fs::read_to_string(format!("/proc/{current}/stat"))
            .ok()
            .and_then(|s| s.rsplit_once(')').map(|(_, v)| v.to_owned()))
            .and_then(|s| {
                s.split_whitespace()
                    .nth(1)
                    .and_then(|p| p.parse::<u32>().ok())
            })
        else {
            return true;
        };
        current = parent;
    }
    pid <= 1
}
fn exited(fd: &std::os::fd::OwnedFd) -> Result<bool, String> {
    use std::os::fd::AsRawFd;
    let mut p = libc::pollfd {
        fd: fd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    let result = unsafe { libc::poll(&mut p, 1, 0) };
    if result < 0 || p.revents & (libc::POLLERR | libc::POLLNVAL) != 0 {
        return Err("Process exit state unavailable".into());
    }
    Ok(p.revents & libc::POLLIN != 0)
}
fn hold(identity: &Identity) -> Result<Held, String> {
    use std::os::fd::{AsRawFd, FromRawFd};
    let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, identity.pid, 0) } as i32;
    if raw < 0 {
        return Err("Stable process handles are unavailable on this system".into());
    }
    let fd = unsafe { std::os::fd::OwnedFd::from_raw_fd(raw) };
    if process(identity.pid).as_ref() != Some(identity) || exited(&fd)? {
        return Err("Application instance changed".into());
    }
    if unsafe {
        libc::syscall(
            libc::SYS_pidfd_send_signal,
            fd.as_raw_fd(),
            0,
            std::ptr::null::<libc::siginfo_t>(),
            0,
        )
    } != 0
    {
        return Err("Application process cannot be controlled".into());
    }
    Ok(Held {
        identity: identity.clone(),
        fd,
    })
}
fn all_exited(held: &[Held]) -> Result<bool, String> {
    for p in held {
        if !exited(&p.fd)? {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn desktop_identity_is_exact_not_a_name_or_suffix_guess() {
        assert!(entry_matches(
            "org.example.Notes",
            "org.example.Notes.desktop"
        ));
        assert!(entry_matches(
            "org.example.Notes.desktop",
            "org.example.Notes.desktop"
        ));
        for v in [
            "",
            "Notes",
            "notes",
            "/apps/org.example.Notes.desktop",
            "org.example.Notes.evil",
        ] {
            assert!(!entry_matches(v, "org.example.Notes.desktop"))
        }
    }
    #[test]
    fn self_missing_and_system_processes_are_rejected() {
        for pid in [0, 1, std::process::id(), u32::MAX] {
            assert!(process(pid).is_none())
        }
    }
    #[test]
    fn stable_process_handles_report_exit_and_do_not_retarget_by_pid() {
        let mut child = std::process::Command::new("/bin/sleep")
            .arg("5")
            .spawn()
            .unwrap();
        let identity = process(child.id()).unwrap();
        let held = hold(&identity).unwrap();
        assert!(!exited(&held.fd).unwrap());
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(exited(&held.fd).unwrap());
        assert!(process(held.identity.pid).is_none());
    }
    #[test]
    fn ancestor_processes_are_never_selected() {
        let parent = unsafe { libc::getppid() } as u32;
        assert!(protected(parent));
        assert!(process(parent).is_none());
    }
    #[test]
    fn only_a_vanished_window_is_skipped_during_enumeration() {
        use x11rb::{errors::ReplyError, protocol::ErrorKind, x11_utils::X11Error};
        let error = |kind| -> Error {
            Box::new(ReplyError::X11Error(X11Error {
                error_kind: kind,
                error_code: 3,
                sequence: 1,
                bad_value: 42,
                minor_opcode: 0,
                major_opcode: 20,
                extension_name: None,
                request_name: Some("GetProperty"),
            }))
        };
        assert!(vanished(&error(ErrorKind::Window)));
        assert!(!vanished(&error(ErrorKind::Access)));
        let unavailable: Error = "Application identity unavailable".into();
        assert!(!vanished(&unavailable));
    }
}
