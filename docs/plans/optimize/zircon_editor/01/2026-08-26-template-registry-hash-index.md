---
title: Editor01 Template Registry Hash Index
category: zircon_editor
report_id: Editor01-template-registry-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Template Registry Hash Index

## Scope

This slice replaces the ordered compiled-document table in `EditorTemplateRegistry` with a
`HashMap`. Repeated template projection and instantiation now use expected constant-time complete
document-ID lookup.

The registry exposes no ordered iteration API. Registration, duplicate rejection, typed missing
document errors, compiled-document ownership, and runtime instantiation contracts are unchanged,
with no secondary index or duplicate compiled payload.

## Performance Workload

The release workload fills 512 document IDs sharing a long prefix and performs 4,096 stable hits.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered-index lookups | 4,096 | 0 |
| Hash-index lookups | 0 | 4,096 |
| Compiled-document clones in index benchmark | 0 | 0 |
| Registration or error-policy changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR01_TEMPLATE_REGISTRY_HASH_INDEX_BENCH_V1`. Acceptance requires HashMap lookup P95 to be at
least 30% below the legacy BTreeMap path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826bs_editor_template_registry_hash_index_preserves_lookup` covers
  registration, complete-ID lookup, HashMap ownership, and typed missing-document errors.
- `optimization_batch_20260826bs_editor_template_registry_hash_index_preserves_duplicate_error`
  covers unchanged duplicate rejection.
- `optimization_batch_20260826bs_editor_template_registry_hash_index_p95` reports paired release
  P50/P95 samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained invalidation, layout/paint scaling, shared visual materialization,
accessibility, and product-scale interaction evidence. This slice only converges compiled-template
registry lookup.
