---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75/2026-09-15-notification-static-key-update.md
related_records:
  - docs/plans/astra/features/runtime/783-overlay-static-key-update.md
  - docs/plans/astra/features/runtime/707-notification-keyboard-filter-borrowing.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/component/state_reducer/notification_center.rs
  - zircon_runtime/src/ui/component/state_reducer/notification_static_key_tests.rs
tests:
  - zircon_runtime/src/ui/component/state_reducer/notification_static_key_tests.rs
  - tools/tests/test_runtime_notification_static_key_update_performance_contract.py
---

# Runtime784 · Notification static-key update

Notification-center selection, focus, and unread-count publication now update fixed state keys in
place, retaining owned keys only for first insertion. Existing notification ordering, disabled
navigation, reference-source invalidation, and selection semantics are unchanged.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime75 / notification state publication | Replace repeated fixed-key `to_string()` setter calls with an in-place update helper. | TDD source contract GREEN `3/3`; lower key-identity/reference-source regressions and ignored `RUNTIME784_NOTIFICATION_STATIC_KEY_UPDATE_BENCH_V1` marker are wired; scoped Rustfmt passes. The current-source Runtime/Editor performance-contract loader passes `1951/1951` across `544` modules in `8.523s`; the merged focused hot-path set passes `145/145` in `0.150s`; adjacent input/compile-contract probe passes `176/176` across `24` modules; local source/model evidence only. | implemented_pending_validation |

## 性能边界

For already materialized notification state, each fixed-key publication removes one temporary map
key allocation; first insertion keeps the same ownership behavior. Local source/model evidence does
not establish allocator, CPU/RSS, or product notification p50/p95/p99 acceptance.

## 受管验证

This slice joins the owner-attributed Runtime/Editor Windows Release batch. No standalone Cargo
process was started and coordinator state was not queried. Keep `implemented_pending_validation`
until current-source compile, lower Rust regression, Release allocation evidence, and product
percentile evidence arrive; tooling production work remains deferred.
