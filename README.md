# Asaaan Launcher

A cross-platform desktop launcher built in checkpoints with Tauri 2, React, TypeScript and Rust.

The behavioral reference is the adjacent Floe checkout at commit 5b26e9d7a0240243251d7421bee5b90da5f52b9b. The [implementation plan](../docs/floe-cross-platform/PLAN.md) defines the eventual parity goal. Checkpoint 2 adds a standalone Rust search core and a clearly labeled reference catalog. Installed-application discovery, actual launch actions, hotkeys and extensions remain future checkpoints.

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

Native validation checks both Cargo workspace members: formatting, Rust tests and Clippy with warnings denied. npm run check also verifies the pinned reference hashes and test mappings. Linux packaging creates a Debian package. On Windows use npm run tauri build -- --bundles nsis; on macOS use npm run tauri build -- --bundles app. Each platform must be built and tested on its own runner.

    npm run install:local
    npm run smoke:local
    npm run start:local

Local installation extracts the Debian package into this project under deployments/linux/current, without root or changing the system package database. The CP002 smoke test loads the installed application, performs a cal search through React/Tauri/Rust, verifies Calendar then Calculator, and only then exits; start:local opens the installed native window normally. Both require a graphical session (or Xvfb for automated smoke testing).

See [checkpoint rules](docs/CHECKPOINTS.md) and [checkpoint 002](docs/checkpoints/002-search.md). Do not begin checkpoint 003 until the current checkpoint is reviewed and its delivery gates are complete.

The pure search crate is in crates/launcher-search. npm run search:test replays 775 matcher and 20 ranking oracle cases and the ported Floe assertions. See [reference provenance](tests/reference/README.md). An optional Swift compiler can recheck the frozen oracle with npm run parity:swift; regular builds do not require Swift.
