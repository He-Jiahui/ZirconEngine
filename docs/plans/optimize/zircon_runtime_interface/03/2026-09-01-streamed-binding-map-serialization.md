record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/binding/model/binding_value/types.rs
related_tests:
  - zircon_runtime_interface/src/ui/binding/model/binding_value/types/map_serialize_performance_tests.rs
  - tools/tests/test_runtime_interface03_binding_map_serialize_performance_contract.py
  - zircon_runtime_interface/src/ui/binding/model/binding_value/types/map_serialize_performance_tests.rs::runtime_interface03_batch38_streamed_binding_map_release_benchmark
---

# Streamed binding map serialization

## Scope

`UiBindingMap` serialization previously collected one borrowed entry wrapper per map item into a
temporary vector before calling the serializer. It now opens an exact-length serde sequence and
serializes each borrowed key/value entry directly. BTree ordering, sequence length, entry field
names, deserialization, and the public wire shape are unchanged.

## Verification

- TDD RED: the focused contract found `collect::<Vec<_>>()` in the production serializer.
- Focused Batch38 static performance contracts after implementation: `2/2` passed.
- Batched static regression: `99/99` passed (`87` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares the streamed serializer with the former allocating serializer
  byte-for-byte for JSON and bincode across 256 ordered entries.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Batch37-39 snapshot `2738` was created by request `5f745c2425c2410a9ba1667effaffbf8`.
The batched managed validation request `278d8e8367c242c492dfd8b62c2e3d56` was rejected before
queueing by the external `E:\Git\zr_vm` dirty-worktree preflight. No validation ticket, Cargo run,
or terminal benchmark result exists for Batch38. This session will not modify that external
repository.

Ownership receipt: exact-path lease request `4556d1dc1b3c427e90708dc6dc1ed39e`; baseline
attribution request `e9800f33ba8c4603a9edf20ea1a155bf` (`attributed`).

## Performance contract

The ignored release benchmark projects 256 ordered entries 50,000 times over 11 alternating
samples. It compares the former intermediate-vector entry pass with direct streaming and requires
at least 50% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.
