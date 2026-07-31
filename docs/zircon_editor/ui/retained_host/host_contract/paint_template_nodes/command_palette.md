---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_command_palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/attributes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/entries.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/entry.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/ids.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/options.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/parse.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/tests.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_command_palette.rs
  - zircon_runtime/src/ui/surface/render/command_palette.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_command_palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/options.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/entries.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/ids.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/parse.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
  - docs/plans/zircon_editor/editor_ui/index.md
tests:
  - zircon_editor/src/tests/host/retained_window/native_material_painter_command_palette.rs
  - zircon_runtime/src/ui/tests/render_command_palette.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests/actions.rs
  - rustfmt --edition 2021 touched CommandPalette render/native painter Rust files
  - git diff --check -- touched CommandPalette render/native painter/doc files
doc_type: module-detail
---

# CommandPalette Native Painter

`template_command_palette.rs` is the retained-host native painter for `CommandPalette` roots projected from runtime UI. It is a component-owned painter, not a generic template-node fallback and not the ordinary dropdown option-row painter.

Closed `CommandPalette` roots are consumed without drawing. This prevents closed overlay shells from falling back to generic surface painting and leaving a blank popup panel in native previews.

Open roots draw the whole palette inside the node frame: popup panel, focused search field, query or placeholder text, and command rows. The panel uses the Workbench popup surface and border colors, the search field uses the inset surface plus focus-ring border, and the row region starts below the search field with the same compact row metrics used by the runtime render extractor.

Rows come from `TemplatePaneNodeData.structured_options`, which is filled by the `pane_component_projection/command_palette/` owner tree from the typed query window projection, selection/focus, recent commands, and query state. The painter does not re-filter commands; it renders the already-projected host contract.

Real editor command rows are projected from `EditorCommandRegistry::command_palette_query_window` over the generation-owned immutable catalog. Descriptors whose effective `when` evaluates to false are removed by that registry query before any `UiValue` or retained-host row is created; the command palette has no separate disabled-row side channel and does not keep unavailable command rows visible. This is the same effective-when gate used by menu and invocation paths.

The native painter intersects the panel list extent with the current clip, derives the exact visible
row range, and expands it by one row on each side for overscan. Only that range is visited. Row data
is borrowed through `ModelRc::get`; the deleted `0..row_count` plus `row_data` path cannot clone or
build text for rows outside the visible window. Absolute catalog row indices still feed row geometry
and paint ordering, so selection/focus/commit identity and pixels do not change when a window begins
after row zero.

Row visual state still goes through `select_workbench_popup_row_style(...)` and `WorkbenchPopupRowState`, so selected, focused, loading-style colors, and recent-command emphasis stay aligned with the popup-row selector contract. The painter draws rows locally because `template_popup_rows.rs` positions dropdown rows outside the owner frame, while `CommandPalette` rows are embedded inside the palette panel.

The runtime equivalent is `zircon_runtime/src/ui/surface/render/command_palette.rs`. Both paths must continue to suppress generic owner text/image/surface output, paint only while open, and consume the already-filtered shared-registry projection without rebuilding enablement rules in the painter.
