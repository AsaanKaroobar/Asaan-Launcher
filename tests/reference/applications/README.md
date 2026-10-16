# Application catalog references

Floe revision: 5b26e9d7a0240243251d7421bee5b90da5f52b9b, https://github.com/thaw-app/Floe. Copied sources/tests remain unchanged under GNU AGPLv3 with their original headers. provenance.json records hashes; test-map.json accounts for all 25 declarations across AppScanTests, CatalogLoaderTests and LauncherCatalogTests.

15 declarations have application-only equivalents, 2 are partially covered and 8 are deferred with explicit reasons. Rust bundle fixtures preserve the one-level scan and same-name identity/origin cases. The runtime uses platform-native discovery and launch adapters. Foreground polling every five seconds and manual refresh reproduce eventual install/remove visibility; native filesystem watcher APIs are deferred.

The catalog worker uses serialized scans and immutable snapshots, with cached search remaining available during refresh. The frontend drops superseded replies. Tests preserve the existing catalog on refresh failure and verify a query typed during initial loading. Full mixed extension-command loading, command autorun, model cancellation and preloaded snapshots remain outside CP004.

Linux subprocess fixtures isolate XDG directories to avoid GLib cache contamination or launching installed user applications. They cover desktop-ID precedence, Hidden/NoDisplay, desktop visibility, TryExec, nested IDs, literal Exec arguments, native field codes, stdout isolation, refresh and removed-entry rejection. Windows metadata parser and macOS bundle filesystem fixtures run portably. They do not certify native macOS/Windows integration or Finder localization.
