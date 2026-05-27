---
related_code:
  - zircon_runtime_interface/src/ui/ecs.rs
  - zircon_runtime_interface/src/ui/ecs/compute.rs
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/pipeline/stage.rs
  - zircon_runtime_interface/src/ui/pipeline/dirty_reason.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/tests/ui_ecs_projection_contracts.rs
  - zircon_runtime/src/ui/surface/ecs_projection.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/tests/ecs_projection.rs
  - zircon_runtime/src/ui/tests/mod.rs
implementation_files:
  - zircon_runtime_interface/src/ui/ecs.rs
  - zircon_runtime_interface/src/ui/ecs/compute.rs
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/ecs_projection.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
plan_sources:
  - .codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
tests:
  - zircon_runtime_interface/src/tests/ui_ecs_projection_contracts.rs
  - zircon_runtime/src/ui/tests/ecs_projection.rs
  - target: cargo test -p zircon_runtime_interface --lib ui_ecs_projection_contracts --locked --offline --jobs 1
  - target: cargo test -p zircon_runtime --lib ecs_projection --locked --offline --jobs 1
doc_type: module-detail
---

# Runtime UI ECS Projection

The runtime UI ECS projection is the first M5 bridge between retained `UiSurface` truth and Bevy-like schedule visibility. It is not a second UI tree and it does not own behavior. `UiSurface` remains authoritative for retained nodes, layout caches, focus/capture/hover state, component-state values, render extract, and hit grid data.

## Contract

`zircon_runtime_interface::ui::ecs` defines the neutral projection DTOs:

- `UiEcsProjectionSnapshot` carries the target `UiTreeId`, roots, projected node records, and recomputed totals.
- `UiEcsNodeProjection` carries stable node identity, parent/child links, component/control metadata, frame, dirty domains, interaction state, render command count, and hit-entry count.
- `UiEcsDirtyDomains` maps retained dirty flags into schedule-visible layout, text, input, picking, accessibility, render, style, and visible-range domains.
- `UiEcsInteractionState` records visible/enabled/disabled, focus/hover/press/capture, focusable/clickable/hoverable, checked/selected/expanded/popup-open, and dragging facts.

The projection is serde-friendly and default-compatible so editor diagnostics, future runtime schedule probes, and host tooling can consume it without importing runtime-private `UiSurface` internals.

`UiSurfaceFrame` and `UiSurfaceDebugSnapshot` both carry a defaulted `UiEcsProjectionSnapshot`. Older serialized frames or snapshots that do not include the field still deserialize to an empty projection, while new runtime diagnostics expose the projection next to arranged nodes, render extract, hit grid, focus state, rebuild stats, layout engine report, and pipeline report.

`UiEcsProjectionSnapshot::diff_from(...)` creates a read-only change-detection packet for the next schedule bridge. `UiEcsProjectionDelta` records previous/current tree ids, node-level `Added` / `Removed` / `Updated` changes, derived dirty domains, stable change reasons, and aggregate counts. Structural changes mark layout, picking, accessibility, and render domains; interaction changes mark input, accessibility, and render domains; render and hit-entry count changes mark their own domains. Added and removed nodes also preserve any retained dirty domains already present on the projected node.

`UiEcsProjectionNodeChange` and `UiEcsProjectionDelta` also expose fast-path classifiers. `changes_component_structure()` is true for added/removed nodes and parent/child/path/component changes; frame-only and interaction-only updates stay out of that bucket. `is_interaction_only()` identifies focus, hover, press, and capture style deltas that only need input/focus/widget/a11y/render attention. `UiEcsProjectionDeltaTotals` carries component-structure, interaction, and render-only counts so diagnostics and future schedule runners can prove hover/click paths do not rebuild component structure.

The same fast paths are available as concrete node-id lists. `component_structure_change_node_ids()`, `interaction_change_node_ids()`, `interaction_only_change_node_ids()`, and `render_only_change_node_ids()` let schedule probes and editor diagnostics render the affected nodes directly instead of pairing aggregate counters with a second manual scan.

`UiEcsProjectionDelta` now has direct change-query helpers as the row-level counterpart to those totals. `change(node_id)` looks up one node change, `changes_by_kind(kind)` and `node_ids_by_change_kind(kind)` split added/removed/updated rows, `changes_requiring_stage(stage)` returns changes whose domains require a single pipeline stage, and `changes_in_dirty_domain(domain)` returns changes carrying one raw dirty domain. Schedule systems should prefer these helpers over interpreting totals when they need to operate on concrete changed nodes.

`UiEcsProjectionScheduleMask` is the schedule-facing aggregate view. Snapshot and delta payloads now carry a defaulted `schedule_mask` field, and both also expose helpers that recompute the mask from the contained node/change records. Older serialized diagnostics that lack the field deserialize with an empty carried mask; callers can use `schedule_mask()` for a read-only recompute or `recompute_schedule_mask()` / `with_recomputed_schedule_mask()` to populate the carried field from the payload content. The mask reports whether the ordered `InputCollect -> Focus -> WidgetBehavior -> TextMeasure -> Layout -> PostLayout -> Picking -> A11yExtract -> RenderExtract -> BatchPrepare` stages need work and which `UiPipelineDirtyReason` values explain that work. Text dirtiness is treated conservatively as text measure plus layout/post-layout/picking/a11y/render/batch work because shaped text can change measured size, hit testing, accessibility names, and extracted render geometry.

`UiEcsProjectionScheduleImpact` is the node-level companion to the aggregate mask. Snapshot and delta payloads carry defaulted `schedule_impacts` rows, and `UiEcsProjectionSnapshot::schedule_impacts()` / `UiEcsProjectionDelta::schedule_impacts()` recompute those rows from current nodes or changed nodes. Each impact carries the stage, `required` flag, sorted node ids, node count, and stage-specific dirty reasons. Older serialized diagnostics that lack the field deserialize with an empty row list; importers can call `recompute_schedule_impacts()` or `with_recomputed_schedule_impacts()` to normalize the payload. This lets diagnostics answer "which nodes made TextMeasure or RenderExtract necessary?" without walking runtime-private `UiSurface` data or flattening the delta into a boolean-only mask.

`UiEcsDirtyDomainImpact` is the lower-level dirty-domain view. Snapshot and delta payloads carry defaulted `dirty_domain_impacts` rows, and the recompute helpers group dirty projected nodes by `layout`, `text`, `input`, `picking`, `accessibility`, `render`, `style`, and `visible_range`. This is intentionally separate from schedule impacts: schedule rows explain which pipeline stages run, while dirty-domain rows explain exactly which nodes dirtied each domain before stage expansion.

Snapshot and delta payloads also expose query helpers for future schedule runners and diagnostics. `schedule_impact(stage)` and `node_ids_requiring_stage(stage)` answer stage-level questions from recomputed impact rows. `dirty_domain_impact(domain)` and `node_ids_in_dirty_domain(domain)` answer raw-domain questions before pipeline expansion. These helpers keep external callers from duplicating the private grouping rules now isolated in `zircon_runtime_interface/src/ui/ecs/compute.rs`.

Snapshot and delta payloads now expose a unified derived-field freshness contract. `derived_fields_are_fresh()` checks that carried totals, the schedule mask, schedule-impact rows, and dirty-domain rows still match the authoritative node/change payload. `recompute_derived_fields()` and `with_recomputed_derived_fields()` refresh those carried fields together, which is the preferred importer path for legacy diagnostics that deserialize with missing derived fields. The individual recompute helpers remain for narrow tools, but schedule or diagnostic consumers should use the unified helper when normalizing a payload before display.

## Runtime Bridge

`UiSurface::ui_ecs_projection()` builds the snapshot from the current retained surface. It reads:

- `UiTree.nodes` for identity, metadata, frames, parent-child links, and dirty flags.
- `UiFocusState` plus `UiSurfaceComponentStateStore` for focus, hover, press, capture, selection, popup, and drag interaction facts.
- The shared disabled gate for effective disabled inheritance, matching pointer, keyboard, and accessibility behavior.
- `UiRenderExtract` and hit-grid entries for render/hit counts.

State-flag dirtiness is folded into input, picking, and render projection domains, matching the existing `UiSurface::dirty_flags()` behavior. A text dirty node therefore reports text, accessibility, and render schedule visibility; a layout dirty node additionally reports layout, picking, accessibility, and render visibility.

`UiSurface::surface_frame()` stores the latest projection on the frame, and `debug_surface_frame_with_options(...)` copies that projection into `UiSurfaceDebugSnapshot`. That keeps direct frame consumers, JSON diagnostics, editor debug reflection, and future schedule probes aligned on the same runtime facts.

`UiSurface::ui_ecs_projection_delta_from(...)` is a thin runtime helper over the interface diff. Callers keep the previous snapshot, ask the current surface for a delta, and use the returned domains to decide whether layout, text, input, picking, accessibility, or render systems need work. `UiSurface::ui_ecs_schedule_mask_from(...)` folds that delta into the schedule mask for callers that only need stage requirements. `UiSurface::ui_ecs_schedule_impacts_from(...)` is the matching convenience helper for callers that need stage rows and affected node ids without manually building a delta first. `UiSurface::ui_ecs_dirty_domain_impacts_from(...)` exposes the domain rows directly for inspectors that want to show, for example, that a single text dirty node only contributed text/accessibility/render domains before layout expansion. `UiSurface::ui_ecs_component_structure_change_node_ids_from(...)`, `ui_ecs_interaction_change_node_ids_from(...)`, `ui_ecs_interaction_only_change_node_ids_from(...)`, and `ui_ecs_render_only_change_node_ids_from(...)` provide the same fast-path node lists from a previous projection for runtime schedule probes. These helpers do not cache or mutate projection records, and the carried schedule/domain fields remain default-compatible for older serialized diagnostics.

## Boundary

The projection is read-only. Runtime systems may query it for scheduling and diagnostics, but updates must still go through retained `UiSurface` mutation paths such as input dispatch, widget reducers, accessibility actions, property mutation, layout, and render extraction.

Editor code may display the projection or use it for authoring diagnostics. It must not mutate runtime state by editing projected records.
