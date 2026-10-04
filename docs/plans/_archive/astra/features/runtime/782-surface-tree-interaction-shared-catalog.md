---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75/2026-09-15-surface-tree-interaction-shared-catalog.md
related_records:
  - docs/plans/astra/features/runtime/781-menu-child-values-iterator.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/v2/surface_tree/interaction.rs
  - zircon_runtime/src/ui/v2/surface_tree/interaction/shared_catalog_tests.rs
tests:
  - zircon_runtime/src/ui/v2/surface_tree/interaction/shared_catalog_tests.rs
  - tools/tests/test_runtime_surface_tree_interaction_shared_catalog_performance_contract.py
---

# Runtime782 · Surface-tree interaction shared catalog

The v2 surface-tree interaction inference path now borrows the process-wide immutable showcase
catalog instead of retaining a private owned registry clone. Capability/category/event semantics
are unchanged.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime75 / v2 surface-tree interaction | Replace the local owned `OnceLock` catalog with `editor_showcase_shared()`. | TDD source contract GREEN `3/3`; lower pointer-identity regression and ignored `RUNTIME782_SURFACE_TREE_SHARED_CATALOG_BENCH_V1` marker are wired; scoped Rustfmt passes. The current-source Runtime/Editor loader passes `1948/1948` across `543` modules, and the focused recent set passes `142/142`; these are local source/model receipts. | implemented_pending_validation |

## 性能边界

The path removes the secondary 69-descriptor registry clone and keeps the shared immutable catalog
as the sole lookup authority. Local source/model evidence does not establish allocator, CPU/RSS, or
product interaction p50/p95/p99 acceptance.

## 受管验证

This slice joins the existing owner-attributed Runtime/Editor Windows Release batch. No standalone
Cargo process was started and coordinator state was not queried. Keep the status
`implemented_pending_validation` until current-source compile, lower Rust regression, Release
allocation evidence, and product percentile evidence arrive; tooling production work remains
deferred.
