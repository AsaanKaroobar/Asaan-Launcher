# Install script (CP014 follow-up)

The user asked for a simple installation path from the repository. `install.sh` provides a user-level installer for Linux and macOS without invoking `sudo`. Windows continues to use the NSIS `.exe`; the Debian package remains the manual Linux package install.

## Behavior

- Reads the latest release version from `releases/latest/download/latest.json` unless `--version` pins a release.
- Downloads the release `SHA256SUMS` and verifies the selected artifact before installing.
- Linux installs the AppImage at `~/.local/share/asaaan-launcher/Asaaan-Launcher.AppImage`, links `~/.local/bin/asaaan-launcher`, and creates a desktop entry and icon. The AppImage is the signed automatic-update channel.
- macOS mounts the release DMG and copies `Asaaan Launcher.app` into `~/Applications`.
- `--uninstall` removes the installed app, command symlink, desktop entry and icon; user settings are left in place.
- Supports `--prefix`, `--bin-dir`, `--no-desktop`, `--version` and `--help`.
- The Linux instructions print the `Ctrl+Alt+Space` default and the Wayland `--toggle` binding.

## Verification

- `sh -n install.sh` and `dash -n install.sh` passed.
- The script was run on this Linux host against the public **0.3.0** release. It downloaded the latest manifest and checksum file, verified the AppImage SHA-256 `171ccdc2c94cff8a345170d7f5c627ca0747b6463ca0bb972e810f58fb37cff8`, installed the AppImage, created the `asaaan-launcher` symlink, desktop entry and icon, and then removed them with `--uninstall`. A temporary prefix and XDG data directory were used.
- macOS was not executed because no macOS host is available here. The macOS path uses the release DMG, `hdiutil` and `ditto`; it remains unqualified.
- The installed AppImage uses the release's signed updater artifacts. The installation script itself is not signed and is not a substitute for the updater signature check.

## Delivery status

- **Implemented:** `install.sh` on `main`; documented in the README.
- **Tested locally:** Linux 0.3.0 install and uninstall; no macOS execution.
- **Installed locally:** temporary test prefix only; existing desktop installations were not replaced.
- **Release asset:** not attached to the published 0.3.0 release. `scripts/stage-release.mjs` now includes `install.sh` in future staged releases; the README uses the `main` raw URL for the current release.

The script is not a new application binary or a release-version change. It does not start CP015.
