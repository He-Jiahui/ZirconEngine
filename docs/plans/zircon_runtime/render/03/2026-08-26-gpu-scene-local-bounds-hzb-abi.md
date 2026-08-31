---
plan: docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
optimization_source: docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
milestone: GS-M1 / Render04 VC-M3 prerequisite
status: implementation_partial_static_validated
runtime_validation: pending_managed_lane
pixel_evidence: pending
performance_evidence: pending_baseline
---

# GPUScene local bounds / HZB ABI implementation record

## Scope and decision

This slice addresses the P0-3 producer/consumer space mismatch without claiming the
complete persistent-render-scene milestone. Before editing, the current mesh expansion,
GPUScene upload, HZB shader, prepared mesh/model bounds, morph deformation bounds, CPU
visibility bounds, and Unreal GPUScene paths were re-read.

The selected ABI follows the existing Zircon resource boundary and Unreal's split:

- `GpuPrimitiveData.local_bounds_*` stores a mesh primitive's local-space bounding sphere.
- `GpuInstanceData.world_from_local` / `prev_world_from_local` stores the instance transform.
- HZB selects the current or previous instance transform and applies it exactly once.
- `RenderMeshBounds` remains the only CPU bounds value type; no parallel HZB-only type was
  introduced.

Unreal evidence: `GPUScene.cpp` packs `LocalObjectBoundsMin/Max` and
`PreSkinnedLocalBoundsMin/Max` separately from instance scene transforms (around lines
2515-2526), supports per-instance local bounds through
`INSTANCE_SCENE_DATA_FLAG_HAS_LOCAL_BOUNDS`, and `GPUSceneWriter.ush` writes instance local
center/extent. The intent copied here is the data-space split, not Unreal's class hierarchy.

## Implemented

1. `PendingMeshDraw` now carries `RenderMeshBounds` in local space.
2. All three production constructors populate it:
   - prepared/model meshes reconstruct bounds from resident `GpuMeshResource` min/max in O(1);
   - dynamic, CPU-morphed, GPU-morphed, and skinned primitives compute bounds once per
     expanded primitive, outside raster sub-draw expansion.
3. `primitive_data_for_pending_draw` writes the local center/radius. The deleted producer
   behavior no longer derives center from `model_matrix[3]` or radius from transform scale,
   removing translation-twice and scale-squared semantics.
   The internal Rust/WGSL fields were hard-cut to `local_bounds_center/radius`, with no
   compatibility alias, so their coordinate space remains review-visible.
4. Invalid/non-finite local bounds are sanitized and marked force-visible for HZB instead of
   becoming a false-cull input.
5. HZB fails open for bounds without a conservative temporal history:
   - skinned geometry;
   - CPU-morphed geometry;
   - GPU morph payloads whose current and previous weights differ.
   Stable GPU morph weights retain the occlusion path.
6. Orthogonal non-uniform scale keeps the existing conservative max-axis sphere transform.
   Non-orthogonal affine transforms (shear) receive a distinct instance flag and fail open;
   singular/non-finite transforms already use the degenerate flag and also fail open.
7. Primitive and instance flag constants are shared through `zr_gpu_scene.wgsl`; HZB no
   longer duplicates their numeric values. A Rust/WGSL ABI parity test locks all flag bits.

## Correctness and cost model

| Path | CPU bounds cost | HZB behavior |
|---|---:|---|
| resident prepared mesh | O(1) | cull when transform is conservative |
| dynamic/morphed/skinned primitive | O(vertices), once before raster splits | fail open when temporal bounds are unsafe |
| HZB invocation | O(1) | one local-to-world transform, no second producer transform |

This is a correctness-first MVP fallback, not the final performance shape. The O(vertices)
dynamic bounds scan must later be folded into deformation artifact production so the
render-thread expansion only borrows a generation-owned bounds revision. No measured speed,
power, or GPU-time improvement is claimed before WPR/GPU timestamp/RenderDoc baselines.

## Tests and static evidence

Added focused tests cover:

- off-center local mesh bounds projection;
- invalid bounds sanitization/fail-open;
- stable versus changed morph weights;
- CPU-morphed temporal fail-open;
- force-HZB-visible primitive flag projection;
- non-orthogonal transform classification;
- Rust/WGSL primitive and instance flag parity.

Static checks completed on 2026-08-26:

- `rustfmt --edition 2021 --check` passed for all touched Rust sources;
- all 3 production `PendingMeshDraw` literals carry `local_bounds`;
- `approximate_transform_radius` is absent from the mesh GPUScene producer;
- `git diff --check` passed (line-ending warnings only);
- touched owner files remain below the module budget: bounds 89 lines, GPUScene sync 687,
  mesh expansion 747, pending draw 263 at the recorded snapshot.

## Explicitly pending

- CPU visibility still constructs a translation/scale proxy sphere before resource resolution.
  P0-3 is not complete until persistent `RenderSceneGeneration` publishes the same bounds and
  revision to CPU view culling and GPUScene.
- Conservative previous bounds for changing skin poses and deformation generations are not
  stored; these paths intentionally remain HZB-visible.
- Managed Cargo/Naga/WGPU tests are pending because the shared validation copy remains blocked
  by the unrelated missing editor animation UI asset. Raw Cargo was not used.
- No RenderDoc capture, rendered PNG, WPR/xperf baseline, GPU timing, power data, or performance
  comparison has been produced for this slice. Those acceptance artifacts remain mandatory.
