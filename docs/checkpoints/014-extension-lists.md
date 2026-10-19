# CP014: Interactive extension Lists

Status: implemented, tested, locally installed and locally run. Public downloadable release pending. Version 0.14.0; branch checkpoint-14. User authorized advancing after CP013; its public publication remains unverified and user controlled. Stop after this checkpoint. Performance upgrades remain deferred.

## Scope

Normal launcher search now includes eight allowlisted bundled view commands. Extension commands share root ranking, aliases, favorites, hidden results and activation history with applications. The headless application search CLI remains application-only. Command identities are manifest-shaped extension/command IDs; arbitrary paths or scripts cannot be provided through the panel.

The resident owns one List session and supervises it through an independent Rust worker, which owns bundled Bun and its process tree. Closing or disposing the panel retains the session for Floe's default 90-second hidden resume interval. Reopening cancels reset and restores query/selection without another host or activation count. Ending a command, normal resident exit and resident pipe EOF stop the worker and owned host, including an unresponsive Bun event loop. This extends CP013 ownership; platform-native Windows/macOS cleanup remains unqualified.

Reference-v1 messages pass through the worker unchanged. The resident retains the decoding/recovery authority and subtree revisions; the panel receives a bounded, typed display snapshot. Action RPCs revalidate the session token, current query, visible selection and action-node revision. No arbitrary event prop, executable or file path is accepted from the frontend. Private command/query/tree packets are excluded from lifecycle logs.

The first display slice includes List sections, title/subtitle/keywords filtering, producer-handled search, selection, loading, EmptyView actions, text accessories, basic Globe/Star icons, primary/secondary Return actions, declared modified shortcuts, searchable action menus, nested submenus/onOpen, basic toasts, errors, Retry and Back. Grid nodes follow Floe's current List interpretation; a full Grid layout is not implemented. Clipboard text is owned by the resident and survives panel disposal.

The original Floe Planets source is copied unchanged into a separate bundled manifest without the original Hello extension's required preference. It exercises List rendering, favorite updates and navigation. Detail/Form display is explicitly unavailable in this checkpoint; a pushed Detail can be popped back to its List. General extension installation, required preferences/arguments, native API dispatch, rich icons/accessories, List detail sidebars, actionable toast UI, detached HUD display, scheduling/store, memory qualification and broader fidelity remain deferred. Unsupported native requests get correlated unavailable replies. Rich/concealed/file clipboard copies are explicitly refused.

## Traceability

Floe remains pinned at 5b26e9d7a0240243251d7421bee5b90da5f52b9b. tests/reference/extension-ui freezes eight sources and independently inventories 88 original Swift declarations: 24 scoped equivalents, five partial mappings and 59 explicitly deferred declarations. The 18 List/ActionState declarations are mapped; Detail's action-panel selection helper is covered without claiming Detail rendering. Added frontend and installed tests cover keyboard routing, menus, asynchronous replies and lifecycle. The original Swift suite has not run on Linux.

CP013's 59-source/376-declaration archive and the imported runtime assertions remain unchanged. The new checker verifies frozen hashes, independently re-enumerates Swift declarations, verifies mapped test names and checks the copied original Planets source byte for byte. This records scoped progress, not complete one-to-one UI parity.

## Validation

- 279 Rust tests pass, including 28 List/view equivalents, three virtual-clock resume cases, command persistence/identity and mixed root-catalog cases, plus typed extension RPC round trips/refusals. Cargo formatting and full-workspace Clippy pass with warnings denied.
- 120 frontend tests, TypeScript and every reference checker pass. All 275 original Bun runtime cases pass with 853 assertions; the CP013 portability patches and original assertions are unchanged.
- All 15 installed List stages pass: root discovery; AND filtering; selection/primary/secondary actions; resident-owned literal Unicode clipboard; nested menu/onOpen/modified shortcut and menu search; panel disposal/resume without another host or launch count; nonempty-query Back and cleanup; EmptyView action; producer search/loading/empty result; unchanged Planets favorite/push/pop; nonfatal/fatal errors; Retry with a new token and no extra count; abnormal exit; watchdog and hung-host normal quit; hard resident crash cleanup of Bun, supervisor and panel.
- The same final installed executable passes all 14 CP013 host stages and all prior native regression suites: Restart/Force Quit (11 stages), Hide/Quit identity (12), Settings (nine), metadata/clipboard/reveal (seven, including real Dolphin selection), customization, personalization, picker (ten), headless search (ten), resident keyboard/focus/rapid reopen, native launch/GIO activation and parent EOF. Performance qualification is skipped; resident-x11 records memorySkipped=true.
- The final installed resident runs on the actual KDE/X11 desktop, with Ctrl+Alt+Space registered and a 750×474 native window. Its running /proc executable hash matches the installation manifest. The native root window/version was visually inspected; the actual-preference image remains only in /tmp. List interactions were observed through native AT-SPI on private Xvfb desktops. Xvfb client-pixel readback was unreliable, so no screenshot is used as evidence of List pixel fidelity.

Initial acceptance exposed application-only identity validation rejecting command usage; command-shaped IDs and persistence regression tests cover the fix. Decimal-key injection was corrected in the private X11 fixture. Native accessibility testing also found that List option descendants are flattened; marked status now appears in the row's accessible name. Initial diagnostics are retained separately from passing final evidence.

## Local artifact

Asaaan Launcher 0.14.0 / CP014, Linux amd64, installed at deployments/linux/current.

Package SHA-256: eadc1c5807e216a3cd38b490c65de040775e8bedab5e4dbc21d73f1208342384.

Installed binary SHA-256: b1ef5e0d5abdfda1338c20b2fb0d3e15342d3ebceb8f6ef1b2e0e5e2f3d0d4db.

The ignored .artifacts/checkpoint-014 archive contains the durable Debian package, installation/runtime metadata, final native/frontend/runtime/build logs, per-suite JSON, initial diagnostics, desktop-run evidence, SHA256SUMS and post-commit validation with the exact source revision. Native acceptance uses this package and executable; the installed app does not depend on the disposable Cargo cache.


## Release gates

Public release, signing, remote CI, clean-machine installation and native macOS/Windows qualification remain pending. No CP015 work is authorized in this task.
