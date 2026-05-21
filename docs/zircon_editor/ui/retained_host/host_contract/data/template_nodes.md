---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_state_layer.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/assets/ui/editor/component_showcase.v2.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.v2.ui.toml
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_state_layer.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/assets/ui/editor/component_showcase.v2.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.v2.ui.toml
plan_sources:
  - user: 2026-05-15 continue Zircon Editor Demo first-screen and .zui showcase plan
  - user: 2026-05-20 migrate Slint Material component behavior into retained Editor UI without direct Slint runtime
  - .codex/plans/Zircon Editor Demo 首屏与 .zui 组件陈列计划.md
  - docs/superpowers/plans/2026-05-20-slint-material-retained-editor-migration.md
tests:
  - zircon_editor/src/ui/retained_host/ui/tests/component_showcase.rs
  - zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_state_layer.rs inline unit tests
  - cargo test -p zircon_editor --lib component_showcase_template_nodes_preserve_scroll_clip_frames --locked --target-dir target/codex-shared-b --message-format short (2026-05-15: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase --locked --target-dir target/codex-shared-b --message-format short -- --test-threads=1 (2026-05-15: passed, 24 passed)
  - cargo test -p zircon_editor --lib template_nodes --locked --target-dir target/codex-shared-b --message-format short (2026-05-15: passed, 7 passed)
  - cargo test -p zircon_editor --lib dual_host_parity_preserves_layout_attributes_and_routes_for_representative_documents --locked --target-dir target/codex-shared-b --message-format short (2026-05-15: passed, 1 passed)
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-shared-b --message-format short (2026-05-15: passed)
  - python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug (2026-05-15: passed)
  - E:\zircon-build\ZirconEngine\zircon_editor.exe --list-operations --headless (2026-05-15: passed, includes Window.UiComponentShowcase.Open)
  - .codex/run-logs/editor-noargs-smoke-polished.png (2026-05-15: no-argument editor smoke screenshot, Component Showcase first screen visible without bottom-log overlap)
doc_type: module-detail
---

# Template Pane Nodes

`TemplatePaneNodeData` is the retained host DTO for editor-authored template panes. It carries the visual and interaction facts that the native host needs after a shared `UiSurface` has already compiled and arranged the source document. The host contract must not rederive template geometry from component names or from per-pane coordinate tables.

## Spatial Authority

`frame` and `clip_frame` are pane-local layout facts projected from the shared runtime surface. `EditorUiHostRuntime::build_host_model_with_surface(...)` prefers `UiArrangedNode.frame` and `UiArrangedNode.clip_frame` from `surface.arranged_tree`; tests and metadata-only paths that pass an uncomputed surface still fall back to the raw `UiTreeNode.layout_cache`. On the normal laid-out pane path, the arranged clip frame is the effective clip after intersecting all clipping ancestors, so a descendant inside a `ScrollableBox` carries the scroll viewport clip even when its own frame extends far below the visible pane.

`has_clip_frame` distinguishes a shared-surface node with an arranged clip from older projection-only synthetic nodes. `pane_component_projection::host_template_node(...)` serializes the optional host-model clip into `has_clip_frame` plus `TemplateNodeFrameData`, while `template_node_conversion.rs` keeps projection-only fallbacks at `has_clip_frame = false`.

## Painting

`host_contract/painter/template_nodes.rs` intersects three regions before emitting paint commands:

- the pane body clip;
- the active frame damage clip, when the native presenter is repainting a subregion;
- the node's own retained clip frame, when `has_clip_frame` is true.

This matters for image and text nodes because image rasterization can be expensive and text can otherwise bleed outside a scroll viewport. The painter skips nodes whose frame does not intersect the effective clip before requesting image pixels, then passes the same clip into quad, image, and text commands.

## Slint Material State Metadata

The retained Material migration carries Slint-derived state-layer and ripple facts directly on `TemplatePaneNodeData` so the native host can paint Material behavior without linking Slint or generated `.slint` code.

The state-layer/ripple fields are:

- `enter_pressed`: keyboard activation metadata equivalent to `FocusTouchArea.enter_pressed`; runtime input routing owns when it is set.
- `state_layer_enabled`: opt-in for drawing a Material state-layer overlay.
- `state_layer_color`: source `StateLayerArea.color` / `Ripple.color`; the painter falls back to the host focus-ring color when it is transparent.
- `ripple_enabled`: opt-in for the retained static press ripple.
- `ripple_pressed_x` and `ripple_pressed_y`: source-compatible press origin metadata from `pressed_x` / `pressed_y` or explicit retained ripple attrs.
- `ripple_unclipped`: host-internal inverse of source `clip_ripple`.

Projection accepts both retained names and source-compatible names. `pane_component_projection::host_template_node(...)` maps `state_layer_enabled` / `display_state_layer`, `state_layer_color` / `ripple_color` / `color`, `ripple_enabled` / `ripple`, `ripple_pressed_x` / `pressed_x`, `ripple_pressed_y` / `pressed_y`, and `clip_ripple`. Older conversion paths in `template_node_conversion.rs` default all M2 metadata to inactive values so existing template nodes do not accidentally draw overlays.

`host_contract/painter/material_state_layer.rs` applies the retained priority `disabled > focus/selected/checked > pressed/enter_pressed > drag > hover/drop/active-drag-target > default`. Disabled uses focus opacity to preserve the source `root.state_layer_opacity: MaterialPalette.state_layer_opacity_focus` behavior for disabled display backgrounds. `ripple_enabled` does not imply `state_layer_enabled`, so callers may request only the static press ripple without the full overlay. Ripple is intentionally static in M2: it draws a width-derived press-origin circle with press opacity while animation timing remains metadata in `editor_material.v2.ui.toml` for a later motion layer.

## Hit Testing

`host_contract/surface_hit_test/template_node.rs` reconstructs a temporary surface for template-node hit testing. When a `TemplatePaneNodeData` row has a clip frame, the reconstructed `UiTreeNode.layout_cache.clip_frame` is seeded before `surface.rebuild()`. The shared hit grid then rejects pointer hits outside the same effective clip used by painting.

## Authoring Contract

Scrollable showcase regions must be authored as real shared scroll containers:

```toml
component = "ScrollableBox"
layout = { container = { kind = "ScrollableBox", axis = "Vertical", gap = 10.0, scrollbar_visibility = "Auto" } }
```

A node whose component name is `ScrollableBox` but whose layout container is `VerticalBox` will arrange like a linear panel and will not contribute a scroll viewport clip to descendants. The Component Showcase smoke regression uses the lower `ContextActionMenuDemo` row in a short viewport to prove that off-screen rows receive the `ComponentShowcaseScroll` clip instead of painting over the bottom log.
