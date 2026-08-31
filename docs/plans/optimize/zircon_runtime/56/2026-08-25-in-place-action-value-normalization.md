---
title: Runtime56 In-place Action Value Normalization
category: zircon_runtime
report_id: Runtime56-in-place-action-value-normalization-2026-08-25
date: 2026-08-25
session_id: root-runtime56-three-task-input-performance-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime56 In-place Action Value Normalization

## Scope

This slice reuses the consumed action-value `BTreeMap` while constructing `InputActionState`. It
preserves action ordering, finite-value filtering, positive and negative zero filtering, `[-1, 1]`
clamping, pressed and transition sets, serde shape, and every public query result. It does not change
Runtime56 mapping, focus, device state, recording, replay, or input authority.

## Implementation

The retired constructor consumed every `(action, value)` pair, filtered and normalized it, and
collected retained pairs into a replacement `BTreeMap`. That rebuilt the tree and moved every retained
owned key even though the caller had already provided the required map. The optimized path calls
`BTreeMap::retain` on the consumed map, removes invalid and zero values in place, and writes clamped
values directly into retained entries. The original tree allocation and retained keys survive.

The regression compares retired and optimized results for finite values, positive and negative zero,
NaN, infinity, and both clamp directions. A source contract requires in-place retain and writeback and
rejects the retired filter-map collection.

## Performance Contract

| Evidence per 2,048-action normalization | Retired path | Optimized gate |
| --- | ---: | ---: |
| Replacement `BTreeMap` builds | 1 | 0 |
| Retained owned-key moves | `N` | 0 |
| Alternating release benchmark | 11 samples x 32 maps | optimized P95 <= 65% of retired P95 |

The benchmark emits `RUNTIME56_IN_PLACE_ACTION_VALUE_NORMALIZATION_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/action/retained-action counts, replacement trees, and retained
key moves.

## Validation

The TDD source probe first observed all five retired-path indicators, then observed in-place retain,
value writeback, and no replacement collection. Rust 1.94.1 formatting and scoped static checks passed
before batching. The managed `runtime56_batch_` release gate covers this slice together with
incremental context ordering and scratch-free binding sorting: 3 source contracts, 10 Rust tests, and
3 performance rows in one Cargo invocation. Dynamic P95 evidence, integration SHA, automatic commit,
and automatic WeCom performance delivery remain coordinator-owned and pending.
