---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75/2026-09-15-menu-child-values-iterator.md
related_records:
  - docs/plans/astra/features/runtime/780-menu-label-borrow.md
  - docs/plans/astra/features/runtime/779-menu-search-filter-accumulator.md
  - docs/plans/astra/features/runtime/764-menu-search-streaming.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu.rs
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/child_values_iterator_tests.rs
  - tools/tests/test_runtime_menu_search_streaming_performance_contract.py
tests:
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/child_values_iterator_tests.rs
  - tools/tests/test_runtime_menu_child_values_iterator_performance_contract.py
---

# Runtime781 · Menu child-values iterator

Menu search-tree child traversal now streams the fixed child-property lookup sequence directly
from the source map, removing one temporary `Vec<&UiValue>` per map node while preserving property
order and descendant semantics.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime75 / menu search tree | Replace the per-node borrowed child-value vector with a lazy iterator over the canonical property names. | TDD source contract GREEN `3/3`; combined Runtime/Editor source-contract batch GREEN `120/120` in `0.080s`; lower property-order regression and ignored `RUNTIME781_MENU_CHILD_VALUES_ITERATOR_BENCH_V1` marker are wired. | implemented_pending_validation |

## 性能边界

The projection removes one fixed-property temporary vector per visited map node and retains the
existing child order. Local source/model evidence does not establish allocator, CPU/RSS, or
product menu p50/p95/p99 acceptance.

## 受管验证

This slice joins the existing owner-attributed Runtime/Editor Windows Release batch. No standalone
Cargo process was started and coordinator state was not queried. Keep the status
`implemented_pending_validation` until current-source compile, lower Rust regression, Release
allocation evidence, and product percentile evidence arrive; tooling production work remains
deferred.
