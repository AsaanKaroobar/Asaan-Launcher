# CP007: Application metadata actions

Status: implemented, tested, locally installed and locally run. Public downloadable release pending. Branch checkpoint-7; version 0.7.0. User explicitly authorized CP007 after CP006. Public CP006 publication is unverified; performance upgrades remain deferred.

Scope: catalog-resolved Copy Path, Copy Name, Copy platform application identifier and native Reveal. Copies preserve the query and show acknowledged feedback; reveal dismisses/reset search only on native success. No Copy File, process control, trash or general file provider. Linux copies/reveals the desktop entry; macOS uses the bundle; Windows StartApps IDs support name/ID copies without inventing a filesystem path. This is scoped platform equivalence; native Windows/macOS qualification remains pending. The name is the native catalog display title (macOS Finder display-name fidelity still requires target qualification).

The resident owns text clipboard writes, including after foreground dismissal and disposal. Typed private pipe requests are bounded and acknowledged without logging text. No usage/history/personalization mutation for metadata actions. Linux reveal uses FileManager1.ShowItems with the file URI, macOS uses open -R with literal arguments; unavailable providers fail visibly. No folder-open fallback is counted as selected-item reveal.

## Implementation and parity boundaries

The menu preserves Open first, the CP006 alias shortcut and favorite/hide suffix. Native descriptors offer only valid identity actions: Linux desktop entry path/name/ID; macOS bundle path/name/identifier (identifier only when present); Windows name/AppID. Windows StartApps IDs are not filesystem paths, so filesystem copy/reveal is omitted there. General file menus, Copy File, Quick Look, process hide/quit/restart/force-quit, trash and full visual/menu hierarchy parity remain deferred. macOS Finder display-name byte fidelity still needs native qualification; CP007 copies the native catalog title.

The native command accepts a closed action enum and an opaque catalog ID. Extra paths/text/commands are rejected. Copy text is bounded to 64 KiB of UTF-8, excludes NUL, and crosses a typed resident request/reply with matching IDs and a five-second deadline. The resident rejects expired requests and rechecks the deadline before dispatching queued Linux clipboard writes. Work leases cover native operations and pending replies. Clipboard text/metadata is excluded from resident protocol logs. Copies preserve the query and show feedback after acknowledgement. Reveal uses a literal file URI through FileManager1.ShowItems on Linux, or argument-separated open -R on macOS; missing targets/providers/refusal/timeout remain errors. No parent-folder fallback is counted as selection.

## Validation

- **137 Rust tests pass**, formatting passes, and Clippy passes with warnings denied. Coverage includes typed metadata/capabilities, UTF-8 limits, deadlines, forbidden fields/actions, opaque IDs, missing paths and three foreground clipboard IPC restarts.
- **67 frontend tests pass** with TypeScript and every pinned-reference check. The new map accounts for all **14 ItemActionTests declarations: 1 ported, 2 partial and 11 deferred**. Earlier maps and the 775 matcher/20 ranking fixtures remain intact; the Floe checkout is unchanged at 5b26e9d7a0240243251d7421bee5b90da5f52b9b.
- The installed normal UI passes clipboard path/name/ID checks using Unicode, spaces and semicolons; native acknowledgement feedback; missing target/provider refusal; provider timeout while hidden with a retained work lease; foreground disposal; clipboard retention in the resident; and accepted reveal without activation. The personalization file remains byte-identical and the fixture app is never launched.
- A real Dolphin FileManager1 provider selects the catalog desktop entry in a private session. Native screenshots: [launcher Actions menu](007-native.png) and [Dolphin selection](007-reveal-native.png).
- The final installed package passes the five rapid grace-period reopen keyboard probe. The CP005/CP006 customization/history probes, reference-search smoke, native GIO application launch, resident three-process lifecycle and parent-pipe EOF checks all pass on the same installed package.

Installed testing caught a native string error being replaced by a generic UI message; the UI now preserves the native reason, with a regression test. The initial accessibility status observer needed the text interface rather than only accessible names. Fixture teardown was also corrected to recognize signal exits and close inherited process pipes. Failed-run and teardown evidence is retained with final success evidence. These fixture corrections do not change product behavior.

No performance qualification or upgrades were attempted. Native macOS/Windows/Wayland execution and remote CI remain unverified. Windows selected-item reveal is explicitly unavailable in this checkpoint. Full Floe parity is not claimed.

## Local artifact

Product: Asaaan Launcher 0.7.0 / CP007, Linux amd64. Installed by extracting the validated Debian package into deployments/linux/current, then run on the real KDE/X11 desktop with Ctrl+Alt+Space registered.

Package SHA-256: 3fb6aa86530aab98806a4c7a6966ad6406769c2b36982a0860e87df15c973daf.

Installed binary SHA-256: 493896e1ef065853fb2a2bd658ebc79acb30f76210cd734e70895e9047f600b2.

The ignored .artifacts/checkpoint-007 directory holds the package, installation manifest, SHA256SUMS, test/native logs and JSON, diagnostic failures, screenshots and a post-commit validation manifest with the exact source revision. Public publication remains user-controlled and unverified. Stop after CP007; CP008 has not started.
