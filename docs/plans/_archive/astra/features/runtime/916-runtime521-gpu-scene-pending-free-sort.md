---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime521-gpu-scene-id-allocator.md
implementation_files:
  - zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs
tests:
  - zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs
---

# Runtime916 Runtime521 GPU-scene pending-free sort fast path

`GpuSceneIdAllocator` now tracks whether appended pending-free spans break the
monotonic start-key invariant and sorts only when required. Monotonic frame
frees therefore avoid the unconditional sort while unordered spans retain the
same coalescing and deferred-reuse behavior.

The focused behavior test covers monotonic and unordered spans. Marker
`RUNTIME521_GPU_SCENE_PENDING_FREE_SORT_BENCH_V1` models 32,768 frames with 64
pending spans and reports zero optimized sort calls versus the legacy sort on
every non-empty frame. Managed Windows Release compile, tests, and the final
performance receipt remain pending.
