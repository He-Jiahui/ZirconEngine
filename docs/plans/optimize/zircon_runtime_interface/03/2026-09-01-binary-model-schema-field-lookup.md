record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/binding/model/model_schema.rs
related_tests:
  - zircon_runtime_interface/src/ui/binding/model/model_schema/field_lookup_performance_tests.rs
  - tools/tests/test_runtime_interface03_model_schema_field_lookup_performance_contract.py
  - zircon_runtime_interface/src/ui/binding/model/model_schema/field_lookup_performance_tests.rs::runtime_interface03_batch36_binary_model_field_lookup_release_benchmark
---

# Binary model schema field lookup

## Scope

Model field lookup previously scanned every field for each request. It now uses binary search for
the canonical sorted field layout and falls back to the former linear lookup when externally
constructed or deserialized schemas are not sorted. The fallback preserves the public constructor,
serialized order, missing-field behavior, and compatibility with legacy unsorted schemas.

## Verification

- TDD RED: the focused contract found no binary lookup and no release benchmark module.
- Focused Batch36 static performance contracts after implementation: `2/2` passed.
- Batched static regression: `93/93` passed (`81` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares binary-plus-fallback and former linear lookup for first, middle,
  last, and missing fields in both sorted and deliberately reordered schemas.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Batch35-36 snapshot `2736` was created by request `94eb9072f7694ceba8e25fe145cef819`.
The batched managed validation request `9f1698aab5e74d038a1c2e076251e711` was rejected before
queueing by the external `E:\Git\zr_vm` dirty-worktree preflight. No validation ticket, Cargo run,
or terminal benchmark result exists for Batch36. This session did not modify that external
repository.

Ownership receipt: exact-path lease request `ecdab54d6ee54359aa5d1ab455eaf1c6`; baseline
attribution request `bebaafa992f349379df0e8bb19e33b3c` (`attributed`).

## Performance contract

The ignored release benchmark performs 32 last-field lookups in a sorted 65,536-field schema over
11 alternating samples. It compares the former linear scan with binary lookup and requires at
least 80% P95 improvement. This threshold applies to the sorted canonical layout only; unsorted
compatibility inputs deliberately retain linear behavior. Terminal nanosecond values must come
from the managed Windows receipt before integration, push, or WeCom reporting.
