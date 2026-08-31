---
title: Runtime77 Analog Navigation Single Normalization
category: zircon_runtime
report_id: Runtime77-analog-navigation-single-normalization-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime77 Analog Navigation Single Normalization

## Scope

This slice removes duplicate control-name normalization from retained analog navigation dispatch.
Supported aliases, axis and direction mapping, repeat timing, user-qualified state keys, dead-zone
reset, and diagnostics remain unchanged.

## Change

- Normalize the authored analog control name once at event admission and pass the borrowed result
  through axis classification, repeat lookup, insertion, and reset.
- Keep the persisted repeat key format exactly `user{user_id}:{normalized_control}:{direction}`.
- Keep both direction-key removals for a recognized axis when the value returns to the dead zone.

## Deterministic Performance Evidence

| Recognized analog event | Before | After |
|---|---:|---:|
| Control normalization passes, active direction | 2 | 1 |
| Control normalization passes, dead-zone reset | 3 | 1 |
| State-key formatting operations, active direction | 1 | 1 |
| State-key formatting operations, dead-zone reset | 2 | 2 |

The ignored release gate runs 17 alternating sample pairs over 32,768 events and emits
`RUNTIME77_ANALOG_NAVIGATION_SINGLE_NORMALIZATION_BENCH_V1`. Acceptance requires single-pass
normalization P95 to be at least 20% below duplicate-normalization P95. Exact Windows timings
remain pending the coordinator run.

## Acceptance

- `runtime77_analog_navigation_preserves_normalized_repeat_keys` covers alias
  normalization, user-qualified state, repeat suppression, and dead-zone cleanup.
- `runtime77_analog_navigation_normalizes_control_once_per_event` requires one
  event normalization and rejects normalization inside state-key construction.
- `runtime77_analog_navigation_single_normalization_p95` reports paired
  P50/P95 samples and enforces the 20% P95 reduction gate.

These tests are grouped with picking-report single-pass projection in one two-task asynchronous
coordinator batch. Terminal timings, integration, record finalization, and automatic WeCom
delivery remain pending.

## Remaining Parent-plan Work

Runtime77 still owns qualified input identity, transactional effects, navigation indexing,
route-scratch reuse, queue backpressure, device coverage, and product-scale performance receipts.
This slice only converges repeated analog-control normalization.
