---
title: Editor57 Single-pass Compact Utility Collapse
category: zircon_editor
report_id: Editor57-single-pass-compact-utility-collapse-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor57 Single-pass Compact Utility Collapse

## Scope

This slice removes repeated full-node scans when compact Asset Browser utility content collapses on
short viewports. It preserves the exact nine preview controls, duplicate controls, finite coordinate
normalization, zero-height collapse, unrelated nodes, and all public Editor/UI contracts. It does not
alter expanded preview layout or utility tab placement.

## Implementation

`collapse_compact_utility_content` previously invoked `set_node_frame` once for each of nine control
IDs, traversing the entire node slice nine times. The optimized path normalizes the target frame once
and uses one node pass with an exact string-pattern match. Every matching duplicate is still updated;
all other controls are skipped.

The regression covers all nine targets, a duplicate target, and an unrelated node with a retained
frame. The ignored release benchmark compares retired and optimized frame output before timing.

## Performance Contract

| Evidence | Retired path | Optimized gate |
| --- | ---: | ---: |
| Full node-slice passes per compact utility collapse | 9 | 1 |
| Alternating release benchmark | 11 samples x 256 collapses x 4,096 nodes | optimized P95 <= 35% of retired P95 |

The benchmark emits `EDITOR57_SINGLE_PASS_COMPACT_UTILITY_COLLAPSE_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/node counts, and full-pass counts.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, and production structure checks are required before
submission. One managed Editor57 Cargo invocation filtered by `editor57_compact_single_pass_`
covers this behavior regression and ignored release benchmark together with compact sources layout.
Dynamic P95 evidence, integration SHA, and automatic WeCom performance delivery remain
coordinator-owned and pending.
