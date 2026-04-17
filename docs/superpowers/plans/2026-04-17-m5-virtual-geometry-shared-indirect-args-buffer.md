---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_graphics/src/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_graphics/src/scene/scene_renderer/deferred/deferred_scene_resources_record_gbuffer_geometry.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/update_stats/mod.rs
  - zircon_render_server/src/types.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_graphics/src/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_graphics/src/scene/scene_renderer/deferred/deferred_scene_resources_record_gbuffer_geometry.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/update_stats/mod.rs
  - zircon_render_server/src/types.rs
plan_sources:
  - user: 2026-04-17 continue the next M5 slice without waiting for confirmation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-indirect-raster-consumption.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-slot-aware-indirect-compaction.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - cargo test -p zircon_graphics virtual_geometry_prepare_reuses_one_shared_indirect_buffer_across_multiple_indirect_draws --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_render --locked
  - cargo test -p zircon_graphics render_server_bridge --locked
  - cargo test -p zircon_graphics virtual_geometry --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Virtual Geometry Shared Indirect Args Buffer

**Goal:** 在已经落地的 slot-aware indirect compaction 之上，把 `Virtual Geometry` fallback raster 的 indirect submission 再推进一层，不再为每条 `MeshDraw` 单独创建一个 `wgpu` indirect args buffer，而是改成同帧共享一份 args buffer，并通过 per-draw offset 在 base/prepass/deferred 三条路径上消费。

**Non-Goal:** 本轮仍然不实现 GPU-generated indirect compaction、multi-draw indirect、visibility-owned indirect buffer、真正的 cluster raster 或更深的 page residency manager。

## Delivered Slice

- `build_mesh_draws(...)` 现在先收集 frame-local pending draw，再一次性编码 `IndexedIndirectArgs[]`，生成单个 shared indirect args buffer。
- `MeshDraw` 不再独占 indirect buffer；它现在只保留：
  - shared `Arc<wgpu::Buffer>`
  - 当前 draw 的 `indirect_args_offset`
- base/prepass/deferred 三条 raster 录制路径不再默认对每个 indirect draw 用 offset `0`，而是显式消费 `MeshDraw.indirect_args_offset`。
- `SceneRenderer` / `RenderServer` stats 现在同时暴露：
  - `last_virtual_geometry_indirect_draw_count`
  - `last_virtual_geometry_indirect_buffer_count`
- `mesh/build_mesh_draws/build/` 与 `submit_frame_extract/update_stats/` 当前也已经拆成 root-only wiring + helper 子模块，shared indirect args 组装与 stats 汇总不再回流成聚合脚本。

## Why This Slice Exists

- 上一轮 slot-aware compaction 已经把相邻 resident segment 压成更少的 indirect draw，但 renderer 仍然维持“每条 draw 一个 args buffer”的局部提交模型。
- 这种模型仍然把 indirect submission 停留在 draw-count 层，缺少更真实的 shared submission contract。
- 把同帧 `IndexedIndirectArgs` 聚合到 shared buffer 后，后续无论继续推进 visibility-owned indirect buffers、GPU-generated compaction，还是更深的 cluster raster，都有了更可信的替换边界。

## Validation Summary

- `virtual_geometry_prepare_reuses_one_shared_indirect_buffer_across_multiple_indirect_draws`
  - 证明同一帧保留两条 indirect draw 时，renderer 仍只复用一份 shared indirect args buffer
- `virtual_geometry_prepare_render`
  - 证明 shared buffer + offset 消费没有破坏已有 slot-aware compaction、fallback filtering、resident-slot tint、segment override 与 streaming coverage
- `render_server_bridge`
  - 证明 shared indirect buffer 统计已经沿 `RenderServer -> RenderStats` 路径向 façade 暴露
- `cargo test -p zircon_graphics virtual_geometry --locked`、`cargo test -p zircon_graphics --lib --locked`、`validate-matrix.ps1 -Package zircon_graphics`
  - 证明改动没有回归当前 `zircon_graphics` 的 M4/M5 主链

## Remaining Route

- visibility-owned unified indirect args buffer，而不是当前 renderer-local build step 聚合
- GPU-generated indirect args compaction / multi-draw indirect
- 更真实的 cluster raster consumption 与 page-table-driven visibility execution
- 与 occlusion、BVH、Hybrid GI scene representation 的更深层联合路径
