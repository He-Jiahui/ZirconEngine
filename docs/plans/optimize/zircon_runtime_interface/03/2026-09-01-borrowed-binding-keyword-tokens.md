record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
related_tests:
  - tools/tests/test_runtime_interface03_binding_keyword_token_performance_contract.py
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs::runtime_interface03_batch27_borrowed_keyword_token_release_benchmark
---

# Borrowed binding keyword tokens

## Scope

The tokenizer previously collected every identifier into a `String` before recognizing the stable
`true`, `false`, and `null` literals, immediately discarding that allocation for keyword tokens.
It now matches those three exact ASCII character slices first and allocates only ordinary
identifiers. Case sensitivity, identifier scanning, fallback text, and token values are unchanged.

## Verification

- TDD RED: the focused contract found allocation before literal dispatch and no benchmark.
- Focused Batch25-28 static performance contracts after implementation: `8/8` passed.
- Batched static regression: `77/77` passed (`65` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares borrowed and former allocating paths for all keywords, casing,
  keyword prefixes, ordinary identifiers, and punctuation fallback.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Managed submission remains blocked before queueing by the external `E:\Git\zr_vm` dirty-worktree
preflight. No ticket or terminal benchmark result exists.

Ownership receipt: exact-path lease request `0dba7dc54c8c4755bd124abbdf6a7326`; baseline
attribution request `4b36b693646d4358955f37e4534d0cfc` (`attributed`).

## Performance contract

The ignored release benchmark recognizes `false` 200,000 times over 11 alternating samples. It
compares the former allocating dispatch with borrowed slice dispatch and requires at least 50% P95
improvement. Terminal nanosecond values must come from the managed Windows receipt before
integration, push, or WeCom reporting.
