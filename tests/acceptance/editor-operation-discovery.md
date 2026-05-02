# Editor Operation Discovery

## Scope
- External tools use `EditorOperationControlRequest::ListOperations` to discover callable editor operations.
- The discovery response should only include operations whose required capabilities are currently enabled, and each visible operation should still report its `required_capabilities` metadata for UI explanation and automation policy.

## Test Inventory
- Built-in discovery case: `Window.Layout.Reset` appears with menu path, undoable metadata, remote-callability metadata, and an empty `required_capabilities` list.
- Plugin capability case: `Weather.CloudLayer.Refresh` is hidden while `editor.extension.weather_authoring` is disabled, appears after the capability is enabled, and reports `required_capabilities = ["editor.extension.weather_authoring"]`.

## Results
- Pending: `cargo test -p zircon_editor --lib editor_runtime_consumes_plugin_registration_reports_with_capability_gate --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-operation-discovery --message-format short --color never -- --test-threads=1 --nocapture`.
- Passed static gate: `rustfmt --edition 2021 --check zircon_app/src/entry/entry_runner/editor.rs zircon_editor/src/core/editor_event/listener.rs zircon_editor/src/ui/host/editor_operation_dispatch.rs zircon_editor/src/ui/host/editor_extension_registration.rs zircon_editor/src/tests/editor_event/runtime.rs`.
- Passed static gate: scoped `git diff --check` for the touched editor/app code, docs, acceptance files, and active session note; only LF/CRLF conversion warnings were reported.

## Acceptance Decision
- Operation discovery now serializes `required_capabilities` for each visible operation.
- Final acceptance remains pending until the focused editor capability-gate regression reaches test execution.
