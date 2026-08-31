---
title: Editor23 Preview Control Hash Index
category: zircon_editor
report_id: Editor23-preview-control-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Preview Control Hash Index

## Scope

This slice replaces ordered-map construction used by preview control-ID lookup with a hash index.
Document traversal order, duplicate control-ID first-wins behavior, missing-control omission,
preview command order, labels, selection, and hit geometry remain unchanged.

## Change

- Build a borrowed `HashMap<&str, &UiNodeDefinition>` for the three preview projection consumers
  instead of a `BTreeMap` that compares and rebalances on every insertion.
- Preserve `entry(control_id).or_insert(node)` so the first node in deterministic document
  traversal remains authoritative for duplicate IDs.
- Keep the index local to each projection call; this slice does not add stale cross-generation
  cache state.

## Deterministic Performance Evidence

| 16,384 uniquely controlled nodes | Before | After |
|---|---:|---:|
| Index insertions per build | 16,384 tree insertions | 16,384 hash insertions |
| Expected construction complexity | `O(N log N)` | `O(N)` |
| Borrowed node/control strings | yes | yes |
| Output sorting work | none | none |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_PREVIEW_CONTROL_HASH_INDEX_BENCH_V1`. Acceptance requires hash-index construction P95 to
be at least 25% below ordered-tree construction P95. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826aq_preview_control_hash_index_preserves_first_duplicate` verifies
  unique lookup and pointer identity for the first duplicate control ID.
- `optimization_batch_20260826aq_preview_control_index_uses_linear_hash_build` requires a local
  hash index and rejects the ordered map while preserving first-wins insertion.
- `optimization_batch_20260826aq_preview_control_hash_index_p95` reports paired P50/P95 samples and
  enforces the 25% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns lossless V2 editing, revisioned transactions, preview fidelity, typed binding
and mock schemas, incremental validation, cook artifacts, and large-asset gates. This slice only
converges preview control-ID lookup construction.
