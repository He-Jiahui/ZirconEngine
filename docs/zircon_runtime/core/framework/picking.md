---
related_code:
  - zircon_runtime/src/core/framework/picking/mod.rs
  - zircon_runtime/src/core/framework/picking/backend.rs
  - zircon_runtime/src/core/framework/picking/debug_feed.rs
  - zircon_runtime/src/core/framework/picking/hit_data.rs
  - zircon_runtime/src/core/framework/picking/hit_record.rs
  - zircon_runtime/src/core/framework/picking/hit_target.rs
  - zircon_runtime/src/core/framework/picking/hover_map.rs
  - zircon_runtime/src/core/framework/picking/pickable.rs
  - zircon_runtime/src/core/framework/picking/pipeline.rs
  - zircon_runtime/src/core/framework/picking/pointer_button.rs
  - zircon_runtime/src/core/framework/picking/pointer_event.rs
  - zircon_runtime/src/core/framework/picking/pointer_event_state.rs
  - zircon_runtime/src/core/framework/picking/pointer_hits.rs
  - zircon_runtime/src/core/framework/picking/pointer_id.rs
  - zircon_runtime/src/core/framework/picking/pointer_input.rs
  - zircon_runtime/src/core/framework/picking/pointer_location.rs
  - zircon_runtime/src/core/framework/picking/pointer_phase.rs
  - zircon_runtime/src/core/framework/picking/primitive_backend.rs
  - zircon_runtime/src/core/framework/picking/ray.rs
  - zircon_runtime/src/core/framework/picking/ray_map.rs
  - zircon_runtime/src/core/framework/picking/report.rs
  - zircon_runtime/src/core/framework/picking/schedule_label.rs
  - zircon_runtime/src/core/framework/picking/settings.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_editor/src/scene/viewport/pointer/viewport_pointer_route.rs
  - zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs
  - zircon_editor/src/scene/viewport/pointer/precision/candidate_score.rs
  - zircon_editor/src/scene/viewport/pointer/precision/precision_candidate_score.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_debug.rs
  - zircon_editor/src/scene/viewport/pointer/constants.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_pointer_route.rs
implementation_files:
  - zircon_runtime/src/core/framework/picking/mod.rs
  - zircon_runtime/src/core/framework/picking/backend.rs
  - zircon_runtime/src/core/framework/picking/debug_feed.rs
  - zircon_runtime/src/core/framework/picking/hit_data.rs
  - zircon_runtime/src/core/framework/picking/hit_record.rs
  - zircon_runtime/src/core/framework/picking/hit_target.rs
  - zircon_runtime/src/core/framework/picking/hover_map.rs
  - zircon_runtime/src/core/framework/picking/pickable.rs
  - zircon_runtime/src/core/framework/picking/pipeline.rs
  - zircon_runtime/src/core/framework/picking/pointer_event.rs
  - zircon_runtime/src/core/framework/picking/pointer_event_state.rs
  - zircon_runtime/src/core/framework/picking/pointer_hits.rs
  - zircon_runtime/src/core/framework/picking/pointer_input.rs
  - zircon_runtime/src/core/framework/picking/pointer_location.rs
  - zircon_runtime/src/core/framework/picking/primitive_backend.rs
  - zircon_runtime/src/core/framework/picking/ray.rs
  - zircon_runtime/src/core/framework/picking/ray_map.rs
  - zircon_runtime/src/core/framework/picking/report.rs
  - zircon_editor/src/scene/viewport/pointer/viewport_pointer_route.rs
  - zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs
  - zircon_editor/src/scene/viewport/pointer/precision/candidate_score.rs
  - zircon_editor/src/scene/viewport/pointer/precision/precision_candidate_score.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_debug.rs
  - zircon_editor/src/scene/viewport/pointer/constants.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_pointer_route.rs
plan_sources:
  - user: 2026-05-08 implement Runtime Picking / Gizmos / Camera / Remote Bevy completion plan
  - .codex/plans/runtime-picking-gizmos-camera-remote-bevy-completion-plan.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/tests/picking/mod.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - cargo test -p zircon_runtime picking --locked
  - cargo test -p zircon_editor viewport --locked
doc_type: module-detail
---

# Runtime Picking Framework Contracts

## Purpose

`zircon_runtime::core::framework::picking` is the neutral runtime contract layer for viewport and world picking. It owns pointer identity, pointer location, viewport ray construction, backend hit output, hit target ordering, pickability semantics, hover-map state, and pointer event vocabulary so editor tools, runtime dev tools, remote tooling, and future render or mesh backends do not each invent their own route model.

The first implementation slice covered the M1 contract from the Bevy completion plan. The M2 slice adds a plain-Rust backend/event layer: generic primitive ray hits, hover map construction from backend output, pointer input DTOs, and Bevy-shaped pointer event state transitions. It still does not implement GPU picking, exact scene mesh traversal, editor selection, undo, or transform gizmo behavior. Those belong to later milestones and should consume these contracts instead of replacing them.

## Reference Evidence

The primary reference is Bevy picking:

- `dev/bevy/crates/bevy_picking/src/backend.rs` defines `PointerHits`, `HitData`, backend ordering, and `ray::RayMap`.
- `dev/bevy/crates/bevy_picking/src/lib.rs` defines `Pickable` and the staged picking system labels.
- `dev/bevy/crates/bevy_picking/src/events.rs` defines the pointer event vocabulary, event order, per-pointer button state, click/release targeting of previous hover, and drag/drop transitions mirrored by Zircon's M2 event state.
- `dev/bevy/crates/bevy_picking/src/hover.rs` defines backend-output-to-hover-map reduction, including the rule that canceled pointers do not receive backend hits and that `Pickable` hover/blocking controls lower hits.

Secondary cross-checks came from editor/runtime engines that keep authoring picking outside the runtime world authority:

- `dev/Fyrox/editor/src/interaction/gizmo/move_gizmo.rs` and `dev/Fyrox/editor/src/scene_viewer/gizmo.rs` keep editor gizmo interaction as a consumer of scene/camera data rather than serialized runtime scene state.
- `dev/godot/scene/3d/physics/ray_cast_3d.cpp` and `dev/godot/editor/scene/3d/gizmos/physics/ray_cast_3d_gizmo_plugin.cpp` separate raycast state, debug drawing, and editor gizmo visualization.

## Ownership Boundary

The runtime framework owns reusable contracts only. It is not the runtime world authority and it is not the editor UX controller. `HitTarget` intentionally contains the three route categories already used by the editor, but the enum now lives in the runtime framework so later gizmos, picking debug overlays, and remote selection can share the same target vocabulary.

The editor remains responsible for Slint/window input adaptation, candidate generation from current overlay DTOs, selection commands, undo, and viewport UX. The existing `ViewportPointerRoute` now adapts to and from runtime `HitTarget`, while its private editor enum can still carry editor-local axis types until the later transform gizmo cutover removes the old handle registry.

## Data Model

The module is split by declaration so the root stays structural:

- `PointerId`, `PointerLocation`, `PointerButton`, and `PointerPhase` describe device-neutral pointer facts.
- `CameraRaySource`, `RayId`, `RayMap`, and `PointerRay` cache pointer/camera rays per viewport.
- `HitData`, `HitRecord`, `PointerHits`, `PickingHoverMap`, and `Pickable` describe backend output and hover filtering.
- `HitTarget`, `PickingAxis`, and `PickingTargetPriority` define the shared picking target vocabulary and the current M1 priority order: handle axis, scene gizmo, then renderable.
- `PickingBackend`, `PickingBackendInfo`, and `PickingBackendCapability` reserve a backend seam without committing to GPU picking or a mesh backend in M1.
- `PointerInput`, `PointerAction`, and `PointerScrollUnit` describe ordered pointer input records that drive pointer events.
- `PickingPointerEvent`, `PickingEventKind`, `PickingEventLabel`, and `PickingEventState` describe runtime-neutral interaction events and the per-pointer/button state machine that emits them.
- `PrimitivePickingBackend`, `PickingPrimitive`, and `PickingPrimitiveShape` provide a small CPU ray primitive backend for overlay, gizmo, and coarse scene-primitive tests. The first shape is a sphere; exact mesh picking remains a later backend concern.
- `PickingPipelineReport` and `PickingPointerPipelineReport` provide a stable diagnostics snapshot of one picking frame: ray count, pointer count, backend output count, raw hit count, hovered hit count, blocking pointer count, and per-pointer top/blocking target details.
- `PickingDebugFeed`, `PickingDebugMetric`, and `PickingDebugPointerRow` convert a pipeline report into stable summary metrics and per-pointer rows for editor overlays, remote tools, and diagnostics panels.
- `PickingPipelineInput`, `PickingPipelineOutput`, `PickingPipelineStageReport`, and `run_picking_pipeline` provide a reusable plain-Rust stage runner for one picking frame. The runner is intentionally independent of ECS schedules, editor windows, or render backend ownership.
- `PickingSettings` and `PickingScheduleLabel` reserve the configuration and pipeline labels later used by pointer events and backend orchestration.

## Behavior

`RayMap::rebuild` clears and repopulates rays from active cameras and current pointer locations. It filters by viewport handle and viewport bounds, so multiple pointers, viewports, and cameras can coexist without producing accidental cross-viewport hits. Ray generation derives aspect ratio from the active viewport size rather than trusting a reused camera snapshot, so off-center rays remain correct when one camera feeds multiple viewport sizes.

`ray_from_viewport_point` supports perspective and orthographic camera snapshots using the same transform direction conventions as the current runtime math layer. It returns `None` for zero-sized viewports, out-of-bounds pointer positions, or invalid ray directions.

`sorted_hits_for_pointer` merges backend outputs for a pointer. The current editor cutover priority sorts `HandleAxis` before `SceneGizmo` before `Renderable` before backend order, so a renderable backend cannot steal a transform-handle route by using a higher backend order. Backend order and depth then break ties within the same target priority. This preserves existing editor route behavior while moving the semantics to a runtime-owned contract. A single backend should place all hits for one pointer in one `PointerHits` value; separate `PointerHits` values represent separate backend outputs, not separate targets from the same backend.

`hovered_hits_for_pointer` applies Bevy-style pickability: non-hoverable hits are skipped, non-blocking hits allow lower hits through, and blocking hits stop lower hover resolution after they are considered.

`PickingHoverMap::from_outputs` converts one frame of backend output into direct hovered hits per pointer using the existing sorting and pickability rules. It is intentionally direct-hover only: hierarchy bubbling is represented by each emitted `PickingPointerEvent` carrying `propagate`, while scene/editor consumers decide how to walk their current ownership graph.

`PrimitivePickingBackend` is a generic CPU ray backend over framework primitives. It casts each `RayMap` ray against registered `PickingPrimitive`s and returns ordinary `PointerHits`. This backend does not inspect the runtime world, editor state, render resources, or selection; it exists so overlays, gizmos, tests, and future render extract adapters can share the same backend contract before exact mesh acceleration data is exposed.

`PickingPipelineReport::from_ray_map_and_outputs` is the M1 diagnostics bridge for dev tools and future editor cutover. It collects pointer ids from both `RayMap` and backend output, so a pointer with a valid camera ray but no backend hits is still visible in diagnostics. Per-pointer reports reuse `sorted_hits_for_pointer` and `hovered_hits_for_pointer` instead of reimplementing ordering; this is deliberate so debug overlays cannot drift from runtime picking semantics. Blocking reports record the first sorted blocking target even when that target is non-hoverable, which makes invisible blockers diagnosable without making them hover events.

`PickingDebugFeed::from_report` is the M2 dev-tools bridge. It does not recompute picking. It only reshapes the already authoritative `PickingPipelineReport` into ordered metrics and pointer rows so an overlay or remote inspector can display ray-only pointers, hovered hit counts, invisible blockers, and top/blocking targets without depending on editor route code.

The editor viewport now consumes this through `ViewportOverlayPointerRouter::debug_feed_at` and through the `ViewportPointerDispatch::picking_debug_feed` field returned by normal move/down dispatch. The method is intentionally a consumer adapter: it uses the UI hit-test stack and editor precision candidates to build one runtime `PointerHits` output for the editor overlay backend, then creates a `PickingPipelineReport` and `PickingDebugFeed`. It does not mutate selection or treat the private editor route enum as the debug authority.

The same dispatch result now carries `ViewportPointerDispatch::runtime_input`, produced from the UI pointer event as a runtime `PointerInput`. This keeps editor input adaptation explicit and runtime-shaped while avoiding editor-owned hover/event synthesis. Move, down, up, and scroll dispatch all resolve through the same runtime route adapter, so editor release and scroll handling cannot drift into a separate route path. Move events currently carry a zero delta because the retained UI event only provides the absolute cursor; a later stateful input collector should compute deltas before driving the full `PickingEventState` path.

The editor router currently exposes only move/down as production convenience methods because those are the viewport controller's active entry points. Up/scroll coverage is exercised through test-only helpers while the production dispatcher and input mapper remain prepared for those event kinds.

The editor precision candidate score used by this adapter carries only screen-space score and projected depth. Editor candidate priority no longer flows through that score object, which prevents the old editor-only resolver semantics from surviving alongside runtime `HitTarget` priority.

`run_picking_pipeline` executes the fixed runtime stage order `Input -> RayMap -> Backend -> Hover -> Events`. It returns all intermediate frame products plus stage counters and the diagnostics report, so editor overlays, remote dev tools, and future runtime debug panels can observe the same picking state without owning the pipeline. When `PickingSettings::enabled` is false, the runner clears `PickingEventState` and returns empty output with every stage marked disabled; this prevents stale hover, press, drag, or click state from surviving a disabled picking frame.

`PickingEventState::dispatch_frame` mirrors Bevy's pointer event order while staying independent of ECS observers:

- hover exits emit `Out`, `Leave`, and any active `DragLeave`,
- hover entries emit `DragEnter`, `Enter`, and `Over`,
- pointers with a same-frame `Cancel` input are removed from the supplied current hover map before hover transitions are calculated, matching Bevy's rule that canceled pointers ignore backend hits for that frame,
- ordered pointer inputs then emit `Press`, `Click`, `Release`, `DragDrop`, `DragEnd`, `DragLeave`, `DragStart`, `Drag`, `DragOver`, `Move`, `Scroll`, or `Cancel` as appropriate,
- `Click` and `Release` target the previous hover map, matching Bevy's touch-release behavior,
- `Cancel` targets the previous hover map and clears the pointer's hover and button interaction state after the cancel event is emitted.

## Intentional Divergence

Bevy models picking through ECS events, resources, observers, and system sets. Zircon keeps the contract as plain Rust DTOs and pure helpers because the current runtime framework layer is the neutral contract spine, not the ECS scheduler. The M2 event state emits a flat event stream with `propagate` metadata rather than directly walking a scene hierarchy. That is deliberate: the runtime framework does not own editor authoring state or the runtime scene's mutable hierarchy, and consumers can map propagated events through the authoritative graph they already own.

Bevy's event payload stores click duration. Zircon M2 omits timing because no shared runtime clock contract is part of this picking milestone yet. A later `core::framework::time` integration can add a timestamp or duration field without changing target and ordering semantics.

The editor route enum is not removed in this slice. It is a private adapter around `HitTarget` because M1 only freezes the runtime contract. M9 will delete or downgrade the remaining editor pointer/handle code after transform gizmo and pointer events have moved onto runtime primitives.

The M2 editor adapter begins that cutover by converting stacked editor precision candidates into runtime `PointerHits` before route resolution. The editor still computes screen-space candidate geometry from authoring overlays, but category ordering now flows through runtime `HitTarget` and `hovered_hits_for_pointer` instead of an editor-only score resolver.

## Test Coverage

`zircon_runtime/src/tests/picking/mod.rs` covers:

- perspective viewport-center pointer to camera ray conversion,
- multi-pointer, multi-viewport, active-camera filtering in `RayMap`,
- two pointers across two active cameras in one viewport,
- same pointer id scoped to independent viewport locations,
- viewport-size aspect correction for off-center rays,
- hit sorting for the required handle/gizmo/renderable priority,
- hit sorting priority remaining stronger than backend order,
- hover resolution for ignored, non-blocking, and blocking hits.
- generic primitive backend ray hits flowing into existing hover rules,
- hover map construction from multiple backend outputs,
- hover transition event order before pointer move,
- click/release targeting the previous hover map,
- drag/drop and scroll event sequencing,
- cancel filtering of current hover plus pointer interaction state clearing.
- picking pipeline diagnostics for ray-only pointers, backend output counts, raw/hovered hit counts, top target selection, and blocking non-hoverable targets.
- picking debug feed summary metrics, ray-only pointer rows, and blocked non-hoverable pointer rows.
- picking pipeline stage execution, report carry-through, and disabled-frame state clearing.

`zircon_editor/src/tests/editing/viewport.rs` covers:

- existing editor router priority regressions,
- conversion between editor `ViewportPointerRoute` and runtime `HitTarget`,
- route priority matching the runtime target contract.

Milestone testing evidence for this slice:

- `cargo test -p zircon_runtime picking --locked` passed on 2026-05-08 before review fixes with 4 focused picking tests passing and existing runtime warning noise.
- During the M2 implementation slice, `cargo fmt` was attempted but stopped on an unrelated locked file handle at `zircon_runtime/tests/native_plugin_loader_contract.rs` with OS error 1224. The touched picking Rust files were then formatted directly with `rustfmt --edition 2021`.
- Review follow-up validation initially exposed unrelated active blockers in manifest locking, UI pool wiring, asset animation conversion, and scene ECS work outside `zircon_runtime::core::framework::picking`. Those blockers were not fixed from the picking lane except for allowing Cargo to refresh generated lock data already required by active platform/plugin manifest changes.
- `cargo metadata --offline --format-version 1` on 2026-05-08 refreshed generated `Cargo.lock` entries for the sibling platform/plugin feature manifest drift, including `gilrs`, first-party runtime plugin packages, `tokio`, and `socket2`. The offline metadata command then stopped because `objc2-io-kit` was not cached locally, but the generated lock resolution was sufficient for the next `--locked` validation.
- `cargo test -p zircon_runtime picking --locked --target-dir "E:\cargo-targets\zircon-runtime-picking-validation" --message-format short --color never` passed on 2026-05-08: 12 focused picking tests passed, 0 failed, 1059 filtered out. Cargo emitted existing warning noise in graphics/UI/native-plugin code outside the picking module.
- `cargo test -p zircon_editor viewport --locked` was not rerun in this validation continuation. Earlier evidence showed it blocked before viewport tests in unrelated active editor UI presentation code, so editor viewport and full-workspace validation remain broader follow-up gates.
- M1 report hardening on 2026-05-21 added `PickingPipelineReport` / `PickingPointerPipelineReport`, plus focused tests for ray-only pointers, backend output counts, raw/hovered hit totals, top target selection, and blocking non-hoverable targets.
- `cargo fmt --all --check` passed on 2026-05-21 after the working tree's stale unclosed app source-guard comment was no longer present.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-runtime-picking-m1-20260521-1320 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib picking --locked --color never --jobs 1` passed on 2026-05-21: 14 focused picking tests passed, 0 failed, 1778 filtered out.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-runtime-picking-m1-20260521-1320 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib input_protocol_types_live_in_runtime_input_surface --locked --color never --jobs 1` passed on 2026-05-21 after updating the source guard to the folder-backed `zircon_app/src/entry/runtime_entry_app/*` input files.
- M1 pipeline hardening on 2026-05-21 added `run_picking_pipeline`, `PickingPipelineInput`, `PickingPipelineOutput`, and `PickingPipelineStageReport`, plus focused tests for fixed stage ordering, report carry-through, and disabled-frame state clearing.
- `cargo fmt --all --check` passed on 2026-05-21 after the M1 pipeline slice.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-runtime-picking-m1-20260521-1320 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib picking --locked --color never --jobs 1` passed on 2026-05-21 after the M1 pipeline slice: 16 focused picking tests passed, 0 failed, 1778 filtered out.
- M1 boundary hardening on 2026-05-21 added ray-map tests for two pointers across two active cameras and for the same pointer id scoped to separate viewport locations.
- `cargo fmt --all --check` passed on 2026-05-21 after the M1 boundary slice.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-runtime-picking-m1-20260521-1320 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib picking --locked --color never --jobs 1` passed on 2026-05-21 after the M1 boundary slice: 18 focused picking tests passed, 0 failed, 1778 filtered out.
- M2 editor adapter on 2026-05-21 routed editor precision candidates through runtime `PointerHits` and `hovered_hits_for_pointer`, removing the editor-only `better_score` / `resolve_best_route` dispatcher path.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-runtime-picking-m1-20260521-1320 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib picking --locked --color never --jobs 1` passed on 2026-05-21 after clearing stale UI test helper field initializers: 18 focused picking tests passed, 0 failed, 1798 filtered out.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-picking-m2-20260521-2330 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib viewport --locked --color never --jobs 1` passed on 2026-05-21 after the M2 editor adapter slice: 78 focused viewport tests passed, 0 failed, 1342 filtered out.
- `cargo fmt --all --check` passed on 2026-05-21 after the M2 editor adapter slice.
- M2 debug-feed hardening on 2026-05-22 added `PickingDebugFeed`, `PickingDebugMetric`, `PickingDebugMetricKind`, and `PickingDebugPointerRow`, plus focused coverage for summary metrics, ray-only pointer rows, and blocked non-hoverable pointer rows.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-runtime-picking-debug-20260522-0045 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib picking --locked --color never --jobs 1` passed on 2026-05-22 after clearing the stale `widget_text_input_pointer.rs` binding helper initializer: 20 focused picking tests passed, 0 failed, 1816 filtered out.
- `rustfmt --edition 2021 --check zircon_runtime/src/core/framework/picking/debug_feed.rs zircon_runtime/src/core/framework/picking/mod.rs zircon_runtime/src/tests/picking/mod.rs zircon_runtime/src/ui/tests/widget_text_input_pointer.rs` passed on 2026-05-22.
- `cargo fmt --all --check` was attempted on 2026-05-22 and stopped on unrelated active UI/A11y formatting differences in `zircon_runtime/src/ui/accessibility/action.rs`; no picking formatting diagnostics were emitted.
- M2 editor debug-feed consumption on 2026-05-22 added `ViewportOverlayPointerRouter::debug_feed_at`, backed by `runtime_debug_feed_for_candidates`, plus focused viewport coverage proving overlapping editor overlay candidates produce a runtime `PickingDebugFeed` whose top target matches the dispatched route.
- `rustfmt --edition 2021 --check zircon_editor/src/scene/viewport/pointer/mod.rs zircon_editor/src/scene/viewport/pointer/viewport_pointer_dispatch.rs zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_event.rs zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_debug.rs zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs zircon_editor/src/scene/viewport/pointer/overlay_router/mod.rs` passed on 2026-05-22 after the editor debug-feed dispatch slice.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-picking-m2-20260521-2330 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib overlay_router_debug_feed_reports_runtime_picking_route_at_point --locked --color never --jobs 1` passed on 2026-05-22 after the dispatch field was added: 1 focused debug-feed test passed, 0 failed, 1422 filtered out.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-picking-m2-20260521-2330 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib viewport --locked --color never --jobs 1` passed on 2026-05-22 after the editor debug-feed dispatch slice: 79 focused viewport tests passed, 0 failed, 1344 filtered out.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-runtime-picking-debug-20260522-0045 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib picking --locked --color never --jobs 1` and the same command with `--offline` were attempted on 2026-05-22 but did not enter tests because unrelated active manifest edits in `zircon_app/Cargo.toml`, `zircon_plugins/net/features/content_download/runtime/Cargo.toml`, and `zircon_plugins/sound/runtime/Cargo.toml` made Cargo require a `Cargo.lock` update. The picking lane left `Cargo.lock` untouched rather than running non-locked validation for another session's manifest changes.
- M2 editor input DTO follow-up on 2026-05-22 added `ViewportPointerDispatch::runtime_input` and `runtime_pointer_input_for_event`, mapping editor UI move/down/up/scroll input into runtime `PointerInput` without moving event-state synthesis into editor code.
- `rustfmt --edition 2021 --check zircon_editor/src/scene/viewport/pointer/mod.rs zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs zircon_editor/src/scene/viewport/pointer/viewport_pointer_dispatch.rs zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_event.rs` passed on 2026-05-22 after the input DTO follow-up.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-picking-m2-20260521-2330 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib overlay_router_ --locked --color never --jobs 1` was attempted on 2026-05-22 after the input DTO follow-up but did not enter tests because unrelated active manifest edits still required a `Cargo.lock` update.
- M2 resolver source-guard follow-up on 2026-05-22 added a direct runtime-route test that keeps `HitTarget` priority authoritative even when UI stack order and projected depth favor a renderable candidate, and removed the unused editor precision priority from `CandidateScore`.
- `rustfmt --edition 2021 --check zircon_editor/src/scene/viewport/pointer/mod.rs zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs zircon_editor/src/scene/viewport/pointer/viewport_pointer_dispatch.rs zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_event.rs zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_debug.rs zircon_runtime/src/core/framework/picking/debug_feed.rs zircon_runtime/src/core/framework/picking/mod.rs zircon_runtime/src/tests/picking/mod.rs` passed on 2026-05-22 before the resolver source-guard follow-up.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-picking-m2-20260521-2330 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib overlay_router_ --locked --color never --jobs 1` was retried on 2026-05-22 but still did not enter tests because unrelated active manifest edits required a `Cargo.lock` update.
- M2 pointer event coverage follow-up on 2026-05-23 made editor up/scroll dispatch use the same runtime route resolver as move/down and added focused source coverage for release and scroll returning runtime input, runtime-picked route, and runtime debug feed output.
- `rustfmt --edition 2021 --check zircon_editor/src/scene/viewport/pointer/mod.rs zircon_editor/src/scene/viewport/pointer/overlay_router/build_dispatcher.rs zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_event.rs zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs zircon_editor/src/scene/viewport/pointer/precision/candidate_score.rs zircon_editor/src/scene/viewport/pointer/precision/precision_candidate_score.rs` passed on 2026-05-23 after the up/scroll debug-feed coverage follow-up.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-picking-m2-20260521-2330 CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib overlay_router_dispatch_maps_release_and_scroll_through_runtime_pointer_input --locked --color never --jobs 1` was attempted on 2026-05-23 but did not enter tests because unrelated active manifest edits in `zircon_app/Cargo.toml`, `zircon_plugins/net/features/content_download/runtime/Cargo.toml`, and `zircon_plugins/sound/runtime/Cargo.toml` still require a `Cargo.lock` update.
- `Get-ChildItem zircon_editor/src/scene/viewport/pointer -Recurse -File -Include *.rs | Select-String -Pattern 'resolve_best_route|better_score'` returned no source matches on 2026-05-23, confirming runtime adapter resolution is now the only pointer source route resolver path.
- `Select-String` source guard for `CandidateScore|priority:` on 2026-05-23 showed `CandidateScore` no longer stores priority; remaining priority matches are candidate z-index inputs, runtime `PickingTargetPriority` constants, and test candidate setup.
- Directly running `E:\cargo-targets\zircon-review-followup\debug\deps\zircon_editor-88ef940533c92e4c.exe overlay_router_debug_feed_reports_runtime_picking_route_at_point --test-threads=1 --nocapture` passed on 2026-05-25: 1 passed, 0 failed, 1463 filtered out.
- Directly running `E:\cargo-targets\zircon-review-followup\debug\deps\zircon_editor-88ef940533c92e4c.exe overlay_router_dispatch_maps_release_and_scroll_through_runtime_pointer_input --test-threads=1 --nocapture` passed on 2026-05-25: 1 passed, 0 failed, 1463 filtered out.
- M2 editor adapter backend-output follow-up on 2026-05-24 packed all scored editor overlay hits for one pointer into one `PointerHits` output, so debug/report backend-output counts now reflect one editor overlay backend rather than one backend output per candidate.
- `rustfmt --edition 2021 --check zircon_editor/src/scene/viewport/pointer/mod.rs zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs` passed on 2026-05-24.
- `git diff --check -- zircon_editor/src/scene/viewport/pointer/mod.rs zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs docs/zircon_editor/scene/viewport/pointer.md docs/zircon_runtime/core/framework/picking.md .codex/sessions/20260524-1739-review-followup-deferred.md` passed on 2026-05-24 with CRLF warnings only.
- Focused Cargo validation for `overlay_router_debug_feed_reports_runtime_picking_route_at_point` had three intermediate non-accepted attempts on 2026-05-24/25: one timed out while compiling, one failed before source compilation on transient root lockfile drift, and one compiled but ran 0 tests because `--exact` did not match the fully qualified lib-test name. The warmed lib-test binary was then run directly with the correct substring filter: `& "E:\cargo-targets\zircon-review-followup\debug\deps\zircon_editor-88ef940533c92e4c.exe" overlay_router_debug_feed_reports_runtime_picking_route_at_point --test-threads=1 --nocapture` passed with 1 passed, 0 failed, 1463 filtered out.

The runtime M2 picking backend/event validation remains accepted by the locked runtime runs above. The latest editor adapter/debug-feed slice is accepted by the 2026-05-22 focused editor viewport run, but workspace-level validation still depends on resolving the unrelated active manifest and lockfile drift before claiming the broader runtime/editor cutover is green.
