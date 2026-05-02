# Editor Menu Operation Bindings

## Scope
- Extension menu items must bind to real `EditorOperation` entries before the extension is accepted into the live editor runtime.
- The existence check uses the same candidate set as component drawers: built-in operations, operations contributed by the same extension, and auto-generated extension View open operations.
- Extension menu paths use Unity-style slash paths and must contain at least a top-level menu and a leaf item, with no empty or whitespace-padded segments.
- `EditorOperationDescriptor.menu_path` follows the same slash-path shape before an operation descriptor enters the registry, because it is projected into discovery and Workbench menu metadata too.
- Menu paths are unique across accepted editor extensions; a later extension cannot reuse a path that is already owned by another extension.

## Test Inventory
- Positive coverage remains in `editor_runtime_projects_plugin_menu_operations_into_remote_callable_reflection`, where a menu item binds to `Weather.CloudLayer.Refresh` after the same extension contributes that operation.
- Negative case: `editor_runtime_rejects_menu_items_to_missing_operations` rejects a syntactically valid menu item path whose operation is not registered.
- Path boundary case: `editor_extension_registry_rejects_invalid_menu_item_paths` rejects empty, single-segment, empty-segment, leading-slash, and trailing-slash menu paths before Workbench projection.
- Operation metadata boundary case: `editor_operation_registry_rejects_invalid_menu_paths` rejects invalid optional operation `menu_path` values at the operation registry boundary.
- Cross-extension conflict case: `editor_runtime_rejects_duplicate_extension_menu_paths_without_registering_operations` rejects duplicate live extension menu paths before the new extension's operations become discoverable.
- Capability coverage remains in `editor_runtime_consumes_plugin_registration_reports_with_capability_gate`, where disabled plugin capabilities hide and block contributed operations.

## Results
- Passed static gate: `rustfmt --edition 2021 --check zircon_editor/src/ui/host/editor_extension_registration.rs zircon_editor/src/tests/editor_event/runtime.rs`.
- Passed static gate: scoped `git diff --check` for menu binding code, runtime regression, docs, acceptance notes, and active session note; only LF/CRLF conversion warnings were reported.
- Passed static gate after operation menu-path validation: `rustfmt --edition 2021 --check zircon_editor/src/core/editor_operation.rs zircon_editor/src/core/editor_extension.rs zircon_editor/src/tests/editor_event/runtime.rs`.
- Passed static gate after operation menu-path validation: scoped `git diff --check` for operation/menu/view validation code, runtime regressions, docs, acceptance notes, and active session note; only LF/CRLF conversion warnings were reported.
- Passed static gate after cross-extension menu conflict validation: `rustfmt --edition 2021 --check zircon_editor/src/ui/host/editor_extension_registration.rs zircon_editor/src/ui/host/editor_extension_views.rs zircon_editor/src/tests/editor_event/runtime.rs zircon_editor/src/core/editor_operation.rs zircon_editor/src/core/editor_extension.rs zircon_app/src/entry/entry_runner/editor.rs`.
- Passed static gate after cross-extension menu conflict validation: scoped `git diff --check` for extension registration/view code, runtime regressions, docs, acceptance notes, and active session note; only LF/CRLF conversion warnings were reported.
- Pending focused test: `cargo test -p zircon_editor --lib editor_operation_registry_rejects_invalid_menu_paths --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-operation-menu-paths --message-format short --color never -- --test-threads=1 --nocapture`.
- Pending focused test: `cargo test -p zircon_editor --lib editor_runtime_rejects_duplicate_extension_menu_paths_without_registering_operations --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-extension-registration-atomicity --message-format short --color never -- --test-threads=1 --nocapture`.
- Blocked: `cargo test -p zircon_editor --lib editor_runtime_rejects_menu_items_to_missing_operations --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-menu-operation-bindings --message-format short --color never -- --test-threads=1 --nocapture` timed out after 15 minutes while compiling/linking the editor test binary and emitted no Rust diagnostic or assertion output.
- Blocked after path validation: `cargo test -p zircon_editor --lib editor_extension_registry_rejects_invalid_menu_item_paths --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-menu-operation-bindings --message-format short --color never -- --test-threads=1 --nocapture` failed before the focused test ran because `zircon_runtime/src/ui/template/asset/loader.rs` imports missing `super::schema::UiAssetMigrationOutcome`. This is an active UI runtime-interface cutover area, so the menu validation slice did not patch it.

## Acceptance Decision
- Runtime menu binding existence validation is implemented and covered by focused regression code.
- Menu path syntax validation is implemented and covered by focused regression code.
- Operation descriptor menu path validation is implemented and covered by focused regression code.
- Cross-extension menu path conflict validation is implemented and covered by focused regression code.
- Final acceptance remains pending until the focused editor test can execute after the active UI runtime-interface Cargo blocker clears.
