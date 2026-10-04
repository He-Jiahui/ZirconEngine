---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-visible-spatial-query-hash-dedup.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/visibility/spatial_query.rs
tests:
  - zircon_runtime/src/graphics/visibility/spatial_query.rs
---

# Runtime951 · visible spatial-query candidate fast path

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09b visible spatial query | Added a renderer-private `CandidateKeySet` that combines lazy hash membership with insertion-order storage. Sorted unique candidate streams now retain their vector without allocating a hash table; mixed streams still materialize hash membership for deduplication. Candidate-order classification and matching entity normalization each use one pass; monotonic ascending input is retained, monotonic descending input is reversed, and only mixed input is sorted before deduplication. | Bounds and ray paths share the candidate owner; fallback, duplicate-hit, descending-output, sorted-public-output, lazy-membership, and source contracts are covered in-file. The current Release owner reruns the unchanged 60% P95 gate against the repaired candidate path. | implemented_pending_validation |

## Deterministic boundary

`CandidateKeySet` never exposes hash iteration order. Candidate keys retain the
index/fallback stream order, while `normalize_entity_ids` preserves the public
sorted-and-unique query contract. Candidate, visited-node, and hit statistics
continue to report the same logical work.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/visibility/spatial_query.rs` | `1CE55EE89E17B52809D8E0CCA822DF10767AA4189296EFA6CB6D3AFF28310F73` |

## Validation handoff

Exact-file Rust 1.94.1 formatting and scoped diff checks pass. The current
source Release owner is being compiled in the grouped Runtime validation
target; its `RUNTIME09B_VISIBLE_SPATIAL_QUERY_HASH_DEDUP_BENCH_V1` P95 receipt
and the managed Cargo/allocator/product gates remain pending. The final-source
package pair is admitted in the central log as Runtime PTY `54276` and Editor
PTY `15395`; no per-task managed run is started.
