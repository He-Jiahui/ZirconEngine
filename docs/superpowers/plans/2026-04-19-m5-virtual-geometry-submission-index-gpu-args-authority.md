---
related_code:
  - zircon_asset/src/lib.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_segments.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
implementation_files:
  - zircon_asset/src/lib.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_segments.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
plan_sources:
  - user: 2026-04-19 把 authority 压进更真实的 visibility-owned / GPU-generated args compaction 与 deeper cluster-raster submission ownership
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-authoritative-indirect-submission-order.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_gpu_args_change_when_only_visible_submission_index_changes -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_args_source_authority -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_authority -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render -- --nocapture
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Submission-Index GPU Args Authority

## Goal

把 `VirtualGeometryPrepareFrame::unified_indirect_draws()` 已经固定下来的 prepare/visibility 排序 authority 再向下压一层：不只让 `submission_index` 决定 shared segment / draw-ref / `MeshDraw.indirect_args_offset` 的 CPU 侧顺序，还要让它真实进入 GPU-generated indirect args 与 cluster-raster coverage。

## Delivered Slice

### 1. 新增红灯回归，证明旧路径还停在 CPU order

新增 `virtual_geometry_prepare_gpu_args_change_when_only_visible_submission_index_changes`：

- 可见实体自己的 `page_id / submission_slot / state / frontier_rank / lod_level / lineage_depth` 固定不变
- 只通过一个“不参与真实 pending draw 的 authoritative helper segment”改变目标实体的 `submission_index`
- 断言目标实体的 GPU-generated indirect args 和最终离屏 raster 都必须变化

这条测试在实现前稳定失败，说明旧路径里 `submission_index` 还没有进入真实 GPU args source。

### 2. `submission_index` 现在进入真实 GPU segment payload

`VirtualGeometryIndirectSegmentInput` 新增 `submission_index` 字段，`segment_input(...)` 会把 `VirtualGeometryIndirectSegmentKey.submission_index` 原样写进 shared segment buffer。

这让 renderer 不再只把 `submission_index` 留在 CPU 排序 key 或 `VirtualGeometryClusterRasterDraw` 临时结构里，而是把它变成真实 GPU 侧可消费的数据合同。

### 3. WGSL indirect-args compute 现在消费 `submission_index`

`virtual_geometry_indirect_args.wgsl` 新增 `submission_index_cluster_offset(...)`，在 `cluster_total_count > 2` 且 visible triangle 仍有空间时，把 `submission_index` 映射成额外的 triangle offset。

结果是：

- 同一实体自己的 page/slot/frontier/lod/lineage 不变
- 但 surrounding authoritative segments 改变顺序时
- 真实 `first_index / index_count` 会继续变化

这条 authority 现在不再只停在 CPU `MeshDraw` 排序或 readback 解读层，而是进入真正的 GPU-generated args / cluster-raster consumption。

### 4. GPU readback 同步更新到新的 segment layout

`read_last_virtual_geometry_indirect_segments()` 的 staging/readback layout 已从 `9 * u32` 扩到 `10 * u32`，保持现有测试返回 tuple 形状不变，但能够正确读取带 `submission_index` 的新 segment buffer。

这保证了新的 segment payload 不会让现有 readback/assertion 失真。

### 5. 额外 shared compile closure

当前工作区同时存在 runtime absorption 的并行迁移，`zircon_asset/src/lib.rs` 暂时恢复了 `ProjectManager / ProjectManifest / ProjectPaths` 的 root re-export，以保持 `zircon_graphics`/`zircon_scene`/tests 在验证本 slice 时可继续编译。

这不是新的长期架构方向，而是当前 dirty worktree 下的最小 shared-layer compile closure。

## Why This Slice Matters

上一刀已经把 authority 压进了：

- prepare-owned unified indirect order
- shared indirect segment buffer
- shared draw-ref buffer
- final `MeshDraw.indirect_args_offset`
- `MeshDraw` 实际提交顺序

但仍然留下一个更深的 leak：

- 如果真实 draw 自己的 page/slot/frontier/lod/lineage 不变
- 只是 surrounding authoritative segments 改变顺序
- GPU-generated indirect args 本身仍然可能完全不变

这会让“visibility-owned authority 已经存在”只在 CPU order 层成立，而没有进入更真实的 GPU args compaction / cluster-raster execution。

本轮补完之后，`submission_index` 已经沿着：

- `prepare.unified_indirect_draws()`
- `VirtualGeometryClusterRasterDraw`
- `VirtualGeometryIndirectSegmentKey`
- `VirtualGeometryIndirectSegmentInput`
- `virtual_geometry_indirect_args.wgsl`

形成一条连续真值链。

## Validation Summary

- 红灯
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_gpu_args_change_when_only_visible_submission_index_changes -- --nocapture`
- 绿灯
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_gpu_args_change_when_only_visible_submission_index_changes -- --nocapture`
- 回归
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_args_source_authority -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_authority -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render -- --nocapture`
  - `cargo check -p zircon_graphics --lib --offline --locked`

## Remaining Gaps

- 这条 authority 现在已经进入 GPU-generated args，但仍然是 renderer-local compute 里基于 CPU-prepared segment payload 做的生成；下一刀仍应继续压向更真实的 visibility-owned / GPU-generated args source，而不是长期停在 CPU sort + GPU post-process 混合模型。
- 更深的 cluster-raster / indirect execution ownership 还需要继续把这份 truth 推到更真实的 GPU compaction / indirect source / residency-manager cascade。
