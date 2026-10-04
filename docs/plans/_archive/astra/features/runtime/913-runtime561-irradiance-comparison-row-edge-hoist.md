---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime561-irradiance-comparison-row-edge-hoist.md
related_records:
  - docs/plans/astra/features/runtime/912-runtime560-irradiance-comparison-face-row-traversal.md
  - docs/plans/astra/features/runtime/914-runtime563-bake-artifact-hash-header-batch.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/core/framework/render/environment/irradiance_comparison.rs
tests:
  - zircon_runtime/src/core/framework/render/environment/irradiance_comparison.rs
---

# Runtime913 Runtime561 Irradiance-Comparison Row-Edge Hoist

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Irradiance row edge classification | Compute `row_is_edge` once per row and reuse it for every texel, preserving edge sample counts and all error accumulators. | Behavior contract compares complete statistics with the legacy inner-loop checks. |
| 性能门禁 | Inner-loop edge checks fall from per-texel evaluation to per-row evaluation; standalone calibration measured 31.45% improvement. | marker `RUNTIME561_IRRADIANCE_ROW_EDGE_HOIST_BENCH_V1`; managed Release receipt remains pending. |

- `irradiance_comparison.rs` contains the Runtime561 contract and marker.
- No tooling changes; standalone calibration is not managed acceptance evidence.

### Grouped validation submission (2026-09-25)

Runtime561 is included in the shared `20260830e` batch covering Runtime552–564:
Runtime development PTY `48614`, Editor development PTY `47311`, Runtime02
Release PTY `82681`, and Editor Release PTY `81946`. All wrappers remain
intentionally unpolled and managed compiler/P95 receipts are pending.
