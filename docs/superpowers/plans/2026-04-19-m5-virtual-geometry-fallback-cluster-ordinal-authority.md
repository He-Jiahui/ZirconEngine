---
related_code:
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
implementation_files:
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
plan_sources:
  - user: 2026-04-19 把 authority 压进更真实的 visibility-owned / GPU-generated args compaction 和更深的 cluster-raster execution
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-fallback-unified-indirect-downshift.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-missing-fallback-cluster-no-draw-closure.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_partial_missing_fallback_clusters_keep_original_cluster_ordinal_in_gpu_args -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_args_source_authority -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
doc_type: milestone-detail
---

# M5 Virtual Geometry Fallback Cluster Ordinal Authority

## Goal

把 synthesized fallback cluster path 的 `cluster_start_ordinal / cluster_total_count` 也变成 visibility-owned truth：submission order 可以重排 draw 的先后，但不能在过滤 `Missing` siblings 之后把 surviving cluster 重新压成新的 coarse ordinal。

## Delivered Slice

### 1. fallback synthesize 现在保留 entity-local cluster ordinal

`VirtualGeometryPrepareFrame::unified_indirect_draws()` 的 fallback path 以前会在过滤 `Missing` clusters 之后重新编号 surviving clusters，于是：

- `cluster_start_ordinal` 变成过滤后的新序号
- `cluster_total_count` 变成 surviving cluster 数

这会让 GPU-generated indirect args 把 surviving cluster 重新扩成更粗的 mesh slice。

现在 fallback synthesize 会先记录每个 entity 的完整 cluster 基数，并保留 entity-local ordinal，再按 authoritative submission key 排序输出 draw。  
结果是：

- draw 的先后顺序仍然服从 `submission_slot / frontier_rank / page`
- 但每条 draw 的 `cluster_start_ordinal / cluster_total_count` 继续服从原始 visibility cluster truth

### 2. partial-missing fallback 不再把 surviving slice 扩回 full-mesh

新的 regression `virtual_geometry_partial_missing_fallback_clusters_keep_original_cluster_ordinal_in_gpu_args` 证明：

- 如果一个 fallback entity 的第一个 cluster 已经 `Missing`
- 第二个 cluster 仍然 `Resident`

那么真实 GPU-submitted segment 会保留 `cluster_start_ordinal = 1, cluster_total_count = 2`，而不是把 surviving cluster 重写成 `ordinal = 0, total = 1`。  
对应的 GPU indirect args 也会只消费 mesh 的后半段，而不是重新扩成 full-mesh draw。

## Why This Slice Matters

这条修正把 “cluster existence truth” 和 “cluster coverage truth” 区分开了：

- `Missing` sibling 可以被过滤掉
- 但 surviving sibling 的 coverage 仍然必须按完整 entity cluster 空间解释

否则 deeper cluster-raster execution 虽然看似已经服从 unified indirect authority，实际仍会因为 fallback reindex 在 GPU args 阶段悄悄扩大 raster coverage。

## Validation Summary

- focused red/green
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_partial_missing_fallback_clusters_keep_original_cluster_ordinal_in_gpu_args -- --nocapture`
- broader regressions
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_args_source_authority -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture`

