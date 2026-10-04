record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs
related_tests:
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program/payload_bitmap_performance_tests.rs
  - tools/tests/test_runtime_interface03_binding_payload_bitmap_performance_contract.py
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program/payload_bitmap_performance_tests.rs::runtime_interface03_batch31_reused_payload_field_bitmap_release_benchmark
---

# Reused binding payload bitmap

## Scope

Compiled binding-program validation previously allocated and zeroed a property-sized `Vec<bool>`
for every binding. The validator now allocates one bitmap before the binding loop and clears only
the property positions touched by a successfully validated payload. Invalid payloads still stop the
whole validation immediately, preserving duplicate, bounds, property-name order, finite-value, and
expression checks.

## Verification

- TDD RED: the focused contract found the bitmap allocation inside the binding loop and no release
  benchmark module.
- Focused Batch31 static performance contracts after implementation: `2/2` passed.
- Batched static regression: `83/83` passed (`71` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares reused and former per-binding allocation for repeated valid
  payloads, duplicate fields, out-of-range fields, descending property names, and empty payloads.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

The Batch31-32 snapshot-create call did not return within the 30-second client window. No snapshot
receipt or validation ticket was obtained, so the mutation was not repeated and no validation
status is polled. The preceding managed preflight also remains blocked by the external
`E:\Git\zr_vm` dirty worktree.

Ownership receipt: exact-path lease request `9b26d25856ee4498abb445fca774fece`; baseline
attribution request `e3685c5252614992aaa0f67e9826d388` (`attributed`).

## Performance contract

The ignored release benchmark validates 512 bindings with four payload fields against 4,096
properties, 64 times over 11 alternating samples. It compares the former per-binding bitmap
allocation with bitmap reuse and requires at least 50% P95 improvement. Terminal nanosecond values
must come from the managed Windows receipt before integration, push, or WeCom reporting.
