---
title: Editor02 History Store Hash Index
category: zircon_editor
report_id: Editor02-history-store-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor02 History Store Hash Index

## Scope

This slice replaces the transaction engine's per-context history-store owner with `HashMap`.
Transaction commit, undo/redo status and replay, save-token checks, context teardown, and history
journal reads now resolve `HistoryContextId` through expected constant-time lookup.

The separate history-generation owner remains a `BTreeMap`, preserving deterministic reset-batch
context order. Global, document, and play-session histories remain isolated; record order inside
each `HistoryStore`, dirty-journal ordering, capacity, merge, and save-token semantics are
unchanged.

## Performance Workload

The release workload fills 4,096 document history contexts and performs 4,096 stable hits for the
final production `HistoryContextId` key.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered history-store lookups | 4,096 | 0 |
| Hash history-store lookups | 0 | 4,096 |
| Generation reset-order changes | 0 | 0 |
| Allocations on history-store hits | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR02_HISTORY_STORE_HASH_INDEX_BENCH_V1`. Acceptance requires hash lookup P95 to be at least 30%
below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bz_history_store_hash_index_isolates_contexts` covers independent
  global, document, and play-session ownership plus context-local removal.
- `optimization_batch_20260826bz_history_store_hash_index_preserves_generation_order_owner` locks
  the split hash-store/tree-generation contract.
- `optimization_batch_20260826bz_history_store_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor02 still owns product undo routing, document-close history lifecycle, shared UI asset and
animation history authority, journal persistence, autosave recovery, CAS save, and close
coordination. This slice only converges per-context history-store lookup.
