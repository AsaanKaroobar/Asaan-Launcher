# Asaaan Launcher

**A keyboard-first desktop launcher for Linux, macOS, and Windows.**

Find applications, run commands, and get back to work without breaking your flow. Asaaan Launcher combines fuzzy search, personal shortcuts, and an extensible command interface in a focused desktop experience.

Built with **Tauri 2 · Rust · React · TypeScript**. The original idea was inspired by [Floe](https://github.com/thaw-app/Floe).

[Getting started](#getting-started) · [Features](#features) · [Development](#development) · [Documentation](#documentation)

## Project status

Asaaan Launcher is under active development. The current source version is **0.2.0** (unreleased), developed through tested checkpoints with explicit implementation, validation, local installation, and release gates. The latest public download is **0.1.1**.

| Platform | Build and packaging | Interactive validation |
| --- | --- | --- |
| Linux | Ubuntu CI; Debian package | Native X11 regression tests; Hyprland Wayland tests; local desktop use |
| macOS | macOS CI; disk image | Manual desktop qualification pending |
| Windows | Windows CI; NSIS installer | Native focus/reopen smoke; broader desktop qualification pending |

Download **[Asaaan Launcher 0.1.1](https://github.com/AsaanKaroobar/Asaan-Launcher/releases/tag/0.1.1)** for Linux x64, Windows x64, or macOS Apple Silicon. The release includes installer checksums. Development builds and validation logs remain available through [GitHub Actions](https://github.com/AsaanKaroobar/Asaan-Launcher/actions/workflows/verify.yml). See the [checkpoint overview](docs/CHECKPOINTS.md) for current delivery gates.

## Features

- **Application search** — fuzzy matching across installed applications, aliases, and keywords, with favorites and usage-based ranking.
- **Personalization** — save favorites, assign aliases, hide results, recall searches, and control search-history recording.
- **Focused presentation** — a minimal launcher, light/dark appearance, a selectable Legacy theme, and centering on the focused application's monitor where supported.
- **Application actions** — open applications, copy names and available metadata, reveal supported application entries, and access guarded running-app controls.
- **Extension Lists** — bundled commands with searchable lists, keyboard selection, actions, feedback, and resumable sessions.
- **Terminal tools** — search applications without opening a window, or pipe lines into an interactive picker.
- **Resident lifecycle** — tray access, a global shortcut on X11, a compositor toggle binding on Wayland, and bounded panel reuse for quick reopening.

Available actions depend on platform capabilities and verified application identity. Extension support currently covers bundled commands and List views; arbitrary extension installation, Detail/Form views, a store, and scheduling are future work.

## Getting started

### Prerequisites

- **Node.js 22.12+ within Node 22, or Node.js 24+**, with npm. CI uses Node 22.
- **Rust 1.95.0**, including rustfmt and Clippy, as pinned in [rust-toolchain.toml](rust-toolchain.toml).
- The [Tauri system prerequisites](https://tauri.app/start/prerequisites/) for your operating system.
- A graphical desktop session to run the native application.

The extension runtime uses the project's pinned Bun dependency, installed through npm. A separate global Bun installation is unnecessary.

### Run in development

Clone the repository and start the native application:

```sh
git clone https://github.com/AsaanKaroobar/Asaan-Launcher.git
cd Asaan-Launcher
npm ci
npm run tauri dev
```

The native development command prepares the bundled extension runtime and starts the frontend automatically.

For frontend-only work, use `npm run dev`. The browser preview cannot exercise native application discovery, operating-system actions, or resident lifecycle behavior.

### Build a package

Run the command for your target operating system on that operating system:

| Platform | Command | Output |
| --- | --- | --- |
| Linux | `npm run package:linux` | Debian `.deb` package |
| macOS | `npm run tauri build -- --bundles dmg` | `.dmg` disk image |
| Windows | `npm run tauri build -- --bundles nsis` | NSIS `.exe` installer |

Packages are written under `src-tauri/target/release/bundle/`. The build prepares the extension runtime automatically. Signing and public release qualification are separate delivery steps.

For a local Linux installation without root:

```sh
npm run install:local
npm run start:local
```

The installer extracts the built Debian package into `deployments/linux/` and updates `deployments/linux/current`. It does not modify the system package database. It uses `dpkg-deb` when available and otherwise extracts with `bsdtar` from libarchive, so local installation also works on non-Debian hosts such as Arch.

## Everyday use

Open the launcher with **Ctrl+Alt+Space** on X11, your compositor binding on Wayland, or the tray menu. Type to search, move through results with the arrow keys, and press **Enter** to open a selection.

| Control | Action |
| --- | --- |
| Ctrl+K / Command+K | Open the selected result's actions |
| Ctrl+Shift+F / Command+Shift+F | Toggle the selected result's favorite status |
| Up at the first result | Recall previous searches |
| Escape in application search | Clear the query, then hide when empty |
| Settings gear | Open General and Applications settings |
| Search tools menu | Access history, refresh, and customization |

Choose the launcher theme in **Settings → General → Launcher theme**. The **Legacy** option retains the earlier green appearance. Extension views have their own navigation: Escape returns from the active extension.

Closing the launcher hides the panel. **Quit** in the tray exits the resident application.

### Wayland and Hyprland

Wayland does not provide a portable global-shortcut API. On Wayland the resident does not install an X11 shortcut grab. Bind the toggle in your compositor instead; on Hyprland:

```ini
bindd = SUPER, SPACE, Asaaan Launcher, exec, asaaan-launcher --toggle
```

After a local install, use the installed path if it is not in `PATH`:

```ini
bindd = SUPER, SPACE, Asaaan Launcher, exec, /path/to/Asaan-Launcher/deployments/linux/current/usr/bin/asaaan-launcher --toggle
```

`--toggle` is forwarded to the running resident through single-instance; the first invocation starts it. The tray remains available (Waybar's `tray` module). On Hyprland the launcher, Settings and picker are floated, sized, centered on the focused monitor's usable area, and focused through Hyprland's local IPC socket. Other Wayland compositors keep their own placement and focus policy. Running-app controls remain unavailable on Wayland and are reported as such.

### Search from the terminal

After a local Linux installation:

```sh
deployments/linux/current/usr/bin/asaaan-launcher --search notes
```

The headless command prints up to eight ranked application results, honoring saved aliases, favorites, and hidden entries. It does not open a window, launch an application, or record search history.

### Choose from piped input

```sh
printf 'Development\nDesign\nDocuments\n' | deployments/linux/current/usr/bin/asaaan-launcher --pick --prompt 'Choose a workspace'
```

Type to filter and press Enter to choose. The picker writes the original selected line to standard output. Add `--index` to return its original zero-based position. Use `--help` for the available command-line modes.

## Development

The application separates persistent desktop coordination from the foreground UI. A Rust resident owns preferences, native actions, and supervised extension sessions. The search panel can be hidden, briefly reused, and disposed independently. Shared Rust crates keep search and lifecycle rules testable outside the webview.

```text
src/                  React interface, themes, and native client
src-tauri/            Tauri integration and platform adapters
crates/               Search, lifecycle, catalog, preferences, extensions
runtime/              Bundled extension host and runtime tests
extensions/           Bundled extension commands and fixtures
scripts/              Validation, packaging, and native acceptance tools
tests/reference/      Frozen behavioral fixtures and test mappings
docs/                 Checkpoint plans, findings, and delivery records
```

### Validate changes

```sh
npm run check
npm run build
npm run runtime:prepare
npm run runtime:test
npm run native:check
```

These commands cover TypeScript, frontend tests, behavioral fixture checks, extension-runtime tests, Rust formatting, workspace tests, and Clippy with warnings denied. A Swift toolchain is optional for fixture regeneration and is not required for normal builds.

After packaging and installing on Linux, run native acceptance in a private X11 session:

```sh
npm run test:resident:isolated
npm run smoke:appearance
npm run smoke:picker
npm run smoke:extension-ui
```

The isolated harness requires Xvfb, KWin X11, D-Bus, X11 utilities, Python GI/AT-SPI bindings, and the relevant desktop libraries. The complete Ubuntu dependency list and additional feature suites are maintained in the [validation workflow](.github/workflows/verify.yml). Tests use private display sessions and temporary preferences. Launch timing can be measured with `npm run benchmark:launch -- --samples 5 --expect-reuse` after local installation.

On a live Hyprland Wayland session, run the native Wayland checks against the installed package:

```sh
npm run test:wayland:hyprland
npm run test:wayland:windows
```

They show and focus only their own launcher windows and skip if another resident is running. CI remains X11-based and does not exercise Hyprland.

## Platform notes

- **Wayland:** global shortcut availability and absolute window placement depend on compositor support. Use the compositor binding with `--toggle`; the tray is also available. Hyprland placement/focus is implemented and tested. Other Wayland compositors keep their own placement policy; focused-monitor positioning is not guaranteed there. Running-app controls remain unavailable.
- **Appearance:** platform fonts, icons, and native materials can differ. The default theme uses an appearance-aware opaque surface.
- **Running-app controls:** Hide, Quit, Restart, and Force Quit are exposed only when the adapter can verify the target and required capabilities. Unsupported applications do not receive guessed process actions.
- **Preferences:** history and personalization are stored locally. Linux uses `$XDG_DATA_HOME/com.asaaan.launcher`, falling back to `~/.local/share/com.asaaan.launcher`. Versioned preference formats mean older builds may not read data saved by newer versions.

## Documentation

- [Checkpoint overview and delivery process](docs/CHECKPOINTS.md)
- [Interactive extension Lists](docs/checkpoints/014-extension-lists.md)
- [Current appearance, monitor placement, and launch measurements](docs/checkpoints/014-floe-theme-placement.md)
- [Panel reuse and launch performance](docs/checkpoints/014-launch-performance-1.md)
- [Platform CI and packaging](docs/checkpoints/014-platform-ci.md)
- [Behavioral test provenance](tests/reference/README.md)

Checkpoint records describe the evidence and limitations at the time of each delivery. Later follow-up records document subsequent changes.

## Contributing

Keep changes focused and include the validation appropriate to the behavior you change. For platform issues, include your operating system, display server where applicable, build version or commit, reproduction steps, and relevant logs with personal information removed.

Review the [checkpoint scope](docs/CHECKPOINTS.md) before proposing larger features. Preserve existing license headers and third-party attribution when adapting code or fixtures.

## License

Distributed under **AGPL-3.0-only**. See [LICENSE](LICENSE), [project attribution](NOTICE.md), and [third-party notices](LICENSES/) for details.
