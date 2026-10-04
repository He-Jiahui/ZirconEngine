record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/template/asset/style.rs
related_tests:
  - zircon_runtime_interface/src/ui/template/asset/style/selector_capacity_performance_tests.rs
  - tools/tests/test_runtime_interface03_selector_token_capacity_performance_contract.py
  - zircon_runtime_interface/src/ui/template/asset/style/selector_capacity_performance_tests.rs::runtime_interface03_batch43_presized_selector_token_release_benchmark
---

# Presized selector tokens

## Scope

The sliced compound tokenizer still grew its token vector from zero capacity. It now counts the
ASCII `.`, `#`, and `:` delimiters and whether the compound begins with a type selector, then
allocates the exact token capacity for valid inputs. UTF-8 continuation bytes cannot match those
ASCII delimiters. Token values, ordering, invalid-input behavior, and public selector shapes are
unchanged.

## Verification

- TDD RED: the focused contract found `Vec::new()` in the sliced production tokenizer.
- Focused Batch41-43 selector performance contracts after implementation: `6/6` passed.
- Batched static regression: `107/107` passed (`95` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares presized and unpresized sliced tokenizers for type-leading and
  delimiter-leading compounds, every token kind, Unicode, repeated delimiters, and empty input.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Batch41-43 snapshot `2740` was created by request `b6dd89d8f16149c3a72adda3edd5e1c8`.
The batched managed validation request `f4af6fced8284097a8aa75db81acf993` was rejected before
queueing by the external `E:\Git\zr_vm` dirty-worktree preflight. No validation ticket, Cargo run,
or terminal benchmark result exists for Batch43. This session will not modify that external
repository.

Ownership receipt: exact-path lease request `c6b5990ad72d4354b7fdb7a668dda446`; baseline
attribution request `1265883784354be4831578e107572e58` (`attributed`).

## Performance contract

The ignored release benchmark tokenizes a 132-token compound 100,000 times over 11 alternating
samples. It compares zero-capacity growth with delimiter-derived capacity and requires at least 20%
P95 improvement. Terminal nanosecond values must come from the managed Windows receipt before
integration, push, or WeCom reporting.
