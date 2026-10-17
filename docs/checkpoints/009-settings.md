# CP009: Basic settings window

Status: implemented, tested, locally installed and locally run. Public downloadable release pending. Version 0.9.0; branch checkpoint-9. User explicitly authorized CP009 after CP008. Public CP008 publication remains unverified and user controlled. Performance upgrades remain deferred.

Scope: a separate settings process with General and Applications pages for existing search-history, alias and hidden-application preferences. Full settings, process actions, search CLI, extensions and later checkpoints are not implemented here.

## Behavior and ownership

Settings opens from the launcher header, tray Settings… or --settings, with optional --page general/applications. Unsupported page names and page flags without --settings exit 64 before GUI startup. A single settings child is reused and brought forward on repeated opens. Its last page is remembered within the resident session. Focus loss keeps it open; closing it or Ctrl/Command+W ends that child after pending operations finish. Resident quit and parent-pipe EOF also end it. A crashed settings child is gone until the next opening starts a new one. No settings WebView is created in the resident.

General exposes Remember searches and Clear search history. Opt-out retains existing queries; Clear removes queries while keeping usage and favorites. Applications shows the full catalog including hidden apps, supports filtering by name/alias/ID, explicit alias Save/Cancel, Hide/Restore and cleanup of stale aliases/hidden records for removed apps. Hiding removes a favorite; restoring preserves alias/usage. Nonempty alias edits and hide operations resolve native catalog IDs; clearing an orphan alias and restoring an orphan do not require the app to remain installed. Store validation still rejects invalid identities and malformed operations.

Both settings and panel private connections use one resident-owned personalization store behind a shared mutex. Requests mutate individual fields and return committed snapshots, so stale client state cannot overwrite unrelated edits. Identical request numbers on different private pipes remain independently correlated. Settings reads do not write preferences. Read failures display an error and Reload, without editable fallback defaults. Save failures retain existing values and alias drafts. Five-second/focus refreshes take launcher edits; pending drafts survive refresh and stale replies cannot replace an acknowledged mutation. The data format remains v2; no whole-snapshot save endpoint is exposed.

## Reference traceability

Pinned copies/hashes cover SettingsProcess.swift, SettingsMode.swift, ApplicationSettingsView.swift, SettingsView.swift, SettingsProcessTests.swift and SettingsReloadTests.swift at Floe 5b26e9d7a0240243251d7421bee5b90da5f52b9b. The reference checkout remains unchanged. The new 36-declaration map records 13 partial equivalents and 23 deferred declarations, including native lazy-process evidence. Existing maps and oracle fixtures remain intact.

The partial classifications are deliberate: only General/Applications and existing preference fields are available; the reference tests also name other pages, full AppSettings, autosave, imports, updater/recorder and unrelated stores. This checkpoint uses explicit alias Save/Cancel and acknowledged field operations rather than reproducing the full debounced AppSettings implementation. Other-window/extension-install exit gates, complete settings search, hotkey recording, permissions, import/export, updater, remaining pages and full native visual fidelity remain deferred. The original Swift/macOS settings baseline has not been run here.

## Validation

- 164 Rust tests pass, Cargo formatting passes and full workspace Clippy passes with warnings denied. New coverage includes process/page/exit policies and simultaneous settings/panel IPC mutations over three restarts with the same request IDs, preserving usage, favorites and aliases.
- 90 frontend tests pass, TypeScript passes and all pinned-reference checks pass. Settings coverage includes acknowledged opt-out/clear, initial/read/save failures, no optimistic reset, alias drafts across refresh/failure, stale response suppression, filtering, orphan restore, page/focus and native close routing.
- The installed package passes nine settings stages: lazy startup; CLI settings without opening search and no preference writes on read; one process on repeated opens; alias edits, orphan alias deletion, hide/restore and orphan restore; history controls preserving usage; focus independence and launcher alias visibility plus its Settings button; close/reopen with remembered page; crash recovery preserving preferences; resident quit and private parent-pipe EOF cleanup.
- The same installed package passes the prior resident keyboard/focus/lifecycle checks (including five rapid reopens), metadata clipboard/reveal with real Dolphin selection, alias/hide migration, favorites/history persistence, picker native checks, reference-search UI, native GIO application launch and resident/EOF smoke tests. Performance sampling was not run.
- The installed settings UI was visually checked on the actual KDE/X11 desktop. Screenshots use a private fixture catalog/history: [General](009-general-native.png), [Applications](009-applications-native.png). The delivered resident is then run with normal local preferences and Settings open.

Two initial settings observer failures were test-harness issues: a Promise used directly in an OR condition, and an accessible row label containing alias text rather than an exact title-only name. The observer now awaits both page checks and reads native text with the live alias query verified. Product settings behavior required no correction from those failures. Initial diagnostics and final success evidence are retained.

An interrupted regression run encountered ENOSPC on the workspace FUSE/NTFS volume despite free capacity/inodes. Removing only the unused 644 MiB debug build cache restored file creation. Source, release build, installed package and previous checkpoint evidence were preserved, and all interrupted phases were rerun successfully.

Only Linux KDE/X11 is locally tested. Windows/macOS/Wayland native execution, remote CI, clean-machine installation, signing and public publication remain unverified. Full Floe parity is not claimed.

## Local artifact

Asaaan Launcher 0.9.0 / CP009, Linux amd64. The tested Debian package is extracted into deployments/linux/current and that installed executable is running locally.

Package SHA-256: 8d57392033f2c57c08de56ab66e3f11f63ef1656d958e9413545b8df29f24e27.

Installed binary SHA-256: fdc4eec5bb148f4e8052151e3695e36d0d7d67c3776f704720bb2f79005fd0bf.

The ignored .artifacts/checkpoint-009 folder holds the package, installation, test/native logs and JSON, initial diagnostics, screenshots, SHA256SUMS and a post-commit manifest with the exact source revision. Public downloadable release remains user controlled and pending. Stop at CP009; CP010 has not started.
