---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/578/2026-08-31-overlay-invalidation-single-pass.md
related_records:
  - docs/plans/astra/features/runtime/893-runtime578-pmrem-invalid-length-preflight.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/refresh/overlay_text.rs
tests:
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/refresh/overlay_text.rs
---

# Editor950 Editor578 Overlay Invalidation Single Pass

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Diagnostic overlay text | Format overlay text directly from current refresh/invalidation counters instead of cloning `HostRefreshDiagnostics` to overwrite three fields; startup text and formatted bytes remain unchanged. | Behavior contract compares direct formatting with the legacy clone-and-overlay sequence. |
| 性能门禁 | 131,072 overlay strings per sample avoid the intermediate diagnostics clone. | ignored marker `EDITOR578_OVERLAY_INVALIDATION_SINGLE_PASS_BENCH_V1` requires optimized P95 ≤90% of legacy; managed Editor Release receipt remains pending. |

- `overlay_text.rs` contains the Editor578 behavior contract and marker.
- No tooling changes; the combined Runtime578/Editor578 release gate remains pending.

### Grouped validation submission (2026-09-25)

Editor578 is included in the exact `optimization_batch_gw` replacement wave:
Runtime development PTY `60225`, Editor development PTY `71656`, Runtime02
Release PTY `21156`, and Editor Release PTY `60969`. The wrappers remain
intentionally unpolled; compiler, functional-test, and P95 receipts are pending.
