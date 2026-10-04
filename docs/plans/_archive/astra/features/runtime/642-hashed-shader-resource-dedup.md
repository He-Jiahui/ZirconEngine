---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/642/2026-09-01-hashed-shader-resource-deduplication.md
related_code:
  - zircon_runtime/src/asset/project/shader_resource_records.rs
  - zircon_runtime/src/asset/project/shader_resource_records/optimization_batch_jc_runtime642_tests.rs
tests:
  - zircon_runtime/src/asset/project/shader_resource_records/optimization_batch_jc_runtime642_tests.rs
---

# Hashed Shader Resource Deduplication

Shader resource deduplication now uses input-sized hash indexes for IDs and locators, retaining an
explicit deterministic final sort. Duplicate diagnostics and accepted record order remain
unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime642 | Replace ordered membership probes with bounded hash indexes | implemented_pending_validation | Source/behavior regressions and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Runtime Cargo and Release p50/p95/p99 remain pending. |
