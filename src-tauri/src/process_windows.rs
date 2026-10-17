//! Native Windows AUMID-to-window matching. Never guesses by title/executable name.
use super::process_actions::{Action, Running};
use asaaan_apps::{Application, Target};
use std::ffi::c_void;
type Handle = *mut c_void;
#[link(name = "user32")]
unsafe extern "system" {
    fn EnumWindows(
        callback: Option<unsafe extern "system" fn(Handle, isize) -> i32>,
        data: isize,
    ) -> i32;
    fn GetWindowThreadProcessId(window: Handle, pid: *mut u32) -> u32;
    fn IsWindowVisible(window: Handle) -> i32;
    fn IsIconic(window: Handle) -> i32;
    fn ShowWindowAsync(window: Handle, command: i32) -> i32;
    fn PostMessageW(window: Handle, message: u32, wparam: usize, lparam: isize) -> i32;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> Handle;
    fn CloseHandle(handle: Handle) -> i32;
    fn GetApplicationUserModelId(process: Handle, length: *mut u32, value: *mut u16) -> i32;
    fn GetExitCodeProcess(process: Handle, code: *mut u32) -> i32;
    fn GetProcessTimes(
        process: Handle,
        created: *mut u64,
        exited: *mut u64,
        kernel: *mut u64,
        user: *mut u64,
    ) -> i32;
}
struct Process(Handle);
impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
fn identity(process: &Process) -> Option<String> {
    let mut length = 0;
    if unsafe { GetApplicationUserModelId(process.0, &mut length, std::ptr::null_mut()) } != 122
        || length == 0
        || length > 4096
    {
        return None;
    }
    let mut value = vec![0u16; length as usize];
    if unsafe { GetApplicationUserModelId(process.0, &mut length, value.as_mut_ptr()) } != 0
        || length == 0
        || length as usize > value.len()
    {
        return None;
    }
    String::from_utf16(&value[..length as usize - 1]).ok()
}
struct Window {
    window: Handle,
    pid: u32,
    process: Process,
    hidden: bool,
    created: u64,
}
unsafe extern "system" fn collect(window: Handle, data: isize) -> i32 {
    let windows = unsafe { &mut *(data as *mut Vec<Handle>) };
    if windows.len() >= 4096 {
        return 0;
    }
    if unsafe { IsWindowVisible(window) } != 0 {
        windows.push(window);
    }
    1
}
pub fn perform(
    app: &Application,
    action: Option<Action>,
    expected_token: Option<&str>,
) -> Result<Option<Running>, String> {
    let Target::Windows(expected) = &app.target else {
        return Err("Application has no native Windows identity".into());
    };
    let mut candidates: Vec<Handle> = vec![];
    if unsafe {
        EnumWindows(
            Some(collect),
            (&mut candidates as *mut Vec<Handle>) as isize,
        )
    } == 0
    {
        return Err("Native window catalog unavailable or too large".into());
    }
    let mut windows = vec![];
    for window in candidates {
        let mut pid = 0;
        unsafe {
            GetWindowThreadProcessId(window, &mut pid);
        }
        if pid == 0 || pid == std::process::id() {
            continue;
        }
        let handle = unsafe { OpenProcess(0x1000, 0, pid) };
        if handle.is_null() {
            continue;
        }
        let process = Process(handle);
        if identity(&process).as_ref() != Some(expected) {
            continue;
        }
        let (mut created, mut exited, mut kernel, mut user) = (0, 0, 0, 0);
        if unsafe { GetProcessTimes(process.0, &mut created, &mut exited, &mut kernel, &mut user) }
            == 0
        {
            continue;
        }
        windows.push(Window {
            created,
            window,
            pid,
            hidden: unsafe { IsIconic(window) } != 0,
            process,
        });
    }
    if windows.is_empty() {
        return if action.is_none() {
            Ok(None)
        } else {
            Err("Application is no longer running or its native identity cannot be verified".into())
        };
    }
    windows.sort_by_key(|w| w.window as usize);
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    for w in &windows {
        (w.window as usize, w.pid, w.created).hash(&mut hash);
    }
    let token = format!("win:{:016x}", hash.finish());
    let running = Running {
        hidden: windows.iter().all(|w| w.hidden),
        can_quit: true,
        token,
    };
    if let Some(action) = action {
        if expected_token != Some(running.token.as_str()) {
            return Err("Application instance changed; reopen its action menu".into());
        }
        if action == Action::Hide && running.hidden {
            return Err("Application windows are already hidden".into());
        }
        // Retain process handles across enumeration/revalidation so PID reuse cannot retarget a request.
        for w in &windows {
            let mut pid = 0;
            let mut exit = 0;
            unsafe {
                GetWindowThreadProcessId(w.window, &mut pid);
            }
            if pid != w.pid
                || unsafe { GetExitCodeProcess(w.process.0, &mut exit) } == 0
                || exit != 259
                || identity(&w.process).as_ref() != Some(expected)
            {
                return Err("Application changed; reopen its action menu".into());
            }
        }
        for w in &windows {
            let accepted = match action {
                Action::Hide if !w.hidden => unsafe { ShowWindowAsync(w.window, 6) },
                Action::Hide => 1,
                Action::Quit => unsafe { PostMessageW(w.window, 0x0010, 0, 0) },
            };
            if accepted == 0 {
                return Err(
                    "Native application request was refused; check its windows before retrying"
                        .into(),
                );
            }
        }
    }
    Ok(Some(running))
}
