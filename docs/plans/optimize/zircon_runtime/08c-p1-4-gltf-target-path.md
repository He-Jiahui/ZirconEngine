# Runtime08C P1-4 glTF Animation Target Path Execution Plan

- Source review: `docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md`, P1-4
- Owner: `optimize-runtime08c-gltf-target-path-r1-01a00797-20260820`
- Status: implementation complete; combined managed validation pending

## Scope

1. Derive every imported clip `target_id` from the exact generated or referenced
   `AnimationSkeletonAsset` bone hierarchy.
2. Reject an animation channel whose node is absent from that skeleton instead
   of producing a clip that fails later during frame evaluation.
3. Lock the non-root animation fixture to the canonical root/child path and run
   it in the serialized multi-task Runtime batch.

This slice does not change the plugin-vs-builtin glTF authority decision,
prepared animation artifacts, skin/inverse-bind ownership, or GPU skinning.
