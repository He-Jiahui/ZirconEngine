---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75/2026-09-14-menu-search-streaming.md
related_records:
  - docs/plans/astra/features/runtime/763-menu-search-projection-capacity.md
  - docs/plans/astra/features/runtime/781-menu-child-values-iterator.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu.rs
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/search_streaming_tests.rs
tests:
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/search_streaming_tests.rs
  - tools/tests/test_runtime_menu_search_streaming_performance_contract.py
---

# Runtime764 · Menu-search streaming

Menu search now streams recursive top-level and descendant options into their retained owner
vectors rather than materializing temporary recursive vectors. Top-level indexes, child order,
identity/label rules, default focus candidates, and retained top-level ID values are unchanged.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime75 / menu search tree construction | Replace recursive array/map `flat_map` output vectors with caller-owned streaming collectors and borrow the top-level ID during descent. | TDD source contract GREEN `4/4`; lower tree-order/focus-index regression and ignored `RUNTIME764_MENU_SEARCH_STREAM_BENCH_V1` marker; original combined Runtime/Editor contract batch passes `58/58` in `0.110s`, with Runtime781's refreshed batch at `120/120` in `0.080s`. | implemented_pending_validation |

## 性能边界

Flat menus avoid recursive intermediate `Vec<MenuSearchOption>` values; nested menus retain one
owner vector per actual parent option. This local source/model evidence does not establish
allocator, CPU/RSS, or product keyboard p50/p95/p99 acceptance.

## 受管验证

This slice joins the existing owner-attributed Runtime/Editor Windows Release batch. No standalone
Cargo process was started and coordinator state was not queried. Keep the status
`implemented_pending_validation` until current-source compile, lower Rust regression, Release
allocation evidence, and product percentile evidence arrive; tooling production work remains
deferred.
