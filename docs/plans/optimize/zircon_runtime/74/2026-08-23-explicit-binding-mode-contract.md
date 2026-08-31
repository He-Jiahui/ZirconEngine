# Runtime74 Explicit Binding Mode Contract

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M1
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-23-explicit-binding-mode-contract.md","docs/zircon_runtime/ui/architecture.md","docs/zircon_runtime/ui/template/pipeline.md","docs/zircon_runtime/ui/v2.md","zircon_runtime/src/ui/template/asset/binding/validation.rs","zircon_runtime/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime/src/ui/tests/asset_binding/compiled_program.rs","zircon_runtime_interface/src/tests/contracts.rs","zircon_runtime_interface/src/ui/template/asset/binding/diagnostic.rs","zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime_interface/src/ui/template/asset/compiler/package/header.rs","zircon_runtime_interface/src/ui/template/document.rs","zircon_runtime_interface/src/ui/template/mod.rs","zircon_editor/src/tests/editing/ui_asset_replay.rs","zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs","zircon_editor/src/ui/asset_editor/binding/binding_inspector/payload_editing.rs","zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/component_property_rows.rs","zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_tree_rows.rs","zircon_editor/src/ui/workbench/reference/builder/nodes.rs","zircon_runtime/src/ui/dispatch/input_manager/manager/tests.rs","zircon_runtime/src/ui/surface/surface/default_interactions.rs","zircon_runtime/src/ui/tests/accessibility_widget_actions.rs","zircon_runtime/src/ui/tests/asset_surface_index/binding_ownership_performance.rs","zircon_runtime/src/ui/tests/event_routing.rs","zircon_runtime/src/ui/tests/pointer_click_semantics.rs","zircon_runtime/src/ui/tests/runtime_input_manager.rs","zircon_runtime/src/ui/tests/runtime_input_reply_routes.rs","zircon_runtime/src/ui/tests/runtime_input_reply_routes/table_pointer_routes.rs","zircon_runtime/src/ui/tests/runtime_window_input_pump.rs","zircon_runtime/src/ui/tests/shared_core/scroll_mutation/property_mutation.rs","zircon_runtime/src/ui/tests/v2_asset.rs","zircon_runtime/src/ui/tests/v2_asset/default_controls.rs","zircon_runtime/src/ui/tests/v2_asset/range_controls.rs","zircon_runtime/src/ui/tests/widget_menu_behavior.rs","zircon_runtime/src/ui/tests/widget_radio_behavior.rs","zircon_runtime/src/ui/tests/widget_range_navigation.rs","zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs","zircon_runtime/src/ui/tests/widget_text_input_ime_context.rs","zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs","zircon_runtime/src/ui/tests/widget_text_input_keyboard_clipboard.rs","zircon_runtime/src/ui/tests/widget_text_input_keyboard_hard_line.rs","zircon_runtime/src/ui/tests/widget_text_input_keyboard_text.rs","zircon_runtime/src/ui/tests/widget_text_input_mui.rs","zircon_runtime/src/ui/tests/widget_text_input_pointer.rs"]

- Date: 2026-08-23
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-004`
- Delivery state: implementation complete; grouped coordinator validation pending

## Scope Delivered

- `UiBindingMode` owns stable `OneTime`, `OneWay`, `TwoWay`, `Event`, and `Command` source names.
  Each mode exposes one typed trigger timing plus explicit target, source, and command publication
  permissions.
- Existing `.zui` documents default to `Event`. Direct Rust constructors explicitly select the
  same default, so introducing the public field does not change existing dispatch behavior.
- `UiCompiledBinding` retains the mode in the persistent program. Compiler schema 8 prevents an
  older artifact without mode identity from being admitted as current.
- Runtime currently has an executor only for `Event`. `OneTime`, `OneWay`, `TwoWay`, and `Command`
  produce stable `unsupported_binding_mode` / `ZUI-BIND-0005` diagnostics and block compilation.
  They are not silently treated as event bindings or claimed as implemented model/command paths.

## TDD and Validation Contract

The interface tests were written before the enum, trigger, permission, and diagnostic
implementations. They lock the exact serialized names, the legacy `Event` default, and the complete
permission matrix. Runtime tests lock artifact retention for `Event` and fail-closed compiler
behavior for every mode that lacks an executor.

The grouped Runtime74 submission `caf7bfeb2eed4e3e9452e78fd45aed36` / request
`a97a2f548668430b997b32ec2891c14b` covered 88 tasks, 62 Cargo groups, 20 new behavior tests, and
18 existing performance rows under validator SHA-256
`E93B9E81B8EFA1225CDA3B5CF5632687E7CA29C1A02C20C4614342A91D6BAFB1`. It failed during
validation-copy `closure_planning` with `validation_copy_state_forbidden`, before Cargo started.
No behavior pass, performance result, or commit is claimed; grouped validation remains pending.

The forward grouped submission `a2c39ddcdd944d588daa96cd7c99d512` / request
`d92db795584a4c4e8a561e6d3df175e1` is queued asynchronously without waiting. It covers 89 tasks,
65 Cargo groups, 30 cumulative new behavior tests, and 18 performance rows under root validator
SHA-256 `D84C8CA2B28C1EE4137D0CCC580FB601ED34F7F4E4084081E1AA0BEC75701ACB`; its 245-path,
7-tombstone source manifest is `6d2edcabe8fb82f2971f30f13d908d13899a148aa747ce75ae863a87c2582063`.
This receipt is submission evidence only; acceptance remains pending.

## Performance

This slice adds no event-dispatch lookup, allocation, parser, or mutation work and therefore adds
no independent performance row. The mode is compiled once and carried as a small artifact field;
unsupported modes stop before publication. Existing Runtime74 release measurements remain pending,
and no performance threshold is claimed for RTB-P1-004 alone.
