---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75/2026-09-14-text-search-streaming-prefix.md
related_records:
  - docs/plans/astra/features/runtime/772-toast-borrowed-setting.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/component/state_reducer/text_search.rs
  - zircon_runtime/src/ui/component/state_reducer/text_search/streaming_prefix_tests.rs
tests:
  - zircon_runtime/src/ui/component/state_reducer/text_search/streaming_prefix_tests.rs
  - tools/tests/test_runtime_text_search_streaming_performance_contract.py
---

# Runtime773 · Text-search streaming Unicode prefix

The non-ASCII prefix matcher now consumes Unicode lowercase scalars lazily instead of creating a
full lowercase candidate `String` for every menu/typeahead probe. ASCII fast paths and existing
query semantics are preserved.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime75 / shared text search | Stream Unicode lowercase prefix comparison and retain multi-scalar case mappings without a temporary candidate buffer. | TDD source contract GREEN `4/4`; lower Unicode expansion regression and ignored `RUNTIME773_TEXT_SEARCH_STREAMING_PREFIX_BENCH_V1` marker are wired; current combined Runtime/Editor source-contract batch passes `120/120` in `0.080s`. | implemented_pending_validation |

## 性能边界

The steady-state non-ASCII prefix path removes the known lowercase candidate allocation. Local
source/model evidence does not establish allocator, CPU/RSS, or product menu/typeahead p50/p95/p99
acceptance.

## 受管验证

This slice joins the existing owner-attributed Runtime/Editor Windows Release batch. No standalone
Cargo process was started and coordinator state was not queried. Keep the status
`implemented_pending_validation` until current-source compile, lower Rust regression, Release
allocation evidence, and product percentile evidence arrive; tooling production work remains
deferred.
