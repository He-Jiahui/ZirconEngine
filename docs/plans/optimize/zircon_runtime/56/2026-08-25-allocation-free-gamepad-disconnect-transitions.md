---
title: Runtime56 Allocation-free Gamepad Disconnect Transitions
category: zircon_runtime
report_id: Runtime56-allocation-free-gamepad-disconnect-transitions-2026-08-25
date: 2026-08-25
session_id: root-runtime56-bulk-button-release-20260825
implementation_status: implementation_complete
validation_status: managed_validation_queued
validation_ticket: 2a6e907071354f28b114db0c90a9074c
---

# Runtime56 Allocation-free Gamepad Disconnect Transitions

## Scope

This slice removes the temporary transition vector created when a gamepad disconnects. It preserves
the source `BTreeMap` iteration order, target-device filtering, zero-value suppression, transition
fields, subsequent axis/button cleanup, button releases, and every emitted frame-state result. It does
not change Runtime12 gamepad polling budgets, disconnect ordering, input ownership, or other pending
changes already present in the touched manager and state files.

## Implementation

The retired manager filtered matching axes into a temporary `Vec<GamepadAxisTransition>` and then
extended the frame transition buffer from that vector. The optimized state helper extends the existing
transition buffer directly from the filtered axis iterator. The destination still owns the same values
in the same order, while the intermediate allocation and second transition-buffer write disappear.

The regression compares retired and optimized output with matching, nonmatching, zero-valued, and
pre-existing transitions. A cross-file source contract requires the state helper call from the
disconnect arm and rejects the retired temporary collection.

## Performance Contract

| Evidence per 4,096-axis disconnect | Retired path | Optimized gate |
| --- | ---: | ---: |
| Temporary transition vectors | 1 | 0 |
| Transition writes per matching axis | 2 | 1 |
| Matching axes | 2,048 | 2,048 |
| Alternating release benchmark | 11 samples x 64 disconnects | optimized P95 <= 80% of retired P95 |

The benchmark emits `RUNTIME56_ALLOCATION_FREE_GAMEPAD_DISCONNECT_TRANSITIONS_BENCH_V1` with both
P95 timings, reduction basis points, sample/iteration/axis/matching-axis counts, temporary vectors,
and transition writes per match.

## Validation

The scoped TDD source probe first observed the missing direct append helper and retained manager-side
temporary vector, then observed direct extension and the single manager call. Rust 1.94.1 formatting
and scoped static checks passed before batching. One managed Runtime56 batch covers this slice together
with indexed focus-loss axis reset, including equivalence, source contracts, and both ignored release
benchmarks. Dynamic P95 evidence, integration SHA, automatic commit, and automatic WeCom performance
delivery remain coordinator-owned and pending.
