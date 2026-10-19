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
