# Asaaan Launcher

Product name is exactly **Asaaan Launcher** (three a's in Asaaan). Machine identifiers use asaaan-launcher and com.asaaan.launcher.

## Checkpoint workflow

- Read docs/CHECKPOINTS.md and the active checkpoint record before changes.
- Work only on the active checkpoint. Do not start later feature work in the same task.
- A checkpoint advances through implemented, tested, locally installed, locally run, and downloadable release published. Record each state independently with evidence.
- Never label a build, browser preview, CI configuration, or release staging folder as a deployed native application.
- Do not advance until the current checkpoint is tested, installed/run locally, and its downloadable release is published. Stop and report completion or the concrete outstanding gate.
- Floe at ../Floe is the pinned, unchanged behavioral reference. Planning and test inventories are in ../docs/floe-cross-platform. Preserve the source-to-test traceability approach there.
- Current scope is CP005: resident-owned favorites, usage suggestions, recent-search recall and controls, equivalent tests, packaging and local run. Aliases, hidden results, full action menus, extensions and broader settings remain later work.
- User reviewed CP004 and explicitly approved CP005. Performance upgrades remain deferred until the complete software is ready. Preserve prior measurements and publication status; public CP004 publication is unverified.
- All new commits must use Dani-Bhai and 62589953+Dani-Bhai@users.noreply.github.com for both author and committer. Verify with git var before committing. Do not rewrite historical commits unless requested.
- Preserve AGPL-3.0 and original attribution if reference code is copied. Use this project's own application IDs and release authorities.
- UI/browser tests supplement native tests; do not claim Windows/macOS support has been tested from Linux results.
