---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-runtime567-hermite-basis-reuse.md
related_records:
  - docs/plans/astra/features/runtime/901-runtime566-subpixel-fraction-fast-path.md
  - docs/plans/astra/features/runtime/903-runtime568-animation-channel-projection-single-dispatch.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/animation/sequence/interpolation.rs
tests:
  - zircon_runtime/src/animation/sequence/interpolation.rs
---

# Runtime902 Runtime567 Animation Hermite Basis Reuse

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Hermite vector sampling | Build one immutable `HermiteBasis` per Vec2/3/4 sample and reuse it across components, preserving arithmetic order; scalar and quaternion branches remain unchanged. | Behavior contract compares shared-basis Vec4 output with the legacy formula at five interpolation points. |
| 性能门禁 | Vec4 basis construction falls from four builds to one per sample; managed gate requires at least 15% P95 improvement. | ignored marker `RUNTIME567_HERMITE_BASIS_BENCH_V1`; standalone calibration measured 28.07% reduction. |

- `interpolation.rs` contains the Runtime567 behavior/source contracts and marker.
- No tooling changes; standalone calibration is not treated as managed acceptance evidence.

### Grouped validation submission (2026-09-25)

Runtime567 is included in the shared `20260831f` batch covering Runtime565–570:
Runtime development PTY `34078`, Editor development PTY `73116`, Runtime02
Release PTY `31967`, and Editor Release PTY `1779`. All wrappers remain
intentionally unpolled and managed compiler/P95 receipts are pending.
