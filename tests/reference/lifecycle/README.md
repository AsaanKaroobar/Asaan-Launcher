# Floe lifecycle parity references

Revision: 5b26e9d7a0240243251d7421bee5b90da5f52b9b, https://github.com/thaw-app/Floe.

The five unchanged Swift sources/tests are test references under GNU AGPLv3. Original source headers are preserved. provenance.json records exact SHA-256 values. test-map.json accounts for every original declaration, including deferred cases and semantic substitutions. npm run lifecycle:parity checks hashes, mapping coverage and target function presence; cargo test executes the Rust equivalents.

18 of 20 declarations have bounded CP003 equivalents. Clipboard store ownership and specialized built-in hotkey routing are future work. The Rust command stand-in is an opaque BuiltIn value: action execution is not implemented. Watchdog fixtures never cooperate with shutdown; Rust uses hard termination for unready processes instead of the reference's POSIX signal escalation. Sequence ordering is checked through actual child-process pipes. Work ownership uses RAII and an atomic count.

The wire format is private Rust-to-Rust tagged NDJSON, not Swift Codable byte compatibility. MAX_PACKET bounds input allocation to 4 MiB; malformed, unknown and truncated packets fail closed. EOF exits the managed native child. A fixture binary exists only for tests and is not part of the Tauri package.
