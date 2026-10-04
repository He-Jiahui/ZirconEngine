---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/73-runtime-ui-style-theme-token-cascade-selector-pseudo-state-invalidation-transition-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/73/2026-09-15-v2-style-rule-capacity.md
related_records:
  - docs/plans/astra/features/runtime/784-notification-static-key-update.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/v2/style/rule_capacity_tests.rs
tests:
  - zircon_runtime/src/ui/v2/style/rule_capacity_tests.rs
  - tools/tests/test_runtime_v2_style_rule_capacity_performance_contract.py
---

# Runtime785 · V2 style rule capacity

The v2 style resolver now reserves the authored stylesheet rule bound before parsing and sorting
rules. Selector order, specificity precedence, declaration cloning, and invalid-selector
diagnostics remain unchanged.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime73 / v2 style collection | Reserve the summed stylesheet rule count before rule materialization. | TDD source contract GREEN `3/3`; lower order-preservation regression and ignored `RUNTIME785_V2_STYLE_RULE_CAPACITY_BENCH_V1` marker are wired; scoped Rustfmt passes; current non-tooling core batch passes `1963/1963` across `549` modules in `14.682s`; merged recent Runtime/Editor/input/style focus passes `316/316` across `61` modules in `5.877s`; local source/model evidence only. | implemented_pending_validation |

## 性能边界

The collector remains `O(R log R)` because specificity ordering still sorts `R` rules. The change
removes geometric vector growth for the known `R`-rule bound; it does not establish allocator,
CPU/RSS, or product style p50/p95/p99 acceptance.

## 受管验证

This slice joins the owner-attributed Runtime/Editor Windows Release batch. No standalone Cargo
process was started and coordinator state was not queried. Keep `implemented_pending_validation`
until current-source compile, lower Rust regression, Release allocation evidence, and product
percentile evidence arrive; tooling production work remains deferred.
