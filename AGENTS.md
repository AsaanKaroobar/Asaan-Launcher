# Asaaan Launcher

Product name is exactly **Asaaan Launcher** (three a's in Asaaan). Machine identifiers use asaaan-launcher and com.asaaan.launcher.

## Checkpoint workflow

- Read docs/CHECKPOINTS.md and the active checkpoint record before changes.
- Work only on the active checkpoint. Do not start later feature work in the same task.
- A checkpoint advances through implemented, tested, locally installed, locally run, and downloadable release published. Record each state independently with evidence.
- Never label a build, browser preview, CI configuration, or release staging folder as a deployed native application.
- Do not advance until the current checkpoint is tested, installed/run locally, and its downloadable release is published. Stop and report completion or the concrete outstanding gate.
- Floe at ../Floe is the pinned, unchanged behavioral reference. Planning and test inventories are in ../docs/floe-cross-platform. Preserve the source-to-test traceability approach there.
- Current scope is CP014: first interactive extension List UI, resident-owned supervised sessions, equivalent tests, packaging and local run. Detail/Form, store and scheduling remain deferred.
- User explicitly approved CP014 after CP013, then authorized incremental launch performance work after the platform CI repair. Active follow-ups: docs/checkpoints/014-launch-performance-1.md, docs/checkpoints/014-floe-theme-placement.md, docs/checkpoints/014-wayland-hyprland.md, docs/checkpoints/014-auto-updater.md and docs/checkpoints/014-install-script.md. Wayland/Hyprland core support is implemented, tested, locally installed and run, and published as 0.2.0 on 2026-10-26. The Tauri updater, signing, AppImage channel, startup prompt and tray check are implemented, tested locally, and published as 0.3.0 on 2026-10-26; public releases remain user controlled. A user-level `install.sh` for Linux and macOS is implemented and Linux-tested on `main`; macOS execution remains unqualified and the script is not attached to the published 0.3.0 assets. CP015 feature work remains deferred; public releases remain user controlled. The user also authorized focused-monitor placement, removing launcher control chrome, and a Floe default theme with the previous design selectable as Legacy.
- All new commits must use Dani-Bhai and 62589953+Dani-Bhai@users.noreply.github.com for both author and committer. Verify with git var before committing. Do not rewrite historical commits unless requested.
- Preserve AGPL-3.0 and original attribution if reference code is copied. Use this project's own application IDs and release authorities.
- UI/browser tests supplement native tests; do not claim Windows/macOS support has been tested from Linux results.
