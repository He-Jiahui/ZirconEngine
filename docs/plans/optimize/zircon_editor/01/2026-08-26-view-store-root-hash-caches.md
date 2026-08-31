---
title: Editor01 View Store Root Hash Caches
category: zircon_editor
report_id: Editor01-view-store-root-hash-caches-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 View Store Root Hash Caches

## Scope

This slice removes ordered-tree lookups from the retained view template store's source-root cache
and unavailable-root membership cache. The per-request root set remains a `BTreeSet`, so watcher
creation order stays deterministic. Watcher ownership also remains in its existing `BTreeMap`.

## Change

- Replace `source_roots: BTreeMap<PathBuf, Option<PathBuf>>` with `HashMap` for repeated source
  lookup and insertion.
- Replace `unavailable_roots: BTreeSet<PathBuf>` with `HashSet` for repeated failure suppression.
- Preserve ordered root batching, watcher creation, invalidation signaling, and metadata fallback.

## Deterministic Performance Evidence

| Representative 8,192 cached roots / 65,536 paired lookups | Before | After |
|---|---:|---:|
| Source-root lookup | O(log n) | average O(1) |
| Unavailable-root lookup | O(log n) | average O(1) |
| Watch registration order | path ordered | unchanged |

The ignored release gate runs 17 alternating samples and emits
`EDITOR01_VIEW_STORE_ROOT_HASH_CACHES_BENCH_V1`. Acceptance requires hash-cache P95 to be at most
60% of ordered-cache P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826ad_editor01_hash_root_caches_preserve_unavailable_membership`
  exercises both persistent root caches without creating an OS watcher.
- `optimization_batch_20260826ad_editor01_view_store_uses_hash_caches_and_ordered_watch_roots`
  requires both hash caches while preserving the local ordered root set.
- `optimization_batch_20260826ad_editor01_view_store_root_hash_caches_performance_evidence`
  checks paired lookup equivalence, reports both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Editor01 still needs generation-bound async template loading, cancellation, last-good publication,
retained-tree patching, startup-scale qualification, and complete retained UI performance evidence.
This slice only improves persistent view store root caches.
