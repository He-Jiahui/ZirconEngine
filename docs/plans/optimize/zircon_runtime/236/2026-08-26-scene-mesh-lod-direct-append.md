# Runtime236 Scene Mesh LOD Direct Append

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime236-editor182-performance-batch-20260826gp-v1`

## Problem

Scene mesh instance reference collection preallocated its final Vec but called `direct_references`
for every LOD, allocating a temporary Vec per level before moving those references into the final
result.

## Optimization

- Share a private LOD append path between standalone and instance reference collection.
- Write every LOD reference directly into the instance's exactly sized final Vec.
- Preserve standalone LOD behavior, reference cloning, ordering, and count APIs.

## Regression Contract

The `optimization_batch_20260826gp_` Runtime tests exercise real `AssetReference` ordering and the
direct-append source contract, and provide an ignored paired release benchmark emitting
`RUNTIME236_SCENE_MESH_LOD_DIRECT_APPEND_BENCH_V1`. It collects 256 LODs with 16 large reference
payloads across 64 builds per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
