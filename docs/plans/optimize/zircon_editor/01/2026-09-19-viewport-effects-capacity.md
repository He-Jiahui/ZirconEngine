---
title: Editor viewport effect projection capacity
category: zircon_editor
report_id: Editor817-viewport-effects-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor817 · viewport effect projection capacity

## Scope

`execute_viewport_event` emits at most three ordered `EditorEventEffect` values
from `viewport_effects`: render, presentation, and reflection invalidation. The
old collector started empty on every event, so a full three-effect projection
could grow its scratch vector unnecessarily. The optimized path derives the
same three predicates once, reserves their exact count, and retains the
existing effect order and empty-event behavior.

No viewport command, feedback, structural-change, chrome-projection, or effect
semantics changed. The empty cancel path still returns a zero-capacity vector;
only effect-bearing paths receive a bounded reservation.

## TDD and deterministic model

- The Python source contract was intentionally RED against the original
  `Vec::new()` collector and missing lower marker, then GREEN at `3/3` after
  the exact-count reservation and lower regression were added.
- The lower regression checks zero-capacity empty output, exact three-effect
  order, and exact full-path capacity. The ignored marker
  `EDITOR817_VIEWPORT_EFFECTS_CAPACITY_BENCH_V1` is wired for the managed
  Windows Release lane.
- For the maximum three effects, the zero-capacity model performs one initial
  geometric growth event and the exact-count model performs zero (`1 -> 0`).
  This is allocation-shape evidence only, not allocator, CPU, RSS, or product
  percentile evidence.

## Local validation

- `tools/tests/test_editor_viewport_effects_capacity_performance_contract.py`:
  `3/3`.
- `python -m py_compile` for the contract: pass.
- Scoped `rustfmt --edition 2021 --check` for
  `zircon_editor/src/ui/host/editor_event_execution/viewport_event.rs`: pass.
- Existing viewport effect semantic regressions remain in the same lower test
  module; managed Cargo/Release execution has not been started in this shared
  checkout.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/editor_event_execution/viewport_event.rs` | `608CB9F7E2230FD8653D43EB6CA867FE01EBDE10337CB49AA4B9E0E93212B9B7` |
| `tools/tests/test_editor_viewport_effects_capacity_performance_contract.py` | `B2A9041C3B1D63771F4F88686C71CDD4CDF7902635A1C8E6AFB4EF881C2A115F` |

## Managed validation boundary

Managed Windows Cargo/Release execution, ignored benchmark timing, allocator
evidence, and viewport input-to-present p50/p95/p99 evidence remain pending
under the shared external `E:\Git\zr_vm` admission gate. This session does not
poll or monitor the coordinator; tooling production remains deferred for the
later Rust migration.
