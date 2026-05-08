---
related_code:
  - zircon_runtime/src/core/framework/picking/mod.rs
  - zircon_runtime/src/core/framework/picking/backend.rs
  - zircon_runtime/src/core/framework/picking/hit_data.rs
  - zircon_runtime/src/core/framework/picking/hit_record.rs
  - zircon_runtime/src/core/framework/picking/hit_target.rs
  - zircon_runtime/src/core/framework/picking/hover_map.rs
  - zircon_runtime/src/core/framework/picking/pickable.rs
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
  - zircon_runtime/src/core/framework/picking/schedule_label.rs
  - zircon_runtime/src/core/framework/picking/settings.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_editor/src/scene/viewport/pointer/viewport_pointer_route.rs
  - zircon_editor/src/scene/viewport/pointer/constants.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_pointer_route.rs
implementation_files:
  - zircon_runtime/src/core/framework/picking/mod.rs
  - zircon_runtime/src/core/framework/picking/backend.rs
  - zircon_runtime/src/core/framework/picking/hit_data.rs
  - zircon_runtime/src/core/framework/picking/hit_record.rs
  - zircon_runtime/src/core/framework/picking/hit_target.rs
  - zircon_runtime/src/core/framework/picking/hover_map.rs
  - zircon_runtime/src/core/framework/picking/pickable.rs
  - zircon_runtime/src/core/framework/picking/pointer_event.rs
  - zircon_runtime/src/core/framework/picking/pointer_event_state.rs
  - zircon_runtime/src/core/framework/picking/pointer_hits.rs
  - zircon_runtime/src/core/framework/picking/pointer_input.rs
  - zircon_runtime/src/core/framework/picking/pointer_location.rs
  - zircon_runtime/src/core/framework/picking/primitive_backend.rs
  - zircon_runtime/src/core/framework/picking/ray.rs
  - zircon_runtime/src/core/framework/picking/ray_map.rs
  - zircon_editor/src/scene/viewport/pointer/viewport_pointer_route.rs
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
- `PickingSettings` and `PickingScheduleLabel` reserve the configuration and pipeline labels later used by pointer events and backend orchestration.

## Behavior

`RayMap::rebuild` clears and repopulates rays from active cameras and current pointer locations. It filters by viewport handle and viewport bounds, so multiple pointers, viewports, and cameras can coexist without producing accidental cross-viewport hits. Ray generation derives aspect ratio from the active viewport size rather than trusting a reused camera snapshot, so off-center rays remain correct when one camera feeds multiple viewport sizes.

`ray_from_viewport_point` supports perspective and orthographic camera snapshots using the same transform direction conventions as the current runtime math layer. It returns `None` for zero-sized viewports, out-of-bounds pointer positions, or invalid ray directions.

`sorted_hits_for_pointer` merges backend outputs for a pointer. The current editor cutover priority sorts `HandleAxis` before `SceneGizmo` before `Renderable` before backend order, so a renderable backend cannot steal a transform-handle route by using a higher backend order. Backend order and depth then break ties within the same target priority. This preserves existing editor route behavior while moving the semantics to a runtime-owned contract.

`hovered_hits_for_pointer` applies Bevy-style pickability: non-hoverable hits are skipped, non-blocking hits allow lower hits through, and blocking hits stop lower hover resolution after they are considered.

`PickingHoverMap::from_outputs` converts one frame of backend output into direct hovered hits per pointer using the existing sorting and pickability rules. It is intentionally direct-hover only: hierarchy bubbling is represented by each emitted `PickingPointerEvent` carrying `propagate`, while scene/editor consumers decide how to walk their current ownership graph.

`PrimitivePickingBackend` is a generic CPU ray backend over framework primitives. It casts each `RayMap` ray against registered `PickingPrimitive`s and returns ordinary `PointerHits`. This backend does not inspect the runtime world, editor state, render resources, or selection; it exists so overlays, gizmos, tests, and future render extract adapters can share the same backend contract before exact mesh acceleration data is exposed.

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

## Test Coverage

`zircon_runtime/src/tests/picking/mod.rs` covers:

- perspective viewport-center pointer to camera ray conversion,
- multi-pointer, multi-viewport, active-camera filtering in `RayMap`,
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

The focused runtime M2 picking backend/event validation is accepted by the 12-test locked run above. Because this is a shared framework API, editor viewport and workspace-level validation still need to be rerun before claiming the broader runtime/editor cutover is green.
