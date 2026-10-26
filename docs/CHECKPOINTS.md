# Checkpoint delivery

User instruction: build in steps. Each checkpoint must be reached, tested, installed and run locally, and then published as a downloadable checkpoint release before work moves forward.

## Current scope

**Active: CP014, first interactive extension List UI. Implemented, tested, locally installed/run; public release [0.2.0](https://github.com/AsaanKaroobar/Asaan-Launcher/releases/tag/0.2.0) published.** User authorized advancing after CP013. Use the existing Bun/reference-v1 host, resident ownership and native panel lifecycle. Public CP013 publication is unverified and user controlled. Detail/Form, store and scheduling remain deferred. The user subsequently authorized incremental launch performance work after platform CI repair, then focused-monitor placement and a Floe default theme with the original design retained as Legacy, then a Windows focus repair, then Wayland/Hyprland support, then automatic updates. Follow-ups: [launch performance increment 1](checkpoints/014-launch-performance-1.md), [Floe theme and placement](checkpoints/014-floe-theme-placement.md), [Wayland/Hyprland support](checkpoints/014-wayland-hyprland.md), and [automatic updates](checkpoints/014-auto-updater.md). Core Wayland/Hyprland support and the first automatic-update implementation are tested, locally installed and run; the updater release is pending and user controlled.

A packaged executable alone is not a local installation. A local installation alone is not a successful native launch. An unsigned local package is not a signed public release. Browser preview is not native-app testing. Record evidence for each independently.

## Gates

1. Implement only the active checkpoint.
2. Run the applicable frontend, native and packaged-app checks. Record failures honestly.
3. Install the tested build locally and run that installed artifact.
4. Publish the same validated artifacts as a downloadable checkpoint release at the selected release destination.
5. Record artifact hashes, source revision and test results in the checkpoint record.
6. Stop and report. Do not begin the next checkpoint within the same task.

If publication, a target OS, signing credentials or a deployment environment is unavailable, keep that gate pending and remain on the current checkpoint. Local Linux success does not prove macOS/Windows parity.

## Release record

The user authorized the initial public release from merged main as **0.1.0**, replacing internal checkpoint version numbering. All three platform jobs passed; the exact Linux release package was installed and run locally. Installer hashes, source revision, and qualification limits are recorded in [release 0.1.0](releases/0.1.0.md). No later feature checkpoint was started.

The user authorized publishing patch **0.1.1** on 2026-10-24. All three release-version platform jobs passed; the exact Windows release installer was installed and run locally, and the exact Linux package passed installed-native CI acceptance. Publication, source revision, installer hashes and qualification limits are recorded in [release 0.1.1](releases/0.1.1.md). The fix remains in CP014.

The user authorized publishing **0.2.0** on 2026-10-26 after the Windows focus repair and Wayland/Hyprland work were synced across `main`, `checkpoint-14-windows-focus` and `checkpoint-15-wayland-hyprland`. All three release-version platform jobs passed for the shared source; the exact Linux installer was installed and passed the native Wayland tests and smokes. Publication, source revision, installer hashes and qualification limits are recorded in [release 0.2.0](releases/0.2.0.md).

## Upcoming sequence (not authorized implementation in this checkpoint)

- CP015: additional extension view support; refine this slice after CP014 acceptance.
- Later checkpoints: additional extension views/actions/preferences, store/background integrations, everyday tools, native services, fidelity and release qualification. Refine scope before starting each.

The broader phases in ../../docs/floe-cross-platform/PLAN.md are planning groups, not permission to implement them all at once.
