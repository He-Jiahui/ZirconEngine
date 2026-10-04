---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75/2026-09-15-menu-typeahead-search-projection.md
related_records:
  - docs/plans/astra/features/runtime/776-menu-typeahead-append-reuse.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu.rs
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/typeahead_append_reuse_tests.rs
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/typeahead_search_projection_tests.rs
tests:
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/typeahead_search_projection_tests.rs
  - tools/tests/test_runtime_menu_typeahead_search_projection_performance_contract.py
---

# Runtime777 · Menu typeahead search projection

Menu typeahead candidates now retain one owned buffer for both matching and state publication.
The multi-scalar decision exits after the second Unicode scalar while preserving pasted input,
repeat-key behavior, and primary/fallback search order.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime75 / menu typeahead | Retain one candidate buffer and borrow it for matching; stop multi-scalar detection after the second scalar. | TDD source contract GREEN `4/4`; combined Runtime/Editor source-contract batch GREEN `120/120` in `0.080s`; lower pasted/active/fallback/Unicode-expansion regression and ignored `RUNTIME777_MENU_TYPEAHEAD_SEARCH_PROJECTION_BENCH_V1` marker are wired. | implemented_pending_validation |

## 性能边界

The candidate projection removes its duplicate owned search string and bounds the multi-scalar
decision to two scalars. Local source/model evidence does not establish allocator, CPU/RSS, or
product keyboard p50/p95/p99 acceptance.

## 受管验证

This slice joins the existing owner-attributed Runtime/Editor Windows Release batch. No standalone
Cargo process was started and coordinator state was not queried. Keep the status
`implemented_pending_validation` until current-source compile, lower Rust regressions, Release
allocation evidence, and product percentile evidence arrive; tooling production work remains
deferred.
