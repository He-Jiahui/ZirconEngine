---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/10-notification-center-toast-decision-history-actions-retention-accessibility-diagnostic-integration-review.md
  - docs/plans/optimize/zircon_editor/10/2026-09-14-activity-projection-capacity.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/activity/view.rs
  - zircon_editor/src/ui/activity/decision/view.rs
  - zircon_editor/src/ui/activity/mod.rs
tests:
  - zircon_editor/src/ui/activity/capacity_tests.rs
  - tools/tests/test_editor_activity_projection_capacity_performance_contract.py
---

# Editor745 · Activity projection exact capacity

Activity toast, progress, log, and visible decision-option projections now reserve their known
source bounds before iterator materialization. Existing ordering, localization, expiry, record
identity, selection IDs, and empty-input behavior are unchanged. The lower regression models the
allocation-growth reduction and the ignored marker `EDITOR745_ACTIVITY_PROJECTION_CAPACITY_BENCH_V1`
is ready for the next managed Release batch.

## 计划完成列表

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Editor10 / Activity projection | Reserve snapshot, record, and decision-option bounds before append | implemented_pending_validation | RED/GREEN source contract `4/4`; merged Activity/notification contracts pass `31/31`; standalone optimized Rust harness passes the two lower tests and reports `legacy_growth_events=11` versus `optimized_growth_events=0` (`legacy_p95_ns=65800`, `optimized_p95_ns=38100`, 57.9% of legacy versus an informational local 70% guard); merged Runtime/Editor performance contracts pass `1836/1836` in `9.621s` and later `14.186s`; the full non-tooling discovery passes `3585/3585` in `478.170s` and a fresh rerun passes `3585/3585` in `775.037s`; scoped Rustfmt and Wiki validation (`272/272`, zero errors) are present. Managed Cargo/Release and product allocation/latency evidence remain pending. |

## 性能边界

The four projections remain `O(N)` over their bounded input. Exact reservation removes geometric
vector growth but is not a product CPU/RSS/p50/p95/p99 measurement. Any future filtering may leave
spare capacity by design.

## 源码指纹

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/activity/view.rs` | `F7839CF1079D6BF7B88DC4BE6BF2686DF6AE2C28E57DEA6086B1BF8964365349` |
| `zircon_editor/src/ui/activity/decision/view.rs` | `8549AA45C5011296C5780D7C546E93A5E3FAAEF07C152BF5A2B4153A03762CBF` |
| `zircon_editor/src/ui/activity/mod.rs` | `488051A5B7BB557D5E6E8BBE08E6E38FE699402C78E611C3689B075F680007F5` |
| `zircon_editor/src/ui/activity/capacity_tests.rs` | `E1CC87521106087A9D099200CE007A5BA88B63FCDF99AE5D208159FB44245A5E` |
| `tools/tests/test_editor_activity_projection_capacity_performance_contract.py` | `5A2165D71BBA2643D717D0C0E9099976398651DAEAD2BF2A7094F99802B5908F` |

## 受管验证

No direct Cargo command or coordinator status query was run for this slice. It joins the existing
multi-task Runtime/Editor Windows Release lane. Keep this record `implemented_pending_validation`
until that lane supplies current-source compile, behavior, allocation, and percentile evidence.
