---
title: Runtime11A Prototype Cache Canonical Revalidation
category: zircon_runtime
report_id: Runtime11A-prototype-cache-canonical-revalidation-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A Prototype Cache Canonical Revalidation

## Scope

This slice removes redundant filesystem canonicalization when a flat UI prototype-store cache hit
revalidates its transitive source paths. Initial explicit paths and every imported source continue
to resolve to physical paths before publication. Cache hits continue to read current metadata,
modified time, and file length for every dependency, so stale source content still invalidates the
entry. It advances Runtime11A UI template loading without changing parsing, import traversal,
prototype compilation, or cache identity.

## Change

- Separate physical-path resolution from metadata snapshot construction.
- Revalidate the entry's already-canonical source paths directly instead of canonicalizing them a
  second time.
- Preserve the original path, modified-time, and length key fields and ordered dependency list.
- Add no watcher, persistent metadata authority, or alternate prototype cache.

## Deterministic Performance Evidence

| Cache hit with 512 canonical dependency paths | Before | After |
|---|---:|---:|
| Redundant `canonicalize` calls | 512 | 0 |
| Metadata freshness checks | 512 | 512 |
| Dependency key rows | 512 | 512 |
| Freshness/ordering changes | 0 | 0 |

Deterministic redundant canonicalization falls by 100%. The ignored release gate runs 17
alternating sample pairs and emits
`RUNTIME11A_PROTOTYPE_CACHE_CANONICAL_REVALIDATION_BENCH_V1`. Acceptance requires canonical-path
revalidation P95 to be at least 20% below legacy re-canonicalization. Exact Windows P50/P95 timings
remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bk_prototype_cache_canonical_revalidation_preserves_key` proves both
  constructors produce the same key for a canonical source.
- `optimization_batch_20260826bk_prototype_cache_canonical_revalidation_skips_resolve` locks the
  cache-hit call site to the canonical constructor and rejects path resolution inside it.
- `optimization_batch_20260826bk_prototype_cache_canonical_revalidation_p95` reports paired release
  P50/P95 samples and enforces the 20% P95 reduction gate.

## Remaining Parent-plan Work

Runtime11A still owns template compilation, retained tree/layout/input/accessibility integration,
resource invalidation, virtualization, and product-scale UI evidence. This slice only converges
flat prototype cache-hit source revalidation.
