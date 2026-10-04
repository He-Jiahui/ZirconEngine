---
related_code:
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs
  - zircon_runtime/src/ui/v2/component_instancer.rs
  - zircon_runtime/src/ui/tests/asset/document_compiler.rs
  - zircon_runtime/src/ui/tests/v2_asset/composite_components.rs
  - zircon_runtime/tests/ui_component_slot_layout_contract.rs
  - zircon_runtime/src/ui/tests/asset_component_reference_layout.rs
  - zircon_runtime/src/ui/tests/asset_prototype_store.rs
implementation_files:
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs
  - zircon_runtime/src/ui/v2/component_instancer.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
tests:
  - zircon_runtime/src/ui/tests/asset_component_reference_layout.rs
  - zircon_runtime/src/ui/tests/asset_prototype_store.rs
  - cargo test -p zircon_runtime --lib ui_document_compiler_applies_reference_instance_props_to_expanded_root --locked -- --nocapture
  - cargo test -p zircon_runtime --lib prototype_compiler_applies_reference_instance_props_to_expanded_root --locked -- --nocapture
  - cargo test -p zircon_runtime --lib ui_document_compiler_applies_reference_instance_style_overrides_after_stylesheets --locked -- --nocapture
  - cargo test -p zircon_runtime --lib prototype_compiler_applies_reference_instance_style_overrides_after_stylesheets --locked -- --nocapture
  - zircon_runtime::ui::tests::asset::document_compiler::component_slot_placeholder_layout_survives_into_mounted_child_slot_contract
  - zircon_runtime::ui::tests::v2_asset::composite_components::ui_v2_composite_component_preserves_slot_placeholder_layout_on_filled_child
  - zircon_runtime/tests/ui_component_slot_layout_contract.rs
doc_type: module-detail
---

# Component Instance Expansion

## Purpose

The legacy template asset compiler, flat prototype-store compiler, and v2 component instancer expand reusable component references into concrete nodes. This document owns their shared instance-override and slot-placeholder inheritance rules while the editor shell is moved onto Runtime UI assets.

The rule is intentionally simple: the component definition supplies the root node, then the instance site patches the expanded root. The instance site must be able to override ordinary props and final style values because Workbench assets use reusable controls such as labels, fields, tabs, and table rows with per-instance text, colors, and state.

## Behavior Model

`component_instance_expander.rs` handles the older `UiDocumentCompiler` path. `prototype_instancer.rs` handles the prototype-store path used by imported reusable assets. Both paths now merge the instance node's `props` into the expanded root attributes after resolving tokens and parameters. That means an instance-level value such as `foreground_color`, `text_tone`, `placeholder`, or `disabled` wins over the component's root defaults.

Style overrides follow the same precedence contract. Stylesheet rules may set a baseline on the expanded root, but an instance's inline style or resolved props patch is applied afterward and remains the effective value seen by later projection and render extraction. The tests use a `Label` component whose stylesheet sets a neutral foreground and primary text tone; the instance then overrides those values to error red and `error` tone.

A named `Slot` placeholder also owns the filled child's layout relative to the component container. Because expansion removes the placeholder itself, both the document compiler and v2 instancer transfer its authored `layout` map to the mounted child's slot attributes before insertion. The legacy compiler first composes the component token environment and resolves component parameters, then resolves the placeholder layout through that same environment. The caller's explicit child-mount values merge last, so an instance can intentionally override component defaults without losing unspecified axes. This is what allows a reusable property-editor row to keep a fixed name column and a stretching value column at every viewport width.

UI v2 applies the same recursive precedence when an instance patches the expanded component root. An instance commonly authors only width or height, while the prototype root owns structural layout such as `container.kind = "HorizontalBox"` and `container.gap`. Replacing the whole layout table would silently turn the expanded root into a free-layout container; recursive merge preserves omitted prototype structure and lets each explicitly authored instance leaf remain the final value.

## Design And Rationale

The Workbench shell relies on shared primitives rather than hand-authored native controls for every visual sample. Without instance root patching, reusable components collapse to their defaults and every authoring-site variant must fork a new component. That would make the shell asset set harder to maintain and would also hide semantic state, such as disabled fields or selected tabs, from the retained host.

The merge happens in the compiler support layer rather than in editor projection because the compiled runtime document is the authority. Both editor and runtime consumers should see the same expanded root metadata regardless of whether the document is projected into a retained host model or materialized into a runtime surface.

## Edge Cases And Constraints

- Instance props are resolved through the same token/parameter map as other compiler values before merging.
- The merge only patches the expanded root; child nodes keep their component-authored values unless they are reached through slots or their own component instances.
- The prototype-store, document-compiler, and v2 paths must remain behaviorally aligned. Add paired tests when changing precedence.
- Placeholder layout is transferred as parent-slot metadata; it does not overwrite the filled control's own internal layout contract.
- Placeholder mount layouts and component-root instance layouts merge recursively; caller- or instance-authored values win at the leaves.

## Test Coverage

Focused coverage currently verifies both compiler paths:

- `ui_document_compiler_applies_reference_instance_props_to_expanded_root`
- `prototype_compiler_applies_reference_instance_props_to_expanded_root`
- `ui_document_compiler_applies_reference_instance_style_overrides_after_stylesheets`
- `prototype_compiler_applies_reference_instance_style_overrides_after_stylesheets`
- `component_slot_placeholder_layout_survives_into_mounted_child_slot_contract`
- `ui_v2_composite_component_preserves_slot_placeholder_layout_on_filled_child`
- `component_slot_layout_resolves_component_tokens_params_and_caller_overrides`

The UI v2 composite test also builds a real runtime surface and computes a 260 logical-pixel row. It requires the filled field to begin after the fixed 105-pixel name column plus the 4-pixel container gap and to consume the exact remaining width. This frame-level assertion protects both root container inheritance and mounted-child slot inheritance rather than accepting metadata that is never used by layout.

The integration contract isolates the legacy compiler from the monolithic `#[cfg(test)]` tree and asserts three precedence layers together: a component token supplies width stretch, a component parameter supplies fixed height, and a caller mount supplies an additional width weight. Its pre-fix binary fails with the literal `$value_slot_stretch`; the production fix resolves the component environment before capturing placeholder layout. After Text 05 restored the missing `UiTextDistanceFieldEffects` import at its canonical Runtime Interface owner, the current-source Windows integration command completed as `1 passed / 0 failed`. This proves the token, parameter, and caller-leaf precedence through the normal Runtime library build rather than a test-only compiler path.
