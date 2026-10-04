---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/577/2026-08-31-mesh-sdf-dimension-fold.md
related_records:
  - docs/plans/astra/features/editor/951-editor577-progress-role-single-dispatch.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/assets/mesh/mesh_sdf/validate.rs
tests:
  - zircon_runtime/src/asset/assets/mesh/mesh_sdf/validate.rs
---

# Runtime894 Runtime577 Mesh-SDF Dimension Fold

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Mesh SDF validation | Validate each bounded dimension while folding the expected voxel count in one pass; preserve invalid-dimension and voxel-count diagnostics under the 256-per-axis bound. | Behavior contract covers bounds and folded count. |
| 性能门禁 | 2,000,000 valid dimension folds avoid the second dimension scan and checked multiplications. | ignored marker `RUNTIME577_MESH_SDF_DIMENSION_FOLD_BENCH_V1` requires optimized P95 ≤90% of legacy; managed Runtime Release receipt remains pending. |

- `validate.rs` contains the Runtime577 behavior contract and marker.
- No tooling changes; the combined Runtime577/Editor577 release gate remains pending.

### Grouped validation submission (2026-09-25)

Runtime577 is included in the exact `optimization_batch_gv` replacement wave:
Runtime development PTY `82613`, Editor development PTY `39013`, Runtime02
Release PTY `63979`, and Editor Release PTY `29732`. The wrappers remain
intentionally unpolled; compiler, functional-test, and P95 receipts are pending.
