record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
related_tests:
  - tools/tests/test_runtime_interface03_binding_token_admission_performance_contract.py
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs::runtime_interface03_batch25_early_token_admission_release_benchmark
---

# Early binding token admission

## Scope

Binding expression source bytes were bounded before tokenization, but the token-count budget was
checked only after scanning the complete source and growing the token vector. The tokenizer now
checks the stable 2,048-token limit after each admitted token and returns the same typed
`BudgetExceeded { budget: "tokens" }` at the first excess token. Valid parsing, probing, source
normalization, and error types are unchanged.

## Verification

- TDD RED: the focused contract found no in-loop token admission and no release benchmark.
- Focused static performance contract after implementation: `2/2` passed.
- Batched static regression: `73/73` passed (`61` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Latest batched static regression after Batch27/28: `77/77` passed (`65 + 9 + 3`).
- Rust behavior coverage compares the early exit with the former full-scan-then-reject oracle and
  requires the identical typed token-budget error.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Managed submission remains blocked before queueing by the external `E:\Git\zr_vm` dirty-worktree
preflight. No ticket or terminal benchmark result exists.

Ownership receipt: exact-path lease request `ff532612ef9644f3ae5a6272547560d6`; baseline
attribution request `2f693f916da04263817a7f516bc1a715` (`attributed`).

Latest four-task union receipt: lease request `0dba7dc54c8c4755bd124abbdf6a7326`; baseline
attribution request `4b36b693646d4358955f37e4534d0cfc` (`attributed`).

## Performance contract

The ignored release benchmark tokenizes a 16 KiB hostile unary-operator source 32 times over 11
alternating samples. It compares the former complete scan with first-excess admission and requires
at least 50% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.
