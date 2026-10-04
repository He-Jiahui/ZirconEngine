record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
related_tests:
  - zircon_runtime_interface/src/ui/template/asset/binding/expression/parser_token_performance_tests.rs
  - tools/tests/test_runtime_interface03_parser_token_take_performance_contract.py
  - zircon_runtime_interface/src/ui/template/asset/binding/expression/parser_token_performance_tests.rs::runtime_interface03_batch30_parser_token_take_release_benchmark
---

# Moved parser token consumption

## Scope

`Parser::next` previously cloned each token before advancing. Tokens that own identifiers, strings,
or unsupported-operator text therefore allocated a second string even though the parser consumes
its stream monotonically. The parser now moves the token out with `mem::replace` and leaves a private
`Consumed` sentinel in the already-passed slot. It retains the indexed lookahead model and never
exposes or revisits the sentinel.

## Verification

- TDD RED: the focused contract found `.cloned()` in production `Parser::next` and no release
  benchmark module.
- Focused Batch30 static performance contracts after implementation: `2/2` passed.
- Batched static regression: `81/81` passed (`69` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares moved and cloned consumption for owned identifiers, literal text,
  unsupported text, integers, floats, and booleans while checking index and end-of-stream parity.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Batch29-30 snapshot `2733` was created by request `3b9f6c5bb61b4d37ab998e819c6a88d4`.
The batched managed validation request `9d7c3bfae2f14d94b6caa0f1498872e1` was rejected before
queueing by the external `E:\Git\zr_vm` dirty-worktree preflight. No validation ticket, Cargo run,
or terminal benchmark result exists for Batch30.

Ownership receipt: exact-path lease request `6dd7af1571e048719a1d50c1a654bb04`; baseline
attribution request `d3f5e584e8a6423fa445b644a5fc8002` (`attributed`).

## Performance contract

The ignored release benchmark consumes a 512-byte owned identifier 200,000 times over 11
alternating samples. It compares cloned consumption with move consumption and requires at least 50%
P95 improvement. The production method also has a static no-clone contract. Terminal nanosecond
values must come from the managed Windows receipt before integration, push, or WeCom reporting.
