# Wayland and Hyprland support (CP014 follow-up)

The user asked for Wayland support and confirmed this Hyprland machine as the test device. The selected scope is core Wayland plus Hyprland window behavior, with `--toggle` and a compositor binding instead of a Wayland global-shortcut service. Running-app controls remain out of scope. Public releases remain user controlled.

Status: implemented, tested on the installed Linux 0.2.0 package, and run locally on Hyprland 0.56.2. Downloadable public release pending.

## Behavior

Wayland has no portable global-shortcut API. On Wayland the resident does not install the X11 global-shortcut backend; it reports a bind hint instead. `asaaan-launcher --toggle` is forwarded to the existing resident through single-instance and toggles the panel. The first invocation starts the resident and presents the panel. A compositor binding is the supported shortcut path, for example:

```ini
bindd = SUPER, SPACE, Asaaan Launcher, exec, asaaan-launcher --toggle
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

- Frontend: **125 vitest tests** passed; `npm run check` passed TypeScript and every parity/check script.
- Rust: **290 workspace tests** passed; rustfmt and strict all-target Clippy passed.
- Installed package: `Asaaan Launcher_0.2.0_amd64.deb`, 44.34 MiB, SHA-256 **b288d4f6914191ee968513c825ff55e2859f1f996811bdd97e82a7875f9797ea**.
- Installed binary SHA-256: **a03d9982f218e5e5bb66be778023d8f65a0ed9fb78e395bbd761c9da99001ef5**.
- Local install: `npm run install:local` extracted the package with `bsdtar` because this Arch host has no `dpkg-deb`; the manifest records `extractedWith: bsdtar`. `current` points at this package.
- Installed run: `npm run start:local`. Resident PID 154819; native panel PID 154841 ran with `--shortcut-wayland`. Hyprland reported `xwayland:false`, `floating:true`, `(393,208) 750x474` on the focused eDP-1 work area, and the panel held focus. An earlier run of the same test path also centered on the second monitor at `(1929,208)`. `--quit` stopped it cleanly.
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

- X11 native suites were not rerun locally because this host lacks Xvfb, KWin X11 and `xwininfo`. The X11 code paths remain unchanged except for shared `--toggle` argument forwarding; the Wayland-only presentation options are gated by session detection. Existing X11 native qualification and hosted CI remain the X11 gate.
- Hosted CI has not run for this change because the branch has not been pushed. CI remains X11-only; it does not exercise Hyprland.
- GNOME, KDE Wayland and other compositors are not tested. The compositor binding path is generic; Hyprland placement/focus is specific.
- Running-app controls, layer-shell panel behavior and Wayland global-shortcut portals remain unimplemented.
- Native macOS and Windows behavior is unaffected and not claimed.
- The downloadable public release of 0.2.0 is pending and user controlled. The latest public download remains 0.1.0.

## Delivery status

- **Implemented:** branch `checkpoint-15-wayland-hyprland`; final source commit recorded in the repository history.
- **Tested locally:** frontend 125, Rust 290, parity/type checks, installed Wayland main and window tests, three installed smokes.
- **Installed locally:** 0.2.0 Debian package extracted with `bsdtar` into `deployments/linux/current`.
- **Run locally:** installed resident and panel on Hyprland 0.56.2; screenshots inspected.
- **Downloadable public release:** pending and user controlled.
