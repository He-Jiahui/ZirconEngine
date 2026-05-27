---
related_code:
  - zircon_runtime/src/ui/surface/ecs_projection.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - zircon_runtime/src/ui/tests/ecs_projection.rs
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/tests/pipeline_report.rs
  - zircon_runtime_interface/src/ui/ecs.rs
  - zircon_runtime_interface/src/ui/ecs/compute.rs
  - zircon_runtime_interface/src/ui/pipeline/mod.rs
  - zircon_runtime_interface/src/ui/pipeline/stage.rs
  - zircon_runtime_interface/src/ui/pipeline/stage_report.rs
  - zircon_runtime_interface/src/ui/pipeline/frame_report.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime_interface/src/tests/contracts.rs
implementation_files:
  - zircon_runtime/src/ui/surface/ecs_projection.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime_interface/src/ui/pipeline/mod.rs
  - zircon_runtime_interface/src/ui/pipeline/stage.rs
  - zircon_runtime_interface/src/ui/pipeline/stage_report.rs
  - zircon_runtime_interface/src/ui/pipeline/frame_report.rs
  - zircon_runtime_interface/src/ui/ecs.rs
  - zircon_runtime_interface/src/ui/ecs/compute.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
plan_sources:
  - docs/superpowers/specs/2026-05-06-ui-debug-reflector-full-closure-design.md
  - docs/superpowers/plans/2026-05-06-ui-debug-reflector-full-closure.md
  - user: 2026-05-06 continue UI Debug Reflector full closure milestone
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - zircon_runtime/src/ui/tests/pipeline_report.rs
  - cargo test -p zircon_runtime --lib surface_debug_snapshot_json_exports --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-diagnostics-20260522 --message-format short --color never (2026-05-22: passed, 2 passed after fallback_reason_counts JSON assertions; lock restored)
  - cargo test -p zircon_runtime_interface ui_surface_debug_snapshot_legacy_layout_report_recovers_fallback_reason_counts --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-snapshot-deser-20260522 --message-format short --color never (2026-05-22: passed, 1 passed after nested legacy layout report aggregate recovery; lock restored)
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - tests/acceptance/ui-debug-reflector-full-closure.md
doc_type: module-detail
---

# Runtime UI Surface Diagnostics

Runtime UI surface diagnostics generate the shared debug snapshot consumed by the editor Debug Reflector. The snapshot source is `UiSurfaceFrame`, which already contains arranged nodes, render extract, hit grid, focus state, and rebuild counters.

## Snapshot Contract

`zircon_runtime_interface::ui::surface::UiSurfaceDebugSnapshot` is the neutral serde contract. It carries schema version/capture context, widget reflector nodes, layout-engine route reports, UI pipeline reports, UI ECS projection facts, render command records, material batches, `UiRenderDebugSnapshot.render_batches`, hit-grid cell records, pick hit-test dumps, sampled overdraw cells, invalidation and damage reports, event records, and overlay primitives.

`pipeline_report` is the Bevy-aligned M7 schedule bridge. `UiSurfaceFrame` and `UiSurfaceDebugSnapshot` both carry the defaulted `UiPipelineFrameReport` so editor diagnostics can display the required `InputCollect -> Focus -> WidgetBehavior -> TextMeasure -> Layout -> PostLayout -> Picking -> A11yExtract -> RenderExtract -> BatchPrepare` order without depending on runtime-private rebuild fields. The current runtime projection is conservative: `UiSurfaceRebuildReport::pipeline_report(...)` fills measured timing/counters for layout, arranged post-layout, picking, and render extract; input/focus/widget/text/a11y/batch remain ordered skipped stages with notes until those owners expose direct timing.

`ecs_projection` is the M5 Bevy-like schedule visibility bridge. `UiSurfaceFrame` and `UiSurfaceDebugSnapshot` both carry a defaulted `UiEcsProjectionSnapshot` derived from retained `UiSurface` nodes, dirty flags, component state, focus/capture/hover facts, render extract counts, hit-grid counts, and the shared effective-disabled gate. Diagnostics consume it as read-only evidence; edits still go through retained runtime mutation paths.

Projection diffing stays in the same read-only boundary. `UiSurface::ui_ecs_projection_delta_from(...)` returns a `UiEcsProjectionDelta` comparing a caller-held previous snapshot with the current surface projection. The delta records added, removed, and updated nodes plus derived schedule domains, so future diagnostics and schedule views can show why a frame needs layout, picking, accessibility, render, text, or input work without interpreting private runtime tree fields.

`UiSurface::ui_ecs_schedule_mask_from(...)` folds that delta into `UiEcsProjectionScheduleMask`. This is the diagnostic-friendly form for displaying M7 stage requirements: it reports ordered pipeline stages and dirty reasons without exposing every node-level change when a caller only needs to know which pipeline domains are active. Projection snapshots and deltas also serialize their defaulted `schedule_mask`, so JSON diagnostics can carry stage requirements directly while older payloads continue to deserialize with an empty mask. Importers can call the recompute helpers when they want to normalize legacy snapshots before displaying schedule rows.

When a diagnostic view needs row-level detail, snapshot and delta payloads carry defaulted `UiEcsProjectionScheduleImpact` rows, and `UiEcsProjectionSnapshot::schedule_impacts()` / `UiEcsProjectionDelta::schedule_impacts()` can recompute them from nodes or changes. Each row groups sorted affected node ids and stage-specific dirty reasons for one active schedule stage, so an editor inspector or JSON importer can show the exact nodes that made `TextMeasure`, `Layout`, `Picking`, `A11yExtract`, `RenderExtract`, or `BatchPrepare` necessary without reading runtime-private tree internals. Legacy diagnostics that lack the carried rows still deserialize; importers can call `recompute_schedule_impacts()` before displaying detailed rows.

For lower-level dirty-domain views, snapshots and deltas also carry defaulted `UiEcsDirtyDomainImpact` rows. These group affected nodes by raw projection domain before pipeline expansion, so diagnostics can show a text edit as text/accessibility/render domain dirtiness and separately explain why the schedule expanded that into layout, picking, a11y, render, and batch stages.

Stage/domain query helpers are the preferred diagnostic read path when a view needs node ids for one row. `node_ids_requiring_stage(...)` and `node_ids_in_dirty_domain(...)` derive their answers through the same grouping logic as the carried impact rows, so schedule views do not need their own partial interpretation of projection dirty flags.

Pipeline diagnostics can compare a pre-rebuild ECS projection mask with the post-rebuild `UiPipelineFrameReport`. Render-only dirtiness, for example, now has focused coverage showing the projection requires `RenderExtract` / `BatchPrepare` while the rebuild report keeps layout, post-layout, and picking skipped.

Diagnostic importers should normalize archived or partial projection payloads with `recompute_derived_fields()` before rendering schedule rows. `derived_fields_are_fresh()` lets the importer detect stale carried totals, schedule masks, stage-impact rows, or dirty-domain rows without reinterpreting runtime-private `UiSurface` state.

Delta fast-path classifiers are also part of the diagnostic contract. `component_structure_change_count`, `interaction_change_count`, and `render_only_change_count` let schedule views separate component rebuild triggers from pointer/focus interaction changes. A hover, focus, press, or capture-only delta reports `interaction_only()` and leaves component-structure counts at zero, which is the evidence future editor diagnostics need before showing a narrow input/focus/widget/a11y/render refresh path.

When a diagnostic pane needs the actual rows behind those counters, it can use `component_structure_change_node_ids()`, `interaction_change_node_ids()`, `interaction_only_change_node_ids()`, and `render_only_change_node_ids()`. These helpers keep fast-path displays tied to the same classifiers as the aggregate totals.

For concrete delta rows, diagnostics should use `change(node_id)`, `changes_by_kind(...)`, `changes_requiring_stage(...)`, and `changes_in_dirty_domain(...)`. These helpers keep row-level schedule panes aligned with the same domain expansion used by the aggregate schedule mask and impact rows.

`render_batches` is the replay-friendly render slice for M7 tools. It carries batch/debug rows, cache status, renderer parity rows, and `UiRenderVisualizerSnapshot` paint rows, batch groups, overlays, overdraw regions, resource bindings, text backend/glyph/decorator counters, and cache reuse/rebuild stats. Editor Debug Reflector reads this packet directly; renderer-specific code does not need to expose private draw commands.

`UiDebugOverlayPrimitiveKind` covers both surface diagnostics and render visualizer replay. Besides selected frames, clip frames, hit cells, hit paths, rejected bounds, overdraw cells, material batch bounds, and damage regions, it now includes render wireframes, text glyph bounds, text baselines, and resource atlas bounds.

Stable hit-test reject reasons live in `UiHitTestRejectReason`, with a separate human-readable message. Editor tests should assert reason codes instead of string-matching messages.

## Runtime Generation

`debug_surface_frame_with_options(...)` builds the baseline snapshot. `debug_surface_frame_for_selection(...)` adds selected-node capture and selection overlay primitives. `debug_surface_frame_for_pick(...)` runs query-aware frame hit testing, stores the pick dump, selects the top hit, and emits hit-path/rejected-bounds overlay primitives.

`UiSurface::debug_snapshot_for_selection(...)`, `UiSurface::debug_snapshot_for_pick(...)`, and `UiSurface::debug_snapshot_json(...)` are convenience methods for callers that already own a retained surface. JSON export remains payload-only; editor code owns paths and import/export UI. The JSON path includes `UiLayoutEngineSelectionReport`, so a Taffy-native or Zircon-fallback route visible in the frame is also visible in serialized diagnostics, including the aggregated `fallback_reason_counts` summary used by the editor reflector. When imported debug snapshots come from older tools that do not contain the aggregate summary, the nested layout report recomputes all route counts from `selections` during serde deserialization instead of trusting stale wire counters.

Surface frame generation now also includes `UiPipelineFrameReport` generated from the latest `UiSurfaceRebuildReport` and the current `UiEcsProjectionSnapshot` generated from retained runtime facts. This keeps `debug_snapshot()`, `debug_snapshot_json()`, and direct `surface_frame()` consumers aligned on one ordered schedule DTO and one schedule-visible ECS projection. Render-only rebuilds therefore report skipped layout/post-layout/picking stages while still exposing render extract dirty reasons and command reuse/rebuild counters.

## Current Limits

Backend render counters are optional until host render backends provide measured draw-call and pipeline data. Runtime damage reporting is currently default/empty; editor host damage integration should populate `UiDamageDebugReport` later without changing the snapshot authority model.

## Validation

M7 diagnostics validation on 2026-05-07 used `E:\zircon-build\targets-ui-m7`: `cargo check -p zircon_runtime_interface --tests`, `cargo test -p zircon_runtime --lib diagnostics`, and `cargo test -p zircon_runtime --lib hit_grid`. These passed with existing runtime warning noise.

The 2026-05-20 layout-engine route export check passed:

```powershell
cargo test -p zircon_runtime --lib surface_debug_snapshot_json_exports_layout_engine_route_report --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-impl --message-format short --color never
```

That focused test proves a computed Taffy-native horizontal surface exports `layout_engine_report` through `debug_snapshot_json(...)`, round-trips it through serde, and preserves the root selection as `Taffy` + `Native`.

The 2026-05-22 fallback-reason count JSON continuation passed:

```powershell
cargo test -p zircon_runtime --lib surface_debug_snapshot_json_exports --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-diagnostics-20260522 --message-format short --color never
```

That focused run passed `2 passed; 0 failed; 1852 filtered out`. The Taffy-native route export now asserts an empty serialized `fallback_reason_counts` array, while the Zircon fallback route export asserts the serialized `zircon_owned_semantics` reason/count pair and the deserialized `UiLayoutEngineSelectionReport.fallback_reason_counts` structure. The run used temporary lockfile backup/restore because unrelated sound/cpal lock drift blocks `--locked --offline`; existing runtime warning noise remains.

The 2026-05-22 nested legacy snapshot import check passed:

```powershell
cargo test -p zircon_runtime_interface ui_surface_debug_snapshot_legacy_layout_report_recovers_fallback_reason_counts --offline --jobs 1 --target-dir D:\cargo-targets\zircon-layout-snapshot-deser-20260522 --message-format short --color never
```

That focused contract test passed `1 passed; 0 failed; 104 filtered out`. It serializes a full `UiSurfaceDebugSnapshot`, removes `fallback_reason_counts`, corrupts the stored aggregate counters, and proves deserialization recovers `request_count`, `fallback_count`, `unsupported_count`, and the `ZirconOwnedSemantics=1` reason summary from the nested route selections. The run used the same temporary lockfile backup/restore workaround.
