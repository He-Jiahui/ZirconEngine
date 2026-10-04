record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs
related_tests:
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program/reference_bitmap_performance_tests.rs
  - tools/tests/test_runtime_interface03_binding_reference_bitmap_performance_contract.py
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program/reference_bitmap_performance_tests.rs::runtime_interface03_batch32_dense_binding_sequence_release_benchmark
---

# Dense binding sequence validation

## Scope

Compiled binding-program validation already requires each referenced binding ID to equal a strictly
increasing expected index. A second binding-sized reference bitmap therefore repeated the same
duplicate, gap, and order proof while allocating and mutating memory. Validation now relies on the
dense sequence invariant and compares the final consumed count with the binding table length to
reject unreferenced trailing bindings.

## Verification

- TDD RED: the focused contract found the redundant `referenced_bindings` allocation and no release
  benchmark module.
- Focused Batch32 static performance contracts after implementation: `2/2` passed.
- Batched static regression: `85/85` passed (`73` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares dense and former bitmap validation for a valid sequence, missing
  tail entries, duplicates, gaps, generation mismatch, and source-index mismatch.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

The Batch31-32 snapshot-create call did not return within the 30-second client window. No snapshot
receipt or validation ticket was obtained, so the mutation was not repeated and no validation
status is polled. The preceding managed preflight also remains blocked by the external
`E:\Git\zr_vm` dirty worktree.

Ownership receipt: exact-path lease request `40ca0857d223497f8fe8f5edefc4905f`; baseline
attribution request `a1f4c6b73ced4124ae73f876681fa62b` (`attributed`).

## Performance contract

The ignored release benchmark validates a dense 128-binding node sequence 10,000 times over 11
alternating samples. It compares the former reference bitmap with the allocation-free dense counter
and requires at least 20% P95 improvement. Terminal nanosecond values must come from the managed
Windows receipt before integration, push, or WeCom reporting.
