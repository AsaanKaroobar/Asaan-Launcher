# Launch performance — increment 1 (CP014 follow-up)

The user explicitly authorized performance work after the platform CI repair. This supersedes the earlier performance deferral; CP015 feature work remains outside this task.

Target: move toward sub-500 ms launch incrementally. First establish a fresh installed Linux baseline, distinguish disposable-panel startup from grace-period reuse, identify the critical path, and validate one focused optimization. Do not retain hidden WebViews indefinitely or weaken readiness/focus/EOF checks to lower the number.

Status: measurement in progress. Native timing results apply to this Linux host only. A passing hosted build does not establish Windows/macOS launch latency. Public release remains user controlled.

Measurements use a private Xvfb/D-Bus/KWin session and the existing scoped keyboard/focus fixture. The reported end-to-end timings include test adapter and focus-observation overhead, with filesystem caches left intact. Each disposable-panel sample follows confirmed panel/helper exit. Compare identical sample counts and environments; keep the original CP014 evidence archive intact.

## Implemented candidate

The user approved panel reuse after discussing its memory tradeoff. Launcher panels now remain available for up to 10 seconds after dismissal with no finite work outstanding. Settings retains its existing 750 ms idle grace. Parent EOF, explicit Quit, readiness watchdogs, stale-sequence rejection and supervision remain unchanged. Unused launcher panels still exit and reap their renderer helpers after the bounded window.

Root search refreshes once per new presentation sequence on reopening. It preserves query/selection and leaves an active extension session under the existing resident supervision. A frontend regression checks updated results, retained input/selection and duplicate focus notifications.

Run `npm run benchmark:launch -- --samples 12 --expect-reuse` after packaging and local installation. By default it waits two seconds between hide/reopen (beyond the old 750 ms grace), checks PID reuse, native focus, editable input, initial results, eventual process/helper disposal and focus restoration. It records median/p95, installed package/binary hashes, environment and hidden/disposed memory. The same harness can benchmark an older installed version using `--binary /absolute/path/to/usr/bin/asaaan-launcher` without `--expect-reuse`. CI adds a five-sample functional reuse check; timing results are recorded without a hardware-dependent pass threshold.

Fresh disposable-panel startup and reopening are separate metrics. Reuse does not remove the first WebView startup. The earlier handshake experiments did not improve end-to-end launch and were removed; icon-loading work is left for a later increment.

Validation and measured results: pending the installed default-policy checks and hosted matrix. No public release has been published.

Initial installed default-policy check: 5/5 two-second reopens reused the same panel; focus median 211 ms, p95 213 ms. Fresh panel focus median 841 ms. Native freshness check renamed a private desktop fixture while hidden and confirmed its updated result with the retained query after reopening. All panels/helpers reaped after 10.04–10.08 seconds following final dismissal. Complete group PSS was 232,234 KiB while hidden versus 40,830 KiB after disposal; this is the explicit transient memory tradeoff. Twelve-sample comparison and final platform results remain pending.

The installed List suite also checks reuse after a two-second pause: the same panel and Bun host must retain the extension query/selection, accept keyboard activation and keep usage count unchanged. Its existing full-disposal/resume test remains in place.

The retained root selection is tracked by application ID for refresh, so inserting a newly installed application before it does not silently change the selected application. The new regression failed with index-only retention and passed after this fix; it also checks that repeated focus notifications do not trigger duplicate refreshes.
