# Platform CI repair (CP014 follow-up)

Scope: repair the existing Ubuntu, macOS and Windows validation matrix. No CP015 feature work. Historical failed runs remain historical records; new runs validate the current checkpoint.

## Observed failures

- Ubuntu failed after packaging, in the first bare-Xvfb smoke command. Both the earliest available run and CP010/CP013 time out there, with a missing org.a11y.Bus service warning. The later local validation used private D-Bus/Xvfb/KWin sessions and WebKit software-renderer flags; CI did not use that same environment.
- macOS CP012/CP013 fails Clippy with warnings denied because process_wait.wait_for_exit and poll_exit are compiled without a macOS caller.
- Windows CP013/CP014 fails 26 runtime cases: POSIX paths resolved as Windows paths, Unix-only executables, cmd.exe newline conventions, JavaScript escaping of Windows bundle paths, POSIX signals unavailable on Windows, and EBUSY while deleting still-open SQLite databases.

Evidence: [CP010](https://github.com/AsaanKaroobar/Asaan-Launcher/actions/runs/39046226452), [CP013](https://github.com/AsaanKaroobar/Asaan-Launcher/actions/runs/39059421529), [original CP014](https://github.com/AsaanKaroobar/Asaan-Launcher/actions/runs/39102098886). Raw job logs and job metadata are retained in ignored .artifacts/platform-fixes.

## Changes

Linux CI uses the same isolated native smoke harness as local validation and explicitly installs accessibility-bus services and software GL support. Each native suite has its own named step and can report independently after a successful installation. Failure evidence and successfully built packages are uploaded even when a later test fails. No test or Clippy gate is made optional.

The shared process wait module is compiled only for its Linux/Windows callers and its pure unit tests. macOS uses its existing native termination path.

Runtime access summaries understand POSIX, Windows drive and UNC paths, including case-insensitive Windows home/own-folder comparisons and native file URLs. Executable recording handles quoted names and Windows separators.

SQLite owner teardown closes connections and prepared statements before deleting test data; failed initialization also releases its handle. An internal WeakMap disposer leaves the public extension LocalStorage API unchanged. A regression checks close/rename/reopen with values preserved.

The original runtime sources/tests stay frozen. tests/reference/extensions/portability-patches.json records exact fixture/teardown adaptations; the checker reapplies these against frozen originals and rejects drift. Windows explicitly skips three unsupported POSIX signal deliveries; the existing stdin-EOF flush case runs on all platforms. Linux/macOS retain all signal assertions. No behavioral assertion is silently removed to make a runner green.

## Validation

Local runtime: 280 passing tests, 862 assertions. Frontend: 120 passing tests plus TypeScript and all scoped reference checks. Remote platform matrix results will be recorded after execution.

Public release, signing and full native desktop acceptance on Windows/macOS remain separate from CI build/test qualification.
