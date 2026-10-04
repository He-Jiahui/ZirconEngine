---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime557-irradiance-hash-batches.md
related_records:
  - docs/plans/astra/features/runtime/908-runtime556-builtin-texture-row-templates.md
  - docs/plans/astra/features/runtime/910-runtime558-animation-target-fixed-hex-display.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/core/framework/render/environment/source_irradiance_cubemap.rs
tests:
  - zircon_runtime/src/core/framework/render/environment/source_irradiance_cubemap.rs
---

# Runtime909 Runtime557 Irradiance Hash Batches

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Irradiance content hashing | Route texel-channel hashing through 64-texel batches, preserving digest bytes while removing per-channel hasher updates. | Behavior contract compares optimized and legacy hash output. |
| 性能门禁 | A 32×32 six-face cube reduces update calls from 18,432 to 96; standalone calibration measured 94.96% improvement. | marker `RUNTIME557_IRRADIANCE_HASH_BATCH_BENCH_V1`; managed Release receipt remains pending. |

- `source_irradiance_cubemap.rs` contains the Runtime557 contract and marker.
- No tooling changes; standalone calibration is not managed acceptance evidence.

### Grouped validation submission (2026-09-25)

Runtime557 is included in the shared `20260830e` batch covering Runtime552–564:
Runtime development PTY `48614`, Editor development PTY `47311`, Runtime02
Release PTY `82681`, and Editor Release PTY `81946`. All wrappers remain
intentionally unpolled and managed compiler/P95 receipts are pending.
