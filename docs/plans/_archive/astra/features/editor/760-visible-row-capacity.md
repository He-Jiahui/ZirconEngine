---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/01/2026-09-14-visible-row-capacity.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata.rs
tests:
  - zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata/capacity_tests.rs
  - tools/tests/test_editor_asset_visible_row_capacity_performance_contract.py
---

# Editor760 · AssetContent visible-row bounded capacity

The shared AssetContent visible-group append now reserves the clipped groups'
known `node_rows` bound before extending the fixed-row projection. Clip
intersection, `partition_point` bounds, row order, visible-group counts, and the
final sort remain unchanged. The virtualized content helper is deliberately out
of scope; this is a bounded-capacity follow-up to PERF-MVP-219, not a claim that
the full visible-query allocation/sort target is closed.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor01 / AssetContent visible groups | Reserve the exact clipped `node_rows` bound before non-virtual visible-row append. | TDD RED/GREEN source contract `4/4`; lower Rust capacity/order/clipping regression; ignored `EDITOR760_VISIBLE_GROUP_ROWS_CAPACITY_BENCH_V1` marker; focused capacity batch `21/21` in `0.127s`; merged Runtime/Editor performance contracts `1844/1844` across `515` modules in `47.661s`; scoped Rustfmt/diff. | implemented_pending_validation |

## 性能边界

The append remains linear in the clipped group rows and avoids geometric output
growth on the common bounded path. The deterministic model is an allocation-shape
check only; managed Cargo/Release and product CPU/RSS/p50/p95/p99 evidence remain
required. The broader PERF-MVP-219 goal (zero visible-query result allocation and
sort) remains open for a later typed-range owner.

## 源码指纹

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata.rs` | `5B25032109E13C5E5D9F9393D3D0D410D2975066265C2087249CFDC8D8903E64` |
| `zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata/capacity_tests.rs` | `E37DA45E0489FBE72D864A8DA4E31553113C306DF846F6D018EECD82CD0696A1` |
| `tools/tests/test_editor_asset_visible_row_capacity_performance_contract.py` | `A4E1E4E4922E2ACFD1B94C0AA4DFFC8CA4BC7519D8B6D07128EC5D7694B28B85` |

## 受管验证

This feature joins the existing multi-task Runtime/Editor Windows Release lane.
No per-task Cargo run or coordinator status query was made. Keep the record
`implemented_pending_validation` until that lane supplies current-source compile,
behavior parity, allocation, and percentile evidence. Tooling production changes
remain deferred.
