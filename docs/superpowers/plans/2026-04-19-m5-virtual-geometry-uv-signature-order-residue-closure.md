---
related_code:
  - zircon_graphics/src/scene/resources/gpu_mesh/gpu_mesh_resource.rs
  - zircon_graphics/src/scene/resources/gpu_mesh/gpu_mesh_resource_from_asset.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - docs/assets-and-rendering/render-framework-architecture.md
implementation_files:
  - zircon_graphics/src/scene/resources/gpu_mesh/gpu_mesh_resource_from_asset.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
plan_sources:
  - user: 2026-04-19 继续把 visibility-owned authority 压进更真实的 GPU-generated args source / compaction ownership
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-repeated-draw-ref-gpu-args-authority.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_cluster_raster_output_is_stable_when_same_segment_primitives_only_change_model_enumeration_order_with_distinct_uvs -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry UV-Signature Order Residue Closure

## Goal

收掉 repeated primitive compaction 里最后一类仍会被模型 primitive 枚举顺序污染的 tie-break residue：当两个 same-segment primitive 拥有完全相同的 position/index，但 texcoord 或 normal 不同、且真实输出会因材质/贴图采样而变化时，renderer 不能继续只靠 `pending_draw_index` 或仅 position/index signature 来决定哪条 draw 拿到 compacted indirect slot。

## Delivered Slice

### 1. `indirect_order_signature` 现在覆盖完整顶点负载，而不是只看 position/index

`GpuMeshResource::indirect_order_signature` 的签名源已从：

- `vertex.position`
- `indices`

扩到：

- `vertex.position`
- `vertex.normal`
- `vertex.uv`
- `indices`

这让 repeated primitive compaction 在 same-segment / same-index-count tie 的场景下，不再把“不同 texcoord 但相同几何”误判成完全等价 primitive。

### 2. 新增透明贴图重叠回归，锁定“只换 primitive 枚举顺序”也不能改结果

新增离屏回归：

- `virtual_geometry_prepare_cluster_raster_output_is_stable_when_same_segment_primitives_only_change_model_enumeration_order_with_distinct_uvs`

它构造两份 glTF：

- 相同 positions
- 相同 indices
- 相同 scene / prepare / visibility-owned segment
- 仅 primitive 枚举顺序不同
- 两个 primitive 使用不同 UV 区域采样同一张 split-color texture

在修补前，这条测试会失败，因为 compaction tie-break 会退回 primitive 导入顺序，最终让透明叠加输出漂移。修补后，两份模型输出一致。

### 3. repeated primitive compaction 的 authority 继续从“几何大致相同”收紧到“真实绘制身份”

这条 closure 没有改变 unified indirect 的更高层 contract：

- segment authority 仍然来自 prepare/visibility
- draw-ref compaction 仍然按 same-segment group 发生
- GPU-generated indirect args 仍然是最终 raster source

变化只在于：用于 same-segment tie-break 的 primitive identity 现在终于覆盖会改变实际像素结果的顶点属性，而不是只覆盖几何轮廓。

## Why This Slice Matters

上一刀虽然已经用 `mesh_signature` 收掉了“position/index 不同但 index_count 相同”的 primitive 顺序残留，但那还不是完整 closure。对真实渲染来说：

- texcoord 会改变纹理采样
- normal 会改变光照

如果 signature 不覆盖这些字段，那么 renderer 仍可能在 repeated primitive compaction 里把“视觉上不同的 primitive”当成等价项，继续让导入顺序决定哪条 primitive 获得 compacted draw slot。这会让更深的 GPU-generated args / cluster-raster ownership 看似收敛，实际上还保留一条资产枚举顺序侧漏。

## Validation Summary

- 红:
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_cluster_raster_output_is_stable_when_same_segment_primitives_only_change_model_enumeration_order_with_distinct_uvs -- --nocapture`
- 绿:
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_cluster_raster_output_is_stable_when_same_segment_primitives_only_change_model_enumeration_order_with_distinct_uvs -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining M5 Todo After This Slice

- `Virtual Geometry`: 继续把 visibility-owned truth 压进更真实的 GPU-generated args source / compaction ownership，减少 `build_mesh_draws(...)` 对 CPU submission reconstruction 的依赖。
- `Virtual Geometry`: 继续把 unified indirect authority 下沉到更深的 cluster-raster submission / indirect execution ownership。
- `Virtual Geometry`: 继续推进 deeper residency-manager cascade / page-table / completion / split-merge frontier policy 收敛。
- `Hybrid GI`: 继续推进 scene-driven screen-probe hierarchy gather / request / RT hybrid lighting continuation。
