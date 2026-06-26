---
related_code:
  - zircon_editor/src/ui/retained_host/ui/template_layout_context.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/native_template_node_panes.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation/pane_conversion.rs
  - zircon_editor/src/ui/workbench/autolayout/layout_tier.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/allocation.rs
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15d-composite-density-and-alignment.md
  - docs/plans/zircon_editor/editor_layout/15e-domain-breakpoint-adaptation.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - cargo fmt -p zircon_editor --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib asset_browser_table_nodes_receive_narrow_context_variant --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
  - cargo test -p zircon_editor --lib table_nodes_receive_context_tier_variant --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never
doc_type: module-detail
---

# Template Layout Context

`template_layout_context.rs` is the retained-host UI projection owner for component-level responsive context tags. It translates an already-known Workbench or pane content width into stable component variant tokens before the node reaches paint leaves.

## Ownership

- Maps context width through `workbench_layout_tier_for_width`.
- Appends `layoutNarrow`, `layoutRegular`, or `layoutWide` to table nodes only.
- Leaves non-table nodes untouched.
- Deduplicates variant tokens so repeated projection does not grow strings.

## Consumers

- `workbench_window_projection.rs` derives context width from `UiHostWindowRoot` and tags Workbench component-showcase table nodes.
- `native_template_node_panes.rs` tags Asset Browser table nodes using the pane content width passed by `pane_conversion.rs`.
- `template_table_rows/cells/allocation.rs` consumes `layoutNarrow` as a paint-local degradation hint.

## Boundary

This module does not paint, allocate columns, or decide drawer geometry. It only bridges layout context to component variants. Paint leaves stay unaware of shell or pane structures; shell autolayout remains the owner of actual Workbench tier classification.
