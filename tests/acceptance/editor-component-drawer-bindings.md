# Editor Component Drawer Operation Bindings

## Scope
- Component drawer `ui.toml` bindings must name real `EditorOperation` entries before the drawer is accepted into the live editor runtime.
- Syntax validation still happens at `EditorExtensionRegistry::register_component_drawer(...)`; runtime registration now adds existence validation against built-in operations, operations contributed by the same extension, and auto-generated extension View open operations.
- Component drawer descriptors now also carry optional template id and data root metadata; invalid template/data-root ids are rejected before the descriptor can enter the live registry.
- Inspector pane payloads preserve drawer UI document, controller, template id, data root, and binding ids so custom drawer projection can be rendered or safely disabled when the plugin is unavailable.
- Editor snapshots now resolve enabled `ComponentDrawerDescriptor` entries into selected dynamic plugin component snapshots only after the current capability snapshot allows the contributing extension.

## Test Inventory
- Positive case: a plugin component drawer with `Weather.CloudLayer.Refresh` should register when the same extension contributes that operation descriptor.
- Negative case: a syntactically valid `Weather.CloudLayer.Refresh` binding should be rejected if no built-in, generated, or extension operation with that path exists.
- Syntax boundary: a short `Weather.Refresh` binding remains rejected at descriptor registration time because it violates the `XXX.YYY.ZZZ` operation path contract.
- Template metadata boundary: a template id with surrounding whitespace is rejected as an invalid component drawer template id.
- Projection case: `inspector_payload_preserves_component_drawer_template_metadata` verifies drawer metadata survives into the Inspector payload.
- Runtime snapshot case: `editor_snapshot_resolves_enabled_component_drawer_for_selected_dynamic_component` verifies an enabled drawer descriptor reaches the selected dynamic component snapshot with UI document, controller, template id, data root, and binding ids intact.
- Capability fallback case: `editor_snapshot_hides_component_drawer_when_extension_capability_is_disabled` verifies a capability-gated drawer stays hidden and the component remains protected.

## Results
- Pending: `cargo test -p zircon_editor --lib editor_runtime_rejects_component_drawer_bindings_to_missing_operations --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings --message-format short --color never -- --test-threads=1 --nocapture`.
- Pending: `cargo test -p zircon_editor inspector_payload_preserves_component_drawer_template_metadata --locked --jobs 1 --message-format short --color never`.
- Pending: `cargo test -p zircon_editor editor_snapshot_resolves_enabled_component_drawer_for_selected_dynamic_component --locked --jobs 1 --message-format short --color never`.
- Pending: `cargo test -p zircon_editor editor_snapshot_hides_component_drawer_when_extension_capability_is_disabled --locked --jobs 1 --message-format short --color never`.
- Passed static gate: `rustfmt --edition 2021 --check zircon_app/src/entry/entry_runner/editor.rs zircon_editor/src/core/editor_event/listener.rs zircon_editor/src/ui/host/editor_operation_dispatch.rs zircon_editor/src/ui/host/editor_extension_registration.rs zircon_editor/src/tests/editor_event/runtime.rs`.
- Passed static gate: scoped `git diff --check` for the touched editor/app code, docs, acceptance files, and active session note; only LF/CRLF conversion warnings were reported.

## Acceptance Decision
- Runtime binding existence validation is implemented and covered by focused regression code.
- Final acceptance remains pending until the focused editor test reaches execution under a quieter Cargo queue.
