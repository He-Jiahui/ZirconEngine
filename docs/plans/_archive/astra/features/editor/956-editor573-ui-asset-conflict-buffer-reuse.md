---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-editor573-ui-asset-conflict-buffer-reuse.md
related_records:
  - docs/plans/astra/features/editor/955-editor572-build-export-target-single-buffer.md
  - docs/plans/astra/features/runtime/898-runtime573-feedback-css-color-single-buffer.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/host/ui_asset_promotion.rs
tests:
  - zircon_editor/src/ui/host/ui_asset_promotion.rs
---

# Editor956 Editor573 UI-Asset Conflict Buffer Reuse

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| UI asset conflict resolution | Reuse one capacity-planned asset-ID buffer while probing occupied suffixes, and materialize document/display values only after selecting the first free path; `.zui` placement and unsuffixed behavior remain unchanged. | Compatibility contracts cover unsuffixed, `.zui`, extensionless, document, and display candidates. |
| 性能门禁 | 8,000 scans across 32 occupied suffixes avoid eager three-string allocation; calibration is preliminary. | ignored marker `EDITOR573_DEFERRED_TARGET_ALLOCATIONS_BENCH_V1` requires at least 30% P95 improvement; managed Editor Release receipt remains pending. |

- `ui_asset_promotion.rs` contains the Editor573 behavior/source contracts and marker.
- No tooling changes; standalone calibration is not treated as managed acceptance evidence.

### Grouped validation submission (2026-09-25)

Editor573 is included in the shared broad `57` batch covering the 571–579
Runtime/Editor records: Runtime development PTY `62085`, Editor development PTY
`29906`, Runtime02 Release PTY `6464`, and Editor Release PTY `16118`. All
wrappers remain intentionally unpolled and managed compiler/P95 receipts are pending.
