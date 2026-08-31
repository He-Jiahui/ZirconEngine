---
title: Runtime74 UI V2 Cache Canonical Revalidation
category: zircon_runtime
report_id: Runtime74-ui-v2-cache-canonical-revalidation-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime74 UI V2 Cache Canonical Revalidation

## Scope

This slice removes redundant filesystem canonicalization when the UI v2 file-store cache
revalidates an in-memory or persistent entry. Explicit request paths and imported source paths
continue to resolve to physical paths before publication. Revalidation still reads current
metadata, modified time, and file length for every dependency, so stale source content continues
to invalidate the entry.

The change advances Runtime74 hot-reload and product integration without changing parsing,
resource-reference traversal, compiled document identity, persistent schema versions, or cache
admission.

## Change

- Separate physical-path resolution from source metadata snapshot construction.
- Revalidate already-canonical in-memory and persistent `source_paths` directly.
- Build the initial transitive source key from the canonical paths published by source collection.
- Preserve ordered dependency identity and all metadata freshness fields.

## Deterministic Performance Evidence

| Cache revalidation with 512 canonical dependency paths | Before | After |
|---|---:|---:|
| Redundant `canonicalize` calls | 512 | 0 |
| Metadata freshness checks | 512 | 512 |
| Dependency key rows | 512 | 512 |
| Freshness/ordering changes | 0 | 0 |

Deterministic redundant canonicalization falls by 100%. The ignored release gate runs 17
alternating sample pairs and emits
`RUNTIME74_UI_V2_CACHE_CANONICAL_REVALIDATION_BENCH_V1`. Acceptance requires canonical-path
revalidation P95 to be at least 20% below legacy re-canonicalization. Exact Windows P50/P95
timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bm_ui_v2_cache_canonical_revalidation_preserves_key` proves the
  legacy and canonical constructors produce the same key for a canonical source.
- `optimization_batch_20260826bm_ui_v2_cache_canonical_revalidation_skips_resolve` locks both
  memory and persistent revalidation to the canonical constructor and rejects path resolution
  inside that constructor.
- `optimization_batch_20260826bm_ui_v2_cache_canonical_revalidation_p95` reports paired release
  P50/P95 samples and enforces the 20% P95 reduction gate.

## Remaining Parent-plan Work

Runtime74 still owns binding, expression, event, command, hot-reload, and product-scale integration
evidence. This slice only converges UI v2 file-cache source revalidation.
