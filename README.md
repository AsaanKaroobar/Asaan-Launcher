# Asaaan Launcher

A cross-platform desktop launcher built in checkpoints with Tauri 2, React, TypeScript and Rust.

The behavioral reference is the adjacent Floe checkout at commit 5b26e9d7a0240243251d7421bee5b90da5f52b9b. The [implementation plan](../docs/floe-cross-platform/PLAN.md) defines the eventual parity goal. Checkpoint 10 adds headless application search with --search, using committed preferences and the existing ranking engine. The separate General/Applications settings process remains in place. The standalone command-line picker remains in place. Catalog-resolved metadata copy/reveal actions and resident clipboard ownership remain in place. Application aliases, a keyboard-accessible Actions menu and hide/restore controls remain in place. Persistent favorites, usage-based suggestions and recent-search recall remain in place. Enter or double-click opens the selected app. The resident tray and Ctrl+Alt+Space disposable panel remain in place. Process controls, trash, general file actions, full settings, extensions and built-in tools remain later work.

## Develop

Install Node.js 22.12+ and Rust 1.95.0, plus the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your operating system.

    npm ci
    npm run tauri dev

Use npm run dev for a browser preview. Browser mode explicitly reports that the native connection is unavailable.

## Validate and package

    npm run check
    npm run build
    npm run native:check
    npm run package:linux

Native validation checks all Cargo workspace members: formatting, Rust tests and Clippy with warnings denied. npm run check also verifies the pinned reference hashes and test mappings. Linux packaging creates a Debian package. On Windows use npm run tauri build -- --bundles nsis; on macOS use npm run tauri build -- --bundles app. Each platform must be built and tested on its own runner.

    npm run install:local
    npm run smoke:local
    npm run smoke:resident
    npm run smoke:applications
    npm run start:local

Local installation extracts the Debian package into this project under deployments/linux/current, without root or changing the system package database. The reference search smoke test loads the installed application, performs a cal search through React/Tauri/Rust, verifies Calendar then Calculator, and only then exits; start:local starts the resident and opens its disposable native panel. Ctrl+Alt+Space toggles it; Escape clears text first, then hides an empty search. Closing the panel hides it, while Quit in the tray exits the resident. Both require a graphical session (or Xvfb for automated smoke testing).

The resident smoke test opens/searches/hides/reaps three distinct foreground processes and verifies a native panel exits after parent-pipe EOF. On an X11 desktop, npm run test:resident:x11 exercises actual shortcut events, Escape, focus return, single-instance forwarding and a 60-second settled resident-memory sample. It creates its own focus fixture; avoid interacting with the keyboard while it runs. Wayland currently exposes a tray fallback with an explicit shortcut-unavailable status.

See [checkpoint rules](docs/CHECKPOINTS.md) and [checkpoint 012](docs/checkpoints/012-process-controls.md). The user explicitly deferred performance upgrades until the complete software is ready. Public checkpoint releases remain user-controlled.

The pure search crate is in crates/launcher-search. npm run search:test replays 775 matcher and 20 ranking oracle cases and the ported Floe assertions. See [reference provenance](tests/reference/README.md). An optional Swift compiler can recheck the frozen oracle with npm run parity:swift; regular builds do not require Swift.

The pure lifecycle crate is in crates/launcher-lifecycle. npm run lifecycle:parity verifies copied source hashes and the explicit ported/deferred test map. No Swift toolchain or test fixture executable is included in release packages.

Native tray routing can be inspected or exercised on a StatusNotifier Linux desktop with /usr/bin/python3 scripts/test-tray-linux.py RESIDENT_PID inspect (or show/hide/quit). It requires the system GIO Python adapter and targets only that PID. CP003’s memory and cold-start feasibility findings are recorded in its checkpoint record.

The application catalog crate is crates/launcher-apps. npm run applications:parity verifies the 25-declaration application test map. Normal use scans installed apps in a worker, keeps cached results during refresh and polls every five seconds while the foreground is open; Refresh applications forces a scan. The reference catalog is now used only by diagnostic smoke mode.

The installed application smoke uses a temporary isolated XDG catalog and launches its own marker-writing probe; it does not open an existing user application. On Linux X11 use npm run test:resident:x11 -- --functional-only for keyboard/focus/process regression without memory qualification. Native macOS/Windows execution and platform icons remain pending qualification as detailed in the checkpoint record.

On a Linux development host with Xvfb and kwin_x11 installed, npm run test:resident:isolated exercises native keyboard/focus/lifecycle behavior in a private display and D-Bus session with temporary preferences. It leaves the current desktop untouched and skips performance qualification.

Favorites are saved by the resident, in insertion order, with Control+Shift+F (Command+Shift+F on macOS) or the selected-result button. Up at the first row recalls earlier searches; typing ends recall. History exposes a local search-recording opt-out and Clear search history. As in Floe, known application activations count before OS delegation, including a launch that later fails. Clearing queries does not erase usage counts or favorites. Data lives in personalization.json under the native app data directory (Linux: $XDG_DATA_HOME/com.asaaan.launcher, or ~/.local/share/com.asaaan.launcher). The file is versioned, atomically replaced and limited to 2 MiB. Corrupt/unsupported files are preserved and require repair or moving aside while the resident is stopped.

Run npm run smoke:personalization on Linux with the isolated X11 prerequisites above. It exercises the installed normal UI, an isolated marker application, favorite/unfavorite, Up recall, recording opt-out, clearing and three resident restarts. It never changes the user’s catalog or history. See tests/reference/personalization for the full scoped test map, including the explicit immediate-save substitution for the reference burst timer.

Use Actions or Control+K (Command+K on macOS) for Open, Set/Edit Alias, favorite and Hide from Search. Right-clicking a row opens its menu. Customize or Control+comma (Command+comma) lists hidden applications and Show Again restores one, including an app that is no longer installed. Hiding removes its favorite and retains alias/usage; restoring does not re-add the favorite. Alias editing is a focused form with Enter to save and Escape to cancel. The same existing alias ranking handles exact matches, prefixes and title fallback.

CP005 version-1 data is read without rewriting and upgraded to version 2 on the first successful mutation, retaining all previous values. Older builds preserve but cannot read version 2. Run npm run smoke:customization for installed UI coverage of migration, alias search/launch, hide, restoring an uninstalled app, clearing the alias and four resident restarts. It uses a private catalog, data directory and desktop. Scoped provenance is under tests/reference/customization. Full OS-specific application actions remain deferred.

The X11 keyboard probe also requires system Python GI and AT-SPI bindings (`python3-gi`, `gir1.2-atspi-2.0` on Debian/Ubuntu). It checks native focus and observes input text through accessibility before asserting clear/hide behavior, including five rapid grace-period reopen cycles.

The Actions menu offers native Copy Name and, when available, Copy Path, the platform application identifier and Reveal in File Manager (Show in Finder on macOS). Copy success preserves the query and reports feedback only after a resident acknowledgement. Reveal resets/dismisses only after native success. Linux copies/selects the `.desktop` catalog entry; macOS uses the application bundle. Windows StartApps identities support name/ID copies and deliberately omit filesystem operations. These adapters require target-specific qualification; only Linux X11 is locally tested.

Run `npm run smoke:actions` for private-session native UI clipboard checks, persistence after foreground disposal, removed targets, FileManager1 refusal/timeout, work-lease cleanup, literal Unicode paths and real Dolphin selection. Linux requires `python3-gi`, `gir1.2-gtk-3.0`, `gir1.2-atspi-2.0`, Xvfb, kwin_x11 and Dolphin. `npm run actions:parity` verifies the 14-declaration scoped reference map. Metadata operations do not record activations or change personalization. Full clipboard history/own-copy markers belong to the later clipboard checkpoint; clipboard text is not logged.

## Command-line picker

From this project on Linux:

    printf 'alpha\nbeta\tdetail\ngamma\n' | deployments/linux/current/usr/bin/asaaan-launcher --pick --prompt 'Choose a line'
    printf 'alpha\n\nbeta\n' | deployments/linux/current/usr/bin/asaaan-launcher --pick --query beta --index

The first command opens a dedicated picker. Type to filter; use arrows, Page Up/Down (nine rows), Home/End, Enter to choose, or Escape to cancel. A click selects a row; double-click chooses it. Return with no match keeps the window open. Closing the window or losing focus cancels. Starting-query text keeps its caret at the end. IME composition retains Return/Escape.

Output is the entire original line, including tabs and surrounding spaces, followed by a newline. --index prints the original zero-based line position, including skipped blank lines. Tabs split the title and displayed detail; matches in the title score ten points above equivalent detail matches. Equal scores retain input order. CRLF counts as one line break. Non-UTF-8 input falls back to Latin-1. Exit status is 0 for a choice, 1 for cancellation/empty input, and 64 for invalid picker arguments or terminal stdin; I/O failures can return 74. --prompt, --query and --index require --pick; unrelated/repeated picker options are rejected. ASAAAN_PICK_AUTO=1 selects the first filtered match without requiring a display. Use --help for usage.

The picker bypasses resident startup, application scanning, preferences and global shortcuts. It can coexist with the resident and uses an ephemeral WebView. npm run smoke:picker checks the installed Linux package on an isolated X11 desktop, including headless output/encoding/TTY cases, native keyboard/focus and coexistence. Native Windows/macOS/Wayland qualification remains pending; Linux results do not certify those targets. Floe's diagnostic picker snapshot environment variable is not implemented.

## Settings

Open Settings from the launcher header, the tray's Settings… item, or the installed executable:

    deployments/linux/current/usr/bin/asaaan-launcher --settings
    deployments/linux/current/usr/bin/asaaan-launcher --settings --page applications

General controls Remember searches and Clear search history. Turning recording off keeps the history already stored; clearing history preserves application usage and favorites. Applications lists the full catalog, including hidden apps, with a filter by name, alias or ID. Set/Edit alias uses explicit Save/Cancel; Hide removes the app from Favorites; Restore keeps its alias and usage. Removed apps with stored aliases or hidden records remain manageable: their alias can be removed and visibility restored.

One settings child runs at a time, independently of the disposable search panel. Reopening brings it forward; a new child remembers the last General/Applications page within the resident session. Closing Settings or Ctrl/Command+W ends that child after pending operations finish. Focus loss keeps Settings open. Resident exit or private parent-pipe EOF ends Settings. Preferences remain serialized by one resident store; clients send field operations and receive committed snapshots. Five-second/focus refreshes take external launcher edits without overwriting an unsaved alias draft.

This checkpoint implements only General and Applications and the existing preference fields. Other pages, hotkey recording, import/export, updater and full native visual fidelity remain later work. Unsupported --page values fail with exit 64. npm run settings:parity checks the scoped reference map; npm run smoke:settings exercises the installed normal UI in a private Linux X11 session. Windows/macOS/Wayland native qualification and public release remain pending. Performance upgrades remain deferred.

## Search from the terminal

    deployments/linux/current/usr/bin/asaaan-launcher --search notes
    deployments/linux/current/usr/bin/asaaan-launcher --search ''
    deployments/linux/current/usr/bin/asaaan-launcher --search=memo

Search prints the first eight ranked installed applications. An empty query browses Favorites, Suggestions and Applications in the same order as the launcher. Output follows Floe's English text format, for example Notes (Application) (alias memo), with [Favorites] or another section prefix when browsing. Aliases, favorite boosts, usage, keywords and hidden apps use the existing shared search engine. Unicode remains literal; control characters in catalog text are escaped so one result cannot inject terminal controls or extra lines.

The command reads the last committed personalization snapshot from the same platform data directory as the resident. It does not record usage/history, write preferences, launch applications, read stdin, open a window, start a resident or forward activation to one already running. Atomic settings publications can be read concurrently. Missing preferences use defaults without creating a file; corrupt, future-version or unreadable preferences fail visibly rather than ignoring hidden-app rules. Old v1 data is interpreted in memory without rewriting it.

Exit codes: 0 for a successful search, including no matches; 64 for invalid/missing/conflicting arguments or queries over 4096 UTF-8 bytes; 69 for catalog failure; 74 for stdout failure; 78 for preference/path failures. --search accepts a single query and cannot be combined with picker, settings or resident-control modes. Use --search=QUERY for a query beginning with --. --help lists the available modes.

This checkpoint covers application results only. Floe's extension commands, built-ins, dictionary and other root providers remain later work. No browser, general file or network provider is started. npm run smoke:search-cli checks the installed Linux command without a display or session bus. Native Windows/macOS execution remains pending qualification.

For this local session, the disposable Cargo target cache was moved to /tmp/asaaan-cp010-target to work around recurring workspace FUSE/NTFS ENOSPC errors; src-tauri/target is an ignored symlink. Installed versions and checkpoint artifacts remain on the workspace volume. If /tmp is cleared, remove the dangling src-tauri/target symlink before rebuilding. This is a local build-cache arrangement, not a runtime dependency of installed packages.

## Graceful running-app controls (CP011)

Open an application's Actions menu for Hide and Quit when native identity can be verified. Hide dismisses the launcher; Quit preserves the query and acknowledges the request without claiming the app has exited. Apps can refuse to close or ask about unsaved work. These actions do not change usage or preferences. Instance fingerprints prevent a stale menu from affecting a replacement process.

Linux X11 checks native desktop ID, local hostname, owner, executable and process start time. Hide minimizes the verified windows; Quit uses WM_DELETE_WINDOW only when every verified window advertises it. Apps without usable native metadata, wrappers with a different executable, headless/background-only apps and Wayland sessions are unsupported. No process-name/title guessing or signal fallback is used. X11 metadata is cooperative and does not defend against hostile clients in the same desktop session.

macOS uses NSWorkspace and canonical bundle paths with native hide/terminate requests. Windows uses process AUMIDs and native window requests; unpackaged apps without a process AUMID and windows hosted by a different process cannot be identified by this adapter. Linux/Windows window-close requests can leave background processes running. Native macOS and Windows validation remains pending; no complete process-action parity is claimed.

Run npm run process:parity for the pinned source contracts and npm run smoke:process for the installed Linux/X11 multi-window acceptance test. Restart and Force Quit are added in CP012 below. Performance upgrades remain deferred.

## Restart and Force Quit (CP012)

Restart asks the app to quit, waits up to ten seconds for its original processes to exit, and only then reopens it. Unsaved-work prompts, refusal, a changed desktop entry, a replacement instance or a continuing background process prevent reopening. No forced fallback or history/usage mutation occurs.

Force Quit… dismisses the launcher and opens a native unsaved-work warning with cancellation as the default. It revalidates the selected instance after confirmation before terminating verified processes. Linux uses stable pidfds; Windows uses retained process handles; macOS uses NSRunningApplication. Linux/X11 is locally qualified; native macOS/Windows validation is pending. Controls remain unavailable where identity or required capabilities cannot be proven. No process tree is guessed or killed.

Run npm run smoke:process-controls for the installed Linux/X11 cancellation, hung-app, stale-instance, refused/background/delayed-exit and changed-entry checks. See the checkpoint report for platform and HUD fidelity limits. Extensions and performance work remain deferred.
