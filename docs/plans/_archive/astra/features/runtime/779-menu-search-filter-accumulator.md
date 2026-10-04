---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75/2026-09-15-menu-search-filter-accumulator.md
related_records:
  - docs/plans/astra/features/runtime/778-menu-search-query-borrow.md
  - docs/plans/astra/features/runtime/764-menu-search-streaming.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu.rs
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/search_filter_accumulator_tests.rs
tests:
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/search_filter_accumulator_tests.rs
  - tools/tests/test_runtime_menu_search_filter_accumulator_performance_contract.py
---

# Runtime779 · Menu search filter accumulator

Recursive menu filtering now reuses one caller-owned result accumulator with checkpoint rollback,
removing per-node child result vectors while preserving preorder IDs, focus order, and unmatched
branch removal.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime75 / menu search filter | Stream recursive matches into one accumulator and roll back unmatched branches by length checkpoint. | TDD source contract GREEN `3/3`; combined Runtime/Editor source-contract batch GREEN `120/120` in `0.080s`; lower preorder/focus-order regression and ignored `RUNTIME779_MENU_SEARCH_FILTER_ACCUMULATOR_BENCH_V1` marker are wired. | implemented_pending_validation |

## 性能边界

The recursive projection removes per-node temporary result containers and retains parent-before-
child output ordering through checkpoints. Local source/model evidence does not establish allocator,
CPU/RSS, or product menu-search p50/p95/p99 acceptance.

## 受管验证

This slice joins the existing owner-attributed Runtime/Editor Windows Release batch. No standalone
Cargo process was started and coordinator state was not queried. Keep the status
`implemented_pending_validation` until current-source compile, lower Rust regression, Release
allocation evidence, and product percentile evidence arrive; tooling production work remains
deferred.
