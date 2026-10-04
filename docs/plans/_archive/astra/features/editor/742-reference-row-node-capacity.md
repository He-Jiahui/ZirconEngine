---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/129-editor-search-filter-query-index-result-find-usage-reference-navigation-current-source-review.md
  - docs/plans/optimize/zircon_editor/129/2026-09-13-reference-row-node-capacity.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/layouts/views/asset_reference_rows.rs
tests:
  - zircon_editor/src/ui/layouts/views/asset_reference_rows.rs
  - tools/tests/test_editor_asset_reference_rows_capacity_performance_contract.py
---

# Editor reference-row node capacity reservation

References and Used By refreshes now reserve the known four-node row count
before appending dynamic rows. The prototype reset, row numbering, ordering,
unknown-kind fallback, and empty-state semantics remain unchanged.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor129 / reference row projection | Reserve `4 * references.len()` node slots before row append. | TDD RED/GREEN source-pressure contract `3/3`; lower Rust capacity/order regression; ignored Release marker `EDITOR742_REFERENCE_ROW_CAPACITY_BENCH_V1`; refreshed 555-module/2065-test Runtime/Editor performance-plus-pressure batch in `24.966s`; scoped Rustfmt and Python compilation. | implemented_pending_validation |

## 性能边界

The projection remains `O(R)` for `R` references and still performs the
required row-owned field copies. The deterministic 4,096-row model changes
geometric growth from 11 capacity steps to one upfront admission. It is not a
managed CPU, RSS, or product p50/p95/p99 measurement.

## 受管验证

This feature joins the existing batched Runtime/Editor Windows Release lane;
no per-task Cargo run or coordinator status query was made. The managed gate
remains pending under the external dirty-worktree/static-overlay admission
blocker. Tooling production changes remain deferred.
