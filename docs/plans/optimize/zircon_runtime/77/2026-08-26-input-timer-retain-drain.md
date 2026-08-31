---
title: Runtime77 Input Timer Retain Drain
category: zircon_runtime
report_id: Runtime77-input-timer-retain-drain-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime77 Input Timer Retain Drain

## Scope

This slice removes clone-and-remove cleanup from the UI input timer expiration path. Typeahead,
submenu hover, tooltip, and toast timers still publish expired targets in `UiNodeId` order and keep
future timers armed.

## Change

- Drain each `BTreeMap` with one `retain` traversal instead of collecting keys and removing every
  expired entry through a second tree lookup.
- Move submenu, tooltip, and toast identifiers out with `std::mem::take`; expired string payloads
  are no longer cloned.
- Preserve the ordered map and therefore the prior deterministic result ordering.

## Deterministic Performance Evidence

| 16,384 tooltip timers, 8,192 expired | Before | After |
|---|---:|---:|
| Full map traversals | 1 | 1 |
| Ordered removals after traversal | 8,192 x `O(log N)` | 0 |
| Expired payload clones | 8,192 | 0 |
| Result order | ascending `UiNodeId` | unchanged |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME77_INPUT_TIMER_RETAIN_DRAIN_BENCH_V1`. Acceptance requires retain-drain P95 to be at least
30% below clone-and-remove P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826al_input_timer_retain_drain_preserves_order_and_pending_entries`
  covers deterministic expired order and retention of future timers.
- `optimization_batch_20260826al_input_timer_uses_single_pass_retain_drains` requires four retain
  boundaries, three moved string payloads, and rejects second-pass removal loops.
- `optimization_batch_20260826al_input_timer_retain_drain_p95` reports paired P50/P95 samples and
  enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Runtime77 still owns full input dispatch, focus, navigation, capture, gesture, drag/drop, IME, and
window-lifecycle product integration. This slice only converges timer expiration cleanup.
