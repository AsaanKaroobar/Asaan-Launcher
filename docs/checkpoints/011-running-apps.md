# CP011: Graceful running-app controls

Status: implemented, tested, locally installed and locally run. Public downloadable release pending. Version 0.11.0; branch checkpoint-11. The user explicitly authorized advancing after CP010; its public release is unverified and user controlled. Restart, Force Quit, extensions and performance upgrades remain deferred.

## Behavior and reference

Floe's AppActions.running matches NSWorkspace applications by standardized bundle URL. Its processActions offers Hide only while visible, dismisses the launcher after Hide, and acknowledges Quit with “Quitting NAME” without waiting for termination or dismissing explicitly. CP011 implements these scoped semantics. Quit acknowledgement means the native request was sent, not that the app exited. Unsaved-work prompts and refusal are respected; there is no termination-signal fallback.

The Actions menu resolves running controls on demand, using only the selected catalog ID. Open remains first, Alias next, process controls precede metadata, and Favorites/Hide from Search remain the suffix. Process replies are cancelled when the menu closes or changes; late inserted rows preserve keyboard focus/navigation. Native errors preserve the query and metadata actions remain usable. Successful Hide clears the query after native acknowledgement; Quit preserves it. No activation, usage, favorites or history mutation occurs.

Each process descriptor carries a native instance fingerprint. The action re-resolves the catalog application and current native state and compares the fingerprint before requesting anything. A closed/replaced process, changed window set or stale menu cannot silently target a replacement instance. Fingerprints are concurrency guards, not authentication secrets. The public request rejects arbitrary paths, PIDs, extra fields, Restart and Force Quit.

## Platform boundaries

Linux/X11: cooperative _GTK_APPLICATION_ID or _KDE_NET_WM_DESKTOP_FILE must exactly match the desktop ID (optional .desktop suffix). Conflicting IDs, remote/missing WM_CLIENT_MACHINE, inaccessible/foreign/self/system processes and executable mismatches fail closed. GIO supplies the desktop executable; /proc verifies its canonical path, owner and process start time. No Exec command is executed for inspection and WM_CLASS/window titles are not identity proofs. The complete verified window set is revalidated before actions. Hide sends ICCCM WM_CHANGE_STATE/IconicState requests for visible verified windows. Quit sends WM_DELETE_WINDOW only when every verified window advertises it. These are window operations: unmatched/dialog/background windows or background processes can remain. Wrappers/interpreters/sandboxed apps without matching native identity may be unsupported. X11 properties are cooperative desktop metadata, not protection against a malicious client in the same X server. Wayland sessions, including partial XWayland views, report unavailable.

macOS: NSWorkspace enumeration and canonical bundle paths select a unique running app; PID and launch date bind the instance. A static JXA AppKit bridge invokes NSRunningApplication.hide/terminate, without process-name matching, shell interpolation or System Events. A five-second helper deadline kills/reaps only the helper, never the selected app. Bundle ambiguity, refusal, stale identity and unavailable metadata fail closed. Native execution remains unverified.

Windows: EnumWindows plus GetWindowThreadProcessId/OpenProcess/GetApplicationUserModelId must prove the catalog AUMID. Retained query-only process handles and creation time guard PID reuse; window ownership and process liveness are rechecked. Hide uses ShowWindowAsync(SW_MINIMIZE); Quit posts WM_CLOSE. Unpackaged desktop apps without a process AUMID, hosted windows with another process identity, inaccessible apps and background-only apps are unsupported. These requests can leave a background process running. Rust types were checked on the Linux host using the actual adapter and minimal catalog/policy types; Windows ABI linking, compilation with the full target and native behavior remain unverified.

## Traceability and validation

Pinned ItemActions.swift and ItemActionTests.swift snapshots are in tests/reference/process-actions at Floe 5b26e9d7a0240243251d7421bee5b90da5f52b9b. The original checkout is unchanged. The existing 14-declaration metadata map remains intact. The new map partially covers anAppThatIsNotRunningOffersNoQuitAndASystemAppNoTrash; Trash and Force Quit assertions remain deferred. Five scoped source contracts/boundaries distinguish partial graceful behavior from the explicitly excluded Force Quit boundary. No dedicated original Hide/Quit suite is invented. Native fixtures provide additional equivalent cases; the original macOS baseline has not run here.

- 182 Rust tests pass, Cargo formatting passes and full workspace Clippy passes with warnings denied. The initial formatting check found one wrapping difference; it was corrected and the full checks rerun.
- 97 frontend tests pass, TypeScript passes and all pinned-reference validators pass. Coverage includes menu order, selected-ID/instance-token dispatch, stale replies, preserved query and unchanged usage, hidden state, graceful-action error handling and keyboard focus after async row insertion.
- The installed Linux package passes 12 process-action stages: stopped app; same title/WM_CLASS without native identity; remote identity; conflicting IDs; executable mismatch; missing close protocol; two-window Hide and launcher dismissal; already-hidden state; refusal without termination/query loss; natural accepted close; stale exited/replaced instances; and byte-identical preference/history data. The fixture is a private owned GTK app with no production bypass; both windows log close requests and can refuse them.
- The same installed package passes all ten headless search CLI stages, live settings/CLI context, resident/focus/keyboard checks including five rapid reopens, metadata clipboard/reveal including real Dolphin selection, customization, personalization, picker, native/GIO launch and resident/EOF smoke checks. Performance qualification was skipped.
- The installed 0.11.0 resident is running on the actual KDE/X11 desktop with Ctrl+Alt+Space registered. Its rendered window and version were visually inspected. Tests use private fixture data. The real-desktop screenshot was inspected in /tmp and is not included in the checkpoint archive. Public downloadable release, native macOS/Windows/Wayland qualification, remote CI, signing and clean-machine installation remain pending. Stop at CP011; CP012 has not started.


## Local artifact

Asaaan Launcher 0.11.0 / CP011, Linux amd64. The Debian package was extracted into deployments/linux/current; all native acceptance used that installed executable.

Package SHA-256: 1dc2fdfcd0fef9959bd7c2d6f9aedacc01c547739d1bac5bf7ab40c375ea215d.

Installed binary SHA-256: f1c3ebafa81f574d1ec8250c1b9c6590d1543d3735fbe4f2d4aff6e860708dc0.

The ignored .artifacts/checkpoint-011 directory holds the durable package, installation manifest, native/frontend/build logs, native acceptance JSON, desktop-run evidence, SHA256SUMS and a post-commit validation manifest with the exact source revision. Installed versions and the archived package do not depend on the disposable /tmp Cargo cache described in CP010. Public release remains user controlled and pending.

Native protocol references: [EWMH window identity and state](https://specifications.freedesktop.org/wm/latest/ar01s05.html), [ICCCM graceful delete/iconify protocols](https://xorg.freedesktop.org/archive/current/doc/xorg-docs/icccm/icccm.html), [Windows process AUMID](https://learn.microsoft.com/en-us/windows/win32/api/appmodel/nf-appmodel-getapplicationusermodelid).
