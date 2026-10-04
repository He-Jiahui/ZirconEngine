---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-runtime568-animation-channel-projection-single-dispatch.md
related_records:
  - docs/plans/astra/features/runtime/902-runtime567-hermite-basis-reuse.md
  - docs/plans/astra/features/runtime/904-runtime569-mesh-sdf-source-hash-batching.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/animation/sequence/conversion.rs
tests:
  - zircon_runtime/src/animation/sequence/conversion.rs
---

# Runtime903 Runtime568 Animation-Channel Projection Single Dispatch

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Animation sequence projection | Share one top-level enum dispatch for finite validation and scene-property projection; vector components scan once while quaternion checks retain typed errors and sample-kind labels. | Behavior contract covers all value variants, non-finite Vec3, and zero-length quaternion errors. |
| 性能门禁 | 20,000,000 mixed Scalar/Vec3/Vec4/Quaternion projections avoid repeated matches; managed gate requires at least 15% P95 improvement. | ignored marker `RUNTIME568_CHANNEL_PROJECTION_BENCH_V1`; standalone calibration measured 31.45% reduction. |

- `conversion.rs` contains the Runtime568 behavior/source contracts and marker.
- No tooling changes; standalone calibration is not treated as managed acceptance evidence.

### Grouped validation submission (2026-09-25)

Runtime568 is included in the shared `20260831f` batch covering Runtime565–570:
Runtime development PTY `34078`, Editor development PTY `73116`, Runtime02
Release PTY `31967`, and Editor Release PTY `1779`. All wrappers remain
intentionally unpolled and managed compiler/P95 receipts are pending.
