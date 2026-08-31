---
title: Runtime63 Subscription Hash Index
category: zircon_runtime
report_id: Runtime63-subscription-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime63-two-task-hash-index-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime63 Subscription Hash Index

## Scope

This slice replaces the subscription table's five unordered key-routing maps with `HashMap`:
token ownership, subtree routing, component-type routing, asset routing, and pending-fact
coalescing. World mutation and inspection publication now use expected constant-time lookup for
every direct subscription key.

All externally visible ordering remains separately owned. World tokens, per-key tokens, and dirty
tokens remain `BTreeSet`; pending facts remain in their existing `Vec`; terminal flush order and
fact coalescing position are unchanged. Limits, overflow diagnostics, ancestry scratch reuse,
watch token allocation, and removal semantics are unchanged.

## Performance Workload

The release workload fills 1,024 component subscription keys with long shared prefixes and
performs 4,096 stable hits for the final key.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered subscription-key lookups | 4,096 | 0 |
| Hash subscription-key lookups | 0 | 4,096 |
| Ordered token-set policy changes | 0 | 0 |
| Allocations on direct routing hits | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME63_SUBSCRIPTION_HASH_INDEX_BENCH_V1`. Acceptance requires hash lookup P95 to be at least
30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the coordinator
run.

## Acceptance

- `runtime63_batch_subscription_hash_index_preserves_targeted_routing` covers
  component routing, token ordering, unwatch removal, and unrelated-key isolation.
- `runtime63_batch_subscription_hash_index_keeps_ordered_token_sets` locks the split
  between hash key routing and ordered token publication.
- `runtime63_batch_subscription_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.
- The managed `runtime63_batch_` release gate covers this task and component schema-generation
  hashing in one Cargo invocation: 2 source contracts, 6 Rust tests, and 2 performance rows.
  Dynamic marker values, integration commit, and WeCom delivery remain coordinator-owned and
  pending.

## Remaining Parent-plan Work

Runtime63 still owns reflection catalog transactions, stable type schemas, compiled property
plans, mutation transactions, inspection publication, and subscription cursors. This slice only
converges subscription-table key routing and fact coalescing indexes.
