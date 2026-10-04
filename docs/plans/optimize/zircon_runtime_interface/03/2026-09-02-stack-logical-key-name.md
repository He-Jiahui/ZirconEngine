record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-02
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter.rs
related_tests:
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter/key_name_performance_tests.rs
  - tools/tests/test_runtime_interface03_logical_key_name_performance_contract.py
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter/key_name_performance_tests.rs::runtime_interface03_batch68_69_stack_logical_key_name_release_benchmark
---

# Stack logical key name

## Scope

Unknown logical keyboard codes previously used `u32::to_string()` on every pressed or released
runtime event. The implementation now reserves the maximum ten decimal bytes and reuses the
adapter's stack decimal encoder. Named keys retain the static name table, while zero, decimal
boundaries, unknown codes, and `u32::MAX` preserve the previous output exactly.

## Verification

- TDD RED: the focused contract found `key_code.to_string()` in the production function and no
  logical-key behavior or release benchmark.
- Focused physical/logical/accessibility/input-batch static contracts after implementation: `8/8`
  passed.
- The Rust behavior oracle compares the stack path with the former formatting implementation for
  named and unknown codes, including decimal boundaries and `u32::MAX`.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number, integration, push, or WeCom claim exists yet.
- Batched request `runtime-interface03-batch68-69-20260902-r1` was rejected before ticket creation
  or Cargo execution by `validation_ticket_external_worktree_dirty` for external worktree
  `E:\\Git\\zr_vm`; it produced no compile, behavior, benchmark, integration, push, or performance
  receipt.

## Performance contract

The ignored release benchmark builds the ten-digit unknown logical key name 1,000,000 times over
11 alternating samples. It compares `u32::to_string()` with stack decimal encoding and requires at
least 20% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt.
