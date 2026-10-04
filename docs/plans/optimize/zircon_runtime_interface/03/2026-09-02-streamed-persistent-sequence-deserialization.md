---
record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-02
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/persistent_sequence.rs
related_tests:
  - zircon_runtime_interface/src/ui/surface/persistent_sequence/deserialization_performance_tests.rs
  - tools/tests/test_runtime_interface03_persistent_sequence_deserialization_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/persistent_sequence/deserialization_performance_tests.rs::runtime_interface03_batch64_65_streamed_persistent_deserialize_release_benchmark
---

# Streamed persistent-sequence deserialization

## Scope

`UiPersistentSequence` deserialization previously decoded the complete flat sequence into a
temporary `Vec<T>`, then moved every value into fixed persistent leaf segments. The custom serde
visitor now reads `SeqAccess` directly into final 64-item leaves and reserves the leaf directory
from the decoder's size hint.

The public flat sequence wire shape, item order, segment boundaries, directory depth/count,
indexed access, and decode error propagation remain unchanged. Only the full-sequence temporary
buffer and second value pass are removed.

## Verification

- TDD RED: the focused static contract found `Vec::<T>::deserialize(...).map(Into::into)` and the
  missing streamed behavior/benchmark module.
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
`41AE9254BF3DE864A42ED8E78683F9364E4F6937C879F269C656A57BFD10B7F8`, static guard
`C6F75317D34698A57E426DE1877DACE5EB2F4775D0F8B29F35A76A2A6CF7DE78`. Initial exact-path lease
request: `18c1f0fe1d904f6fad5ff1bef1bf2db0`.

## Performance contract

The ignored release benchmark deserializes four 262,144-item serde sequences per sample over 11
alternating samples. It compares the former full-vector decoder with direct leaf construction and
requires at least 10% P95 improvement. Terminal P50/P95 nanosecond values must come from the
managed Windows receipt before integration, push, or WeCom reporting.
