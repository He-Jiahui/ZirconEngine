# Editor Component Drawer Operation Bindings

## Scope
- Component drawer `ui.toml` bindings must name real `EditorOperation` entries before the drawer is accepted into the live editor runtime.
- Syntax validation still happens at `EditorExtensionRegistry::register_component_drawer(...)`; runtime registration now adds existence validation against built-in operations, operations contributed by the same extension, and auto-generated extension View open operations.

## Test Inventory
- Positive case: a plugin component drawer with `Weather.CloudLayer.Refresh` should register when the same extension contributes that operation descriptor.
- Negative case: a syntactically valid `Weather.CloudLayer.Refresh` binding should be rejected if no built-in, generated, or extension operation with that path exists.
- Syntax boundary: a short `Weather.Refresh` binding remains rejected at descriptor registration time because it violates the `XXX.YYY.ZZZ` operation path contract.

## Results
- Pending: `cargo test -p zircon_editor --lib editor_runtime_rejects_component_drawer_bindings_to_missing_operations --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings --message-format short --color never -- --test-threads=1 --nocapture`.
- Passed static gate: `rustfmt --edition 2021 --check zircon_app/src/entry/entry_runner/editor.rs zircon_editor/src/core/editor_event/listener.rs zircon_editor/src/ui/host/editor_operation_dispatch.rs zircon_editor/src/ui/host/editor_extension_registration.rs zircon_editor/src/tests/editor_event/runtime.rs`.
- Passed static gate: scoped `git diff --check` for the touched editor/app code, docs, acceptance files, and active session note; only LF/CRLF conversion warnings were reported.

## Acceptance Decision
- Runtime binding existence validation is implemented and covered by focused regression code.
- Final acceptance remains pending until the focused editor test reaches execution under a quieter Cargo queue.
