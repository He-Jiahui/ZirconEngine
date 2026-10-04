---
record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-02
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/persistent_sequence.rs
related_tests:
  - zircon_runtime_interface/src/ui/surface/persistent_sequence/construction_performance_tests.rs
  - tools/tests/test_runtime_interface03_persistent_sequence_construction_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/persistent_sequence/construction_performance_tests.rs::runtime_interface03_batch62_63_streamed_persistent_sequence_release_benchmark
---

# Streamed persistent-sequence construction

## Scope

`UiPersistentSequence::from_iter` previously collected every item into one temporary `Vec` before
moving the same items into fixed 64-item persistent leaf segments. Retained layout, hit-test, and
arranged-frame domains therefore paid for an avoidable full-sequence allocation and memory pass.

Iterator construction now fills each owned leaf segment directly and builds the existing bounded
directory from those segments. `From<Vec<T>>` shares the same constructor. Flat iteration order,
segment boundaries, directory depth/count, indexed access, serde shape, non-`Clone` item support,
and copy-on-write behavior remain unchanged.

## Verification

- TDD RED: the focused static contract found the full `collect::<Vec<_>>()` intermediate and the
  missing behavior/benchmark module.
- Focused static performance contract after implementation: `2/2` passed.
- Complete RuntimeInterface03 static performance-contract discovery: `132/132` passed.
- Scoped Rust 1.94.1 formatting and scoped `git diff --check`: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending a later batched
  coordinator submission; no terminal performance number is claimed yet.
- Batched request `runtime-interface03-batch62-63-20260902-r1` was rejected before ticket creation
  by `validation_ticket_external_worktree_dirty` for external worktree `E:\\Git\\zr_vm`; no
  Cargo, terminal performance, commit, push, or WeCom evidence exists.

Current shared source/test hashes after adjacent Batch63-67 coverage: `persistent_sequence.rs`
`4B7D5E1B94034DDB0A7DBC49F5A663A5B36938A0CE841C71B72A1AD9975D6893`, Rust behavior/benchmark
`A3A37709E8B658E897934A326E6EAA61F6B2921F7FA9682270AB726333F47C35`, static guard
`4BD41A9492042A12AA0E412E15DF661603BD5EA935581BD6CDC554BAECCF458F`. Initial exact-path lease
request: `db7df0f7fc7e4756b0d22744251a0d8b`.

## Performance contract

The ignored release benchmark constructs eight 131,072-item sequences per sample over 11
alternating samples. It compares the former whole-vector buffer with direct leaf segmentation and
requires at least 20% P95 improvement. Terminal P50/P95 nanosecond values must come from the
managed Windows receipt before integration, push, or WeCom reporting.
