record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/binding/model/binding_value/projection.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_value/types.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_call.rs
related_tests:
  - zircon_runtime_interface/src/ui/binding/model/binding_value/projection/native_repr_performance_tests.rs
  - tools/tests/test_runtime_interface03_native_binding_projection_performance_contract.py
  - zircon_runtime_interface/src/ui/binding/model/binding_value/projection/native_repr_performance_tests.rs::runtime_interface03_batch37_single_buffer_native_binding_release_benchmark
---

# Single-buffer native binding projection

## Scope

Native binding serialization previously built a `String` for every nested value, collected those
strings into temporary vectors, and joined them at each array, record, map, enum, and call level.
The projection now writes recursively into one caller-owned `String`; numeric formatting, string
escaping, BTreeMap ordering, optional values, entity/asset forms, and collection-view field order
are unchanged. The existing owned `String` entry points remain available to callers.

## Verification

- TDD RED: the focused contract found no recursive output-buffer helper and no release benchmark.
- Focused Batch37 static performance contracts after implementation: `2/2` passed.
- Batched RuntimeInterface03 static regression: `83/83` performance contracts passed.
- Behavior coverage compares the single-buffer projection with an allocating oracle across nested
  arrays, records, maps, enums, escaped strings, assets, and entities; call projection is checked
  as well.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Batch37-39 snapshot `2738` was created by request `5f745c2425c2410a9ba1667effaffbf8`.
The batched managed validation request `278d8e8367c242c492dfd8b62c2e3d56` was rejected before
queueing by the external `E:\Git\zr_vm` dirty-worktree preflight. No validation ticket, Cargo run,
or terminal benchmark result exists for Batch37. This session will not modify that external
repository.

Ownership receipt: exact-path lease request `0af1951bf0854d8aa09dc349b75b747f`; baseline
attribution request `694896a977734daf82bc84d3d4cc1436` (`attributed`).

## Performance contract

The ignored release benchmark projects one nested binding call 200,000 times over 11 alternating
samples. It compares the former per-node allocation and join strategy with recursive single-buffer
writing and requires at least 50% P95 improvement. Terminal nanosecond values must come from the
managed Windows receipt before integration, push, or WeCom reporting.
