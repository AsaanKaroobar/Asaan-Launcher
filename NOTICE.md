# Project and reference

Asaaan Launcher is an independent cross-platform launcher project. The foundation shell was created from the official Tauri React/TypeScript template.

The behavioral reference is Floe, commit 5b26e9d7a0240243251d7421bee5b90da5f52b9b, at https://github.com/thaw-app/Floe. Floe is licensed under AGPL-3.0, with origin-specific terms in its LICENSES directory. The adjacent reference checkout is unchanged. CP002 includes a Rust port of the fuzzy matcher, ranking and pure result merge, plus original search sources/tests used only as parity references. Original copyright and license headers are retained in those files.

This project is distributed under AGPL-3.0. Retain original headers, licenses, notices and credits when importing reference or third-party code in later checkpoints. Dependencies retain their respective licenses.

CP003 ports the lifecycle policy and finite-work lease ownership from Floe PanelLifetime.swift and PanelWork.swift. Original lifecycle sources and tests are preserved with provenance under tests/reference/lifecycle; original copyright is retained in the Rust policy/test files. Native process supervision and Tauri integration are implemented for this project.

CP004 adapts Floe application bundle scan depth, duplicate identity/origin policy and scoped app-catalog tests. Unchanged original sources/tests and their hashes are in tests/reference/applications. Native GIO, bundle/open and Windows StartApps adapters use this project’s own application identities.

Checkpoint 005 personalization and recall behavior is adapted from Floe Settings.swift, Model.swift, Model+Actions.swift and Model+Keys.swift, © 2026 René Jiménez, AGPL-3.0. Unchanged scoped reference files and test provenance are under tests/reference/personalization.

Checkpoint 006 aliases, hidden results and action/keyboard behavior are adapted from Floe Model+Actions.swift, Model+Keys.swift, ApplicationSettingsView.swift and SettingsView.swift, © 2026 René Jiménez, AGPL-3.0. Unchanged scoped references and test mappings are under tests/reference/customization.

CP007 application metadata contracts translate AppActions/FileActions.copies from Floe ItemActions.swift (© 2026 René Jiménez, AGPLv3). Pinned source and explicit partial/deferred tests are under tests/reference/application-actions.

CP008 translates the picker model and session behavior from Floe Picker.swift/PickerPanel.swift and PickerTests.swift (© 2026 René Jiménez, AGPLv3). Pinned sources and all 22 test declarations are tracked under tests/reference/picker.
