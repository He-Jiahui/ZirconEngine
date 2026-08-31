---
title: Editor01 Assets Activity Indexed Layout
category: zircon_editor
report_id: Editor01-assets-activity-indexed-layout-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Assets Activity Indexed Layout

## Scope

This slice removes repeated full-node scans from Assets Activity content layout. Row geometry,
badge/type overlap, metadata measurement, file-name compaction, empty-state behavior, and the
historical first-match rule for duplicate control IDs remain unchanged.

## Change

- Build one capacity-sized `HashMap<SharedString, usize>` at the start of the layout pass.
- Preserve first-match semantics with `entry(...).or_insert(index)`.
- Reuse the index for panel lookup, row/badge/type/name/metadata frame writes, metadata text lookup,
  item-name compaction, hidden controls, and the empty state.
- Clone only Slint shared-string handles into the index; control-ID bytes are not deep-copied.

## Deterministic Performance Evidence

| 4,096 nodes and 2,048 reverse-order target lookups | Before | After |
|---|---:|---:|
| Control-ID comparisons | 6,292,480 | 4,096 index inserts + 2,048 average `O(1)` probes |
| Duplicate-ID target | first node | first node |
| Layout traversal complexity | `O(rows * nodes)` | average `O(nodes + rows)` |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR01_ASSETS_ACTIVITY_INDEXED_LAYOUT_BENCH_V1`. Acceptance requires indexed lookup P95 to be
at most 60% of linear-scan P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826aj_activity_index_preserves_first_duplicate_match` verifies indexed
  frame mutation, missing lookup, and first-duplicate behavior.
- `optimization_batch_20260826aj_activity_layout_uses_one_control_index` requires the single shared
  index and rejects mutable full-node scans in the layout implementation.
- `optimization_batch_20260826aj_activity_layout_index_p95` reports both P95 values and enforces the
  60% threshold.

## Remaining Parent-plan Work

Editor01 still needs end-to-end retained geometry, damage, large-catalog, queue-age, and product P95
qualification. This slice only converges the Assets Activity row-layout lookup boundary.
