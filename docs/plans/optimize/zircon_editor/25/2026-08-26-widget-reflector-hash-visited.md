---
title: Editor25 Widget Reflector Hash Visited Membership
category: zircon_editor
report_id: Editor25-widget-reflector-hash-visited-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor25 Widget Reflector Hash Visited Membership

## Scope

This slice removes logarithmic ordered-set membership from widget-reflector row projection. Root,
child, cycle, duplicate-root, and orphan behavior remain unchanged because rows are published from
snapshot root/child order and the snapshot node map, never by iterating the visited set.

## Change

- Replace the recursive `BTreeSet<UiNodeId>` visited index with `HashSet<UiNodeId>`.
- Preserve root and child traversal order, cycle suppression, and the ordered orphan fallback.
- Keep reflector snapshots, row payloads, selection, and diagnostics contracts unchanged.

## Deterministic Performance Evidence

| Representative 65,536 visits / 8,192 unique nodes | Before | After |
|---|---:|---:|
| Membership class | O(log n) | average O(1) |
| Visit insert attempts | 65,536 | 65,536 |
| Published row order | snapshot-defined | snapshot-defined |

The ignored release gate runs 17 alternating samples and emits
`EDITOR25_WIDGET_REFLECTOR_HASH_VISITED_BENCH_V1`. Acceptance requires hash-visited P95 to be at
most 60% of ordered-set P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826w_editor25_hash_visited_preserves_tree_and_orphan_order` covers a
  duplicate root, a cycle, depth projection, and ordered orphan fallback through the product model.
- `optimization_batch_20260826w_editor25_widget_reflector_uses_hash_visited_membership` requires
  the production hash boundary and rejects ordered-set membership.
- `optimization_batch_20260826w_editor25_widget_reflector_hash_visited_performance_evidence`
  checks admission equivalence, reports both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Editor25 still needs virtualized metric series, provider health/cadence receipts, multi-session
selection, capture analysis, typed watch/events, and a complete remote diagnostics consumer. This
slice only reduces reflector traversal overhead.
