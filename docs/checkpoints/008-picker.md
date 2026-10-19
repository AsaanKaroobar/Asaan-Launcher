# CP008: Standalone command-line picker

Status: implemented, tested, locally installed and locally run. Public downloadable release pending. Version 0.8.0; branch checkpoint-8. User explicitly authorized CP008 after reviewing CP007. Public CP007 publication remains unverified; performance upgrades remain deferred.

Scope: stdin lines through --pick, optional --prompt/--query/--index, title/detail fuzzy filtering, native window keyboard/mouse selection and exact full-line/index output. This completes the picker behavior slice in phase 2 of the broader plan. Application process controls, trash, general file providers, extensions, search CLI and full settings remain later work.

## Behavior and parity

Blank and whitespace-only lines are dropped while original positions are retained. CRLF counts as one newline; other Swift newline characters are handled. Surrounding spaces and tabs remain in output. The first tab column is the title; nonempty remaining columns form the displayed detail. Title/detail fuzzy matches use the existing oracle-tested engine, with a ten-point detail penalty and original-position ties. Empty queries preserve input order. Non-UTF-8 input falls back to Latin-1.

The starting query filters immediately and its caret begins at the end. New queries reset selection, and stale native replies cannot replace the latest results or activate an old selection. Arrows, Page Up/Down (nine rows), Home/End, Return and Escape work; composing input retains Return/Escape. Click selects and double-click chooses. Return with no matches leaves the window open; Escape cancels immediately even with a query. Closing or settled focus loss cancels. Successful output is the original whole line or original zero-based index plus newline. Exit codes: 0 chosen, 1 cancelled/empty, 64 invalid usage/terminal stdin; I/O failures can return 74. Native choices resolve an input index against filtered original rows, never frontend-provided output text. A completion guard prevents duplicate writes.

Picker startup bypasses the resident, single-instance forwarding, catalog scans, preferences and global shortcuts. Automatic selection via ASAAAN_PICK_AUTO=1 runs before GTK/Tauri initialization and needs no display. Interactive WebViews use private browsing and an owned temporary data directory removed on exit. The picker can coexist with a running resident.

All 22 declarations from Floe PickerTests.swift have explicit equivalents: 17 pure model tests (including every original parameter set), five React session tests, plus encoding/CLI/stale-response/keyboard/native integration coverage. Immutable sources, hashes and mapping live in tests/reference/picker. Picker.swift, PickerPanel.swift, DebugOptions.swift and Shortcuts.swift remain pinned to reference commit 5b26e9d7a0240243251d7421bee5b90da5f52b9b. The reference checkout is unchanged. These translated assertions supplement the existing matcher/ranking oracle; the original macOS picker baseline has not been executed here.

## Validation

- 156 Rust tests pass, formatting passes, and Clippy passes with warnings denied.
- 77 frontend tests pass, TypeScript passes, and all pinned-reference mapping/hash checks pass.
- The installed package passes all ten picker native stages: exact text/index/column/space/CRLF/Unicode/Latin-1 output without a display; empty/no-match results; malformed options and real terminal stdin; initial filter/caret append; keyboard selection; Return with no matches then correction; Escape; native focus cancellation; unchanged application data and removal of temporary profiles; no remaining resident/panel processes from standalone use; and coexistence with the resident.
- The same installed package passes the resident keyboard/focus/lifecycle test (including five rapid grace-period reopens), native metadata clipboard/reveal actions with real Dolphin selection, alias/hide/restore migration, favorites/history/opt-out persistence, reference-search UI, native GIO application launch, three-process resident smoke and parent-pipe EOF checks.
- The same installed package is running on the real KDE/X11 desktop. [Native picker screenshot](008-native.png).

Installed validation caught WebKit creating cache/salt files in the normal application directory despite incognito mode. The final implementation supplies a private temporary directory as well; the unchanged-data and temporary-cleanup assertions pass. An intermediate package predated the final malformed-option guard and was rejected by the installed test, then rebuilt. A lifecycle recovery fixture also exposed a transient executable-copy ETXTBSY race: Unix now atomically publishes a symlink to the built fixture instead of copying executable contents. Its original first-failure/recovery assertions remain intact. Initial failures and final evidence are retained.

Native Windows/macOS/Wayland execution, clean-machine installs and remote CI remain unverified. Only Linux X11 is locally qualified. Floe's diagnostic picker snapshot environment variable and full native visual fidelity remain deferred; middle title truncation is not yet reproduced. No performance qualification or upgrades were attempted. Full Floe parity is not claimed.

## Local artifact

Asaaan Launcher 0.8.0 / CP008, Linux amd64. The Debian package is extracted into deployments/linux/current and the installed artifact is run locally.

Package SHA-256: 665a1ec973266c0398bb1abbb523df01d0e5027d419897d629d6e96a65f18c82.

Installed binary SHA-256: 5ed1eb739121ce42c82ef9fba8a03263f490af22f181b8758acdbe223d29e18a.

The ignored .artifacts/checkpoint-008 folder holds the validated package, installation manifest, SHA256SUMS, test/native logs and JSON, failure diagnostics, screenshot and a post-commit validation manifest with the exact source revision. Public downloadable release remains pending and user-controlled. Stop at CP008; CP009 has not started.

User explicitly approved advancing to CP009 on 2026-10-17. Public CP008 publication remains unverified.
