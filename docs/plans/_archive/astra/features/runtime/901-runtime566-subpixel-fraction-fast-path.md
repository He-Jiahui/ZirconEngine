---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-runtime566-subpixel-fraction-fast-path.md
related_records:
  - docs/plans/astra/features/runtime/900-runtime565-event-sampling-mask.md
  - docs/plans/astra/features/runtime/902-runtime567-hermite-basis-reuse.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/text/atlas/raster_key/tests.rs
tests:
  - zircon_runtime/src/text/atlas/raster_key/tests.rs
---

# Runtime901 Runtime566 Text Subpixel Fraction Fast Path

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Glyph raster key fraction | Finite screen coordinates use `fract()` and add one only for negative fractions, preserving `[0,1)` fractions and all signed subpixel bins. | Behavior contracts compare optimized bins with `rem_euclid` at positive, negative, integer, and boundary inputs. |
| 性能门禁 | 30,000,000 coordinates per sample avoid unconditional `rem_euclid(1.0)`; managed gate requires at least 15% P95 improvement. | ignored marker `RUNTIME566_SUBPIXEL_FRACTION_BENCH_V1`; standalone calibration measured 33.20% reduction. |

- `raster_key/tests.rs` contains the Runtime566 benchmark and signed-fraction contract.
- No tooling changes; standalone calibration is not treated as managed acceptance evidence.

### Grouped validation submission (2026-09-25)

Runtime566 is included in the shared `20260831f` batch covering Runtime565–570:
Runtime development PTY `34078`, Editor development PTY `73116`, Runtime02
Release PTY `31967`, and Editor Release PTY `1779`. All wrappers remain
intentionally unpolled and managed compiler/P95 receipts are pending.
