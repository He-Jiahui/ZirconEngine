record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/template/asset/style.rs
related_tests:
  - zircon_runtime_interface/src/ui/template/asset/style/selector_parse_performance_tests.rs
  - tools/tests/test_runtime_interface03_selector_parse_performance_contract.py
  - tools/tests/test_runtime_interface03_selector_tokenization_performance_contract.py
  - zircon_runtime_interface/src/ui/template/asset/style/selector_parse_performance_tests.rs::runtime_interface03_batch42_sliced_selector_compound_release_benchmark
---

# Sliced selector compounds

## Scope

Top-level selector parsing previously accumulated every compound into a temporary `String` before
tokenization. It now advances through the original UTF-8 source with borrowed slices, splits leading
whitespace separately, and passes each compound slice directly to the Batch41 tokenizer. Descendant
and child combinators, whitespace handling, Unicode, trailing combinator errors, and empty selector
errors are unchanged.

## Verification

- TDD RED: the focused contract found `String::new` plus per-character `compound.push` in the
  production selector parser.
- Focused Batch41-42 static performance contracts after implementation: `4/4` passed.
- Batched static regression: `107/107` passed (`95` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- The Batch41 tokenizer guard was updated to delimit its function body at the new adjacent
  `split_whitespace_prefix` helper; its no-char-vector contract remains unchanged.
- Rust behavior coverage compares the sliced parser with the former `Peekable<Chars>` parser for
  child/descendant combinations, Unicode, compact child syntax, leading/trailing whitespace, empty
  input, repeated combinators, and trailing combinators.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Batch41-43 snapshot `2740` was created by request `b6dd89d8f16149c3a72adda3edd5e1c8`.
The batched managed validation request `f4af6fced8284097a8aa75db81acf993` was rejected before
queueing by the external `E:\Git\zr_vm` dirty-worktree preflight. No validation ticket, Cargo run,
or terminal benchmark result exists for Batch42. This session will not modify that external
repository.

Ownership receipt: exact-path lease request `da6de9baaa7043d28ac036ad9677370b`;
shared Batch41 guard lease request `a55d6613ade74666859b9a835a21a959`; final baseline attribution
request `5368bfc9be154788a6309e8c611961dc` (`attributed`).

## Performance contract

The ignored release benchmark parses a 24-segment selector with long compound strings 50,000 times
over 11 alternating samples. It compares temporary compound allocation with source slicing and
requires at least 20% P95 improvement. Terminal nanosecond values must come from the managed Windows
receipt before integration, push, or WeCom reporting.
