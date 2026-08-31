---
title: Runtime56 Indexed Focus-loss Axis Reset
category: zircon_runtime
report_id: Runtime56-indexed-focus-loss-axis-reset-2026-08-25
date: 2026-08-25
session_id: root-runtime56-bulk-button-release-20260825
implementation_status: implementation_complete
validation_status: managed_validation_queued
validation_ticket: 2a6e907071354f28b114db0c90a9074c
---

# Runtime56 Indexed Focus-loss Axis Reset

## Scope

This slice removes repeated linear transition searches when focus loss releases active gamepad axes.
It preserves clearing the live axis map, zero-value suppression, the first-match behavior for an
existing transition, original transition order and `previous_value`, sorted append order for new
transitions, and all other focus-loss state changes. It does not change Runtime12 producer budgets,
event coalescing, input ownership, or the existing host-request frame-start work already present in the
touched file.

## Implementation

The retired path consumed every axis and linearly searched the complete transition vector for a match.
With `A` axes and `T` existing transitions, focus loss could therefore perform up to `A * T` equality
probes. The optimized path consumes the same ordered axis `BTreeMap`, walks existing transitions once,
and removes matched keys through the existing tree index. Remaining nonzero axes are appended in the
same key order. Duplicate transition behavior remains first-match because the first transition removes
the only matching key.

The regression compares retired and optimized output with existing, missing, zero-valued, unrelated,
and duplicate transitions. A source contract requires the transition pass, indexed key removal, and
single append phase and rejects the retired nested search.

## Performance Contract

| Evidence per 1,024-axis / 512-transition reset | Retired path | Optimized gate |
| --- | ---: | ---: |
| Linear transition-probe upper bound | 524,288 | 0 |
| Indexed tree removals | 0 | 512 |
| Transition-vector passes | up to 1,024 | 1 |
| Alternating release benchmark | 11 samples x 8 resets | optimized P95 <= 25% of retired P95 |

The benchmark emits `RUNTIME56_INDEXED_FOCUS_LOSS_AXIS_RESET_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/axis/transition counts, the retired linear-probe upper bound,
and optimized tree removals.

## Validation

The scoped TDD source probe first observed the missing indexed reset helper, then observed one existing
transition pass, indexed removals, and one append phase. Rust 1.94.1 formatting and scoped static checks
passed before batching. One managed Runtime56 batch covers this slice together with allocation-free
disconnect transition extension, including equivalence, source contracts, and both ignored release
benchmarks. Dynamic P95 evidence, integration SHA, automatic commit, and automatic WeCom performance
delivery remain coordinator-owned and pending.
