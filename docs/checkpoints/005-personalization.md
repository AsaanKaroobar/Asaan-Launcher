# CP005: Favorites and launch history

Status: implemented, tested, packaged, installed and running locally for user review. Downloadable publication and native macOS/Windows/Wayland qualification remain pending. Branch checkpoint-5; product version 0.5.0.

The user reviewed CP004 and authorized the next checkpoint. Performance upgrades remain deferred until the complete software is ready. Public CP004 publication remains unverified. New commits use Dani-Bhai for author and committer.

## Scope

Favorites, launch-use counts and recent searches now survive disposable-panel and resident restarts. Favorites retain insertion order, remain separate from up to five usage suggestions, and use the existing Floe ranking and age weights. A favorite action and Control+Shift+F (Command+Shift+F on macOS) operate on the selected application. Up at the top recalls earlier searches newest first; typing a new query ends recall. History controls allow opting out of future search recording and clearing saved searches; usage counts are separate from query history.

The resident is the only writer. Foregrounds send typed operations over inherited private pipes, correlate acknowledgements and hold finite-work leases until the reply. A foreground never saves a full snapshot. As in Floe Model.activate, a known catalog selection records use and query before OS delegation; an OS launch failure therefore still counts that activation. Unknown catalog IDs are rejected before recording. Search and favorite operations validate catalog IDs. Personalization payloads are excluded from lifecycle logs.

Persistence is a versioned atomic replacement under the application's native data directory, using owner-only file permissions on Unix. Writes are acknowledged only after the new file is synced and published. Corrupt or unsupported files are preserved and surfaced as warnings; failed writes leave the previous in-memory and on-disk snapshot in place. Writes are immediate rather than Floe's one-second burst timer, a documented durability substitution; batching/performance remains later work. The file is limited to 2 MiB so snapshots fit inside bounded private IPC packets. Only local storage is used.

Aliases, hidden-result management, full action menus, extension/runtime compatibility, complete settings/import/sync and native platform qualification remain later work. Favorites are tied to platform catalog IDs; unresolved IDs are retained and omitted from visible results until available again.

## Validation

Validated on Linux on 2026-10-16:

- npm run check passed: typechecking, all reference-hash/mapping checks and 41 frontend tests.
- npm run native:check passed: formatting, 112 Rust tests and clippy with warnings denied. This includes 15 personalization-store cases, a real three-foreground resident-ownership test and a catalog personalization integration test; earlier ranking/catalog/lifecycle tests remain green.
- The scoped provenance map accounts for all 35 declarations in the copied UsageStore, Settings and ItemAction suites: 7 ported, 4 partial, 24 deferred. Timer-dependent cases explicitly record the immediate-write substitution; full settings, hidden rows, command/tool providers and action menus remain deferred.
- The installed normal UI passed a private KDE/X11 probe with four actual marker-application launches across three resident restarts. It verified favorite/unfavorite persistence, Up recall, deduplicated queries, recording opt-out, query clearing, retained use counts and foreground exit after each activation. The probe uses only temporary XDG data and private D-Bus/display state.
- The same final installed artifact passed legacy reference-search, native GIO launch/Exec-argument, three resident search/presentation/reap, private-panel EOF and actual shortcut/typing/Escape/focus-return/process-cleanup checks.
- The normal app is running on the user's Linux desktop for review. [Native screenshot](005-native.png). Floe remains unchanged. No performance qualification is claimed; CP003 measurements remain recorded and upgrades are deferred.

Package: asaaan-launcher_0.5.0_amd64.deb

Package SHA-256: cc2e728187a5e2bdcbe6c34e87e357c49145807d41423dcf9667d2e18f7a4d44

Installed binary SHA-256: 4a36ad6c4782507e4601586788f047e3b2d79d5635499e4a322c8d1a73e2fb85

The package, SHA256SUMS, installation manifest, validation manifest with source commit, probe JSON and logs are staged under .artifacts/checkpoint-005. Staging is not publication. macOS/Windows native builds, platform icons and Wayland qualification remain pending; Linux results do not certify those targets. The CI workflow includes the new native probe but its remote execution is not claimed here.

## Delivery

Test, package, install and run the validated Linux artifact locally for review. Downloadable publication and macOS/Windows/Wayland native qualification remain independent pending gates. Stop after CP005; CP006 is not started.
