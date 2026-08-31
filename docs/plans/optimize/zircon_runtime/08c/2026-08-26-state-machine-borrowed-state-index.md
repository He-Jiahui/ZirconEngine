---
title: Runtime08c State Machine Borrowed State Index
category: zircon_runtime
report_id: Runtime08c-state-machine-borrowed-state-index-2026-08-26
date: 2026-08-26
session_id: root-runtime08c-two-task-borrowed-index-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime08c State Machine Borrowed State Index

## Scope

This slice removes repeated state-vector scans from state-machine evaluation when projected lookup
work is large enough to justify an index. Current-state fallback, transition order, invalid target
skipping, duplicate state-name first-match behavior, transition duration handling, graph ownership,
and returned parameter ownership remain unchanged.

## Change

- Estimate state lookup work from state and transition counts before choosing the lookup strategy.
- Keep small state machines on the allocation-free linear path below 128 projected comparisons.
- Build one borrowed `&str -> &AnimationStateAsset` HashMap for larger machines.
- Preserve the old first-state rule with entry `or_insert`, then reuse the lookup for current,
  entry, transition-target, and final graph resolution.

## Deterministic Performance Evidence

| 2,048 states, 256 invalid targets before one valid target, two evaluations per sample | Before | After |
|---|---:|---:|
| State-name comparisons per sample | 1,056,770 | 0 |
| Borrowed state-index visits per sample | 0 | 4,096 |
| State hash lookups per sample | 0 | 518 |
| Small-machine index allocations below threshold | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME08C_STATE_MACHINE_BORROWED_STATE_INDEX_BENCH_V1`. Acceptance requires indexed evaluation
P95 to be at least 90% below repeated state scans. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `runtime08c_batch_state_machine_index_preserves_first_state_and_transition_order`
  covers invalid-target skipping, transition priority, duplicate-name first match, active state,
  and graph projection.
- `runtime08c_batch_state_machine_uses_borrowed_state_index` requires the adaptive
  borrowed HashMap, first-entry preservation, and removal of the repeated lookup helper.
- `runtime08c_batch_state_machine_borrowed_state_index_p95` reports paired P50/P95
  samples and enforces the 90% P95 reduction gate.
- The managed `runtime08c_batch_` release gate covers this task and animation-reference borrowed
  deduplication in one Cargo invocation: 2 source contracts, 6 Rust tests, and 2 performance rows.
  Dynamic marker values, integration commit, and WeCom delivery remain coordinator-owned and
  pending.

## Remaining Parent-plan Work

Runtime08c still owns compiled state-machine programs, stable parameter slots, instance caches,
transition arbitration, interruption, nested machines, layers, cadence, and product-scale
receipts. This slice only converges manager-side state lookup.
