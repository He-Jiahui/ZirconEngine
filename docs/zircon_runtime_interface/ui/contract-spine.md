---
related_code:
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime_interface/src/ui/focus.rs
  - zircon_runtime_interface/src/ui/navigation.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/ui/picking.rs
  - zircon_runtime_interface/src/ui/text.rs
  - zircon_runtime_interface/src/ui/widget.rs
  - zircon_runtime_interface/src/ui/template/asset/document.rs
  - zircon_runtime_interface/src/ui/template/document.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/template/asset/compiler/node_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs
  - zircon_runtime/src/ui/template/asset/schema/legacy_template.rs
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/node_pool.rs
  - zircon_runtime/src/ui/tree/node/focus.rs
  - zircon_runtime/src/ui/tests/focus_navigation.rs
  - zircon_runtime/src/ui/tests/asset_contract_spine.rs
  - zircon_runtime/src/ui/tests/mod.rs
  - zircon_editor/src/ui/asset_editor/palette/instantiate.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/palette_descriptor_registry.rs
  - zircon_editor/src/tests/support.rs
  - zircon_editor/src/ui/asset_editor/document_diff.rs
  - zircon_editor/src/ui/asset_editor/tree/tree_editing.rs
  - zircon_runtime_interface/src/tests/mod.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
implementation_files:
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime_interface/src/ui/focus.rs
  - zircon_runtime_interface/src/ui/navigation.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/ui/picking.rs
  - zircon_runtime_interface/src/ui/text.rs
  - zircon_runtime_interface/src/ui/widget.rs
  - zircon_runtime_interface/src/ui/template/asset/document.rs
  - zircon_runtime_interface/src/ui/template/document.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/template/asset/compiler/node_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs
  - zircon_runtime/src/ui/template/asset/schema/legacy_template.rs
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/node_pool.rs
  - zircon_runtime/src/ui/tree/node/focus.rs
  - zircon_editor/src/ui/asset_editor/palette/instantiate.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/palette_descriptor_registry.rs
  - zircon_editor/src/tests/support.rs
  - zircon_editor/src/ui/asset_editor/document_diff.rs
  - zircon_editor/src/ui/asset_editor/tree/tree_editing.rs
plan_sources:
  - .codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md
  - docs/ui-and-layout/bevy-ui-text-widgets-focus-a11y-m0-gap-audit.md
  - user: 2026-05-08 continue M1 Contract Spine implementation
  - user: 2026-05-08 include M3 tab/directional navigation in M2 focus milestone
tests:
  - zircon_runtime_interface/src/tests/mod.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
  - zircon_runtime/src/ui/tests/asset_contract_spine.rs
  - target: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked
  - target: cargo check -p zircon_runtime_interface --locked
  - target: cargo test -p zircon_runtime --lib ui_asset_compiler_preserves_m1_contract_sections_from_source_nodes --locked
  - target: cargo test -p zircon_runtime --lib focus_navigation --locked
doc_type: module-detail
---

# UI Contract Spine

## Purpose

This document records the M1 Contract Spine for the Bevy UI/Text/Widgets/Focus/A11y completion plan. The work is contract-only inside `zircon_runtime_interface::ui`; it defines neutral DTOs that later runtime and editor milestones can consume without moving focus, picking, widget behavior, text editing, render extraction, or accessibility authority into the editor.

M1 does not implement widget reducers, text shaping, AccessKit bridging, or editor styling. M2/M3 now consume the focus/navigation DTOs for runtime-owned focus mutation and contract-aware navigation, while later milestones still own picking convergence, headless widgets, text shaping/editing completion, accessibility snapshots, and editor styling.

## Contract Families

`ui::focus` defines `UiInputFocus`, `UiFocusVisible`, `UiFocusChangeEvent`, `UiFocusedInput`, and `UiFocusContract`. The state distinguishes the focused node, previous focus, pending autofocus, focus-ring visibility reason, focus-change reason, and focused-input bubbling route.

`ui::navigation` defines `UiTabIndex`, `UiNavigationGroup`, `UiNavigationGroupId`, `UiDirectionalNavigation`, and `UiNavigationContract`. These DTOs let runtime M3 represent tab order, modal trap groups, nested navigation groups, manual directional overrides, and blocked edges.

`ui::picking` defines `UiPickPolicy`, `UiPickMode`, `UiPointerCapture`, and `UiPointerCaptureKind`. The policy intentionally uses one neutral vocabulary for pointer, hover, focus, accessibility, capture, and text-hit eligibility so Zircon does not inherit Bevy's split between UI picking and focus policy.

`ui::accessibility` defines neutral a11y roles, states, actions, nodes, diagnostics, `UiAccessibilityTreeSnapshot`, and `UiAccessibilityContract`. The snapshot is AccessKit-independent: runtime M9 may map it to AccessKit, but the shared contract stores node ids, paths, role/name/description, bounds, state, actions, label links, tooltip text, focused node, and diagnostics such as missing accessible names.

`ui::widget` defines the headless widget event vocabulary requested by M1: `Activate`, `ValueChange`, `TextEditChange`, `OpenChanged`, and `SelectionChanged`. `UiWidgetContract` carries defaulted template-facing widget metadata such as disabled, checked, value, label-for, and tooltip. These widget events intentionally remain separate from `UiComponentEventKind` in M1 because no concrete component-envelope variant or reducer bridge exists yet.

`ui::text` defines `UiTextEdit` and `UiTextCursorStyle` around the existing `UiEditableTextState` and `UiTextEditAction` render-surface text DTOs. It records the edit source and before/after state while leaving actual editing behavior and shaping to later runtime text milestones.

`ui::surface::render` now exposes `UiRenderExtractKind` and `UiRenderStats` beside the existing `UiRenderExtract`. The existing extract shape remains compatible with legacy `{ tree_id, list }` JSON because the new classification and stats DTOs are separate rather than required fields.

`ui::template::UiTemplateNode` has defaulted `focus`, `navigation`, `picking`, `a11y`, and `widget` sections. `ui::template::asset::UiNodeDefinition` mirrors those sections as optional source-authoring tables so old `.ui.toml` files continue to deserialize with no authored contract sections, while new source assets can opt into structured M1 contracts with `[root.focus]`, `[root.navigation]`, `[root.picking]`, `[root.a11y]`, and `[root.widget]` tables. Runtime asset compilation copies native-node sections into compiled `UiTemplateNode` defaults and treats component-instance sections as explicit root contract overrides. Legacy-template migration only emits optional source sections when the old compiled template node had a non-default contract value, preventing default migrated instance nodes from erasing component-root contracts.

`ui::tree::UiTreeNode` now carries defaulted `focus: UiFocusContract` and `navigation: UiNavigationContract` fields in the retained runtime tree. These fields are the runtime consumption point for compiled template contract sections, avoiding a second focus/navigation schema under `zircon_runtime`.

`ui::surface::UiFocusState` now stores runtime focus facts that need to be visible to hosts and tests: previous focus, pending autofocus, focus-visible policy, focus-change events, and focused-input route records. `captured`, `pressed`, and `hovered` remain on the same state object so cleanup can clear focus/capture/hover coherently when nodes are hidden, disabled, or despawned.

## M2/M3 Runtime Consumption

Runtime focus is source-aware. Programmatic calls record `Programmatic`, autofocus records `Autofocus`, pointer focus records `Input` with pointer-interaction visibility, and navigation records `Navigation` with keyboard-navigation visibility. Accepted `enabled`, `visible`, `visibility`, and `focusable` mutations reconcile focus and can emit `Disabled` or `Hidden`; node-pool detach emits `Despawned`.

Focused input bubbling uses `UiFocusedInput` rather than host-local state. Keyboard, navigation, text, and IME dispatch store the focused node, bubble route, accepted handler/owner, and acceptance result in `UiFocusState::focused_inputs`.

Contract-aware navigation uses `UiNavigationContract` fields on `UiTreeNode`: tab indices and group order define Next/Previous, non-modal groups contribute ordering, modal groups trap traversal, manual directional targets can point at a node or group, blocked edges suppress movement, and spatial fallback selects the nearest candidate by node center when no manual override exists. Runtime filters manual directional targets through the current modal group so authored overrides cannot escape an active dialog trap.

## Boundary Rules

The interface crate owns declarations and narrow helpers only. Runtime code must still own focus cleanup, tab/directional traversal, picking hit-grid policy application, widget behavior, text editing mutation, render extraction, a11y snapshot generation, and platform bridge mapping.

Editor code may consume these DTOs for authoring and presentation, but it must not become the source of truth for runtime focus, picking, widget, text, render, or a11y semantics.

## Test Coverage

`zircon_runtime_interface/src/tests/ui_contract_spine.rs` covers serde roundtrips and default compatibility for the new focus/navigation/picking, a11y snapshot/diagnostics, widget/text/cursor, render kind/stats, compiled-template sections, and source `.ui.toml` section contracts. It also verifies that legacy TOML without new sections still deserializes with safe defaults. `zircon_runtime/src/ui/tests/asset_contract_spine.rs` covers the runtime compiler path that preserves authored source sections through component expansion.
