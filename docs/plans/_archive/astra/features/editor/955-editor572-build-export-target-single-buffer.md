---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-editor572-build-export-target-single-buffer.md
related_records:
  - docs/plans/astra/features/runtime/897-runtime572-obj-face-component-single-scan.md
  - docs/plans/astra/features/editor/956-editor573-ui-asset-conflict-buffer-reuse.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export/target_rows/identity.rs
tests:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export/target_rows/identity.rs
---

# Editor955 Editor572 Build-Export Target Single Buffer

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Export target key construction | Normalize the profile directly into one capacity-planned target buffer, preserving lowercasing, separator runs, trailing trimming, and the `target` fallback for standalone and appended keys. | Compatibility/source contracts cover standalone normalization and append-to-existing behavior. |
| 性能门禁 | 200,000 duplicate-platform target constructions avoid the intermediate normalized profile allocation. | ignored marker `EDITOR572_BUILD_EXPORT_TARGET_SINGLE_BUFFER_BENCH_V1` requires at least 25% P95 improvement; managed Editor Release receipt remains pending. |

- `identity.rs` contains the Editor572 behavior/source contracts and marker.
- No tooling changes; this record keeps the managed Release gate pending.

### Grouped validation submission (2026-09-25)

Editor572 is included in the shared broad `57` batch covering the 571–579
Runtime/Editor records: Runtime development PTY `62085`, Editor development PTY
`29906`, Runtime02 Release PTY `6464`, and Editor Release PTY `16118`. All
wrappers remain intentionally unpolled and managed compiler/P95 receipts are pending.
