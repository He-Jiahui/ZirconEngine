record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs
related_tests:
  - tools/tests/test_runtime_interface03_unicode_escape_stack_performance_contract.py
  - zircon_runtime_interface/src/ui/template/asset/binding/expression.rs::runtime_interface03_batch26_stack_unicode_escape_release_benchmark
---

# Stack Unicode escape decode

## Scope

Every binding-string `\uXXXX` escape previously collected four `char` values into a temporary
heap `String` before calling `u32::from_str_radix`. The decoder now copies the same four ASCII
bytes into a fixed stack array and calls the same radix parser. Index advancement, accepted radix
syntax, invalid-input behavior, and Unicode scalar validation are unchanged.

## Verification

- TDD RED: the focused contract found `collect::<String>()` in the production decoder and no
  release benchmark.
- Focused Batch25/26 static performance contracts after implementation: `4/4` passed.
- Batched static regression: `73/73` passed (`61` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Latest batched static regression after Batch27/28: `77/77` passed (`65 + 9 + 3`).
- Rust behavior coverage compares stack and former allocating decoders for boundary scalars,
  malformed digits, non-ASCII input, and the radix parser's signed-looking edge syntax.
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

The ignored release benchmark decodes a four-digit Euro-sign escape 200,000 times over 11
alternating samples. It compares the former heap-allocating decoder with the fixed stack decoder
and requires at least 50% P95 improvement. Terminal nanosecond values must come from the managed
Windows receipt before integration, push, or WeCom reporting.
