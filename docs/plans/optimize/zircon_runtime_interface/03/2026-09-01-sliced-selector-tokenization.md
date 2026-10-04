record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/template/asset/style.rs
related_tests:
  - zircon_runtime_interface/src/ui/template/asset/style/selector_token_performance_tests.rs
  - tools/tests/test_runtime_interface03_selector_tokenization_performance_contract.py
  - zircon_runtime_interface/src/ui/template/asset/style/selector_token_performance_tests.rs::runtime_interface03_batch41_sliced_selector_token_release_benchmark
---

# Sliced selector tokenization

## Scope

Compound selector parsing previously copied the complete UTF-8 input into `Vec<char>` before
scanning and then collected every token from a char slice into another string. The tokenizer now
tracks UTF-8 byte boundaries with `char_indices`, borrows each source slice, and allocates only the
final token string. Type, class, ID, state, host, part, Unicode, and invalid empty-token behavior are
unchanged.

## Verification

- TDD RED: the focused contract found the complete `Vec<char>` materialization in production.
- Focused Batch41 static performance contracts after implementation: `2/2` passed.
- Batched static regression: `103/103` passed (`91` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares sliced and former allocating tokenizers for every token kind,
  Unicode identifiers, empty tokens, invalid empty parts, and repeated delimiters.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Batch41-43 snapshot `2740` was created by request `b6dd89d8f16149c3a72adda3edd5e1c8`.
The batched managed validation request `f4af6fced8284097a8aa75db81acf993` was rejected before
queueing by the external `E:\Git\zr_vm` dirty-worktree preflight. No validation ticket, Cargo run,
or terminal benchmark result exists for Batch41. This session will not modify that external
repository.

Ownership receipt: exact-path lease request `cf337e6b54984990ae5a178c25e15df0`; baseline
attribution request `2e50c905441e44e2b007f2a6d6db53cd` (`attributed`).

## Performance contract

The ignored release benchmark tokenizes one long compound selector with 32 classes 100,000 times
over 11 alternating samples. It compares full char-vector materialization with source slicing and
requires at least 20% P95 improvement. Terminal nanosecond values must come from the managed Windows
receipt before integration, push, or WeCom reporting.
