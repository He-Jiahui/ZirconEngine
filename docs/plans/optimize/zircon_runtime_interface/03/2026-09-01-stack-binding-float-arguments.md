record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
related_tests:
  - tools/tests/test_runtime_interface03_binding_float_array_performance_contract.py
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs::runtime_interface03_batch28_stack_float_arguments_release_benchmark
---

# Stack binding float arguments

## Scope

Typed `vec2`, `vec3`, and `vec4` literal parsing previously allocated a temporary `Vec<f64>` and
converted it to the already-known `[f64; N]` return type. The parser now fills the fixed array in
place. Numeric coercion, comma admission, missing-number errors, right-parenthesis validation, and
the public typed values are unchanged.

## Verification

- TDD RED: the focused contract found `Vec::with_capacity(N)` plus `try_into` and no benchmark.
- Focused Batch25-28 static performance contracts after implementation: `8/8` passed.
- Batched static regression: `77/77` passed (`65` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares fixed-array and former allocating parsers, including final parser
  cursor position.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Managed submission remains blocked before queueing by the external `E:\Git\zr_vm` dirty-worktree
preflight. No ticket or terminal benchmark result exists.

Ownership receipt: exact-path lease request `0dba7dc54c8c4755bd124abbdf6a7326`; baseline
attribution request `4b36b693646d4358955f37e4534d0cfc` (`attributed`).

## Performance contract

The ignored release benchmark parses four numeric arguments 200,000 times over 11 alternating
samples while reusing the same token buffer. It compares the former temporary-vector path with the
stack array and requires at least 20% P95 improvement. Terminal nanosecond values must come from the
managed Windows receipt before integration, push, or WeCom reporting.
