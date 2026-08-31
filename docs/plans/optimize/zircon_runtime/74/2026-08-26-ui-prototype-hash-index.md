---
title: Runtime74 UI Prototype Hash Index
category: zircon_runtime
report_id: Runtime74-ui-prototype-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime74 UI Prototype Hash Index

## Scope

This slice replaces the ordered legacy `UiPrototypeStore` asset index with a `HashMap`. Recursive
prototype instancing, component reference resolution, import validation, and canonical/alias lookup
now use expected constant-time complete-ID lookup.

The store exposes no ordered iteration API, so canonical inserts, alias inserts and replacement,
Arc-backed prototype ownership, component validation, length reporting, and typed missing-import
errors are unchanged without retaining a duplicate ordered index.

## Performance Workload

The release workload stores 1,024 prototype IDs sharing a long prefix and performs 4,096 stable
lookups of the same complete ID.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered-index lookups | 4,096 | 0 |
| Hash-index lookups | 0 | 4,096 |
| Prototype payload clones | 0 | 0 |
| Alias or validation-policy changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME74_UI_PROTOTYPE_HASH_INDEX_BENCH_V1`. Acceptance requires HashMap lookup P95 to be at least
30% below the legacy BTreeMap path. Exact Windows P50/P95 timings remain pending the coordinator
run.

## Acceptance

- `optimization_batch_20260826bs_ui_prototype_hash_index_preserves_canonical_and_alias_lookup`
  covers complete canonical/alias keys, Arc identity, and row count.
- `optimization_batch_20260826bs_ui_prototype_hash_index_preserves_alias_replacement` covers
  replacement semantics without duplicate rows.
- `optimization_batch_20260826bs_ui_prototype_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Runtime74 still owns the broader template asset compiler, UI v2 migration, style resolution,
retained rendering, and product interaction contract. This slice only converges legacy prototype
lookup.
