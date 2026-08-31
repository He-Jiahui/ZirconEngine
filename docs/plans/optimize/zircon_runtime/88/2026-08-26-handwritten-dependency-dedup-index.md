---
title: Runtime88 Handwritten Dependency Dedup Index
category: zircon_runtime
report_id: Runtime88-handwritten-dependency-dedup-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime88 Handwritten Dependency Dedup Index

## Scope

This slice removes repeated `Vec::contains` scans while handwritten Scene, Material, and Model
dependencies are merged into import outcomes. Existing dependency order, first occurrence,
candidate order, typed extraction, subasset labels, and import entry boundaries remain unchanged.
It advances Runtime88 dependency ingestion without claiming completion of source ownership,
reverse dependency closure, reconciliation, generation commit, or reload delivery.

## Change

- Build one borrowed membership set over an import entry's existing dependency URIs.
- Classify all extracted candidates against that set before extending the dependency Vec.
- Clone only accepted candidates; existing dependency URIs are borrowed by the index.
- Preserve the original stable order for existing and newly accepted dependencies.

## Deterministic Performance Evidence

| 4,096 existing and 4,096 distinct candidate dependencies | Before | After |
|---|---:|---:|
| Pairwise URI comparisons | 25,163,776 | 0 |
| Existing dependency index-build visits | 0 | 4,096 |
| Candidate membership probes | 0 | 4,096 |
| Dependency order changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs over 2,048 existing and 2,048 candidate
URIs and emits `RUNTIME88_HANDWRITTEN_DEPENDENCY_DEDUP_INDEX_BENCH_V1`. Acceptance requires indexed
deduplication P95 to be at least 75% below the legacy repeated Vec scan. Exact Windows timings
remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826be_handwritten_dependency_index_preserves_stable_order` covers
  existing, duplicate candidate, and newly accepted URI order.
- `optimization_batch_20260826be_handwritten_dependency_index_eliminates_pairwise_work` locks the
  25,163,776-comparison model and rejects `dependencies.contains` in the append helper.
- `optimization_batch_20260826be_handwritten_dependency_index_p95` reports paired release P50/P95
  samples and enforces the 75% P95 reduction gate.

## Remaining Parent-plan Work

Runtime88 still owns typed source events, compound source ownership, rename/error folding,
reconciliation, generation-qualified dependency deltas, targeted reimport, reload publication, and
fault/soak evidence. This slice only converges handwritten dependency merge work.
