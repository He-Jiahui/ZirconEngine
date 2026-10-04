---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime560-irradiance-comparison-face-row-traversal.md
related_records:
  - docs/plans/astra/features/runtime/911-runtime559-animation-target-short-segment-hash.md
  - docs/plans/astra/features/runtime/913-runtime561-irradiance-comparison-row-edge-hoist.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/core/framework/render/environment/irradiance_comparison.rs
tests:
  - zircon_runtime/src/core/framework/render/environment/irradiance_comparison.rs
---

# Runtime912 Runtime560 Irradiance-Comparison Face-Row Traversal

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Irradiance comparison traversal | Traverse exact face and row chunks without per-texel modulo/division recovery of face-local coordinates; statistics remain exact. | Behavior contract compares complete-cube statistics with the legacy loop. |
| 性能门禁 | Six-face cube scans avoid repeated texel-index division; standalone calibration measured 63.50% improvement. | marker `RUNTIME560_IRRADIANCE_FACE_ROW_TRAVERSAL_BENCH_V1`; managed Release receipt remains pending. |

- `irradiance_comparison.rs` contains the Runtime560 contract and marker.
- No tooling changes; standalone calibration is not managed acceptance evidence.

### Grouped validation submission (2026-09-25)

Runtime560 is included in the shared `20260830e` batch covering Runtime552–564:
Runtime development PTY `48614`, Editor development PTY `47311`, Runtime02
Release PTY `82681`, and Editor Release PTY `81946`. All wrappers remain
intentionally unpolled and managed compiler/P95 receipts are pending.
