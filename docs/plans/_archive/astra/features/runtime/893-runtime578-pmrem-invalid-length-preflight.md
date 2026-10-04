---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/578/2026-08-31-pmrem-invalid-length-preflight.md
related_records:
  - docs/plans/astra/features/editor/950-editor578-overlay-invalidation-single-pass.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/assets/texture/ibl_pmrem.rs
tests:
  - zircon_runtime/src/asset/assets/texture/ibl_pmrem.rs
---

# Runtime893 Runtime578 PMREM Invalid-Length Preflight

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| PMREM texture construction | Validate selected RGBA16F mip-chain length through the canonical helper before copying the artifact slice into an owned `Vec`; valid payloads and existing errors remain unchanged. | Behavior contract covers exact and truncated lengths. |
| 性能门禁 | 250,000 invalid payload checks per sample avoid copying a 128-face, eight-mip payload. | ignored marker `RUNTIME578_PMREM_INVALID_LENGTH_PREFLIGHT_BENCH_V1` requires optimized P95 ≤90% of legacy; managed Runtime Release receipt remains pending. |

- `ibl_pmrem.rs` contains the Runtime578 behavior contract and marker.
- No tooling changes; the combined Runtime578/Editor578 release gate remains pending.

### Grouped validation submission (2026-09-25)

Runtime578 is included in the exact `optimization_batch_gw` replacement wave:
Runtime development PTY `60225`, Editor development PTY `71656`, Runtime02
Release PTY `21156`, and Editor Release PTY `60969`. The wrappers remain
intentionally unpolled; compiler, functional-test, and P95 receipts are pending.
