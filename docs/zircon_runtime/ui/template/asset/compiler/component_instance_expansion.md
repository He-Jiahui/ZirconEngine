---
related_code:
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs
  - zircon_runtime/src/ui/tests/asset_component_reference_layout.rs
  - zircon_runtime/src/ui/tests/asset_prototype_store.rs
implementation_files:
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - zircon_runtime/src/ui/tests/asset_component_reference_layout.rs
  - zircon_runtime/src/ui/tests/asset_prototype_store.rs
  - cargo test -p zircon_runtime --lib ui_document_compiler_applies_reference_instance_props_to_expanded_root --locked -- --nocapture
  - cargo test -p zircon_runtime --lib prototype_compiler_applies_reference_instance_props_to_expanded_root --locked -- --nocapture
  - cargo test -p zircon_runtime --lib ui_document_compiler_applies_reference_instance_style_overrides_after_stylesheets --locked -- --nocapture
  - cargo test -p zircon_runtime --lib prototype_compiler_applies_reference_instance_style_overrides_after_stylesheets --locked -- --nocapture
doc_type: module-detail
---

# Component Instance Expansion

## Purpose

The legacy template asset compiler and flat prototype-store compiler both expand reusable component references into concrete template nodes. This document owns the shared instance override rule used by those two compiler paths while the editor shell is being moved onto runtime UI assets.

The rule is intentionally simple: the component definition supplies the root node, then the instance site patches the expanded root. The instance site must be able to override ordinary props and final style values because Workbench assets use reusable controls such as labels, fields, tabs, and table rows with per-instance text, colors, and state.

## Behavior Model

`component_instance_expander.rs` handles the older `UiDocumentCompiler` path. `prototype_instancer.rs` handles the prototype-store path used by imported reusable assets. Both paths now merge the instance node's `props` into the expanded root attributes after resolving tokens and parameters. That means an instance-level value such as `foreground_color`, `text_tone`, `placeholder`, or `disabled` wins over the component's root defaults.

Style overrides follow the same precedence contract. Stylesheet rules may set a baseline on the expanded root, but an instance's inline style or resolved props patch is applied afterward and remains the effective value seen by later projection and render extraction. The tests use a `Label` component whose stylesheet sets a neutral foreground and primary text tone; the instance then overrides those values to error red and `error` tone.

## Design And Rationale

The Workbench shell relies on shared primitives rather than hand-authored native controls for every visual sample. Without instance root patching, reusable components collapse to their defaults and every authoring-site variant must fork a new component. That would make the shell asset set harder to maintain and would also hide semantic state, such as disabled fields or selected tabs, from the retained host.

The merge happens in the compiler support layer rather than in editor projection because the compiled runtime document is the authority. Both editor and runtime consumers should see the same expanded root metadata regardless of whether the document is projected into a retained host model or materialized into a runtime surface.

## Edge Cases And Constraints

- Instance props are resolved through the same token/parameter map as other compiler values before merging.
- The merge only patches the expanded root; child nodes keep their component-authored values unless they are reached through slots or their own component instances.
- The prototype-store path and document-compiler path must remain behaviorally aligned. Add paired tests when changing precedence.
- This document does not describe v2 `Slot` placeholder traversal. That path is owned by `docs/zircon_runtime/ui/v2.md`.

## Test Coverage

Focused coverage currently verifies both compiler paths:

- `ui_document_compiler_applies_reference_instance_props_to_expanded_root`
- `prototype_compiler_applies_reference_instance_props_to_expanded_root`
- `ui_document_compiler_applies_reference_instance_style_overrides_after_stylesheets`
- `prototype_compiler_applies_reference_instance_style_overrides_after_stylesheets`
