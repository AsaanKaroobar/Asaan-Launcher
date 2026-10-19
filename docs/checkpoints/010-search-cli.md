# CP010: Headless application search CLI

Status: implemented, tested, locally installed and locally run. Public downloadable release pending. Version 0.10.0; branch checkpoint-10. User explicitly approved advancing after CP009; public CP009 publication remains unverified and user controlled. Performance upgrades remain deferred.

Scope: --search QUERY prints ranked installed applications with the same committed alias, favorite, usage, keyword and hidden-app context as the launcher. Other root providers, application process controls, extensions and full settings remain later work.

## Behavior and reference contract

Floe SearchCommand.swift assigns the query, takes model.results.prefix(8), and prints an optional [section] prefix, title, parenthesized item kind and optional (alias value). This checkpoint preserves that English format for applications, including empty-query Favorites/Suggestions/Applications sections and a trailing newline per row. Empty matches print nothing and exit 0. The command uses the same Store/search engine as the native panel, with hidden filtering before the eight-row display limit. Aliases appear on title matches too, not only alias matches. There is no new ranking algorithm.

The command exits before Tauri/GTK, WebView, single-instance forwarding, shortcuts, resident, settings or optional-source startup. It does not read stdin, launch applications, record searches/usage or save preferences. It reads an atomic committed personalization snapshot directly, so a running resident is not required. CLI and resident share the dirs 7 platform data resolver and com.asaaan.launcher identity; a bundled-configuration test guards against unhandled directory overrides. Linux uses XDG data/home rules, macOS Application Support and Windows roaming app data through the same library already used by Tauri. Native Windows/macOS qualification remains pending.

Missing preferences use defaults without creating their directory. v1 migration stays in memory. Corrupt, future, oversized or unreadable stores fail closed instead of searching with default hidden-app rules. Exit codes are 0 for success (including no matches), 64 for invalid/conflicting arguments or query limits, 69 for catalog errors, 74 for output failures and 78 for preference/path errors. Queries accept --search QUERY or --search=QUERY; the latter permits leading --. Queries are limited to 4096 UTF-8 bytes and NUL is refused. stdout is reserved for results and errors go to stderr.

Intentional differences: only application rows are currently available; Floe's extension/built-in/dictionary/other root providers are deferred. English kind/section labels are used. Unicode remains literal, but control characters in catalog text are escaped to prevent terminal controls or forged output rows; the reference prints raw text. Preference errors are explicit rather than silently ignoring unavailable saved rules. Full Floe CLI parity is not claimed.

## Traceability and validation

Pinned copies/hashes of SearchCommand.swift, DebugOptions.swift, RootItem.swift and RankingTests.swift are in tests/reference/search-cli at Floe 5b26e9d7a0240243251d7421bee5b90da5f52b9b. The original checkout is unchanged. The map reuses all 38 existing ranking declaration equivalents, and ten additional scoped contracts cover the CLI boundary. SearchCommand.swift has no dedicated original command test suite; its output/startup behavior is tested from source-level contracts, without inventing original test declarations. The prior 775 matcher/20 ranking fixtures and all earlier mappings remain intact. The original macOS command baseline has not been executed here.

- 175 Rust tests pass, Cargo formatting passes and full workspace Clippy passes with warnings denied. CLI coverage includes parsing/mode conflicts, eight-row format/order, sections, alias annotations, hidden filtering, boosts, keywords, Unicode/controls and injected scanner failure. A configuration test checks shared data identity.
- 90 frontend tests pass, TypeScript passes and every pinned-reference check passes. No new frontend behavior is introduced.
- The installed command passes ten headless stages with no display/session bus: exact ranked/sectioned output; alias/keyword/Unicode/control/no-match behavior; usage errors; stdin left open; unchanged preference bytes/timestamp/history and no activation/config/cache directories; invalid/future/oversized/unreadable files preserved with no stdout; missing-store defaults without creating a directory; read-only v1 migration; concurrent atomic publications producing complete snapshots; and stdout /dev/full returning 74 without panic.
- The installed Settings test edits an alias and hides the app while the resident runs, then verifies --search immediately sees each committed change without opening the panel or rewriting preferences. Existing Settings ownership/reuse/close/crash/EOF behavior remains covered.
- The same installed package passes prior resident keyboard/focus/lifecycle checks including five rapid reopens, metadata copy/reveal with real Dolphin selection, customization/history migration and persistence, picker native checks, reference search UI, native GIO launch and resident/EOF smoke checks. Performance qualification was not run.
- The installed resident/panel is running on the real KDE/X11 desktop with Ctrl+Alt+Space registered. Headless CLI acceptance used private fixture catalogs/preferences. Native macOS/Windows/Wayland execution, remote CI, clean-machine installation, signing and public release remain unverified.

Initial validation was blocked by recurring workspace FUSE/NTFS ENOSPC on new directory/file creation despite free capacity/inodes. The disposable 5.3 GiB Cargo target cache was copied and verified in /tmp/asaaan-cp010-target before removing the old cache and linking src-tauri/target to it; source, installed versions and all prior evidence were retained. The ignored cache link supports existing build/deploy paths. If /tmp is cleared, remove the dangling link before rebuilding. Installed versions and this checkpoint's archived package are stored on the workspace volume and do not depend on that cache. The first Clippy run also found a test-only unnecessary vec allocation; it was corrected and full native checks rerun. Initial logs are retained.

## Local artifact

Asaaan Launcher 0.10.0 / CP010, Linux amd64. The tested Debian package is extracted into deployments/linux/current and that installed binary is used for all native acceptance and the local desktop run.

Package SHA-256: 1bf1ac73a07d517274787e91755867a3607254a533787d1220b2e9ccc966d4f4.

Installed binary SHA-256: 44d2aea494821eca42dbd7a01f0e89574f7d0d82690a1f10c054d9ec26f8938d.

The ignored .artifacts/checkpoint-010 folder holds the durable package copy, installation, reference/test/native logs and JSON, cache-relocation diagnostics, SHA256SUMS and a post-commit manifest with the exact source revision. Public downloadable release remains user controlled and pending. Stop at CP010; CP011 has not started.
