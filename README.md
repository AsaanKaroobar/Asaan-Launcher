# Asaaan Launcher

A cross-platform desktop launcher built in checkpoints with Tauri 2, React, TypeScript and Rust.

The behavioral reference is the adjacent Floe checkout at commit 5b26e9d7a0240243251d7421bee5b90da5f52b9b. The [implementation plan](../docs/floe-cross-platform/PLAN.md) defines the eventual parity goal. Checkpoint 3 adds a resident tray process, Ctrl+Alt+Space shortcut and disposable search panel on top of checkpoint 2’s shared search core. The catalog remains clearly labeled sample data. Installed-application discovery, actual launch actions and extensions remain future checkpoints.

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
    npm run start:local

Local installation extracts the Debian package into this project under deployments/linux/current, without root or changing the system package database. The CP003 search smoke test loads the installed application, performs a cal search through React/Tauri/Rust, verifies Calendar then Calculator, and only then exits; start:local starts the resident and opens its disposable native panel. Ctrl+Alt+Space toggles it; Escape clears text first, then hides an empty search. Closing the panel hides it, while Quit in the tray exits the resident. Both require a graphical session (or Xvfb for automated smoke testing).

The resident smoke test opens/searches/hides/reaps three distinct foreground processes and verifies a native panel exits after parent-pipe EOF. On an X11 desktop, npm run test:resident:x11 exercises actual shortcut events, Escape, focus return, single-instance forwarding and a 60-second settled resident-memory sample. It creates its own focus fixture; avoid interacting with the keyboard while it runs. Wayland currently exposes a tray fallback with an explicit shortcut-unavailable status.

See [checkpoint rules](docs/CHECKPOINTS.md) and [checkpoint 003](docs/checkpoints/003-resident.md). CP004 has not started.

The pure search crate is in crates/launcher-search. npm run search:test replays 775 matcher and 20 ranking oracle cases and the ported Floe assertions. See [reference provenance](tests/reference/README.md). An optional Swift compiler can recheck the frozen oracle with npm run parity:swift; regular builds do not require Swift.

The pure lifecycle crate is in crates/launcher-lifecycle. npm run lifecycle:parity verifies copied source hashes and the explicit ported/deferred test map. No Swift toolchain or test fixture executable is included in release packages.

Native tray routing can be inspected or exercised on a StatusNotifier Linux desktop with /usr/bin/python3 scripts/test-tray-linux.py RESIDENT_PID inspect (or show/hide/quit). It requires the system GIO Python adapter and targets only that PID. CP003’s memory and cold-start feasibility findings are recorded in its checkpoint record.
