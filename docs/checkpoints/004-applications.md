# CP004: Installed applications and launch

Status: implemented, tested, packaged, installed and running locally for user review. Public downloadable publication and native macOS/Windows qualification remain pending. Branch checkpoint-4; product version 0.4.0.

The user reviewed CP003, explicitly approved CP004 and deferred performance upgrades until the complete software is ready. CP003 memory/startup findings remain recorded and do not block this transition. Public CP003 publication is historically unverified. Every new commit must use Dani-Bhai for both author and committer.

## Scope

The normal launcher now searches installed applications. Enter opens the selected application, a single click selects, and a double-click opens that row. The panel hides after the operating system accepts the launch request. Failed launches keep the query/results visible with an error for retry. IME composition keys are ignored and repeated Enter is suppressed while launch is pending.

X11 show requests realize/map the GTK window and include a timestamped EWMH activation message identifying [direct user intent](https://specifications.freedesktop.org/wm/latest/ar01s09.html). The shortcut toggles on key release so hiding cannot interrupt an in-flight modifier key event in WebKit. Native focus-gained events refocus the search field; launcher key routing also works during webview remapping. A private KDE/X11 test session exercises shortcut reopen, Escape, focus return and process cleanup without sharing the user’s keyboard or preferences.

The foreground loads the catalog in a worker; the resident does not scan. Snapshots are retained while refreshing. Manual refresh and a five-second foreground polling interval discover installed/removed apps; a new foreground starts with a fresh scan. Queries use the existing Floe-equivalent fuzzy/ranking core. Same-name apps remain separate, with an origin label; duplicate identities collapse. No sample command rows appear during normal use. The reference catalog remains available through the existing diagnostic smoke role.

Native launch receives only a catalog ID. Rust resolves it to its stored target and revalidates Linux entries before launching; no arbitrary path, shell command or Exec string is accepted from the webview. Finite-work leases cover pending scan/launch work. Spawned application stdout/stderr and stdin are detached from the private panel protocol. A successful launch means OS delegation was accepted; it does not prove another application's window rendered.

## Platform adapters

- Linux: GIO desktop discovery and launching. Native XDG precedence, localized display names, visibility rules, TryExec, Terminal/DBus activation and field-code interpretation remain with GIO. Linux application icons are loaded through the GTK icon theme and passed as PNG data; missing icons use a letter fallback.
- macOS: application roots plus one-level child folders, bundle metadata (XML/binary Info.plist), identity deduplication and /usr/bin/open -a with an explicit argument. Finder/localized display names, native icons and packaged focus/launch qualification remain pending on macOS.
- Windows: [Get-StartApps](https://learn.microsoft.com/en-us/powershell/module/startlayout/get-startapps) metadata includes Start-menu and packaged-app identities, queried with a constant encoded PowerShell script, no profile and a twenty-second timeout. AppsFolder identities are passed directly to Explorer without a command shell. Native icons and actual Windows execution remain pending qualification.

Other providers, extension host, built-in tools, per-app action menus, favorites/usage persistence and settings are later checkpoints. This checkpoint provides the primary open-application action.

## Parity and tests

Pinned source: Floe 5b26e9d7a0240243251d7421bee5b90da5f52b9b. All 25 original declarations in the scoped AppScan/CatalogLoader/LauncherCatalog suites are mapped under tests/reference/applications: 15 ported application equivalents, 2 partial, 8 deferred. The map explains substitutions and gaps; it does not mark mixed extension/builtin cases complete. Existing 38 ranking declarations, frozen Swift search fixtures and lifecycle mappings are retained.

Validation on Linux, 2026-10-16:

- npm run check passed: typechecking, frozen search/lifecycle/application provenance checks, and 33 frontend tests.
- npm run native:check passed: formatting, 95 Rust tests and clippy with warnings denied. This includes 18 application catalog tests and 6 native XDG fixture tests.
- The packaged, locally installed binary passed reference-search, actual isolated application-launch (including literal arguments and Exec field codes), three resident presentation/search/dismiss/reap cycles, and private-panel EOF checks.
- Private KDE/X11 keyboard regression passed three cold opens, singleton forwarding, same-process shortcut hide/reopen during the grace period, physical typing, Escape clear-then-hide, focus restoration and exit of every observed UI helper process. Memory sampling was explicitly skipped; CP003 measurements remain the performance baseline.
- The installed normal application was opened on the user's Linux desktop; [native screenshot](004-native.png) records the real catalog and Linux icons.
- The pinned Floe working tree remains unchanged.

Artifacts are staged locally under .artifacts/checkpoint-004 with validation.json, installation.json, test logs and SHA256SUMS. The validation manifest records the source commit; staging is not publication.

Package: asaaan-launcher_0.4.0_amd64.deb

Package SHA-256: 8a940cf583684a98e20f00d6d977aaeef218a83ca2979aba47522085848942de

Installed binary SHA-256: 15e1bb3b411c3544471802fb25de17561438e67d0b7e35d3b6272f7057a2fe99

Windows/macOS execution, Wayland packaged qualification, Finder display-name fidelity and native Windows/macOS icons remain unverified or deferred as specified above. Linux test success is not certification of those targets.

## Delivery gates

Package, install and run the validated Linux artifact locally for user review. Public publication and native macOS/Windows qualification remain independent pending gates. Performance upgrades are explicitly deferred by the user's instruction. Stop after CP004; CP005 is not started.
