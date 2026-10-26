# Wayland and Hyprland support (CP014 follow-up)

The user asked for Wayland support and confirmed this Hyprland machine as the test device. The selected scope is core Wayland plus Hyprland window behavior, with `--toggle` and a compositor binding instead of a Wayland global-shortcut service. Running-app controls remain out of scope. Public releases remain user controlled.

Status: implemented, tested on the installed Linux 0.2.0 package, run locally on Hyprland 0.56.2, and published as [Asaaan Launcher 0.2.0](https://github.com/AsaanKaroobar/Asaan-Launcher/releases/tag/0.2.0) on 2026-10-26. The Windows focus repair from 0.1.1 is included.

## Behavior

Wayland has no portable global-shortcut API. On Wayland the resident does not install the X11 global-shortcut backend; it reports a bind hint instead. `asaaan-launcher --toggle` is forwarded to the existing resident through single-instance and toggles the panel. The first invocation starts the resident and presents the panel. A compositor binding is the supported shortcut path. The default binding is **Ctrl+Alt+Space**, matching the X11 shortcut; for example:

```ini
bindd = CTRL ALT, SPACE, Asaaan Launcher, exec, asaaan-launcher --toggle
```

The tray remains available. The Legacy header and tray status show the same Wayland hint.

Hyprland exposes a local IPC socket for window actions. The resident/panel uses it only for its own window, matched by process ID:

- `j/clients` finds the mapped window owned by the current process.
- A window hidden on another workspace is moved to the active workspace before presentation.
- `setfloating`, `resizewindowpixel`, `movewindowpixel` and `focuswindow` float, size, center in the monitor's reserved-aware usable area and focus the window.
- The launcher is 750 × 474, Settings is 960 × 640 and the picker is 750 × 474.
- The adapter uses the local Unix socket directly. It does not invoke `hyprctl` or a shell, and it never targets another application's window.
- Other Wayland compositors keep their own placement/focus policy. The Wayland presentation fixes still apply; Hyprland-specific actions do not.

Three WebKitGTK/Wayland presentation defects were observed on Intel UHD 620 and corrected only on Wayland:

- The first hidden-created surface could be transparent and blend with the window behind it. An explicit opaque window background keeps the material opaque.
- The same surface could remain on its pre-search frame after results arrived. A one-pixel height configure round-trip after show forces a fresh buffer; the final size is restored and the window is centered afterward.
- GTK's natural pre-map size could briefly exceed the requested content size (observed at 768 × 828). A Wayland-only maximum inner size prevents that flash before the requested size applies.

The picker now reacquires its search-field focus when the native window gains focus, with a short post-load retry. On Wayland the picker window is active before WebKit's initial layout; without the retry, keys reached the window but not the field.

Wayland sessions continue to report running-app Hide/Quit/Restart as unavailable. The existing X11 identity checks fail closed on Wayland; no compositor-specific process control was added.

## Verification

Environment: Hyprland 0.56.2, native Wayland (`xwayland: false`), two monitors at scale 1.25, Waybar tray and the Hyprland portal present. The tests operate on the live session, show/hide only their own launcher windows, and send keyboard events only to the launcher's Hyprland address.

- Frontend: **126 vitest tests** passed after the Windows focus repair was merged; `npm run check` passed TypeScript and every parity/check script.
- Rust: **290 workspace tests** passed; rustfmt and strict all-target Clippy passed.
- Installed package: CI-produced `Asaaan-Launcher_0.2.0_linux_amd64.deb`, 44.34 MiB, SHA-256 **63c778fc0d7f7726b2da9c383a1d2f6e302335de72f501ff6d8e0709ce9a4978**.
- Installed binary SHA-256: **0f0d99687635f2a00612e479a7a868f650a97c26760c28048d6bebb1ba22399e**.
- Local install: the exact CI Linux installer was extracted with `npm run install:local` using `bsdtar` because this Arch host has no `dpkg-deb`; the manifest records `extractedWith: bsdtar`. `current` points at this package.
- Installed run: the exact CI Linux installer passed the 7-stage Wayland/Hyprland test, the 2-stage picker/Settings test, and the native frontend, application-launch and resident smokes. The resident reported `ASAAAN_SHORTCUT Wayland · bind asaaan-launcher --toggle`; the panel was native Wayland (`xwayland:false`), floating, `750x474`, correctly centered on the focused monitor, and focused. An earlier local integrated-build run also centered on the second monitor at `(1929,208)`; it is historical and not the release artifact.
- `npm run test:wayland:hyprland` (7 stages): Wayland resident startup; native/floating/size/centered/focused panel; search results painted on the first hidden-created presentation; `cal` typed through Hyprland key delivery and filtered; Escape cleared then hid; grace-period `--toggle` reused the same panel process and regained focus; focus returned to the previous window and quit removed the panel.
- `npm run test:wayland:windows` (2 stages): the standalone picker was native Wayland, floating, 750 × 474, centered and focused, typed `dev`, and returned `Development`; Settings was native Wayland, floating, 960 × 640, centered and focused, then closed on resident quit.
- `npm run smoke:local` passed the installed native frontend/Tauri/Rust `cal` search.
- `npm run smoke:applications` passed an isolated native GIO launch on Wayland.
- `npm run smoke:resident` passed three resident panel cycles, search verification and panel IPC cleanup.

Evidence is under the ignored `.artifacts/wayland-hyprland/`: `evidence.json`, `windows.json`, native screenshots, the installed-desktop screenshot, and the existing smoke JSONs.

Reproduce after installing the package:

```sh
npm run test:wayland:hyprland
npm run test:wayland:windows
```

Both commands require a live Hyprland Wayland session and skip if another resident is running. The screenshots demonstrate native rendering and geometry; they are not a pixel-fidelity certification.

## Limits and outstanding gates

- X11 native suites were not rerun on this local Arch host because it lacks Xvfb, KWin X11 and `xwininfo`. The Ubuntu CI job ran the full installed-native X11 suite at the release source and passed; the X11 code paths remain unchanged except for shared `--toggle` argument forwarding, and the Wayland-only presentation options are gated by session detection.
- Hosted CI passed all three platform jobs for `7b901fa` ([main run](https://github.com/AsaanKaroobar/Asaan-Launcher/actions/runs/40280852894)); the Wayland and Windows branch runs for the same commit also passed. CI is X11-based on Linux and does not exercise Hyprland; the local Wayland tests above are the Hyprland qualification.
- GNOME, KDE Wayland and other compositors are not tested. The compositor binding path is generic; Hyprland placement/focus is specific.
- Running-app controls, layer-shell panel behavior and Wayland global-shortcut portals remain unimplemented.
- macOS and Windows CI packaging passed for 0.2.0; manual 0.2.0 desktop qualification remains pending. The Windows focus fix was interactively qualified for 0.1.1 and is included here.
- The 0.2.0 downloadable public release is published and user controlled. The latest public download is 0.2.0.

## Delivery status

- **Implemented:** final release source `7b901fa927b91433b5d9534119f8f3dea829e249`, combining Wayland commit `82e8b55` with Windows focus repair `98f864f`. `main`, `checkpoint-15-wayland-hyprland` and `checkpoint-14-windows-focus` were synced at this commit. A subsequent documentation-only commit records the delivery without changing the qualified runtime source.
- **Tested locally:** frontend 126, Rust 290, parity/type checks, installed Wayland main and window tests, three installed smokes against the exact CI Linux installer.
- **Installed locally:** the exact CI 0.2.0 Debian package extracted with `bsdtar` into `deployments/linux/current`; package SHA-256 `63c778fc…a4978`, binary SHA-256 `0f0d9968…2399e`.
- **Run locally:** installed resident/panel, picker and Settings on Hyprland 0.56.2; native Wayland rendering and geometry inspected.
- **Downloadable public release:** [0.2.0](https://github.com/AsaanKaroobar/Asaan-Launcher/releases/tag/0.2.0) published on 2026-10-26; assets and SHA-256 checksums downloaded anonymously and verified.
