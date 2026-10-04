---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75/2026-09-14-menu-typeahead-option-id-borrow.md
related_records:
  - docs/plans/astra/features/runtime/764-menu-search-streaming.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/component/state_reducer/keyboard.rs
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu.rs
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/typeahead_option_id_tests.rs
tests:
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/typeahead_option_id_tests.rs
  - tools/tests/test_runtime_menu_typeahead_option_id_borrow_performance_contract.py
---

# Runtime765 · Menu typeahead option-ID borrow

Menu typeahead now resolves its current index against the existing ordered `OptionEntry` slice,
instead of cloning every option ID into a transient vector. Current-value precedence, first-match
selection, search behavior, focus handling, and output ordering are unchanged.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime75 / menu keyboard typeahead | Borrow `OptionEntry.id` while resolving the current index; remove the per-keystroke cloned ID vector. | TDD source contract GREEN `4/4`; lower current-value precedence regression and ignored `RUNTIME765_MENU_TYPEAHEAD_OPTION_ID_BORROW_BENCH_V1` marker; combined Runtime/Editor contract batch passes `62/62` in `0.061s`. | implemented_pending_validation |

## 性能边界

For `N` parsed options, this removes `N` cloned IDs from the current-index path while retaining the
same ordered first-match lookup. Local source/model evidence does not establish allocator, CPU/RSS,
or product keyboard p50/p95/p99 acceptance.

## 受管验证

This slice joins the existing owner-attributed Runtime/Editor Windows Release batch. No standalone
Cargo process was started and coordinator state was not queried. Keep the status
`implemented_pending_validation` until current-source compile, lower Rust regression, Release
allocation evidence, and product percentile evidence arrive; tooling production work remains
deferred.
