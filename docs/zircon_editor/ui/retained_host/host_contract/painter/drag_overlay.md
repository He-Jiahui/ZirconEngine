---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_drag_overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/drag_overlay.rs
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_drag_overlay.zui
  - zircon_editor/src/tests/host/retained_window/native_material_painter_drag_overlay.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/drag_overlay_tests.rs
  - zircon_runtime/src/ui/surface/render/drag_overlay.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_drag_overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/drag_overlay.rs
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_drag_overlay.zui
plan_sources:
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
  - docs/plans/zircon_editor/editor_ui/index.md
tests:
  - zircon_editor/src/tests/host/retained_window/native_material_painter_drag_overlay.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/drag_overlay_tests.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/workbench_primitives.rs
  - zircon_runtime/src/ui/tests/render_drag_overlay.rs
  - rustfmt --edition 2021 touched DragOverlay Workbench/native Rust files
  - git diff --check -- touched DragOverlay Workbench/native files
doc_type: module-detail
---

# DragOverlay Native Painter

`template_drag_overlay.rs` is the retained-host native painter for editor `DragOverlay` overlay roots. It is component-owned: closed overlays are consumed without drawing, and open overlays paint their own preview chip and drop indicator instead of falling through to generic template-node surface or text output.

The retained data comes from `pane_component_projection/drag_overlay.rs`. That projector maps `payload_kind`, `payload_label`, `payload_reference`, `cursor_x/y`, `offset_x/y`, `preview_width/height`, `drop_allowed`, `drop_target_*`, `drop_indicator_edge`, and `drop_indicator_text` into dedicated `TemplatePaneNodeData` fields. It also treats `dragging = true` as an open overlay so native preview paint does not depend on a separate `popup_open` flag.

The Workbench primitive is `workbench_drag_overlay.zui`. It exposes the same descriptor-facing drag visual contract under asset governance and is imported by `workbench_window.v2.ui.toml` for reachability. This keeps authored Workbench surfaces, runtime render extract, and the retained native host on the same payload/cursor/drop-target vocabulary.

The runtime render equivalent is `zircon_runtime/src/ui/surface/render/drag_overlay.rs`. Both paths consume closed overlays, suppress generic owner text/image/surface output, draw cursor-offset preview chips, and preserve allowed versus blocked drop-indicator color. The native painter uses a compact payload marker rather than the runtime icon command because the host command layer currently has no tinted vector-icon primitive; the marker still preserves the state color lane needed by native pixel tests.

Executable Cargo evidence remains part of the M3.S4 testing stage. Current focused tests are in place for Workbench asset governance, retained-host projection, native pixels, and runtime render extract.
