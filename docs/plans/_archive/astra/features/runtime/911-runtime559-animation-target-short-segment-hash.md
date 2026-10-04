---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime559-animation-target-short-segment-hash.md
related_records:
  - docs/plans/astra/features/runtime/910-runtime558-animation-target-fixed-hex-display.md
  - docs/plans/astra/features/runtime/912-runtime560-irradiance-comparison-face-row-traversal.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/core/framework/animation/target_id.rs
tests:
  - zircon_runtime/src/core/framework/animation/target_id.rs
---

# Runtime911 Runtime559 Animation-Target Short-Segment Hash

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Animation target hashing | Frame short segments once within a 64-byte update while retaining the long-segment two-update fallback; empty and boundary values remain byte-identical. | Behavior contract covers empty, 56-byte, 57-byte fallback, and longer segments. |
| 性能门禁 | Four-segment paths reduce segment updates from eight to four; standalone calibration measured 19.68% P95 improvement. | marker `RUNTIME559_SHORT_SEGMENT_HASH_BENCH_V1`; managed Release receipt remains pending. |

- `target_id.rs` contains the Runtime559 contract and marker.
- No tooling changes; standalone calibration is not managed acceptance evidence.

### Grouped validation submission (2026-09-25)

Runtime559 is included in the shared `20260830e` batch covering Runtime552–564:
Runtime development PTY `48614`, Editor development PTY `47311`, Runtime02
Release PTY `82681`, and Editor Release PTY `81946`. All wrappers remain
intentionally unpolled and managed compiler/P95 receipts are pending.
