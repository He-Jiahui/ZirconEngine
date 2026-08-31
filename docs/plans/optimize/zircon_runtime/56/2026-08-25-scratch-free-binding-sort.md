---
title: Runtime56 Scratch-free Binding Sort
category: zircon_runtime
report_id: Runtime56-scratch-free-binding-sort-2026-08-25
date: 2026-08-25
session_id: root-runtime56-three-task-input-performance-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime56 Scratch-free Binding Sort

## Scope

This slice removes stable-sort scratch allocation while normalizing action-binding button and axis
lists. It preserves ascending button order, the existing gamepad/axis/direction ordering key,
duplicate removal, action ownership, serde shape, and every public constructor result. Equal elements
are removed immediately after sorting, so their internal relative order is not observable. It does not
change Runtime56 action evaluation, context priority, device state, recording, or replay authority.

## Implementation

The retired constructors used stable slice sorting even though both lists contain no payload whose
relative order must survive equality. The optimized constructors use in-place unstable sorting with
the same total ordering, then retain the existing `dedup` steps. This eliminates the stable-sort
scratch buffer without introducing a new allocation or changing the normalized binding.

The regression compares retired and optimized bindings across mixed button variants, mixed axis keys,
and duplicates. A source contract requires the two unstable sorts and rejects both retired stable-sort
calls in production.

## Performance Contract

| Evidence per 4,096-button + 4,096-axis binding | Retired path | Optimized gate |
| --- | ---: | ---: |
| Stable sorts | 2 | 0 |
| Stable-sort scratch buffers | up to 2 | 0 |
| Alternating release benchmark | 11 samples x 32 bindings | optimized P95 <= 80% of retired P95 |

The benchmark emits `RUNTIME56_SCRATCH_FREE_BINDING_SORT_BENCH_V1` with both P95 timings, reduction
basis points, sample/iteration/button/axis counts, and stable sorts per binding.

## Validation

The scoped TDD source probe first observed both retired stable sorts, then observed one button
`sort_unstable` and one axis `sort_unstable_by` with neither retired call remaining. Rust 1.94.1
formatting and scoped static checks passed before batching. The managed `runtime56_batch_` release gate
covers this slice together with incremental context ordering and in-place action-value normalization:
3 source contracts, 10 Rust tests, and 3 performance rows in one Cargo invocation. Dynamic P95
evidence, integration SHA, automatic commit, and automatic WeCom performance delivery remain
coordinator-owned and pending.
