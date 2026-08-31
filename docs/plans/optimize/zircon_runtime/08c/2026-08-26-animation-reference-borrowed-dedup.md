---
title: Runtime08c Animation Reference Borrowed Dedup
category: zircon_runtime
report_id: Runtime08c-animation-reference-borrowed-dedup-2026-08-26
date: 2026-08-26
session_id: root-runtime08c-two-task-borrowed-index-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime08c Animation Reference Borrowed Dedup

## Scope

This slice replaces repeated vector scans in animation graph and state-machine direct-reference
collection with one shared borrowed deduplicator. First-seen order, UUID-plus-locator identity,
graph/state traversal order, duplicate suppression, and returned owned references remain unchanged.

## Change

- Add a shared collector with `HashSet<&AssetReference>` membership and a separate ordered output.
- Clone each accepted reference exactly once into the returned vector; the HashSet owns no resource
  locator or reference clone.
- Route graph Clip nodes, state kinds, BlendSpace samples, sub-machines, GraphRef states, and layers
  through the same collector.
- Size graph collection from its actual Clip-node count instead of all graph nodes.

## Deterministic Performance Evidence

| 2,048 unique references, two collections per sample | Before | After |
|---|---:|---:|
| Reference equality comparisons per sample | 4,192,256 | 0 |
| Borrowed hash inserts per sample | 0 | 4,096 |
| Output reference clones per sample | 4,096 | 4,096 |
| Additional reference clones retained by membership set | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME08C_ANIMATION_REFERENCE_BORROWED_DEDUP_BENCH_V1`. Acceptance requires borrowed hash dedup
P95 to be at least 90% below repeated vector scans. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `runtime08c_batch_animation_reference_dedup_preserves_first_seen_owned_order`
  covers value-based duplicate suppression, first-seen order, and output ownership.
- `runtime08c_batch_animation_reference_uses_borrowed_hash_dedup` requires the shared
  borrowed HashSet collector across graph and state-machine reference paths.
- `runtime08c_batch_animation_reference_borrowed_dedup_p95` reports paired P50/P95
  samples and enforces the 90% P95 reduction gate.
- The managed `runtime08c_batch_` release gate covers this task and the borrowed state-name index
  in one Cargo invocation: 2 source contracts, 6 Rust tests, and 2 performance rows. Dynamic
  marker values, integration commit, and WeCom delivery remain coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime08c still owns compiled asset reference tables, dependency generation publication, graph and
state-machine caches, residency, reload, and product-scale animation receipts. This slice only
converges direct-reference collection.
