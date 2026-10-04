record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
related_tests:
  - zircon_runtime_interface/src/ui/template/asset/binding/expression/integer_token_performance_tests.rs
  - tools/tests/test_runtime_interface03_integer_token_parse_performance_contract.py
  - zircon_runtime_interface/src/ui/template/asset/binding/expression/integer_token_performance_tests.rs::runtime_interface03_batch29_stack_integer_token_release_benchmark
---

# Stack integer token parse

## Scope

The binding tokenizer previously collected every numeric token into a temporary `String`, even
when an integer was valid and immediately converted into `i64`. The integer path now performs
checked decimal accumulation directly over the scanned character slice. It accumulates negative
values with checked subtraction so `i64::MIN` remains representable, and materializes the original
text only for floats, a lone minus sign, or integer overflow diagnostics.

## Verification

- TDD RED: the focused contract found no slice-based integer parser and no release benchmark.
- Focused Batch29 static performance contracts after implementation: `2/2` passed.
- Batched static regression: `79/79` passed (`67` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares the stack and former allocating parsers for zero, signed values,
  `i64` limits, both overflow directions, a lone minus sign, and float fallback while checking cursor
  parity.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Batch29-30 snapshot `2733` was created by request `3b9f6c5bb61b4d37ab998e819c6a88d4`.
The batched managed validation request `9d7c3bfae2f14d94b6caa0f1498872e1` was rejected before
queueing by the external `E:\Git\zr_vm` dirty-worktree preflight. No validation ticket, Cargo run,
or terminal benchmark result exists for Batch29.

Ownership receipt: exact-path lease request `b4fe3da77f794578b043528c58669d2c`; baseline
attribution request `a11b1de1ead34905ab6eec1b8c6d3b50` (`attributed`).

## Performance contract

The ignored release benchmark parses `i64::MAX` 200,000 times over 11 alternating samples. It
compares the former allocating implementation with checked slice accumulation and requires at least
20% P95 improvement. The production valid-integer path also has a static no-allocation ordering
contract. Terminal nanosecond values must come from the managed Windows receipt before integration,
push, or WeCom reporting.
