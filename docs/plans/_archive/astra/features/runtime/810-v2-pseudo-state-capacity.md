---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/73-runtime-ui-style-theme-token-cascade-selector-pseudo-state-invalidation-transition-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/73/2026-09-19-v2-pseudo-state-capacity.md
related_records:
  - docs/plans/astra/features/runtime/672-single-node-pseudo-state-fast-path.md
  - docs/plans/astra/features/runtime/785-v2-style-rule-capacity.md
implementation_files:
  - zircon_runtime/src/ui/v2/style/runtime_state.rs
tests:
  - zircon_runtime/src/ui/v2/style/runtime_state.rs
  - tools/tests/test_runtime_v2_pseudo_state_capacity_performance_contract.py
---

# Runtime810 · v2 pseudo-state collector capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime73 / static v2 selector path | Reserve the authored props/state lower bound plus painter allowance before pseudo-state alias collection. | TDD source/model contract `3/3`; lower alias/order/capacity regression and ignored `RUNTIME810_V2_PSEUDO_STATE_CAPACITY_BENCH_V1` marker are wired; scoped Rustfmt and Python compilation pass; deterministic `4,097`-slot model removes `12→0` growth events. | implemented_pending_validation |

## 性能边界

This is an allocation-shape optimization for static arena-node selector path
construction. The sort/dedup and painter resolution work remain unchanged; no
claim is made about allocator bytes, CPU, RSS, or product style latency.

## 受管验证

Keep this record at `implemented_pending_validation` until the owner-attributed
Windows Release batch compiles the Runtime UI style module, executes the lower
regression and ignored marker, and supplies allocation plus style p50/p95/p99
evidence. Tooling production work remains deferred.
