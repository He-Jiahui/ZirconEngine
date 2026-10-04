---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/579/2026-08-31-origin-axis-borrowed-defaults.md
related_records:
  - docs/plans/astra/features/runtime/892-runtime579-shader-stage-preflight.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/popup_frame/origin.rs
tests:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/popup_frame/origin.rs
---

# Editor949 Editor579 Origin-Axis Borrowed Defaults

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Popup origin-axis projection | Borrow string attributes and static defaults through `Cow` instead of allocating a `String` for absent axis attributes; authored values, empty fallback, offsets, and placement remain unchanged. | Behavior contract verifies missing attributes return a borrowed default. |
| 性能门禁 | 1,000,000 four-axis projections per sample avoid absent-axis `String` allocations. | ignored marker `EDITOR579_ORIGIN_AXIS_BORROWED_DEFAULTS_BENCH_V1` requires optimized P95 ≤90% of legacy; managed Editor Release receipt remains pending. |

- `origin.rs` contains the Editor579 behavior contract and marker.
- No tooling changes; the combined Runtime579/Editor579 release gate remains pending.

### Grouped validation submission (2026-09-25)

Editor579 is included in the exact `optimization_batch_gx` replacement wave:
Runtime development PTY `48108`, Editor development PTY `74679`, Runtime02
Release PTY `71568`, and Editor Release PTY `5702`. These wrappers remain
intentionally unpolled; compiler, functional-test, and P95 receipts are pending.
