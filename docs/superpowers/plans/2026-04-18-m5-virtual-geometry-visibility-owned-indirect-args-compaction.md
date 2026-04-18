---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_args.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_draw_refs.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
plan_sources:
  - user: 2026-04-18 把 renderer-side authoritative ordering 继续下沉到真正的 visibility-owned unified indirect args buffer / GPU compaction
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-authoritative-indirect-submission-order.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_uses_one_visibility_owned_indirect_segment_across_multi_primitive_model -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_authority -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu -- --nocapture
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Visibility-Owned Indirect Args Compaction

## Goal

把上一轮已经落地的 authoritative submission ordering 再往下推进一层，不只让 `segment_buffer`、`draw_ref_buffer` 与 `MeshDraw.indirect_args_offset` 排序正确，还让真正的 unified indirect args/draw-ref 记录数量开始服从 visibility-owned truth，而不再机械地等于 renderer CPU draw 数。

## Non-Goal

- 本轮不把 args buffer ownership 彻底移出 mesh build step。
- 本轮不实现真正 GPU-generated multi-draw compaction 或 visibility 直接编码 args buffer。
- 本轮不改动 `virtual_geometry_indirect_args.wgsl` 的 raster trim 语义；这里只收口 record cardinality 与 runtime plumbing。

## Delivered Slice

### 1. 以 visibility-owned signature 做 unified indirect record compaction

`build_shared_indirect_args_buffer(...)` 现在会先按既有 authoritative submission key 排序，再把拥有相同：

- `mesh_index_count`
- `VirtualGeometryIndirectSegmentKey`

的 pending draw 折叠成一条 shared indirect args / draw-ref 记录。

对多 primitive、同 segment、同 mesh-index-count 的情况，`args_count` 与 draw-ref count 会真实缩小，而不是继续一条 primitive draw 复制一份完全相同的 indirect record。

### 2. 多个 `MeshDraw` 现在可以共享同一条 indirect args record

compaction 后每个 pending draw 仍然保留自己的 `MeshDraw`，但 `indirect_args_offset` 会回填到 compacted record 的偏移，而不是永远等于排序后 draw 序号。

结果是：

- `draw_count` 仍然表示真实 GPU draw call 数
- `args_count` / draw-ref count 现在表示真实 unique indirect record 数
- 多个 primitive draw 可以共享同一个 visibility-owned args entry，同时继续各自绑定自己的 mesh/index buffer 执行

### 3. Runtime stats / readback 不再假定 `args_count == draw_count`

`BuiltMeshDraws` 现在显式携带 `indirect_args_count`，并沿：

- `build_compiled_scene_draws(...)`
- `virtual_geometry_indirect_stats(...)`
- `render_frame_with_pipeline(...)`
- renderer last-state readback

整条链传递。

因此 `read_last_virtual_geometry_indirect_args()` 与 `read_last_virtual_geometry_indirect_draw_refs()` 现在读取的长度来自真实 compacted args count，而不是旧的 per-draw 假设。

## Why This Slice Matters

上一轮已经让 submission order 服从 `submission_slot / frontier_rank / page / lineage` 真值，但真正的 args cardinality 仍然绑在 CPU pending draw 数上。

这会留下一个明显断层：

- 排序是 visibility-owned 的
- 但 args/draw-ref record 个数仍然是 renderer-owned 的

这轮补上之后，renderer-side authoritative ordering 已经进一步下沉成 visibility-owned indirect record compaction，为下一条把 authority 继续移向真正的 visibility-generated / GPU-generated args source 打平了 contract。

## Validation Summary

- red test：`virtual_geometry_prepare_uses_one_visibility_owned_indirect_segment_across_multi_primitive_model`
  - 先证明旧实现仍然生成 2 条 args record
- green + regressions：
  - `virtual_geometry_unified_indirect`
  - `virtual_geometry_submission_authority`
  - `virtual_geometry_prepare_render`
  - `virtual_geometry_runtime`
  - `virtual_geometry_gpu`
  - `cargo check -p zircon_graphics --lib --offline --locked`

## Remaining Route

- 把当前 compaction authority 从 renderer mesh-build 阶段继续下沉到真正的 visibility-owned / GPU-generated unified indirect args buffer
- 继续推进更深的 cluster raster consumption / indirect execution，而不只停在 shared args build
- 或切回 split-merge frontier residency cascade，把同一套 page-table truth 继续推进到更完整的 host/runtime/GPU completion 闭环
