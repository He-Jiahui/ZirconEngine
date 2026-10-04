---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-runtime565-event-sampling-power-of-two-mask.md
related_records:
  - docs/plans/astra/features/runtime/901-runtime566-subpixel-fraction-fast-path.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/core/runtime/events/diagnostics.rs
tests:
  - zircon_runtime/src/core/runtime/events/diagnostics.rs
---

# Runtime900 Runtime565 Event-Diagnostics Sampling Mask

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Event diagnostics sampling | Power-of-two intervals use `sample_index & (interval - 1)` while zero remains disabled and arbitrary intervals retain modulo semantics; counters and sample indices remain unchanged. | Behavior contracts compare power-of-two, non-power-of-two, and zero decisions with the legacy predicate. |
| 性能门禁 | 50,000,000 predicate calls per sample use the mask fast path for a runtime-variable interval of 64. | ignored marker `RUNTIME565_EVENT_SAMPLING_MASK_BENCH_V1` requires the managed Release evidence; standalone calibration measured 95.24% reduction. |

- `diagnostics.rs` contains the Runtime565 behavior/source contracts and marker.
- No tooling changes; standalone calibration is not treated as managed acceptance evidence.

### Grouped validation submission (2026-09-25)

Runtime565 is included in the shared `20260831f` batch covering Runtime565–570:
Runtime development PTY `34078`, Editor development PTY `73116`, Runtime02
Release PTY `31967`, and Editor Release PTY `1779`. All wrappers remain
intentionally unpolled and managed compiler/P95 receipts are pending.
