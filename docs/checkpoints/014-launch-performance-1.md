# Launch performance — increment 1 (CP014 follow-up)

The user authorized incremental launch performance work after the platform CI repair and explicitly approved panel reuse after discussing its memory tradeoff. CP015 feature work remains outside this delivery. Public releases remain user controlled.

## Change

A dismissed launcher panel stays available for **10 seconds**, then exits when no finite work is outstanding. Reopening within that window reuses the panel and its WebView. Background startup still creates no panel; a fresh opening still starts a new WebView. Settings keeps its original 750 ms idle grace.

A reused root view keeps its query and selected application. Reopening forces a native catalog/preferences refresh once per presentation sequence. Selection is preserved by application ID if refreshed results change order. The regression for an inserted application failed with index-only retention, then passed with ID retention. Duplicate focus notifications do not cause duplicate refreshes. Active extensions retain their existing resident supervision.

Parent EOF, explicit Quit, readiness watchdogs, stale-sequence checks, finite-work leases, native process identity checks and crash cleanup remain in place. The original 750 ms launcher disposal policy is intentionally changed; hidden-memory parity during reuse is not claimed.

## Installed Linux measurements

Twelve samples per build, private Xvfb/D-Bus/KWin session, software GL, no compiler/test workload running alongside measurement. Timings include the existing keyboard adapter and OS-focus observation overhead. Filesystem caches were not flushed. A fresh panel sample starts only after all previous panel/helpers have exited; this is not an OS cold-boot benchmark. The resident is already running. The same harness and private catalog fixture were used for the baseline and final build.

| Focus-confirmed presentation | Baseline | Final build |
| --- | ---: | ---: |
| Fresh panel, median | 813 ms | 815 ms |
| Reopen after a two-second pause, median | 836 ms | 210 ms |
| Reopen after a two-second pause, nearest-rank p95 | 863 ms | 213 ms |
| Reopens reusing the panel | 0/12 | 12/12 |

Reopen median improves by **74.8%**. The sub-500 ms target is reached for reopens inside the reuse window. Fresh panel startup remains around 0.82 seconds and remains a future optimization target. These are Linux observations, not Windows/macOS latency claims.

The final benchmark also renamed a private desktop fixture while hidden, then verified the updated native result with the retained query after reopening. Every cycle restored previous-window focus and eventually reaped the panel and its renderer helpers after roughly 10.1 seconds following final dismissal.

## Memory tradeoff

In the root-search fixture with no active extension or Settings window, complete summed Linux PSS for the resident and descendants was **227.1 MiB while hidden** and **40.1 MiB after disposal** (four processes versus the resident alone). This transient memory cost buys fast reuse. Other active services can add memory. Summed RSS is recorded separately and double-counts shared pages; neither metric is macOS physical footprint. The original resident memory budget is not newly certified by this work.

## Verification and delivery

- Frontend: **121 tests**, TypeScript and all reference checks passed.
- Rust: **279 Linux tests**, formatting and Clippy with warnings denied passed.
- Final installed benchmark: **12/12 reuse cases**, native focus, keyboard input, refreshed catalog/query, process/helper disposal and focus restoration passed.
- Installed native regressions on the final package: **passed**. Completed all host, List, search, process-control, Settings, clipboard/reveal, customization, personalization, picker, rendering/activation/EOF and resident checks.
- List coverage includes a two-second reuse check preserving query/selection, keyboard actions, Bun host identity and usage count, plus the existing full-disposal/resume check. There are **16 installed List stages**.
- Local package installed: **0.14.0 / CP014**. User-desktop run: **passed**; resident PID 256226 registered the shortcut and presented the native panel. Its executable hash matches the installed binary.
- Final hosted matrix: **passed on Ubuntu, macOS and Windows**, [run 39128672956](https://github.com/AsaanKaroobar/Asaan-Launcher/actions/runs/39128672956), code/workflow revision **b402ea52e41dad2d6daed6c0e1c88569d1a83568**.
- Downloadable workflow installers: **uploaded, downloaded and hash-verified for all three platforms**. Public release: **not published**. Signing and Windows/macOS native desktop acceptance remain pending.

Hosted qualification:

| Runner | Rust | Bun runtime | Frontend | Installer |
| --- | ---: | ---: | ---: | --- |
| Ubuntu 24.04 x64 | 279 | 280 passed | 121 | Debian amd64 |
| macOS 14 ARM64 | 268 | 280 passed | 121 | aarch64 DMG |
| Windows 2022 x64 | 266 | 277 passed, 3 POSIX-only skips | 121 | x64 NSIS |

Ubuntu passed all installed-native suites and its five-sample reuse/freshness check (5/5 same-panel reopens; median 212 ms). Its fresh-panel median was 445 ms on that runner; this separate hardware observation does not establish a local cold-start improvement. All native evidence package hashes match the downloaded Linux installer.

| CI installer | SHA-256 |
| --- | --- |
| Linux Debian | cc46e9c655bdc99ce368e08fe9c3201e9b6aff6e9437b3c6fbcec9eede35a864 |
| macOS DMG | 963309308f87b728448048f8f1136287e1006ddd6eb2926838a680333d7cb04f |
| Windows NSIS | 425d84d468bbe3317f8e309a14c9be9283042ee4e1c2a24615ca691b390723a3 |

Local Debian package SHA-256: **4126d8d7d983b4fa01b46d19c7b4903aa9ff642fd995abe923a8af9e2187298b**.

Installed binary SHA-256: **ec50b2501db144a1601e04fcd823daca7bebeb3360ae69211a6cc4b3b36c71bb**.

Raw paired samples, installation manifests, memory/process evidence, failed-before/passed-after selection regression, validation logs and final CI metadata/artifacts are retained under ignored **.artifacts/launch-performance-1**. The original CP014 65-file archive seal was verified unchanged; the pinned Floe checkout also remains unchanged. All commits use Dani-Bhai as author and committer. The final result-record commit changes only this document; executable code and workflow remain those verified at b402ea52e41dad2d6daed6c0e1c88569d1a83568.

## Reproduce

After packaging and local installation:

```sh
npm run benchmark:launch -- --samples 12 --expect-reuse
```

The harness creates its own display, bus, preferences and catalog; it does not inject keys into the user's desktop. It records installed package/binary hashes and environment. To compare an older installed build, supply `--binary /absolute/path/to/usr/bin/asaaan-launcher` without `--expect-reuse`. CI runs a five-sample functional reuse check and uploads the evidence; it records timing without a hardware-dependent latency pass threshold. Set `ASAAAN_TRACE_STARTUP=1` for opt-in monotonic native startup stage logs (no paths or queries).
