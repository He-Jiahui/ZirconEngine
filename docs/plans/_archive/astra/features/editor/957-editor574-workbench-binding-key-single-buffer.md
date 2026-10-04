---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-31-editor574-workbench-binding-key-single-buffer.md
related_records:
  - docs/plans/astra/features/runtime/899-runtime574-component-table-value-move.md
  - docs/plans/astra/features/editor/956-editor573-ui-asset-conflict-buffer-reuse.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/install.rs
tests:
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/install.rs
---

# Editor957 Editor574 Workbench-Binding Key Single Buffer

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Workbench extension binding keys | Reserve exact byte length and append view, slash, and control segments directly, preserving empty and normal segment behavior. | Behavior/source contracts cover representative extension bindings and empty segments. |
| 性能门禁 | 500,000 binding keys avoid general `format!` construction. | ignored marker `EDITOR574_BINDING_KEY_SINGLE_BUFFER_BENCH_V1` requires at least 15% P95 improvement; managed Editor Release receipt remains pending. |

- `install.rs` contains the Editor574 behavior/source contracts and marker.
- No tooling changes; managed Release evidence remains pending.

### Grouped validation submission (2026-09-25)

Editor574 is included in the shared broad `57` batch covering the 571–579
Runtime/Editor records: Runtime development PTY `62085`, Editor development PTY
`29906`, Runtime02 Release PTY `6464`, and Editor Release PTY `16118`. All
wrappers remain intentionally unpolled and managed compiler/P95 receipts are pending.
