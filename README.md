# Asaaan Launcher

A cross-platform desktop launcher built in checkpoints with Tauri 2, React, TypeScript and Rust.

The behavioral reference is the adjacent Floe checkout at commit 5b26e9d7a0240243251d7421bee5b90da5f52b9b. The [implementation plan](../docs/floe-cross-platform/PLAN.md) defines the eventual parity goal. Checkpoint 7 adds catalog-resolved native metadata copy/reveal actions, with resident clipboard ownership after foreground disposal. Application aliases, a keyboard-accessible Actions menu and hide/restore controls remain in place. Persistent favorites, usage-based suggestions and recent-search recall remain in place. Enter or double-click opens the selected app. The resident tray and Ctrl+Alt+Space disposable panel remain in place. Process controls, trash, general file actions, full settings, extensions and built-in tools remain later work.

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

See [checkpoint rules](docs/CHECKPOINTS.md) and [checkpoint 007](docs/checkpoints/007-application-metadata.md). The user explicitly deferred performance upgrades until the complete software is ready. CP008 has not started.

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
