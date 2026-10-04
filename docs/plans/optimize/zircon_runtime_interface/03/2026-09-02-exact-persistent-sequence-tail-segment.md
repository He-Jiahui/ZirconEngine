---
record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-02
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/persistent_sequence.rs
related_tests:
  - zircon_runtime_interface/src/ui/surface/persistent_sequence/tail_segment_performance_tests.rs
  - tools/tests/test_runtime_interface03_persistent_sequence_tail_segment_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/persistent_sequence/tail_segment_performance_tests.rs::runtime_interface03_batch66_67_exact_persistent_tail_release_benchmark
---

# Exact persistent-sequence tail segment

## Scope

Persistent-sequence iterator construction previously reserved 64 items for every leaf, including
a short final leaf whose exact remaining length was already available from an exact iterator.
Converting that oversized `Vec` to `Arc<[T]>` could require avoidable capacity contraction.

The constructor now reserves the exact current-leaf size when the iterator exposes equal lower
and upper bounds. Iterators without an exact bound retain the original 64-item capacity. Item
order, segment boundaries, directory shape, indexing, serde shape, and copy-on-write behavior
remain unchanged.

## Verification

- TDD RED: the focused static contract found the unconditional 64-item segment capacity and the
  missing exact-tail behavior/benchmark module.
- Focused Batch66-67 static performance contracts after implementation: `4/4` passed.
- Complete RuntimeInterface03 static performance-contract discovery: `142/142` passed.
- Scoped Rust 1.94.1 formatting and scoped `git diff --check`: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending a later batched
  coordinator submission; no terminal performance number is claimed yet.
- Batched request `runtime-interface03-batch66-67-20260902-r1` was rejected before ticket creation
  by `validation_ticket_external_worktree_dirty` for external worktree `E:\\Git\\zr_vm`; no
  Cargo, terminal performance, commit, push, or WeCom evidence exists.

Current source/test hashes: `persistent_sequence.rs`
`4B7D5E1B94034DDB0A7DBC49F5A663A5B36938A0CE841C71B72A1AD9975D6893`, Rust behavior/benchmark
`572AC87772F92E100BF61DAA33ACE0674229B8A9BCDF144A8782F4D000535025`, static guard
`60DF5286ADC047E8AE4A7A6F653F950F8081739FD2CB0C8B70D5B1F9D0B5145C`. Initial exact-path lease
request: `d47df01a5de54d018d46b6339cfcaa1c`.

## Performance contract

The ignored release benchmark constructs 50,000 exact-size 65-item sequences per sample over 11
alternating samples. It compares a fixed 64-item tail allocation with exact tail capacity and
requires at least 10% P95 improvement. Terminal P50/P95 nanosecond values must come from the
managed Windows receipt before integration, push, or WeCom reporting.
