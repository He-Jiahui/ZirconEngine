---
title: Runtime74 UI V2 Prototype Hash Index
category: zircon_runtime
report_id: Runtime74-ui-v2-prototype-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime74 UI V2 Prototype Hash Index

## Scope

This slice adds a synchronized `HashMap` lookup index to `UiV2PrototypeStore`. Recursive component
instancing and import validation now use expected constant-time asset and alias lookup, while the
existing `BTreeMap` remains the ordering authority for deterministic `documents()` iteration.

Canonical inserts, alias inserts, replacement behavior, Arc-backed document ownership, declared
asset validation, root reachability, and ordered document traversal are unchanged. The deliberate
tradeoff is one additional owned key and Arc handle per canonical or alias row; document payloads
remain shared rather than duplicated.

## Performance Workload

The release workload stores 1,024 prototype IDs sharing a long prefix and performs 4,096 stable
lookups of the same complete ID.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered-index lookups | 4,096 | 0 |
| Hash-index lookups | 0 | 4,096 |
| Document payload clones | 0 | 0 |
| Ordered traversal-policy changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME74_UI_V2_PROTOTYPE_HASH_INDEX_BENCH_V1`. Acceptance requires hash lookup P95 to be at least
30% below ordered lookup. Exact Windows P50/P95 timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bq_ui_v2_prototype_hash_index_preserves_order_and_aliases` covers
  canonical lookup, alias lookup, Arc identity, and ordered document iteration.
- `optimization_batch_20260826bq_ui_v2_prototype_hash_index_keeps_indexes_in_sync` covers alias
  replacement and complete synchronization between the ordered authority and hash index.
- `optimization_batch_20260826bq_ui_v2_prototype_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Runtime74 still owns the broader UI v2 asset, compiler, style, retained rendering, and product
interaction contract. This slice only converges prototype and alias lookup during validation and
component instancing.
