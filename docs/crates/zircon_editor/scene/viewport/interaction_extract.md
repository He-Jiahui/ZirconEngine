---
related_code:
  - zircon_editor/src/scene/viewport/interaction_extract/mod.rs
  - zircon_editor/src/scene/viewport/interaction_extract/key.rs
  - zircon_editor/src/scene/viewport/interaction_extract/cache.rs
  - zircon_editor/src/scene/viewport/interaction_extract/extract.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_render_snapshot.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_pointer_route.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_sync.rs
tests:
  - zircon_editor/src/scene/viewport/interaction_extract/tests.rs
  - zircon_editor/src/scene/viewport/pointer/tests.rs
  - tools/tests/test_editor05_viewport_interaction_extract_contract.py
doc_type: module-detail
---

# Viewport Interaction Extract

## Purpose

`ViewportInteractionExtractCache` is the editor viewport's generation-scoped fact source for transform handles, scene gizmos, and runtime render meshes used by coarse picking. It prevents render snapshot construction and pointer routing from rebuilding independent authoring projections for the same scene generation.

## Ownership and invalidation

The cache belongs to `SceneViewportController`. Its immutable key contains runtime `world_generation`, active selection, complete viewport settings, camera snapshot, and viewport size. Any change produces a new `Arc<ViewportInteractionExtract>`; a stable key returns the existing `Arc` before invoking handle or gizmo builders.

Controller clones retain the same cached `Arc` and router identity. There is no permanent cache outside the controller and no manual invalidation API; scene mutation must advance `world_generation`, while selection/settings/camera/resize changes are part of the key.

## Render and pointer flow

The render path first builds the runtime viewport packet, then seeds the interaction cache from `packet.scene.meshes`. It copies the shared handle/gizmo slices into the runtime packet's currently owned overlay vectors.

The pointer path resolves the same key. If render already seeded the generation, pointer receives the same `Arc` without a scene traversal. If pointer arrives first, the cache builds one runtime viewport packet only to acquire its camera/layer/active-state-filtered mesh extract, then publishes the shared interaction object. The pointer router stores shared slices and uses `Arc::ptr_eq` as its stable-generation early-out.

Renderable candidate generation consumes sorted `RenderMeshSnapshot` values and collapses multiple primitives of one entity. It never scans `Scene::nodes()` or repeats active-hierarchy filtering. This removes editor-local total-node regeneration, but it is not yet a cursor-query spatial broad phase: a BVH/pick-id backend remains required before claiming 1/1k/10k visit counts proportional to query hits.

## Hard-cut rules

- Do not reintroduce `ViewportPointerSceneKey`; generation authority lives in the controller cache key.
- Do not add a pointer-only `build_scene_gizmos` wrapper or a renderable `Scene::nodes()` scan.
- Do not retain an unbounded/global cache without the full generation key.
- Do not claim the runtime mesh extract as frustum/BVH proof; it is currently camera/layer/active-state filtered.
