---
related_code:
  - zircon_editor/src/scene/viewport/projection.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/precision_candidates_from_layout.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/handle_candidate.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/projected_ring_segments.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/renderable_candidate.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/scene_gizmo_candidate.rs
plan_sources:
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/05/failure-2026-07-18-viewport-pointer-candidate-regeneration.md
doc_type: module-detail
---

# Viewport projection context

`ViewportProjectionContext` is the per-camera/per-viewport owner of the clamped viewport and the
precomputed projection × view matrix. Pointer precision-candidate construction creates exactly one
context before walking handles, scene gizmos, and renderables. Leaf projectors borrow it for every
point, line, circle, and 48-segment ring; they cannot reconstruct a matrix per point.

The context also owns `world_units_per_pixel`, using the same camera and clamped viewport as screen
projection. This keeps hit radius/depth calculations coherent across handle, gizmo, and renderable
priority paths. The deleted free `projected_point(world, camera, viewport)` API has no compatibility
wrapper; callers that project a batch must retain a context.

`project_point` and the free `world_units_per_pixel` remain only for isolated drag/handle calculations
outside the pointer candidate batch. They each construct a short-lived context, so future multi-point
callers must migrate instead of looping over those convenience functions.

This slice removes per-point matrix construction but does not claim the full viewport failure fixed.
The changed-generation path still needs one shared render/pointer gizmo extract and a runtime-visible
or spatial candidate backend so node visits follow query hits instead of total scene nodes.
