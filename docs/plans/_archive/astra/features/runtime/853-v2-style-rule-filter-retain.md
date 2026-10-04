---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/73/2026-09-20-v2-style-rule-filter-retain.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/v2/style/rule_capacity_tests.rs
tests:
  - tools/tests/test_runtime_v2_style_rule_filter_retain_performance_contract.py
---

# Runtime853 · V2 style-rule filter retain

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| V2 static/runtime style filtering | Replace the second `filter(...).collect()` rule buffer with an in-place pseudo-state `retain`, preserving compiled rule order and state classification. | TDD source/model contract `4/4`; existing V2 style-capacity and pseudo-state contracts join a focused `10/10` batch; lower order/capacity regression and ignored `RUNTIME853_V2_STYLE_RULE_FILTER_RETAIN_BENCH_V1` marker are wired. The deterministic 4,096-rule model changes one filter buffer per build to zero; the current expanded source-contract loader passes `4072/4072` across `962` files in `139.499s` (performance-or-contract filename filter, tooling/export/coordinator excluded). Managed Cargo/Release, allocator, and product style p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Complexity boundary

This slice changes only the temporary rule-filter allocation shape. It does not
alter selector parsing, specificity sorting, token resolution, rule matching,
asset authority, or tooling production.
The shared `style.rs` snapshot also carries the adjacent Runtime785 capacity,
wildcard-selector, and Runtime854 selector-path repairs; this entry attributes
only the Runtime853 retain cutover.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/v2/style.rs` | `C7A9BDABD7E0DA0CC98595662DFD7858CEDB2B6827C323D1339B20174A145DCE` |
| `zircon_runtime/src/ui/v2/style/rule_capacity_tests.rs` | `419BDDD7F4A02FCFBE353DFDF269563B0CE8A21718ACD286D05C987D8DB37DE9` |
| `tools/tests/test_runtime_v2_style_rule_filter_retain_performance_contract.py` | `C179BF71E7C8F7C0F5478E70FEBEDE7156580EA0419892F222DA315E89BE42E5` |

## Managed gate

No managed Windows Cargo/Release validation command is started locally and
coordinator status is not polled. Keep this entry `implemented_pending_validation` until the combined
owner-attributed Windows Release lane proves current-source compilation, lower
test reachability, allocator behavior, and style product p50/p95/p99 evidence.
