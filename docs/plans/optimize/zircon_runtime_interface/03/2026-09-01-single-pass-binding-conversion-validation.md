record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/binding/model/conversion.rs
related_tests:
  - zircon_runtime_interface/src/ui/binding/model/conversion/validation_performance_tests.rs
  - tools/tests/test_runtime_interface03_binding_conversion_validation_performance_contract.py
  - zircon_runtime_interface/src/ui/binding/model/conversion/validation_performance_tests.rs::runtime_interface03_batch55_61_single_pass_conversion_id_release_benchmark
---

# Single-pass binding conversion validation

## Scope

`UiBindingConversionId` validation previously scanned the identifier once with `split('.')` to
find empty segments and then scanned it again with `char_indices()` to validate characters. The
validator now performs one character scan, detects empty segments at separators, records the first
invalid character, and reports it after segment validation so the existing error precedence is
unchanged. Empty, over-limit, Unicode, separator, and punctuation behavior remain covered by the
former two-pass oracle.

## Verification

- TDD RED: the focused contract found the old `split('.')` two-pass implementation.
- Focused static conversion performance contract: `2/2` passed.
- Rust behavior coverage compares the single-pass validator and former two-pass implementation,
  including empty segments, invalid characters, Unicode, length limits, and precedence.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.
- Current Batch55-61 submission `runtime-interface03-batch55-61-20260901-r1` was rejected before
  ticket creation by `validation_ticket_external_worktree_dirty` for external worktree
  `E:\\Git\\zr_vm`; no Cargo, terminal performance, commit, or push evidence exists.

Ownership receipt: exact-path lease request `de377b057d204d12890534015bff8425`; baseline
attribution request `fd5e904cc67246208df0088260557a27` (`attributed`).

## Performance contract

The ignored release benchmark validates a representative conversion identifier 250,000 times over
11 alternating samples. It compares the former split-plus-character scan with the single-pass
validator and requires at least 20% P95 improvement. Terminal nanosecond values must come from the
managed Windows receipt before integration, push, or WeCom reporting.
