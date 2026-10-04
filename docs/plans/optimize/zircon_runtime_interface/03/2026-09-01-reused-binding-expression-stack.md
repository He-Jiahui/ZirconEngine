record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs
related_tests:
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program/expression_stack_performance_tests.rs
  - tools/tests/test_runtime_interface03_binding_expression_stack_performance_contract.py
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program/expression_stack_performance_tests.rs::runtime_interface03_batch33_reused_binding_expression_stack_release_benchmark
---

# Reused binding expression stack

## Scope

Compiled binding-program validation previously allocated a DFS `Vec` for every payload and target
expression. One stack with the existing eight-entry inline capacity is now created per program
validation and borrowed by every expression check. The helper clears the stack on entry and before
both successful and failed returns, preserving node/depth budgets and making repeated calls
independent.

## Verification

- TDD RED: the focused contract found no expression stack outside the binding loop and no release
  benchmark module.
- Focused Batch33 static performance contracts after implementation: `2/2` passed.
- Batched static regression: `87/87` passed (`75` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares reused and former allocating stacks for valid branching input,
  invalid property ID, non-finite literal, excessive depth, and a valid call after failures while
  asserting the borrowed stack is empty after every result.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Batch33-34 snapshot `2735` was created by request `bf76138479f841be829b173b6786fe80`.
The batched managed validation request `c8d37d27613041a7bd2260ddaa7f65f4` was rejected before
queueing by the external `E:\Git\zr_vm` dirty-worktree preflight. No validation ticket, Cargo run,
or terminal benchmark result exists for Batch33.

Ownership receipt: exact-path lease request `9aa4e638e2844bc5a7f23c67cc0b0e79`; baseline
attribution request `f594c991ed984d17baf547e38b0fe90e` (`attributed`).

## Performance contract

The ignored release benchmark validates a small branching compiled expression 200,000 times over
11 alternating samples. It compares per-expression allocation with stack reuse and requires at
least 50% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.
