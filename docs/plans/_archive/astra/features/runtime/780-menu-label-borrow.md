---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75/2026-09-15-menu-label-borrow.md
related_records:
  - docs/plans/astra/features/runtime/779-menu-search-filter-accumulator.md
  - docs/plans/astra/features/runtime/764-menu-search-streaming.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu.rs
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/label_borrow_tests.rs
tests:
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/label_borrow_tests.rs
  - tools/tests/test_runtime_menu_label_borrow_performance_contract.py
---

# Runtime780 · Menu label borrow

Menu search-tree label selection now borrows the source value and clones only when a retained option
node needs ownership, preserving multi-ID labels and ID fallback semantics.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime75 / menu search tree | Borrow map labels through lookup and remove the intermediate owned label string. | TDD source contract GREEN `3/3`; combined Runtime/Editor source-contract batch GREEN `120/120` in `0.080s`; lower multi-ID/empty-label fallback regression and ignored `RUNTIME780_MENU_LABEL_BORROW_BENCH_V1` marker are wired. | implemented_pending_validation |

## 性能边界

The path removes one intermediate label allocation per map node while retaining required node-owned
text. Local source/model evidence does not establish allocator, CPU/RSS, or product menu p50/p95/p99
acceptance.

## 受管验证

This slice joins the existing owner-attributed Runtime/Editor Windows Release batch. No standalone
Cargo process was started and coordinator state was not queried. Keep the status
`implemented_pending_validation` until current-source compile, lower Rust regression, Release
allocation evidence, and product percentile evidence arrive; tooling production work remains
deferred.
