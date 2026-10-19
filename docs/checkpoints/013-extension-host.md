# CP013: Extension host foundation

Status: implemented, tested, locally installed and locally run. Public downloadable release pending. Version 0.13.0; branch checkpoint-13. User explicitly authorized advancing after CP012. Public CP012 publication is unverified and user controlled. Extension List UI (CP014) and performance upgrades remain deferred.

## Scope

The shared asaaan-extensions crate discovers manifests in folder-name order, drops malformed entries independently, preserves view/no-view/menu-bar declarations, arguments/preferences/keywords/defaults/intervals, and selects explicit macOS/Windows/Linux platform lists and defaults. It does not expose extension rows in the launcher yet. Source/store origin and custom icon overrides are deferred. Safe ASCII slugs prevent extension and command names becoming traversal or drive paths; this is an intentional stricter boundary than Floe.

The native host starts one bundled Bun 1.4.3 process per command with literal arguments and private stdin/stdout/stderr pipes. The original runtime, renderer, bridge, bundler, SQLite LocalStorage and cache implementations remain attributed and retain their frozen dependency lockfile. Runtime data goes under this project's platform data directory, independent of Floe's stores. The default Linux Debian resource directory follows Tauri's product-name directory. Missing runtime/resources fail explicitly without trying a system Bun.

NDJSON decoding retains bytes across UTF-8 splits and complete-line order, skips unreadable/nonobject lines, and never consumes partial EOF as a complete message. Reference version 1 resolves only against the advertised previous sequence, requests a whole render once per unresolved episode, retains only the latest tree and preserves referenced subtree identity/revisions across renders. Separate hosts never share revisions. Resource limits and the default JSON nesting bound are documented in tests/reference/extensions/README.md.

Bounded reader/writer queues isolate pipe IO. The heartbeat follows the three-second tick and strict greater-than-eight-second unanswered-ping rule, and a pong clears unresponsiveness. Stopping closes input and grants up to one second for cooperative shutdown before force cleanup of the owned Unix host group or Windows Job. The Windows kill-on-close Job implementation is included; native Windows execution and assignment/tree-cleanup behavior remain unqualified. Hard-killing the Unix owner can bypass Drop cleanup. Interactive resident ownership and panel EOF integration are not implemented until CP014; this headless foundation is not a claim that resident extension lifecycle is complete.

The diagnostic --extension-check FIXTURE COMMAND runs only bundled hello/diagnostics/host-probe commands, reads NDJSON input and prints decoded data/messages. Closing stdin stops the host; SIGINT/SIGTERM cancels on Unix. A 25-second deadline bounds this diagnostic mode. Every native request with a valid id gets a correlated explicit unavailable reply; no real native API dispatcher, scheduling, store install, preferences/secrets resolution, detached UI or memory qualification is enabled. Imported Mac-specific API shortcuts, direct Finder/Trash/application helpers and access-path display still need portability work before general extension exposure. Extension processes remain unsandboxed, as in the reference.

## Traceability and validation

Floe remains pinned at 5b26e9d7a0240243251d7421bee5b90da5f52b9b. tests/reference/extensions freezes 59 scoped sources and a 376-declaration inventory: 260 runtime declarations and 48 scoped Swift equivalents are mapped; 68 session/UI/other declarations stay explicitly deferred. Mapping status does not imply native UI/API parity. The parity checker verifies hashes, inventory completeness, mapped Rust tests and exact preserved runtime assertions. Two runtime test files adapt only executable selection and private data paths. The original Swift suite has not run on Linux.

- 243 Rust tests pass, including 51 extension-foundation cases. Cargo formatting and full workspace Clippy pass with warnings denied. The added compact-reference expansion case verifies that small reference messages cannot evade the expanded-node budget.
- 275 Bun runtime cases pass with 853 assertions using pinned Bun 1.4.3. The original test assertions are preserved; only subprocess executable selection and private support paths are adapted. 101 frontend tests, TypeScript and all reference checks pass.
- All 14 installed host stages pass: pinned runtime integrity; planets rendering, actions, reference update, full-render and early/late ping; protocol/stdout isolation; correlated native-request refusal; fatal/nonfatal errors; abnormal exit; watchdog and bounded hung-host cleanup; EOF and cancellation cleanup of an owned SIGTERM-refusing descendant; bounded stderr; oversized input refusal; exclusive CLI modes/path validation; explicit missing-resource/runtime failures without PATH fallback; and unchanged personalization bytes.
- The same final installed executable passes all prior native suites: 11 Restart/Force Quit stages, 12 Hide/Quit identity stages, nine Settings stages, resident keyboard/focus/rapid reopening, seven metadata/clipboard/reveal stages including real Dolphin selection, customization, personalization, ten picker stages, ten headless search stages, GIO activation, native rendering and resident/parent-EOF smoke. Performance qualification was skipped.
- The actual KDE/X11 desktop runs the installed 0.13.0 resident with Ctrl+Alt+Space registered. The native window/version was visually inspected. The actual-preference screenshot stays in /tmp and is excluded from the evidence archive.

Initial runtime runs exposed subprocesses resolving an older developer Bun; the adapted tests now use process.execPath. Installed acceptance found Tauri’s Linux resource folder uses the product name, and the lookup now matches the actual package. Initial diagnostics are retained alongside the successful final evidence. Final review also added the cooperative stop grace before Windows Job termination; native Windows behavior still needs qualification.

## Local artifact

Asaaan Launcher 0.13.0 / CP013, Linux amd64; extracted installation at deployments/linux/current. The final package and all native acceptance results use the same executable.

Package SHA-256: 691e7a15142ba5d1213a9cc9378332732eb04cdb168b4666498012385248410d.

Installed binary SHA-256: e3bef57d93d7eea22a265d6b300bfb39a12d8c9c8a009e3ebb0480079f333d2f.

The ignored .artifacts/checkpoint-013 archive contains the durable Debian package, runtime metadata, installation manifest, native/frontend/runtime/build logs, per-suite JSON, private-fixture reveal image, desktop-run evidence, SHA256SUMS and post-commit validation with the exact source revision. The package and installed version do not depend on the disposable Cargo cache.

Public release, signing, remote CI, clean-machine installation and native macOS/Windows qualification remain pending. Stop at CP013. CP014 has not started.
