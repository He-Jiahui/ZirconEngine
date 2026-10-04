---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-runtime574-component-table-value-move.md
related_records:
  - docs/plans/astra/features/editor/957-editor574-workbench-binding-key-single-buffer.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/component/catalog/editor_showcase/descriptor_builders.rs
tests:
  - zircon_runtime/src/ui/component/catalog/editor_showcase/descriptor_builders.rs
---

# Runtime899 Runtime574 Component-Table Value Move

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Editor showcase layout tables | Accept fixed arrays by value and move each `toml::Value` into the table instead of cloning borrowed values; keys and serialized values remain unchanged. | Behavior/source contracts cover the four-field scrollable layout table and production clone removal. |
| 性能门禁 | 120,000 table constructions avoid cloned-value insertion. | ignored marker `RUNTIME574_TABLE_VALUE_MOVE_BENCH_V1` requires at least 15% P95 improvement; managed Runtime Release receipt remains pending. |

- `descriptor_builders.rs` contains the Runtime574 behavior/source contracts and marker.
- No tooling changes; standalone calibration is not treated as managed acceptance evidence.

### Grouped validation submission (2026-09-25)

Runtime574 is included in the shared broad `57` batch covering the 571–579
Runtime/Editor records: Runtime development PTY `62085`, Editor development PTY
`29906`, Runtime02 Release PTY `6464`, and Editor Release PTY `16118`. All
wrappers remain intentionally unpolled and managed compiler/P95 receipts are pending.
