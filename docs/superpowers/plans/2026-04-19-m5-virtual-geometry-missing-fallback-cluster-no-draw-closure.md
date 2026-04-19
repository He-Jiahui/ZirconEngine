---
related_code:
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
implementation_files:
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
plan_sources:
  - user: 2026-04-19 把 virtual_geometry_cluster_draws authority 继续下沉到更真实的 visibility-owned / GPU-generated args source
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-fallback-unified-indirect-downshift.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-missing-segment-authority-closure.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_missing_fallback_clusters_do_not_emit_zero_count_indirect_records -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_args_source_authority -- --nocapture
doc_type: milestone-detail
---

# M5 Virtual Geometry Missing-Fallback Cluster No-Draw Closure

## Goal

把 synthesized fallback cluster path 里的 `Missing` 真值也彻底收口到 prepare-owned unified indirect authority：一旦 `visible_clusters` 已经明确告诉 renderer “这个 cluster 不应提交”，就不能再在 fallback synthesize 阶段生成零计数或 placeholder indirect record。

## Delivered Slice

### 1. fallback synthesize 现在先过滤 `Missing` clusters

`VirtualGeometryPrepareFrame::unified_indirect_draws()` 的 fallback path 过去虽然会在后续 GPU args 阶段把 `Missing` 变成 `index_count = 0`，但它仍可能为这些 clusters 保留 ghost segment / draw-ref 记录。

现在 fallback synthesize 会先把 `state == Missing` 的 cluster 直接排除出 authoritative draw 列表，因此：

- shared `segment_buffer` 不再出现 page `0` / slot `0` 或其它 ghost fallback segment
- shared `draw_ref_buffer` 不再为 `Missing` fallback cluster 保留占位 record
- GPU-generated indirect args 不再需要依赖 “零计数记录” 才表达 no-draw

### 2. “有 cluster truth 但全部 Missing” 的 entity 现在是 authoritative no-draw

这条收口专门区分了两种情况：

- entity 根本没有任何 cluster truth：仍允许保留迁移期 placeholder fallback
- entity 已经有 cluster truth，但这些 cluster 全部是 `Missing`：现在直接视为 authoritative no-draw，不再复活 placeholder full-mesh fallback

这样 synthesized fallback path 与 explicit segment path 在 “不应提交 draw” 这件事上终于统一成同一条 contract。

## Why This Slice Matters

在更深的 visibility-owned / GPU-generated args source 路线上，ghost indirect record 会制造两个问题：

- renderer 看起来没有真正 obey `Missing`，只是把错误 submission 留到更后面再用 `index_count = 0` 掩盖
- draw-ref / args cardinality 会继续偏离 authoritative segment truth，破坏后续 compaction 与 stats closure

这条 closure 落地后，`Missing` 已经能在 fallback synthesize 阶段直接终止整条 submission path，而不是延迟到 GPU args output 再被动收敛。

## Validation Summary

- focused red/green
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_missing_fallback_clusters_do_not_emit_zero_count_indirect_records -- --nocapture`
- broader regressions
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_args_source_authority -- --nocapture`

