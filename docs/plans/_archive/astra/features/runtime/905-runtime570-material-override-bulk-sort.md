---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-runtime570-material-override-bulk-sort.md
related_records:
  - docs/plans/astra/features/runtime/904-runtime569-mesh-sdf-source-hash-batching.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/core/framework/render/renderer_common.rs
tests:
  - zircon_runtime/src/core/framework/render/renderer_common.rs
---

# Runtime905 Runtime570 Material-Override Bulk Sort

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Material override bulk construction | Collect bulk slots once, stable-sort by key, and deduplicate in place while copying later values into retained slots; incremental insert and last-write-wins semantics remain unchanged. | Behavior contract compares unsorted duplicate input with incremental insertion and serde normalization. |
| 性能门禁 | 4,096 reverse-order slots avoid repeated binary-search insertion and suffix shifts; managed gate requires at least 65% P95 improvement. | ignored marker `RUNTIME570_MATERIAL_OVERRIDE_BULK_SORT_BENCH_V1`; standalone calibration measured 99.75% reduction. |

- `renderer_common.rs` contains the Runtime570 behavior/source contracts and marker.
- No tooling changes; standalone calibration is not treated as managed acceptance evidence.

### Grouped validation submission (2026-09-25)

Runtime570 is included in the shared `20260831f` batch covering Runtime565–570:
Runtime development PTY `34078`, Editor development PTY `73116`, Runtime02
Release PTY `31967`, and Editor Release PTY `1779`. All wrappers remain
intentionally unpolled and managed compiler/P95 receipts are pending.
