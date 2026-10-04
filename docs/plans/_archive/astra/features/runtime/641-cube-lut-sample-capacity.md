---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/641/2026-09-01-preallocated-cube-lut-samples.md
related_code:
  - zircon_runtime/src/asset/assets/texture/cube_lut.rs
  - zircon_runtime/src/asset/assets/texture/cube_lut/optimization_batch_jb_runtime641_tests.rs
tests:
  - zircon_runtime/src/asset/assets/texture/cube_lut/optimization_batch_jb_runtime641_tests.rs
---

# Cube LUT Sample Capacity

The `.cube` parser now reserves RGBA samples from the validated size declaration, capped by source
length. Sample order, channel conversion, supported-size checks, and malformed-row diagnostics are
unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime641 | Reserve bounded 3D LUT samples after size admission | implemented_pending_validation | Source regression and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Runtime Cargo and Release p50/p95/p99 remain pending. |
