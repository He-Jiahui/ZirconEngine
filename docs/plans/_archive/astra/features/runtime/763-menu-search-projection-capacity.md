---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75/2026-09-14-menu-search-projection-capacity.md
related_records:
  - docs/plans/astra/features/runtime/759-keyboard-option-id-streaming.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu.rs
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/search_capacity_tests.rs
tests:
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/search_capacity_tests.rs
  - tools/tests/test_runtime_menu_search_capacity_performance_contract.py
---

# Runtime763 · Menu-search projection capacity

Menu-search filtered membership and preorder ID projection now reserve each known direct bound
before adding retained IDs. The recursive structure, search output order, deduplication, and focus
behavior remain unchanged; no global pre-count walk was added.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime75 / menu search filter projection | Reserve direct array/flags ID-set bounds and root/direct-child flattened-ID bounds before recursive append. | TDD source contract GREEN `4/4`; lower nested-ID/capacity regression and ignored `RUNTIME763_MENU_SEARCH_PROJECTION_CAPACITY_BENCH_V1` marker; combined Runtime/Editor contract batch passes `58/58` in `0.110s`. | implemented_pending_validation |

## 性能边界

Direct containers avoid modeled geometric growth without adding a second tree traversal. This local
source/model evidence does not establish allocator, CPU/RSS, or product keyboard p50/p95/p99
acceptance.

## 受管验证

This slice joins the existing owner-attributed Runtime/Editor Windows Release batch. No standalone
Cargo process was started and coordinator state was not queried. Keep the status
`implemented_pending_validation` until current-source compile, lower Rust regression, Release
allocation evidence, and product percentile evidence arrive; tooling production work remains
deferred.
