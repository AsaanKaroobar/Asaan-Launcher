# Automatic updates (CP014 follow-up)

The user authorized adding the Tauri updater plugin as a CP014 follow-up: signed updater artifacts, AppImage as the Linux update channel, a startup prompt with Install/Later, and a tray **Check for updates…** action. Silent installation is not implemented. Published as 0.3.0 on 2026-10-26; public releases remain user controlled.

## Behavior

The windowless resident owns update checks so the panel, Settings and extension sessions can be stopped before the running application is replaced.

- A startup check runs 15 seconds after the resident starts and is skipped in smoke-test modes.
- The tray menu has **Check for updates…** for a manual check.
- When a newer signed release is available, a native dialog reports the current and available versions with **Install** and **Later**.
- **Later** leaves everything running and downloads nothing. **Install** downloads and verifies the artifact, stops supervised children, installs, and restarts.
- No update is installed silently.
- The updater endpoint is `https://github.com/AsaanKaroobar/Asaan-Launcher/releases/latest/download/latest.json`.
- Signature verification and `requireSignedVersion` are enabled. The signature must carry the announced version.

### Linux

AppImage is the automatic-update channel. The plugin signs the raw `.AppImage`; the updater replaces the running AppImage, preserves its permissions, and restarts it. The `.deb` remains the manual-install artifact; on a non-AppImage Linux run the Install action explains that automatic installation requires the AppImage instead of attempting a privileged package operation.

### Windows and macOS

The same plugin and signed manifest cover Windows NSIS and macOS updates. Windows updates run the NSIS installer; macOS replaces the app bundle. Both builds pass CI packaging, but manual desktop qualification of the 0.3.0 updater remains pending. The macOS app is not notarized and the installers are unsigned, so macOS automatic updates may be blocked by Gatekeeper until signing/notarization is added. Windows SmartScreen may warn about unsigned installers.

### Compatibility

0.2.0 and earlier do not contain the updater and cannot receive automatic updates. The first updater-capable release is **0.3.0**; users on 0.2.0 must install 0.3.0 manually. The 0.3.0 release publishes `latest.json`, so 0.3.0 and later can update to 0.3.1 and later.

## Signing

A new password-protected Tauri updater key was generated outside the repository:

- Private key: `~/.config/asaaan-launcher/updater/asaaan-updater.key` (0600)
- Password: `~/.config/asaaan-launcher/updater/password` (0600)
- Public key: `~/.config/asaaan-launcher/updater/asaaan-updater.key.pub`

The public key is stored in `src-tauri/tauri.conf.json`. GitHub Actions secrets **TAURI_SIGNING_PRIVATE_KEY** and **TAURI_SIGNING_PRIVATE_KEY_PASSWORD** are configured for this repository. The private key/password are not in the repository. Losing either prevents signing future updates; back both up securely. The `.tauri/` and `*.key` patterns are ignored by git.

## Release pipeline

The updater needs updater artifacts and a `latest.json` manifest in the release:

- Linux AppImage and `.AppImage.sig`
- macOS updater archive and `.sig`
- Windows updater installer and `.sig`
- `latest.json` with `linux-x86_64`, `darwin-aarch64`, and `windows-x86_64` entries
- `SHA256SUMS`

`npm run release:stage -- --source <downloaded-ci-artifacts> --version 0.3.0 --notes "See the release notes on GitHub."` stages renamed assets, generates `latest.json` and `SHA256SUMS`, and is intended to be followed by `gh release create`. The staging script supports both raw-signature artifacts (Tauri 2.14 signs the AppImage and NSIS installer directly) and the older tarball/zip updater forms. The endpoint reads `latest.json` from the latest GitHub release.

## Verification

- Frontend: **126 tests**; typecheck and parity checks pass.
- Rust: **290 tests**; rustfmt and strict all-target Clippy pass.
- Installed 0.3.0 Linux `.deb` (exact CI artifact): package SHA-256 `12fc6e1737ec2d1941f007725a7291aad88e9b5ac9df45553b5591aae8ebfb60`; binary SHA-256 `f2602842ffb1919bed88c2eac43161c1ca78c3817f33a8dede7b3f65e9d2c860`.
- The installed `.deb` passed the 7-stage Wayland/Hyprland test and the 2-stage picker/Settings test. The startup updater check ran; with only 0.2.0 published (no latest.json) it correctly logged a non-fatal check error.
- Local end-to-end updater test with a test HTTP endpoint:
  - Test 0.3.0 AppImage SHA-256 `922277e50ce0f33d75277e6f6214150e59e38aa45a40374887653cc657701da2`.
  - Signed 0.3.1 target AppImage SHA-256 `bb3625f96de3a048914cafa3e4d7fe63b1efee6c267ef1d76cece1fc9ddbce25`.
  - The **Update available** window appeared. Escape/Later closed it with no download.
  - Return/Install downloaded the signed AppImage, verified the signed version, replaced the AppImage (final SHA-256 exactly the 0.3.1 target), and restarted as 0.3.1.
  - Evidence: `.artifacts/updater/evidence.json`, `later-cancel.log`, `install-restart.log`, `local-server.log`.
- Release staging script tested with both raw-signature and archive-signature fixture layouts; `latest.json` URLs, signatures, and `SHA256SUMS` matched the staged files.
- Hosted CI passed all three platform jobs for `137f733` ([run 40292409934](https://github.com/AsaanKaroobar/Asaan-Launcher/actions/runs/40292409934)): signed Linux `.deb` + AppImage, macOS DMG + `.app.tar.gz`, and Windows NSIS + signature. The exact CI Linux `.deb` passed the installed Wayland/picker/Settings tests and smokes.

## Limits and outstanding gates

- 0.2.0 cannot auto-update; manuals install 0.3.0 is required.
- Linux automatic updates are AppImage-only. Debian `.deb` users update manually.
- macOS automatic updates require signing/notarization for reliable installation; current builds are unsigned.
- Windows updater installation is CI-packaged but not manually qualified for 0.3.0.
- The prompt is a native dialog; on Hyprland it can appear without taking focus, so the user may need to select it. A future panel banner or notification could improve visibility.
- The startup check contacts GitHub every resident start; no daily cache or opt-out preference is implemented. The check is skipped in smoke tests.
- No automatic update check is performed by the panel or picker child processes.
- The public 0.3.0 release is published; see [release 0.3.0](../releases/0.3.0.md).

## Delivery status

- **Implemented:** updater plugin, signed updater artifacts, startup prompt, tray check, AppImage channel, release staging.
- **Tested locally:** frontend, Rust, installed `.deb`, local prompt/cancel/install/restart.
- **Installed locally:** 0.3.0 `.deb`, installed with `bsdtar`.
- **Run locally:** Hyprland Wayland tests, picker/Settings, updater end-to-end.
- **Downloadable public release:** [0.3.0](https://github.com/AsaanKaroobar/Asaan-Launcher/releases/tag/0.3.0) published on 2026-10-26; all ten assets and checksums downloaded anonymously and verified; `latest.json` serves version 0.3.0.
