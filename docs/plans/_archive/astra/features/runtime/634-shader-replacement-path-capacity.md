---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/634/2026-09-01-preallocated-shader-replacement-paths.md
related_code:
  - zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies/optimization_batch_ix_runtime634_tests.rs
tests:
  - zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies/optimization_batch_ix_runtime634_tests.rs
---

# Shader Replacement Path Capacity

Targeted shader replacement now reserves affected import-path membership from removed shader
counts plus the ready-shader iterator bound. Saturating arithmetic preserves bounded behavior for
unknown iterator upper bounds.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime634 | Reserve affected shader replacement paths before reconciliation | implemented_pending_validation | Source regression and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Runtime Cargo and Release p50/p95/p99 remain pending. |
