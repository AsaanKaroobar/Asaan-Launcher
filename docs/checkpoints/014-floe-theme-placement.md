# Floe theme and focused-monitor placement (CP014 follow-up)

The user authorized removing launcher control chrome, matching Floe aesthetically, retaining the former appearance as **Legacy**, and centering on the focused monitor. CP015 extension features and public releases remain outside this delivery.

## Behavior

**Floe** is the default. The main launcher has no native title bar, branded control header, shortcut HUD, connection status footer or version label. Its 750 × 474 content, inset 43-pixel search capsule, 38-pixel result rows, section headings, neutral selection, and bottom Settings/actions bar follow the pinned reference. Existing history, refresh and customization controls are available through Search tools. **Settings → General → Launcher theme** selects Floe or Legacy. Legacy restores the prior green appearance, branded header and connection footer. Acknowledged preferences are persisted atomically by the resident; failed saves retain the previous theme. Reads of existing preference files default to Floe without rewriting them. Older binaries that reject unknown preference fields cannot read files after this version saves the theme; downgrade is not qualified.

Floe uses system light/dark appearance. macOS Liquid Glass, SF Symbols and platform fonts are not pixel-identical on other systems: this delivery uses an opaque, appearance-aware cross-platform material. Native glass rendering is not claimed. AGPL/GPL source attribution is retained in the adapted styling.

The active application's window is examined before the launcher takes focus on every presentation, including reuse. Linux/X11 reads EWMH active-window geometry; Windows reads the foreground window bounds; macOS reads the frontmost normal window's CoreGraphics metadata, without requesting screen capture or Accessibility permission. The selected monitor's usable work area is centered using target DPI and preserved logical panel dimensions. Missing foreground geometry falls back to the pointer monitor, then primary display. Wayland restricts global focus/pointer geometry and absolute positioning; final placement is compositor controlled there.

Native GTK tests exposed stale pre-map outer dimensions (including the default 640 × 480 surface before Settings maps). Placement therefore uses requested content dimensions, matching Tao’s normal centering convention. Native acceptance checks both the undecorated launcher and Settings geometry. Pixel tests also exposed stale/blank frames in the private Xvfb session on a host with both NVIDIA and Mesa EGL providers. The test harness now explicitly selects Mesa and uses normal WebKit compositing, removing the inherited legacy WebKit fallback overrides. This restores continuous painting on WebKitGTK 2.52.6; no timing, resize, surface recreation or graphics override is added to the application. Native acceptance passes eight stages, including initial painting, live input repainting, visible Legacy selection, reuse and a fresh panel.

The standalone picker retains its original visible startup. Its initial input focus and end-of-query caret are applied after the React DOM commit, without depending on an animation frame; this fixes the native append-at-caret regression exposed by normal WebKit compositing. The main launcher’s single-row grid is scoped away from the picker, whose header/search/footer layout is retained and checked using native field coordinates and a screenshot. Existing picker exit, Unicode, cancellation, focus-loss and coexistence tests pass all ten stages.

## Reference traceability

Pinned Floe revision: **5b26e9d7a0240243251d7421bee5b90da5f52b9b**, unchanged.

| Reference | Equivalent verification |
| --- | --- |
| Launcher/LauncherLayout.swift | 750 × 474 native geometry and centering assertions; placement arithmetic tests |
| Thaw/SearchPanelStyle.swift; Launcher/Views.swift | Native search-field pixel evidence, compact row/field metrics, default chrome absence |
| Launcher/RowSelection.swift | Section headings and existing keyboard/selection regressions |
| Launcher/PanelBottomBar.swift | Accessible Settings gear, retained actions, tools/history native test |
| Launcher/LauncherAppearance.swift; ThawUI tokens | Appearance-aware default and persisted Legacy selection; glass fidelity explicitly partial |

## First-launch optimization assessment

The final matched five-sample Linux benchmark measured **736 ms median / 816 ms maximum (sample p95) to focused input for a fresh panel**, versus **838 ms / 860 ms** for the prior installed reuse build under the same corrected Mesa harness. Reopening measured **212 ms median** for both builds, with reuse verified in all five cycles. DOM presentation medians were 715 ms final versus 815 ms baseline; observed results medians were 906 ms versus 1081 ms. These small sequential samples are encouraging, but do not establish a repeatable cold-start improvement or meet the sub-500 ms fresh-panel target. The resident is already running, filesystem caches are not flushed, tracing is enabled, and shortcut adapter/observation overhead is included. This is neither an OS cold-boot test nor an exact first-painted-frame measurement. Earlier measurements under the old graphics harness are not directly comparable.

Native traces put window construction around 70 ms and substantial remaining time in WebView/frontend startup. Lifecycle-hook and identity-bootstrap experiments did not produce a reliable end-to-end gain and were reverted. No prewarming or unverified startup workaround is shipped.

Next incremental experiments, one at a time:

1. Load application icons only as result rows become visible, preserving fallback icons and keyboard scrolling. Compare time to focused input and painted results with the same installed-package harness.
2. Split Settings/Picker modules from the launcher's initial frontend bundle. Measure before claiming benefit; the current whole bundle is modest.
3. Move nonessential catalog refresh/extension discovery work behind first presentation, preserving fresh results and resident session restoration. Measure separately from WebView startup.
4. Optional background prewarming could remove much of first-opening WebView cost, but changes idle RAM and the current lazy-panel policy; it is not implemented by this change.

## Delivery status

- **Implemented:** runtime source `177209865f8bcfc0281bbb82f87429a7c827cf52` on `checkpoint-14-floe-theme-placement`; all new commits use Dani-Bhai for author and committer. A subsequent documentation-only commit records delivery without changing the qualified runtime source.
- **Tested locally:** 125 frontend tests, TypeScript/parity checks, 285 Rust tests, rustfmt and strict all-target Clippy pass. Final installed-package regressions pass extension host/search CLI, 16-stage extension UI, process controls/processes, Settings, actions, customization, personalization, ten-stage picker, native/application/resident smoke and resident lifecycle. Eight-stage native appearance checks include live painting, theme switching, reuse/fresh presentation and Settings geometry.
- **Hosted qualification:** [run 39170254436](https://github.com/AsaanKaroobar/Asaan-Launcher/actions/runs/39170254436) passes all three jobs for the exact runtime source: Ubuntu 24.04 `.deb`, macOS 14 `.dmg`, and Windows 2022 NSIS. These establish native checks and packaging; interactive Windows/macOS GUI, mixed-DPI monitor placement and Wayland behavior have not been manually qualified.
- **Installed locally:** version 0.14.0, package `Asaaan Launcher_0.14.0_amd64.deb`; SHA-256 `86e87ff12c21d3b7a09f642cc9ac0598eefd58c9be3b709efcaa7f60ef2a1cf4`. Installed binary SHA-256 `092ead881f19c8243de7ae3d6bbcd64f3dbe22a108b0b4d49f2c9d892a3b3c7f`. `deployments/linux/current` points to this version.
- **Run locally:** the final installed resident executable hash matches the package. On the real two-monitor Linux desktop, a focused application on the right display yields launcher bounds `(2505,281,750,474)`, exactly centered in that display’s work area. Earlier candidate evidence also exercises the nonprimary left display; it is separately labeled and is not final-package evidence. Synthetic input is confined to private test displays.
- **Evidence:** `.artifacts/floe-theme-placement/final/` contains selected final logs, screenshots, matched benchmarks, installation/source manifests, CI results, the local package and `SHA256SUMS`. Previous CP014/platform/performance sealed archives retain all 244 recorded file hashes. Floe reference remains pinned and unchanged.
- **Downloadable public release:** pending and user controlled. No public release, merge or CP015 feature work was performed.

## Windows focus repair (2026-10-22)

Scope: the CP014 launcher on Windows interrupts typing and immediately closes Actions. The user authorized committing and pushing this repair on a new branch and running the GitHub build matrix before moving to publication. No later checkpoint features are included.

The native `Focused(true)` handler reacquired WebView focus and emitted `panel-shown` for the current resident sequence. Repeated notifications caused the frontend to blur/refocus the search and close overlays even when the resident had not requested another presentation. Native editing focus is now acquired by the explicit presentation path only. The frontend ignores duplicate presentation sequences before changing focus, closing overlays or exchanging extension state. A new sequence still performs the existing reopening behavior.

Regression traceability: `src/SearchPanel.test.tsx`, “duplicate native focus notifications preserve typing and an open actions menu”, checks caret/query preservation, an Actions menu surviving repeated notifications, one readiness acknowledgement, and normal focus/menu reset on the next presentation. It failed before the fix because the repeated notification called `blur()` again, then passed after the fix. Existing `src/Applications.test.tsx` reopening coverage checks retained query, selection identity and refreshed catalog. These are local port regressions supplementing the existing pinned Floe lifecycle/selection mappings, not a new claim of native reference parity.

Delivery gates for this repair (separate from historical Linux delivery above):

- **Implemented:** Windows focus repair `98f864f4376369a4d5cf93a8dda3272b987f55d8` on `checkpoint-14-windows-focus`, based on `a3dcfd07fd33a2ea9000706a649a47ef3b79895b`. Author and committer are Dani-Bhai. A subsequent documentation-only commit records hosted qualification without changing the validated runtime source.
- **Setup:** Node 24.20.0/npm 11.19.0, `npm ci`, bundled Bun 1.4.3 and frozen extension dependencies prepared on Windows x64. Visual C++ build tools are present. The named Rust 1.95.0 toolchain is installed and verified. Native workspace tests used the existing `stable` alias, verified to resolve to exactly the pinned compiler, `rustc 1.95.0 (59807616e 2026-04-14)`, while the named toolchain initialized.
- **Tested:** `npm run check` passes all parity checks and 126 frontend tests; `npm run build` passes; `npm run runtime:test` passes 277 tests, with three explicitly POSIX-only signal tests skipped. `cargo +stable test --workspace --locked --release`, `cargo +stable fmt --all --check`, and `cargo clippy --workspace --locked --release --all-targets -- -D warnings` pass on Windows. `npm run tauri build -- --bundles nsis` succeeds.
- **Installed locally:** the NSIS installer exits 0, installing into `deployments/windows/focus-repair/`. Installer SHA-256: `da76942f5ecfb39c4375537930567ace39a6ff948098320da570624591c5af5e`. Installed executable SHA-256: `e053aa98aa9475f7b04d6f801590223e08be19220d782614843191d231a7e79d`. Tauri's NSIS bundle patches exactly three bytes of the build executable, changing its bundle-type marker from `UNK` to `NSS`; the installation manifest records the differing offsets and both hashes.
- **Run locally:** installed native smoke passes reference-result rendering, DOM-focused presentation, exactly one readiness acknowledgement per presentation, hide/reopen in the same panel and exit after private-pipe EOF. The installed resident starts as PID 21308, registers Ctrl+Alt+Space, and presents its supervised panel. The user explicitly confirms “Typing and Actions both work” after being asked to test search, Ctrl+K and hide/reopen. This qualifies the reported Windows issue on this desktop; mixed-DPI placement and other platforms were not requalified by this repair. Computer Use window enumeration was unavailable, so interactive acceptance is user-observed. The smoke run exits 0 with a WebView2 shutdown diagnostic (`Chrome_WidgetWin_0`, error 1412); no assertion fails.
- **Evidence:** `.artifacts/windows-focus/installation.json`, `panel-smoke.json`, the smoke harness and resident logs retain artifact hashes, native protocol/startup observations and validation results. The installed binary remains running for the user. The local manifest records the precommit source base; the branch commit records the repair source.
- **Hosted qualification:** all three jobs pass [run 39804282489](https://github.com/AsaanKaroobar/Asaan-Launcher/actions/runs/39804282489) for exact source `98f864f4376369a4d5cf93a8dda3272b987f55d8`: Windows 2022 x64 NSIS, macOS 14 ARM64 DMG, and Ubuntu 24.04 amd64 DEB. Each passes frontend/runtime/native checks and packaging. Ubuntu also passes the full installed-native suite, including appearance, live input, actions, picker, Settings, process controls, extension Lists, resident focus/reopening and bounded reuse. All installers, completed logs, run/artifact metadata and Linux native evidence were downloaded into `.artifacts/windows-focus/`.
- **Exact CI Windows artifact installed/run:** NSIS installation exits 0 into `deployments/windows/focus-repair/`. The installed executable SHA-256 is `c2e8ef9fef32d48d6e3226f1d12368e52de360dee4ce8b10bfc954f05265e227`. The installed GitHub artifact passes the same native focused-presentation/reuse/EOF smoke (`ci-panel-smoke.json`). Resident PID 36684 starts from this installation, registers Ctrl+Alt+Space and presents its panel (`ci-resident.stderr.log`). Earlier manual typing/Actions acceptance above applies to the local build of the same runtime changes; it is not a second manual acceptance of the CI binary. macOS desktop acceptance remains pending.
- **Downloadable public release:** pending and user controlled. No release is published by this repair.

Downloaded CI installer hashes (validation artifacts retain version 0.1.0; a new public patch release must update version metadata and qualify its resulting artifacts):

| Platform | SHA-256 |
| --- | --- |
| Linux amd64 DEB | `5a6a9f555b8723894be9059c60e02f4e49dc055e0f7250caa4627d54fb6e9cff` |
| macOS ARM64 DMG | `f3d9ff926f8e33495881538693c04960fd3a83571344c9694277bb190204d7a4` |
| Windows x64 NSIS | `efe7f62f04885028442b884721a858b7826194e30302d70202a6c1d803069c53` |
