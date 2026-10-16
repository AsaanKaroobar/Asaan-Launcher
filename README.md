# Asaaan Launcher

A cross-platform desktop launcher built in checkpoints with Tauri 2, React, TypeScript and Rust.

The behavioral reference is the adjacent Floe checkout at commit 5b26e9d7a0240243251d7421bee5b90da5f52b9b. The [implementation plan](../docs/floe-cross-platform/PLAN.md) defines the eventual parity goal. This first checkpoint is a native application shell; launcher features are not implemented yet.

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

Native validation runs formatting, Rust tests and Clippy with warnings denied. Linux packaging creates a Debian package. On Windows use npm run tauri build -- --bundles nsis; on macOS use npm run tauri build -- --bundles app. Each platform must be built and tested on its own runner.

    npm run install:local
    npm run smoke:local
    npm run start:local

Local installation extracts the Debian package into this project under deployments/linux/current, without root or changing the system package database. The smoke test loads the installed application and verifies a React-to-Rust round trip before exiting; start:local opens the installed native window normally. Both require a graphical session (or Xvfb for automated smoke testing).

See [checkpoint rules](docs/CHECKPOINTS.md) and [checkpoint 001](docs/checkpoints/001-foundation.md). Do not begin checkpoint 002 until all current gates, including release publication, are complete.
