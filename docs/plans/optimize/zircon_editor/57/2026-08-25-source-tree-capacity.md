---
title: Editor57 Source-tree Row Capacity
category: zircon_editor
report_id: Editor57-source-tree-row-capacity-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor57 Source-tree Row Capacity

## Scope

This slice preallocates the exact additional row capacity required while rebuilding Asset Browser
source-tree nodes. It preserves prototype discovery, removal of prior dynamic rows, fallback row
behavior, folder order, pointer control IDs, selection, recursive counts, depth, and all surrounding
static template nodes. It does not change Editor57's folder model, expansion, navigation, selection,
or virtualization contracts.

## Implementation

The retired path removed the previous dynamic rows and immediately pushed the replacement rows into
the retained `Vec`. A cold large tree therefore grew geometrically and repeatedly moved the already
built `ViewTemplateNodeData` values. The optimized path computes `folder_tree.len().max(1)` after
prototype validation and row removal, then reserves that many additional slots before either the
fallback push or the folder loop. A warm vector with sufficient retained capacity performs no new
allocation.

The regression compares retired and optimized row identity, text, count, selection, and depth across
128 folders. A source contract requires the reserve to precede the production row loop.

## Performance Contract

| Evidence for a cold 2,048-row source tree | Retired path | Optimized gate |
| --- | ---: | ---: |
| Row-vector capacity growth events | measured geometric growth, greater than 1 | exactly 1 |
| Warm rebuild growth events | capacity dependent | 0 when retained capacity is sufficient |
| Alternating release benchmark | 11 samples x 32 rebuilds | optimized P95 <= 90% of retired P95 |

The benchmark emits `EDITOR57_SOURCE_TREE_CAPACITY_BENCH_V1` with both P95 timings, reduction basis
points, sample/iteration/row counts, and measured cold capacity-growth counts.

## Validation

The TDD source probe first observed no row-capacity reservation, then observed the reservation before
the production folder loop. Rust 1.94.1 formatting and scoped diff checks passed before submission.
One managed Editor batch covers retired/optimized row equivalence, the source contract, and the
ignored release benchmark. Dynamic P95 evidence, integration SHA, automatic commit, and automatic
WeCom performance delivery remain coordinator-owned and pending.
