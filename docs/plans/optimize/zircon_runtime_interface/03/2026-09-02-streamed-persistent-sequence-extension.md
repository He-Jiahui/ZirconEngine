---
record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-02
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/persistent_sequence.rs
related_tests:
  - zircon_runtime_interface/src/ui/surface/persistent_sequence/extend_performance_tests.rs
  - tools/tests/test_runtime_interface03_persistent_sequence_extend_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/persistent_sequence/extend_performance_tests.rs::runtime_interface03_batch64_65_streamed_persistent_extend_release_benchmark
---

# Streamed persistent-sequence extension

## Scope

`UiPersistentSequence::extend` previously cloned all retained items into a whole-sequence `Vec`,
appended new items there, then moved the combined values into persistent leaf segments. Extension
now chains borrowed clones of existing items with the appended iterator and streams the combined
sequence directly into final leaves.

The existing-item clone count, appended-item ownership, flat order, segment boundaries, directory
depth/count, indexed access, serde shape, and copy-on-write behavior remain unchanged. The
optimization removes only the complete intermediate buffer and second value pass.

## Verification

- TDD RED: the focused static contract found `self.to_vec()` in `Extend` and the missing streamed
  behavior/benchmark module.
- Focused Batch64-65 static performance contracts after implementation: `4/4` passed.
- Complete RuntimeInterface03 static performance-contract discovery: `138/138` passed.
- Scoped Rust 1.94.1 formatting and scoped `git diff --check`: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending a later batched
  coordinator submission; no terminal performance number is claimed yet.
- Batched request `runtime-interface03-batch64-65-20260902-r1` was rejected before ticket creation
  by `validation_ticket_external_worktree_dirty` for external worktree `E:\\Git\\zr_vm`; no
  Cargo, terminal performance, commit, push, or WeCom evidence exists.

Current source/test hashes: `persistent_sequence.rs`
`4B7D5E1B94034DDB0A7DBC49F5A663A5B36938A0CE841C71B72A1AD9975D6893`, Rust behavior/benchmark
`EEAE59C234D47E740239CC80C4712BC9F7FA8DB2893C3BC72D589B2022394F03`, static guard
`B64F96180E2F631BD690A4B35AFC71A4CB9F6C659553DE7DCB4722AD218775ED`. Initial exact-path lease
request: `48ba342049f04161a301bb8cabe5699e`.

## Performance contract

The ignored release benchmark appends 4,096 items to a retained 131,072-item sequence eight times
per sample over 11 alternating samples. It compares the former whole-vector buffer with direct
leaf segmentation and requires at least 20% P95 improvement. Terminal P50/P95 nanosecond values
must come from the managed Windows receipt before integration, push, or WeCom reporting.
