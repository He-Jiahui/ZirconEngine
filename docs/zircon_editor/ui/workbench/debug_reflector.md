---
related_code:
  - zircon_editor/src/ui/workbench/debug_reflector/mod.rs
  - zircon_editor/src/ui/workbench/debug_reflector/model.rs
  - zircon_editor/src/ui/workbench/debug_reflector/selection.rs
  - zircon_editor/src/ui/workbench/debug_reflector/export.rs
  - zircon_editor/src/ui/workbench/debug_reflector/overlay.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/debug_reflector_overlay.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.ui.toml
  - zircon_editor/src/ui/slint_host/ui/tests.rs
implementation_files:
  - zircon_editor/src/ui/workbench/debug_reflector/mod.rs
  - zircon_editor/src/ui/workbench/debug_reflector/model.rs
  - zircon_editor/src/ui/workbench/debug_reflector/selection.rs
  - zircon_editor/src/ui/workbench/debug_reflector/export.rs
  - zircon_editor/src/ui/workbench/debug_reflector/overlay.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/debug_reflector_overlay.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.ui.toml
  - zircon_editor/src/ui/slint_host/ui/tests.rs
plan_sources:
  - docs/superpowers/specs/2026-05-07-debug-observatory-design.md
  - docs/superpowers/plans/2026-05-07-debug-observatory-m0-m1.md
  - docs/superpowers/specs/2026-05-06-ui-debug-reflector-full-closure-design.md
  - docs/superpowers/plans/2026-05-06-ui-debug-reflector-full-closure.md
  - user: 2026-05-06 continue UI Debug Reflector full closure milestone
tests:
  - zircon_editor/src/ui/workbench/debug_reflector/tests.rs
  - zircon_editor/src/tests/host/pane_presentation.rs
  - zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs
  - zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs
  - zircon_editor/src/tests/host/slint_runtime_diagnostics_template_body.rs
  - zircon_editor/src/tests/host/slint_window/ui_debug_reflector.rs
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
  - zircon_editor/src/tests/host/slint_window/shell_window.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - tests/acceptance/ui-debug-reflector-full-closure.md
doc_type: module-detail
---

# Editor UI Debug Reflector

The editor debug reflector is the workbench-side consumer for `UiSurfaceDebugSnapshot`. It does not own layout, hit-test, or render truth. Runtime produces the snapshot from `UiSurfaceFrame`; editor code turns that snapshot into strings, rows, details, JSON export/import results, and overlay toggle state.

## Model

`EditorUiDebugReflectorModel` is a read model for pane projection. It stores summary strings, flattened node rows, detail lines, subsystem sections, and warnings. `from_snapshot(...)` derives all values from `UiSurfaceDebugSnapshot`, including tree rows, selected-node details, render stats, render visualizer stats, renderer parity stats, hit-grid stats, overdraw stats, invalidation, and damage status.

The M7 render visualizer section consumes `UiSurfaceDebugSnapshot.render_batches.visualizer`. It exposes paint element count, batch groups, draw calls, visualizer overlay count, overdraw regions, resource binding count, text backend/glyph/decorator counters, and paint-cache reuse/rebuild counters. The renderer parity section consumes `render_batches.parity` and reports the canonical paint/batch row counts that runtime and editor renderers can compare without reading backend-private draw commands.

When the current pane pipeline has no active shared UI surface frame, `no_active_surface()` returns a stable placeholder. Runtime Diagnostics no longer relies on that placeholder for its own body: `apply_presentation` first builds the pane `body_surface_frame`, then `refresh_runtime_diagnostics_debug_reflector_from_body_surface(...)` converts the host-projected template nodes into a diagnostics-only `UiSurfaceFrame` and derives a normal `EditorUiDebugReflectorModel` from it. The placeholder remains valid for future workflows that intentionally show no selected surface.

## Selection And Export

`EditorUiDebugReflectorSelection` derives selected state from the snapshot, preferring the pick top hit when a pick dump exists and otherwise using `capture.selected_node`. The selection helper records the pick point by borrowing `UiHitTestQuery`, so Debug Reflector projection never consumes or mutates runtime UI snapshot state.

`snapshot_to_json(...)` and `load_snapshot_json(...)` wrap serde JSON export/import for editor callers. Runtime owns the payload shape; editor owns user-visible file handling and parse errors.

## Runtime Diagnostics Pane

Milestone 3 embeds the first reflector view in Runtime Diagnostics. The pane payload carries `ui_debug_reflector_summary`, node rows, detail lines, and export status. `pane_payload_projection.rs` injects those fields as template root attributes, while `runtime_diagnostics_body.ui.toml` exposes reflector summary/export/detail/list anchors.

Debug Observatory M1 adds a direct payload-build seam for an explicitly supplied active `UiSurfaceDebugSnapshot`. When `PanePayloadBuildContext::with_active_ui_debug_snapshot(...)` is used, Runtime Diagnostics projects the supplied snapshot into summary rows, details, export status, and shared overlay primitives before host conversion. When no active snapshot is supplied, the pane keeps the stable no-active placeholder and the later host-body refresh path can still derive a diagnostics-only snapshot from the pane's own `body_surface_frame`.

The Slint host conversion path uses `runtime_diagnostics.rs` to project those anchors and append text rows for reflector details. It rewrites the template nodes once with payload values before adding the generated text rows, so the host model does not keep stale authored placeholder labels beside live payload text. This keeps Runtime Diagnostics separate from Module Plugins in host contract data and lets the native painter draw the text/list section through normal template-node rendering.

The host-scene conversion seam is part of the reflector acceptance boundary because Runtime Diagnostics eventually travels through `apply_presentation.rs::to_host_contract_pane(...)` like every other workbench pane. That conversion must be payload-driven, not string-only. A host-scene pane may use a synthetic pane kind while carrying a real host-owned native-body payload; non-empty native-body buckets are therefore preserved even when `kind` is not the canonical editor pane label. The regression `host_scene_projection_converts_host_owned_panes_to_host_contract_panes` locks this boundary so Debug Reflector and other native panes do not lose their template nodes before the host contract rebuilds `body_surface_frame`.

The live M7 projection refresh is deliberately narrow. It only rebuilds the Runtime Diagnostics pane a second time when the first `body_surface_frame` produced a fresh reflector model. All other panes keep the existing single-pass presentation conversion. The generated model includes visibility, enabled state, input policy, clickable/hoverable/focusable flags, render/hit counts, focus/capture/pressed/hovered state, hit path, bubble route, reject reason, material batch breaks, and dirty flags so the pane mirrors the shared snapshot instead of local host guesses.

## Snapshot-Derived Overlay

Milestone 4 carries overlay records through the same Runtime Diagnostics payload instead of adding editor-local geometry. `RuntimeDiagnosticsPanePayload.ui_debug_reflector_overlay_primitives` stores shared `UiDebugOverlayPrimitive` records, and host conversion maps them into `RuntimeDiagnosticsPaneData.overlay_primitives` for the native painter.

`EditorUiDebugReflectorOverlayState` filters shared primitive kinds for selected frames, clip frames, render wireframes, hit cells, hit paths, rejected bounds, overdraw cells, material batch bounds, text glyph/baseline debug, resource atlas bounds, and damage. If damage is enabled and the snapshot has `UiDamageDebugReport.damage_region` but no explicit damage primitive, the overlay state synthesizes a `DamageRegion` primitive from the shared damage report.

The M7 overlay replay path also converts `UiRenderVisualizerOverlay` rows into `UiDebugOverlayPrimitive` records. `Wireframe`, `ClipScissor`, `BatchBounds`, `OverdrawHeat`, `TextGlyphBounds`, `TextBaseline`, and `ResourceAtlas` visualizer overlays therefore use the same Runtime Diagnostics pane payload and native painter as hit-grid, reject, material, and damage overlays. The editor still does not derive geometry locally; it only translates shared render visualizer rows into the existing overlay transport.

`draw_debug_reflector_overlay(...)` is a focused painter helper. `workbench.rs` only calls it after normal Runtime Diagnostics pane content is painted and before the shell debug refresh marker. The helper offsets primitive frames by the pane content origin and clips them to the pane content frame, so the overlay never changes layout, hit testing, render extraction, dispatch state, or host surface-frame authority.

## Boundaries

The editor reflector must not rebuild a hit grid, infer layout from host rectangles, or query Slint widget internals. When a pane body surface is available, it must first be converted into a runtime `UiSurfaceDebugSnapshot` and then passed through this module. Runtime Diagnostics follows that rule for its own host-projected body surface; selecting some other pane's live surface remains a separate workflow.

## Validation

M7 debug-reflector validation on 2026-05-07 used `E:\zircon-build\targets-ui-m7-current`: `rustfmt --edition 2021 --check` over the reflector/runtime-diagnostics touched files, `cargo check -p zircon_editor --lib`, `cargo test -p zircon_editor --lib ui_debug_reflector`, `cargo test -p zircon_editor --lib presenter::tests`, and `cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate`. These passed with existing warning noise. The live Runtime Diagnostics regression is covered by `runtime_diagnostics_live_body_surface_populates_debug_reflector_rows_and_overlays` inside the `ui_debug_reflector` filter.
