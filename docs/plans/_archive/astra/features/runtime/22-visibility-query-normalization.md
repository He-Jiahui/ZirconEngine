---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/213-runtime-visibility-gpu-scene-culling-batching-instancing-hzb-virtual-geometry-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/09b/2026-09-09-static-index-query-candidate-normalization.md
---

# Visibility query candidate normalization

`VisibilityStaticIndex` now keeps its persistent ordered storage but avoids
building a temporary ordered set for each bounded bounds/ray query. Overflow and
cell memberships are collected into one contiguous candidate vector, sorted and
deduplicated once, so the public stable-key order and conservative fallback are
unchanged while query allocation work is reduced.

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime213/09B | static index bounds/ray candidate 收集、跨 cell 去重与顺序保持 | `implemented_pending_validation` | `visibility_static_index_query_normalizes_overlapping_cell_memberships` 与 `optimization_batch_runtime213_static_index_query_uses_vec_normalization` 已落源；Runtime09B/Runtime213 visibility 合同批次、Rust parse-only、scoped diff-check 通过。受管 Windows Cargo、正半径预算 workload 与 release p50/p95/p99 仍待异步验证；外部 `E:/Git/zr_vm` dirty，故暂不封存。 |
