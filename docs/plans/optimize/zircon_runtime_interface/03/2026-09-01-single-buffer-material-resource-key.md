record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/brush.rs
related_tests:
  - zircon_runtime_interface/src/ui/surface/render/brush/material_key_performance_tests.rs
  - tools/tests/test_runtime_interface03_material_resource_key_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/brush/material_key_performance_tests.rs::runtime_interface03_batch55_61_single_buffer_material_resource_key_release_benchmark
---

# Single-buffer material resource key

## Scope

`UiMaterialBrushPayload::resource_key` previously sent every variant-qualified material ID through
`format!`. The implementation now allocates the final byte capacity once and appends the material
ID, separator, and variant directly. Unqualified IDs still preserve the former single clone, and
revision, atlas, UV, fallback, empty-string, separator, and Unicode behavior remain unchanged.

## Verification

- TDD RED: the focused contract found the `format!` path and no behavior/benchmark module.
- Focused Batch55-56 static performance contracts after implementation: `4/4` passed.
- The behavior oracle compares direct construction with the former formatting implementation and
  verifies the complete published resource key metadata.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Full-workspace rustfmt remains externally blocked by unrelated Rust-edition parse errors in
  Editor animation and Runtime prepared-geometry files; those paths were not modified.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.
- Batched managed request `runtime-interface03-batch55-56-20260901-r2` was rejected before ticket
  creation or Cargo execution because the external `E:\Git\zr_vm` worktree was dirty. The request
  produced no compile, behavior, benchmark, integration, push, or performance receipt.
- Batched request `runtime-interface03-batch55-59-20260901-r1` was accepted for asynchronous
  reconciliation, but coordinator post-response timed out with request `ed8c6d3ced7d448d9d805f8302709365`;
  no terminal Cargo or benchmark output is available yet.
- Current Batch55-61 submission `runtime-interface03-batch55-61-20260901-r1` was rejected before
  ticket creation by `validation_ticket_external_worktree_dirty` for external worktree
  `E:\\Git\\zr_vm`; no Cargo, terminal performance, commit, or push evidence exists.

## Performance contract

The ignored release benchmark builds a 361-byte qualified material ID 250,000 times over 11
alternating samples. It compares the former formatting path with the exact-capacity buffer and
requires at least 20% P95 improvement. Terminal nanosecond values must come from the managed
Windows receipt before integration, push, or WeCom reporting.
