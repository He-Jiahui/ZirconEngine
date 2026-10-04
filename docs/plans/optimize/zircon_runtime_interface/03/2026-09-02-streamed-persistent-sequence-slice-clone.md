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
  - tools/tests/test_runtime_interface03_persistent_sequence_slice_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/persistent_sequence/construction_performance_tests.rs::runtime_interface03_batch62_63_streamed_persistent_slice_release_benchmark
---

# Streamed persistent-sequence slice cloning

## Scope

`UiPersistentSequence::from_slice` previously cloned the complete source slice into a temporary
`Vec`, then moved those cloned items into fixed leaf segments. It now clones each item directly
into its final owned segment through the common streamed constructor.

The item clone count, flat order, segment boundaries, directory depth/count, indexed access,
serde shape, and copy-on-write behavior remain unchanged. The optimization only removes the
whole-sequence temporary allocation and memory pass.

## Verification

- TDD RED: the focused static contract found `items.to_vec()` and no slice behavior/benchmark.
- Focused Batch62-63 static performance contracts after implementation: `4/4` passed.
- Complete RuntimeInterface03 static performance-contract discovery: `134/134` passed.
- Scoped Rust 1.94.1 formatting and scoped `git diff --check`: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending a later batched
  coordinator submission; no terminal performance number is claimed yet.
- Batched request `runtime-interface03-batch62-63-20260902-r1` was rejected before ticket creation
  by `validation_ticket_external_worktree_dirty` for external worktree `E:\\Git\\zr_vm`; no
  Cargo, terminal performance, commit, push, or WeCom evidence exists.

Current shared source/test hashes: `persistent_sequence.rs`
`4B7D5E1B94034DDB0A7DBC49F5A663A5B36938A0CE841C71B72A1AD9975D6893`, Rust behavior/benchmark
`A3A37709E8B658E897934A326E6EAA61F6B2921F7FA9682270AB726333F47C35`, static guard
`3229044A2D889121E2F26F009D5563149F218A252549DBCA6A8959FB4B99457E`. Initial exact-path lease
request: `63e33e6a832d425dbc5b5c0c6e73744a`.

## Performance contract

The ignored release benchmark clones one retained 131,072-item slice into eight sequences per
sample over 11 alternating samples. It compares the former whole-vector buffer with direct leaf
segmentation and requires at least 20% P95 improvement. Terminal P50/P95 nanosecond values must
come from the managed Windows receipt before integration, push, or WeCom reporting.
