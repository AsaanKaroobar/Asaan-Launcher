# CP006: Application actions and search customization

Status: implemented, tested, locally installed and locally run. Public downloadable release pending. Branch checkpoint-6; product version 0.6.0.

The user reviewed CP005 and explicitly authorized this checkpoint. Performance upgrades remain deferred. Public CP005 publication is unverified. Every commit uses Dani-Bhai for author and committer.

## Scope

The selected application exposes Open, Set/Edit Alias, favorite and Hide from Search through an in-panel Actions menu. Control+K (Command+K on macOS), the Actions button and right-click open it. Arrow/Home/End navigation, Enter and Escape operate the menu. The alias editor is a focused form: Enter saves, Escape cancels; IME composition does not submit. Alias values preserve case, Unicode and literal spaces; empty removes an alias. Matching uses the existing Floe alias scoring, including exact-alias priority, prefix scores and title fallback.

Hiding stores the catalog's native display title and removes the favorite. Hidden IDs are filtered out of browse, suggestions and ranked/alias search. Customize (Control+comma / Command+comma) stays accessible with no results and lists saved hidden entries. Show Again removes the hidden marker, including for uninstalled applications, while retaining alias, usage and query history; it does not re-add a favorite. The resident remains the only writer. Stale catalog replies cannot undo an acknowledged customization change, and keyboard dispatch checks the live input before clearing or launching results. Native mutation types derive hidden titles from the catalog and reject unknown IDs for alias/hide/favorite; restore accepts valid saved IDs without requiring a currently installed app.

Checkpoint 5 version-1 personalization files are read into version 2 with empty alias/hidden maps and all previous values retained. Reads do not rewrite disk; the first acknowledged mutation atomically publishes version 2. Unsupported or corrupt files remain preserved. Older applications reject version 2 instead of overwriting it. The existing 2 MiB cap and private-pipe acknowledgement/work-lease model remain in place.

This provides a scoped Actions menu and customization dialog. Full Floe application actions (Finder/file reveal, clipboard operations, process hide/quit/restart/force-quit/trash), full settings/import/sync and other providers remain deferred. Set Alias is a shortcut into the alias editor; Floe edits app aliases in its Applications settings. This is functional translation, not full visual/settings parity.

## Validation

- Frontend: **58 tests** pass, including 17 customization/keyboard tests. TypeScript and all pinned-reference checks pass. Native: **123 tests** pass; formatting and Clippy pass with warnings denied.
- New customization map covers all 38 copied declarations: **8 ported, 2 partial and 28 deferred**. Existing search/lifecycle/application/personalization maps remain intact. The search oracle retains 775 matcher and 20 ranking fixtures. Floe remains unchanged at 5b26e9d7a0240243251d7421bee5b90da5f52b9b.
- The final installed package passes the normal-UI customization probe: seed CP005 version-1 data, upgrade on alias save, alias search/launch, hide/favorite removal, restore an uninstalled ID, clear aliases and ordinary title launch across **four resident restarts**. Saved usage, existing query history and recording opt-out survive.
- The same artifact passes the CP005 normal-UI favorite/history/opt-out/clear probe (three restarts/four launches), reference-search smoke, GIO marker launch, three resident foreground lifecycles and parent-pipe EOF cleanup.
- The final X11 keyboard probe verifies **five rapid grace-period hide/reopen cycles**, typed text and Escape clearing through AT-SPI, shortcut toggling, single-instance forwarding, focus return and renderer/helper cleanup across three foreground processes. Memory samples are explicitly skipped. Native screenshot: [installed Linux window](006-native.png). The final resident is running on the real KDE/X11 desktop with its shortcut registered.

Rapid reopen testing initially failed in both concurrent and serial sessions: GTK received an unmodified key while WebKit did not deliver a DOM keydown or input text after direct native hiding. The combobox accessibility focus flag alone was misleading because aria-activedescendant reports the selected row as focused. Input-context reset and GTK focus clearing did not fix the failure and were removed. Resident-requested hides now pass through the frontend event loop before unmapping, and the native acknowledgement accepts only the current sequence. Requested visibility is tracked independently, so newer toggles supersede pending hides. Presentation also reacquires editing focus; keyboard dispatch checks live input before clear/launch. The installed sequenced-hide build passes the acceptance run and the strengthened five-reopen probe. This is a functional regression correction, not performance qualification or proof of an upstream root cause. Temporary diagnostic commands and the experimental dependency are absent from the delivered source/package. Failed-run diagnostics are retained beside successful evidence.

No performance or stress qualification is claimed. CP003 memory/cold-start evidence and deferred upgrades remain unchanged. Remote CI is configured but execution is unverified. Native macOS, Windows and Wayland qualification remain pending.

## Artifact evidence

- Product/version: Asaaan Launcher 0.6.0, CP006, Linux amd64.
- Package SHA-256: 1c6aebdf3fa8a4a588e0c505228566c40bcafcdb8a5520b6c7a6e63e4aad2776.
- Installed binary SHA-256: cb954c8d5572536abc64bfd5af74cb1039de6518d0fd4c3fd02d1be505a503d0.
- Local staging: .artifacts/checkpoint-006/asaaan-launcher_0.6.0_amd64.deb, installation.json, SHA256SUMS, validation.json, native JSON/logs and screenshot. Package inspection excludes the test fixture executables. The staged package matches the installation manifest.
- The post-commit validation manifest records the exact source revision.

## Delivery

The tested Linux artifact is packaged, installed and running locally for user review. Public publication and native macOS/Windows/Wayland qualification remain independent pending gates. Stop after CP006; CP007 is not started.
