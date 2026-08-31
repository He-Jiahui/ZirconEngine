---
title: Editor23 Tree Wrap Control ID Hash Index
category: zircon_editor
report_id: Editor23-tree-wrap-control-id-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Tree Wrap Control ID Hash Index

## Scope

This slice removes repeated full document scans while generating the wrapper control ID for a UI
asset tree edit. ASCII label normalization, empty normalized labels, base-first selection, smallest
numeric suffix, wrapper node/mount construction, selection, and tree mutation remain unchanged. It
does not claim the parent plan's broader hierarchy, schema, transaction, or designer milestones are
complete.

## Change

- Traverse document nodes once and borrow existing control IDs into a local hash set.
- Probe the wrapper base and numeric suffix candidates against that set.
- Preserve the exact suffix start at two and first free suffix behavior.
- Keep the index scoped to one wrap operation so no stale tree-side cache is introduced.

## Deterministic Performance Evidence

| 2,048 dense IDs, four lookups per sample | Before | After |
|---|---:|---:|
| Candidate string comparisons per sample | 8,400,896 | 0 |
| Candidate hash probes per sample | 0 | 8,196 |
| One-time control ID index insertions per sample | 0 | 8,192 |
| Selected ID changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_TREE_WRAP_CONTROL_ID_HASH_INDEX_BENCH_V1`. Acceptance requires hash-indexed wrapper ID
selection P95 to be at least 90% below repeated node scans. Exact Windows timings remain pending
the coordinator run.

## Acceptance

- `optimization_batch_20260826bc_tree_wrap_control_id_preserves_smallest_gap` covers sparse
  suffixes, unused bases, normalization, and empty-base compatibility.
- `optimization_batch_20260826bc_tree_wrap_control_id_uses_one_hash_index` requires one node
  traversal and hash membership helper and rejects repeated `iter_nodes().any` scans.
- `optimization_batch_20260826bc_tree_wrap_control_id_hash_index_p95` reports paired release
  P50/P95 samples and enforces the 90% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns lossless tree editing, schema-backed drop receipts, multi-selection, hierarchy
search, transaction-safe save, preview fidelity, and large-asset gates. This slice only converges
wrapper control ID selection.
