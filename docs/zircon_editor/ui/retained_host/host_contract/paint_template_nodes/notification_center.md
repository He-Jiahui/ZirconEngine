---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/attributes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/entries.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/entry.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/options.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/parse.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/tests.rs
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_notification_center.zui
  - zircon_editor/src/tests/host/retained_window/native_material_painter_notification_center.rs
  - zircon_runtime/src/ui/surface/render/notification_center.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/attributes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/entries.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/entry.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/options.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/parse.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/tests.rs
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_notification_center.zui
plan_sources:
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
  - docs/plans/zircon_editor/editor_ui/index.md
tests:
  - zircon_editor/src/tests/host/retained_window/native_material_painter_notification_center.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center/tests.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/workbench_primitives.rs
  - zircon_runtime/src/ui/tests/render_notification_center.rs
  - rustfmt --edition 2021 touched NotificationCenter Workbench/native Rust files
  - git diff --check -- touched NotificationCenter Workbench/native files
doc_type: module-detail
---

# NotificationCenter Native Painter

`template_notification_center.rs` is the retained-host native painter for editor `NotificationCenter` overlay roots. It is component-owned: closed centers are consumed without drawing, and open centers paint their own panel, header, rows, severity markers, and empty state rather than falling through to generic template-node surface/text output.

Rows come from `TemplatePaneNodeData.structured_options`. `pane_component_projection/notification_center/` fills those rows from `notifications`, `selected_notification_id`, `focused_index`, `visible_limit`, `title`, and `empty_text`. `parse.rs` accepts TOML table entries, pipe-string entries, and arrays; `options.rs` preserves the retained-host row contract so authored Workbench assets and runtime state share the same native painter input.

`TemplatePaneOptionData.description`, `tone`, and `unread` are NotificationCenter row metadata. The native painter uses `description` for body text, `tone` for info/success/warning/error marker color, and `unread` for unread row emphasis and header count. Existing popup-row painters continue to ignore these fields.

The Workbench primitive is `workbench_notification_center.zui`. It keeps the same popup shell props under `.zui` governance as the descriptor: open flags, placement/anchor metadata, portal/modal flags, notification entries, selected/focused row state, visible limit, title, unread count, and empty text. `workbench_window.zui` imports the primitive so asset reachability checks can see the production reference chain.

The runtime render equivalent is `zircon_runtime/src/ui/surface/render/notification_center.rs`. Both paths must consume closed centers, suppress generic owner text/image/surface output, keep disabled rows visible, and preserve selected/focused/unread/severity visual distinctions. Reducer behavior and broader executable Cargo evidence remain later M3 work.
