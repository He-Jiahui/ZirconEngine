---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_mesh_draw_build_context.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
plan_sources:
  - user: 2026-04-19 继续把这套 authority 压进更真实的 visibility-owned / GPU-generated args compaction
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-visibility-owned-indirect-args-compaction.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-prepare-owned-args-source-authority.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-fallback-unified-indirect-downshift.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics --offline virtual_geometry_args_source_authority -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_submission_execution_order -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_prepare_render -- --nocapture
  - cargo check -p zircon_graphics --lib --offline
doc_type: milestone-detail
---

# M5 Virtual Geometry Mesh-Build Authority Convergence

## Goal

把 `Virtual Geometry` 在 renderer mesh-build 里的最后一层 CPU fallback duplication 收掉，让 `build_mesh_draws(...)` 和 `extend_pending_draws_for_mesh_instance(...)` 明确只消费 `prepare -> build_virtual_geometry_cluster_raster_draws(...)` 产出的 visibility-owned cluster-raster truth，而不是继续保留一套平行的 fallback segment bookkeeping。

## Delivered Slice

### 1. mesh-build 的 authoritative segment / draw-ref source 现在只来自 `virtual_geometry_cluster_draws`

`build_mesh_draws(...)` 不再另外计算 `authoritative_fallback_segment_keys`，也不再把 fallback entity 的 segment key / draw-ref 再拼接一遍。当前 authoritative source 已经固定为：

- `VirtualGeometryPrepareFrame::unified_indirect_draws()`
- `build_virtual_geometry_cluster_raster_draws(...)`
- `MeshDrawBuildContext.virtual_geometry_cluster_draws`
- shared `segment_buffer / draw_ref_buffer / indirect args`

这让 renderer 末端不再维护一条和 prepare-owned unified indirect 平行的 fallback authority。

### 2. `extend_pending_draws_for_mesh_instance(...)` 现在只认 prepare-owned cluster-raster truth

当 `Virtual Geometry` feature 开启时：

- 如果 entity 已经有 `cluster_raster_draws`，pending draw 会直接按这些 draw 生成 `indirect_draw_ref`
- 如果没有 authoritative cluster-raster truth，函数现在直接把它视为 authoritative no-draw，不再补任何 renderer 私造的 full-mesh fallback draw

也就是说，missing-segment fallback 的正常路径已经完全前移到 prepare/unified-indirect 主链；renderer mesh-build 不再负责为这类 entity 再扩一遍 fallback segment slices，也不再保留所谓的 “last-ditch full-mesh fallback” 作为常驻兜底语义。

### 3. mixed explicit + fallback regression 现在显式锁定 “不重复 authority record”

新增的 `virtual_geometry_mixed_explicit_and_fallback_entities_reuse_one_prepare_owned_args_source` 覆盖了这条更窄的混合场景：

- 一个 entity 走显式 `cluster_draw_segments`
- 另一个 entity 没有显式 segments，只走 synthesized fallback slices

回归要求 shared indirect segment / draw-ref / args 只保留 prepare-owned 两条记录，不允许 mesh-build 再追加幽灵 fallback-only record。

## Why This Slice Matters

上一刀已经把 missing-segment fallback slices 前移到 `VirtualGeometryPrepareFrame::unified_indirect_draws()`；但 mesh-build 内部还保留着一套旧的 fallback bookkeeping 和参数入口，看起来像是 renderer 末端仍然拥有第二份 authority。

这会带来两个问题：

- 代码结构上继续误导后续 M5 工作，让人误以为 mesh-build 还能独立补全 fallback truth
- 更深的 GPU-generated args compaction / cluster-raster submission ownership 往下推进时，仍然可能被这条死路径重新分叉

本轮收掉它之后，接下来的主链更清晰：

- 往下继续把 `virtual_geometry_cluster_draws` 这份 authority 压进更真实的 GPU-generated args source / cluster-raster execution
- 或切回 runtime-side deeper residency-manager / split-merge frontier cascade

## Validation Summary

- focused regressions
  - `cargo test -p zircon_graphics --offline virtual_geometry_args_source_authority -- --nocapture`
  - `cargo test -p zircon_graphics --offline virtual_geometry_unified_indirect -- --nocapture`
  - `cargo test -p zircon_graphics --offline virtual_geometry_submission_execution_order -- --nocapture`
  - `cargo test -p zircon_graphics --offline virtual_geometry_prepare_render -- --nocapture`
  - `cargo check -p zircon_graphics --lib --offline`
