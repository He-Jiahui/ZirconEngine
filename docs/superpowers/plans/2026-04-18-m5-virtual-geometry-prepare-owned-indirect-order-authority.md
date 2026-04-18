---
related_code:
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/virtual_geometry_cluster_raster_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
implementation_files:
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/virtual_geometry_cluster_raster_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
plan_sources:
  - user: 2026-04-18 继续下一刀
  - user: 2026-04-18 把 renderer-side authoritative ordering 继续下沉到真正的 visibility-owned unified indirect args buffer / GPU compaction
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-visibility-owned-indirect-args-compaction.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - cargo test -p zircon_graphics --offline virtual_geometry_prepare_frame_sorts_unified_indirect_draws_by_submission_authority -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_submission_authority -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_prepare_render -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_gpu -- --nocapture
  - cargo check -p zircon_graphics --lib --offline
doc_type: milestone-detail
---

# M5 Virtual Geometry Prepare-Owned Indirect Order Authority

## Goal

把 unified indirect 的第一份 authoritative ordering 从 renderer mesh-build 再往上提一层，固定在 `VirtualGeometryPrepareFrame::unified_indirect_draws()` 这一层，而不是继续让 renderer 在 `pending_draws` 末端用 full segment key 重新发明第一份顺序。

## Delivered Slice

### 1. `prepare.unified_indirect_draws()` 现在先做 authoritative submission sort

`VirtualGeometryPrepareFrame::unified_indirect_draws()` 不再只是把 `cluster_draw_segments` 机械映射成 indirect draw records。

它现在会在 prepare 层按以下 authority 排序：

- `submission_slot`
- `frontier_rank`
- `entity`
- `cluster_start_ordinal`
- `page_id`
- `cluster_span_count / cluster_total_count`
- `lod_level`
- `lineage_depth`
- `state`

这意味着 renderer 不再需要负责“第一份排序”；renderer 只负责消费已经排序好的 prepare-owned indirect draw contract。

### 2. prepare-owned order 继续下沉成 internal `submission_index`

`build_virtual_geometry_cluster_raster_draws(...)` 现在会按 sorted `unified_indirect_draws()` 枚举出 internal `submission_index`，并把它继续带进：

- `VirtualGeometryClusterRasterDraw`
- `VirtualGeometryIndirectSegmentKey`

随后 `build_shared_indirect_args_buffer(...)` 会把这条 `submission_index` 作为真实 segment / draw-ref 排序的首要 key。

因此 renderer 末端虽然仍会做 shared-buffer compaction，但它已经不再负责发明 segment order，只是在消费 prepare-owned order truth。

### 3. 旧的 args compaction 与 draw execution contract 保持兼容

上一轮加的 `(mesh_index_count, segment_key)` compaction 还在。

这轮没有改变：

- 多 primitive draw 共享同一条 indirect args record 的语义
- `draw_count` 与 `args_count` 分离的 stats/readback plumbing
- fallback slot authority、frontier rank、lineage depth 对 cluster-raster consumption 的影响

它只是把“先排顺序，再 compaction”里的“排顺序”前移到 prepare 层。

## Why This Slice Matters

上一轮虽然已经让 args/draw-ref cardinality 开始服从 visibility-owned truth，但第一份 authoritative order 仍然是在 renderer 里根据 pending mesh draws 重建出来的。

这会留下一个架构裂缝：

- visibility / prepare 负责 segment ownership
- renderer 仍负责第一份 submission ordering

本轮把这条 ordering authority 提前到 prepare 层之后，下一轮继续下沉到真正 visibility-generated / GPU-generated args source 就更薄了，因为 renderer 已经只剩“消费和执行”，而不再承担第一份排序生成。

## Validation Summary

- red -> green
  - `virtual_geometry_prepare_frame_sorts_unified_indirect_draws_by_submission_authority`
- regressions
  - `virtual_geometry_runtime`
  - `virtual_geometry_unified_indirect`
  - `virtual_geometry_submission_authority`
  - `virtual_geometry_prepare_render`
  - `virtual_geometry_gpu`
  - `cargo check -p zircon_graphics --lib --offline`

## Remaining Route

- 把当前 prepare-owned ordering + renderer-side compaction 继续下沉到真正的 visibility-owned / GPU-generated unified indirect args source
- 或切回 split-merge frontier residency cascade，把 final page-table truth 再推进到更完整的 runtime/GPU completion/policy 闭环
