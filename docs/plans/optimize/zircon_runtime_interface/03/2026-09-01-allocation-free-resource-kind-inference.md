record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_kind.rs
related_tests:
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_kind/performance_tests.rs
  - tools/tests/test_runtime_interface03_resource_kind_inference_performance_contract.py
  - zircon_runtime_interface/src/ui/template/asset/resource_ref/resource_kind/performance_tests.rs::runtime_interface03_batch54_borrowed_resource_kind_release_benchmark
---

# Allocation-free resource kind inference

## Scope

UI resource kind inference previously lowercased the complete path and URI, collected path
segments, and formatted compound names. It now walks path segments from the end by borrowed
slices, compares names and URI extensions with ASCII-insensitive matching, and recognizes the
same compound names without temporary strings. Path precedence, URI query/fragment trimming,
supported extensions, and the generic fallback remain unchanged.

## Verification

- TDD RED: the focused resource-kind contract passed its pre-existing shape assertions but the new
  release benchmark and behavior oracle were absent; implementation then supplied both.
- Focused Batch54 static performance contract after implementation: `2/2` passed.
- Behavior coverage compares borrowed inference with the former allocating implementation across
  compound path names, mixed case, Unicode-adjacent paths, query/fragment URIs, and fallback types.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

## Performance contract

The ignored release benchmark infers one path/URI pair 200,000 times over 11 alternating samples.
It compares the former lowercasing/segment-vector implementation with borrowed inference and
requires at least 20% P95 improvement. Terminal nanosecond values must come from the managed
Windows receipt before integration, push, or WeCom reporting.

Combined Batch53-54 snapshot `2748` was created by request `6218dd60cbea490b9064cb14f11e0666`.
The corrected batched managed validation request `runtime-interface03-batch53-54-20260901-r2`
was rejected before ticket creation by `validation_ticket_external_worktree_dirty` for external
worktree `E:\\Git\\zr_vm`. No Cargo run, commit, push, or terminal performance value exists; the
external worktree remains untouched.
