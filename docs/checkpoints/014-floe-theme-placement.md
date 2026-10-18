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

No cold-start speed improvement is claimed in this UI/placement change. The previous matched local benchmark was **815 ms median for a fresh panel**, versus **210 ms for reuse**. That fresh-panel benchmark has a resident already running and is not an OS cold-boot test. Prior native traces put window construction around 70 ms, frontend identity around 460–500 ms, and input focus around 650–710 ms before harness observation overhead. Lifecycle-hook and identity-bootstrap experiments did not produce a reliable end-to-end gain and were reverted.

Next incremental experiments, one at a time:

1. Load application icons only as result rows become visible, preserving fallback icons and keyboard scrolling. Compare time to focused input and painted results with the same installed-package harness.
2. Split Settings/Picker modules from the launcher's initial frontend bundle. Measure before claiming benefit; the current whole bundle is modest.
3. Move nonessential catalog refresh/extension discovery work behind first presentation, preserving fresh results and resident session restoration. Measure separately from WebView startup.
4. Optional background prewarming could remove much of first-opening WebView cost, but changes idle RAM and the current lazy-panel policy; it is not implemented by this change.

## Delivery status

Theme and placement implementation are in place. Frontend checks pass (125 tests), and installed Linux appearance acceptance passes all eight stages. Full native regressions and hosted matrix are in progress. Local package, native acceptance, hosted matrix, source revision and artifact hashes will be recorded after final verification. Public release remains user controlled.
