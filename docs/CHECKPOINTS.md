# Checkpoint delivery

User instruction: build in steps. Each checkpoint must be reached, tested, installed and run locally, and then published as a downloadable checkpoint release before work moves forward.

## Current scope

**Active: CP013, extension host foundation.** User authorized advancing after CP012. Preserve Floe’s Bun/NDJSON boundary, reference-v1 tree decoding and bounded process cleanup. Extension List UI is CP014. Public CP012 publication is unverified and user controlled. Extension List UI and performance upgrades remain deferred.

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

- CP014: first interactive extension List UI, using the CP013 host and existing resident/panel ownership.
- Later checkpoints: additional extension views/actions/preferences, store/background integrations, everyday tools, native services, fidelity and release qualification. Refine scope before starting each.

The broader phases in ../../docs/floe-cross-platform/PLAN.md are planning groups, not permission to implement them all at once.
