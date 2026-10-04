---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75/2026-09-14-table-borrowed-sort-setting.md
related_records:
  - docs/plans/astra/features/runtime/770-collection-single-resolution.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/component/state_reducer/table.rs
  - zircon_runtime/src/ui/component/state_reducer/table/borrowed_sort_setting_tests.rs
tests:
  - zircon_runtime/src/ui/component/state_reducer/table/borrowed_sort_setting_tests.rs
  - tools/tests/test_runtime_table_borrowed_sort_setting_performance_contract.py
---

# Runtime771 · Table borrowed sort setting

Table sort, mode, and column-width payload reads now borrow String/Enum data through read-only
comparison and projection paths. The sort-direction transition retains its required current-column
clone before mutating the same state map. Alias, ordering, and fallback behavior remain unchanged.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime75 / table sort and column-width events | Propagate borrowed textual payloads and read-only settings through normalization, comparison, and field projection; retain the necessary current-column clone before same-map mutation. | TDD source contract GREEN 4/4; lower sort/width semantic regression and ignored RUNTIME771_TABLE_BORROWED_SORT_SETTING_BENCH_V1 marker; current combined Runtime/Editor contract batch passes 91/91 in 0.415s. | implemented_pending_validation |

## 性能边界

Read-only sort comparison/mode and column-width handling removes transient textual clones; the
sort-direction state-derived column and retained state/sort-model writes still own required
strings. Local source/model evidence does not establish allocator, CPU/RSS, or product
table-latency p50/p95/p99 acceptance.

## 受管验证

This slice joins the existing owner-attributed Runtime/Editor Windows Release batch. No standalone
Cargo process was started and coordinator state was not queried. Keep the status
implemented_pending_validation until current-source compile, lower Rust regression, Release
allocation evidence, and product percentile evidence arrive; tooling production work remains
deferred.
