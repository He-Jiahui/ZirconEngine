---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75/2026-09-14-menu-typeahead-text-normalization.md
related_records:
  - docs/plans/astra/features/runtime/774-text-search-streaming-contains.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu.rs
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/typeahead_text_normalization_tests.rs
tests:
  - zircon_runtime/src/ui/component/state_reducer/keyboard/menu/typeahead_text_normalization_tests.rs
  - tools/tests/test_runtime_menu_typeahead_text_normalization_performance_contract.py
---

# Runtime775 · Menu typeahead text normalization

Typeahead normalization now writes the final lowercase search buffer directly rather than
materializing a filtered temporary string first. Control filtering, Unicode trimming, internal
whitespace, and lowercase expansion behavior remain compatible.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime75 / menu typeahead | Build the retained normalized search string once while tracking only a trailing-whitespace offset. | TDD source contract GREEN `3/3`; focused menu/typeahead/search batch GREEN `21/21` in `0.092s`; lower equivalence regression and ignored `RUNTIME775_MENU_TYPEAHEAD_TEXT_NORMALIZATION_BENCH_V1` marker are wired; current combined Runtime/Editor source-contract batch passes `120/120` in `0.080s`. | implemented_pending_validation |

## 性能边界

The steady-state typeahead path removes one full intermediate text buffer per normalized event.
Local source/model evidence does not establish allocator, CPU/RSS, or product keyboard p50/p95/p99
acceptance.

## 受管验证

This slice joins the existing owner-attributed Runtime/Editor Windows Release batch. No standalone
Cargo process was started and coordinator state was not queried. Keep the status
`implemented_pending_validation` until current-source compile, lower Rust regression, Release
allocation evidence, and product percentile evidence arrive; tooling production work remains
deferred.
