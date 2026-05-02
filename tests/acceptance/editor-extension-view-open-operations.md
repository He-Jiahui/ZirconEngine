# Editor Extension View Open Operations

## Scope
- Extension views automatically contribute `View.<id>.Open` operations for menu projection, remote discovery, and journal metadata.
- A view id must therefore be valid inside a dotted `EditorOperationPath`; ids with slash, whitespace, empty dotted segments, or other operation-path illegal characters are rejected when the extension registry accepts the view descriptor.
- The validation happens before the descriptor is inserted, so a rejected view cannot leak into the Workbench view registry without a matching operation.

## Test Inventory
- Positive coverage remains in `editor_runtime_registers_plugin_views_as_activity_descriptors`, `editor_runtime_projects_plugin_views_into_view_menu_operations`, and `operation_control_request_lists_registered_operations_for_remote_discovery`.
- Negative case: `editor_extension_registry_rejects_view_ids_that_cannot_form_open_operation_paths` rejects plugin view ids that would make `View.<id>.Open` invalid and verifies the registry remains unchanged.

## Results
- Passed static gate: `rustfmt --edition 2021 --check zircon_editor/src/core/editor_extension.rs zircon_editor/src/tests/editor_event/runtime.rs`.
- Passed static gate: scoped `git diff --check` for view open-operation validation code, runtime regression, operation architecture doc, this acceptance note, and active session note; only LF/CRLF conversion warnings were reported.
- Pending focused test: `cargo test -p zircon_editor --lib editor_extension_registry_rejects_view_ids_that_cannot_form_open_operation_paths --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-extension-view-open-operations --message-format short --color never -- --test-threads=1 --nocapture`.

## Acceptance Decision
- Registry-time view open operation path validation is implemented and covered by focused regression code.
- Final acceptance remains pending until the focused editor test can execute after the active UI runtime-interface Cargo blocker clears.
