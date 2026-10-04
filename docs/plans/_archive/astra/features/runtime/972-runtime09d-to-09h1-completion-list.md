---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09d/2026-09-10-versioned-asset-owner-use-point.md
  - docs/plans/optimize/zircon_runtime/09e/2026-08-26-indexed-shadow-preemption-lookup.md
  - docs/plans/optimize/zircon_runtime/09f1/2026-08-26-ibl-bake-pipeline-fast-hit.md
  - docs/plans/optimize/zircon_runtime/09f1/2026-08-27-borrowed-ibl-artifact-dispatch.md
  - docs/plans/optimize/zircon_runtime/09f2/2026-08-24-lightmap-slot-index.md
  - docs/plans/optimize/zircon_runtime/09h1/2026-08-26-taa-bind-group-mru-fast-path.md
related_records:
  - docs/plans/astra/features/runtime/966-runtime09d-versioned-asset-owner-use-point.md
  - docs/plans/astra/features/runtime/967-runtime09e-indexed-shadow-preemption-lookup.md
  - docs/plans/astra/features/runtime/968-runtime09f1-ibl-pipeline-fast-hit.md
  - docs/plans/astra/features/runtime/969-runtime09f1-borrowed-ibl-artifact-dispatch.md
  - docs/plans/astra/features/runtime/970-runtime09f2-immutable-lightmap-slot-index.md
  - docs/plans/astra/features/runtime/971-runtime09h1-taa-bind-group-mru-fast-path.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
---

# Runtime09D–09H1 · rendering and residency completion list

The six Runtime slices below are implemented in the current source and have
focused tests or release models prepared. They remain pending managed
compilation, terminal marker receipts, allocator evidence, and product-level
qualification where their parent plans require it.

| Slice | Optimization | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Runtime09D | Resolve the current asset manager at semantic-residency use points instead of retaining a stale strong owner. | Managed Runtime/WGPU compilation and resource-streaming/product gates. | implemented_pending_validation |
| Runtime09E | Index shadow preemption incumbents and bound challenger traversal by sorted priority. | `RUNTIME09E_SHADOW_PREEMPTION_BENCH_V1` must clear the 95% P95 gate and allocator tests. | implemented_pending_validation |
| Runtime09F1 pipeline | Probe the complete IBL pipeline key before component caches. | `RUNTIME09F1_IBL_PIPELINE_FAST_HIT_BENCH_V1` must clear the 50% P95 gate. | implemented_pending_validation |
| Runtime09F1 dispatch | Borrow IBL blobs and clone only the selected artifact. | Allocation/byte, P50/P95 gates and borrowed behavior parity. | implemented_pending_validation |
| Runtime09F2 | Normalize immutable lightmap slots and use partition-point lookup. | `RUNTIME09F2_LIGHTMAP_SLOT_BENCH_V1` plus stable-frame allocation gate. | implemented_pending_validation |
| Runtime09H1 | Directly hit the TAA cache's LRU tail for stable frames. | `RUNTIME09H1_TAA_BIND_GROUP_MRU_FAST_PATH_BENCH_V1` must clear the 50% P95 gate. | implemented_pending_validation |

The current grouped Runtime/Editor validation submission covers these source
paths together with the earlier Runtime09c/09h2 batches. No per-task Cargo run
is started and no asynchronous result is inferred in this list.
