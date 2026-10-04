---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-frame-batching-hash-entity-sets.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/batching_result.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result.rs
tests:
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/batching_result.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result/optimization_batch_io_runtime625_tests.rs
---

# Runtime950 · frame-batching entity-set fast path

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09b frame visibility batching | Replaced the three ordered membership trees with `EntitySet`: a lazy hash membership index plus an insertion-order `Vec`. Monotonic input (the normal stable-mesh order) stays allocation-free and only materializes a `HashSet` once mixed ordering or a non-adjacent duplicate requires it. The output boundary preserves ascending public IDs by reversing descending input or sorting only non-monotonic input. | Source contracts cover unordered duplicate input, ascending input, descending input, mixed-input membership materialization, post-materialization out-of-order admission, the three capacity-reserved owners, and the unchanged sorted output boundary. The ignored Release P95 marker is pending a binary containing this final lazy-owner repair. | implemented_pending_validation |

## Deterministic boundary

`EntitySet::insert` is the only admission path. Duplicate IDs never enter the
published vector. `sorted_entity_ids` consumes the set at the visibility-context
boundary, so downstream consumers still receive ascending, unique vectors.
Static/dynamic mobility classification and all batch, BVH, relevance, and
history projections remain unchanged.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/visibility/context/from_extract_with_history/batching_result.rs` | `85D6366FF09543E4F3952E3038B5BF2B9F54C9B341ACB98841EC7882B34BE385` |
| `zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result.rs` | `A831D187ADC842D36CE5FB4E32ED0F1884F6F40A95FC196E33330F29D0B26B54` |
| `zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result/optimization_batch_io_runtime625_tests.rs` | `D7DCB0EB992C20FF51971CB8E010DF3E8A08E6AD281CA9DF734BE64C922CD49B` |

## Validation handoff

Exact-file Rust 1.94.1 formatting and scoped diff checks pass. The current
source Release owner is being compiled in the grouped Runtime validation
target; its `RUNTIME09B_FRAME_BATCHING_HASH_ENTITY_SETS_BENCH_V1` P95 receipt
and the managed Cargo/allocator/product gates remain pending. The final-source
package pair is admitted in the central log as Runtime PTY `54276` and Editor
PTY `15395`; no per-task managed run is started.
