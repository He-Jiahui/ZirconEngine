---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/579/2026-08-31-shader-stage-preflight.md
related_records:
  - docs/plans/astra/features/editor/949-editor579-origin-axis-borrowed-defaults.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/assets/shader/entry_point.rs
tests:
  - zircon_runtime/src/asset/assets/shader/entry_point.rs
---

# Runtime892 Runtime579 Shader-Stage Preflight

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Shader entry-point projection | Resolve and validate the authored stage before cloning the entry-point name; valid descriptors and aliases stay unchanged while rejected metadata avoids discarded allocation. | Behavior contract compares valid aliases and invalid stages with the legacy construction order. |
| 性能门禁 | 250,000 rejected descriptors per sample avoid copying a 1,600-byte entry name. | ignored marker `RUNTIME579_SHADER_STAGE_PREFLIGHT_BENCH_V1` requires optimized P95 ≤50% of legacy; managed Runtime Release receipt remains pending. |

- `entry_point.rs` contains the Runtime579 behavior contract and marker.
- No tooling changes; the combined Runtime579/Editor579 release gate remains pending.

### Grouped validation submission (2026-09-25)

Runtime579 is included in the exact `optimization_batch_gx` replacement wave:
Runtime development PTY `48108`, Editor development PTY `74679`, Runtime02
Release PTY `71568`, and Editor Release PTY `5702`. These wrappers remain
intentionally unpolled; compiler, functional-test, and P95 receipts are pending.
