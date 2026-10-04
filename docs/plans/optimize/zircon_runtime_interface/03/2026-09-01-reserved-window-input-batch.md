record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/window/pump.rs
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter.rs
related_tests:
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter/batch_capacity_performance_tests.rs
  - tools/tests/test_runtime_interface03_window_input_batch_capacity_performance_contract.py
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter/batch_capacity_performance_tests.rs::runtime_interface03_batch55_61_window_input_batch_capacity_release_benchmark
---

# Reserved window input batches

## Scope

`runtime_events_to_window_input_pump_batch` previously started every converted event batch with an
empty `Vec`, causing repeated growth for the normal bounded event bursts. The adapter now consumes
the iterator once, uses its lower size hint to reserve the batch, and preserves the existing event
conversion and error order. The pump API remains compatible; `Default` still constructs an empty
batch for callers that do not have a size hint.

## Verification

- TDD RED: the focused contract found the default allocation path and missing capacity API and
  benchmark module.
- The behavior oracle compares reserved and default batches for an ordered 256-event burst.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.
- Current Batch55-61 submission `runtime-interface03-batch55-61-20260901-r1` was rejected before
  ticket creation by `validation_ticket_external_worktree_dirty` for external worktree
  `E:\\Git\\zr_vm`; no Cargo, terminal performance, commit, or push evidence exists.

## Performance contract

The ignored release benchmark builds 25,000 256-event batches over 11 alternating samples. It
compares default Vec growth with lower-bound reservation and requires at least 50% P95 improvement.
Terminal nanosecond values must come from the managed Windows receipt before integration, push, or
WeCom reporting.
