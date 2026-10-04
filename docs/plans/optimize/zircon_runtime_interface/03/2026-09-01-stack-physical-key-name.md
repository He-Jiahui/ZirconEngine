record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter.rs
related_tests:
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter/key_name_performance_tests.rs
  - tools/tests/test_runtime_interface03_physical_key_name_performance_contract.py
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter/key_name_performance_tests.rs::runtime_interface03_batch55_61_stack_physical_key_name_release_benchmark
---

# Stack physical key name

## Scope

Unknown physical keyboard codes previously used `format!("KeyCode{key_code}")` on every input
adaptation. The implementation now writes the `u32` decimal digits into a ten-byte stack buffer
and appends them to one preallocated output string. Every named key remains on the existing static
name table; zero, digit boundaries, unknown codes, and `u32::MAX` preserve identical text.

## Verification

- TDD RED: the focused contract found the formatting path and no behavior/benchmark module.
- Focused Batch55-56 static performance contracts after implementation: `4/4` passed.
- The behavior oracle compares stack encoding with the former formatting implementation across
  named and unknown codes, including decimal boundaries and `u32::MAX`.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Full-workspace rustfmt remains externally blocked by unrelated Rust-edition parse errors in
  Editor animation and Runtime prepared-geometry files; those paths were not modified.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.
- Batched managed request `runtime-interface03-batch55-56-20260901-r2` was rejected before ticket
  creation or Cargo execution because the external `E:\Git\zr_vm` worktree was dirty. The request
  produced no compile, behavior, benchmark, integration, push, or performance receipt.
- Batched request `runtime-interface03-batch55-59-20260901-r1` was accepted for asynchronous
  reconciliation, but coordinator post-response timed out with request `ed8c6d3ced7d448d9d805f8302709365`;
  no terminal Cargo or benchmark output is available yet.
- Current Batch55-61 submission `runtime-interface03-batch55-61-20260901-r1` was rejected before
  ticket creation by `validation_ticket_external_worktree_dirty` for external worktree
  `E:\\Git\\zr_vm`; no Cargo, terminal performance, commit, or push evidence exists.

## Performance contract

The ignored release benchmark builds the ten-digit unknown physical key name 1,000,000 times over
11 alternating samples. It compares the former formatting path with stack decimal encoding and
requires at least 20% P95 improvement. Terminal nanosecond values must come from the managed
Windows receipt before integration, push, or WeCom reporting.
