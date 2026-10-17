# Checkpoint delivery

User instruction: build in steps. Each checkpoint must be reached, tested, installed and run locally, and then published as a downloadable checkpoint release before work moves forward.

## Current scope

**Active: CP011, graceful running-app controls.** User authorized advancing after CP010. Verify native app identity and offer Hide/Quit only where supported. Restart and Force Quit are CP012. Public CP010 publication is unverified and user controlled. Performance upgrades remain deferred.

A packaged executable alone is not a local installation. A local installation alone is not a successful native launch. An unsigned local package is not a signed public release. Browser preview is not native-app testing. Record evidence for each independently.

## Gates

1. Implement only the active checkpoint.
2. Run the applicable frontend, native and packaged-app checks. Record failures honestly.
3. Install the tested build locally and run that installed artifact.
4. Publish the same validated artifacts as a downloadable checkpoint release at the selected release destination.
5. Record artifact hashes, source revision and test results in the checkpoint record.
6. Stop and report. Do not begin the next checkpoint within the same task.

If publication, a target OS, signing credentials or a deployment environment is unavailable, keep that gate pending and remain on the current checkpoint. Local Linux success does not prove macOS/Windows parity.

## Upcoming sequence (not authorized implementation in this checkpoint)

- CP002: exact shared search/fuzzy ranking fixtures and tests against Floe.
- CP003: resident lifecycle, global shortcut and disposable panel feasibility.
- Later checkpoints: application discovery/actions, extension host, everyday tools, native integrations, fidelity and release qualification. Refine scope before starting each.

The broader phases in ../../docs/floe-cross-platform/PLAN.md are planning groups, not permission to implement them all at once.
