---
record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-02
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/persistent_sequence.rs
related_tests:
  - zircon_runtime_interface/src/ui/surface/persistent_sequence/directory_performance_tests.rs
  - tools/tests/test_runtime_interface03_persistent_sequence_directory_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/persistent_sequence/directory_performance_tests.rs::runtime_interface03_batch66_67_moved_persistent_directory_release_benchmark
---

# Moved persistent-sequence directory nodes

## Scope

Persistent-sequence directory construction previously chunked an owned vector of `Arc` child
nodes, cloned every child reference into a parent, then immediately dropped the old vector. Every
directory level therefore paid for avoidable atomic reference-count increments and decrements.

Directory promotion now consumes the owned child vector and moves each child reference into its
parent. Fanout, child order, directory depth/count, segment identity, flat iteration, indexing,
serde shape, and copy-on-write behavior remain unchanged.

## Verification

- TDD RED: the focused static contract found `chunks(...).children.to_vec()` and the missing
  move-promotion behavior/benchmark module.
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
`8D0C0088E2202E057F6A453EDF2E1CC82B96738EAEA71B708403D5571F461293`, static guard
`DF2946E4967D73D3BAEE8BF29DB649014175CF94A2A9A1F63B66BBB4E2339350`. Initial exact-path lease
request: `e5f85455fe4d48d9a1de75861a2fd23f`.

## Performance contract

The ignored release benchmark promotes 16,384 legal full leaf segments into directories eight
times per sample over 11 alternating samples. It compares cloned child references with owned
child moves and requires at least 10% P95 improvement. Terminal P50/P95 nanosecond values must
come from the managed Windows receipt before integration, push, or WeCom reporting.
