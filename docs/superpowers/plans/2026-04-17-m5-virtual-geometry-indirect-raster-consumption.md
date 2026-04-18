---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_graphics/src/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_graphics/src/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_new/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - docs/assets-and-rendering/index.md
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_graphics/src/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_graphics/src/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_new/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/mod.rs
plan_sources:
  - user: 2026-04-17 continue next task after Virtual Geometry prepare segment consumption
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-prepare-consumption.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-gpu-uploader-readback.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-cluster-refine.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics virtual_geometry_prepare_segments_submit_indirect_raster_draws_when_feature_enabled --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_render --locked
  - cargo test -p zircon_graphics virtual_geometry --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Virtual Geometry Indirect Raster Consumption

**Goal:** 在已有 `Virtual Geometry` preprocess、runtime host、GPU completion、hierarchy refine、prepare-segment contract 之上，把当前 fallback raster 从直接 `draw_indexed(...)` 提交推进到真正的 renderer-local `draw_indexed_indirect(...)` 基线。

**Non-Goal:** 本轮仍然不实现 Nanite-like cluster raster、GPU-generated indirect command compaction、multi-draw batching、page streaming residency manager 或 unified visibility-owned indirect args buffer。

## Delivered Slice

- `build_mesh_draws(...)` 现在会在 `VirtualGeometry` feature 显式开启时，为 prepare 驱动的 fallback draw 生成真实的 indexed indirect args buffer。
- `MeshDraw` 新增可选 `indirect_args_buffer`，并保留原有 `first_index + draw_index_count` 作为 CPU-visible draw range 合同。
- `BaseScenePass`、`NormalPrepassPipeline` 与 deferred geometry pass 现在都会在 `indirect_args_buffer` 存在时走 `draw_indexed_indirect(...)`，否则继续回退到原来的 direct indexed draw。
- `SceneRenderer` 新增 renderer-local 的 `last_virtual_geometry_indirect_draw_count()` 可观测性，便于测试确认 prepare segment 确实进入了 indirect raster 路径。

## Renderer Contract

- indirect args 仍然是 renderer-local 细节，不会暴露到 `RenderServer` 或外部 façade。
- `Virtual Geometry` prepare 仍然只负责提供 `visible_entities` 与 `cluster_draw_segments`。
- renderer 负责把 segment 合同翻译成：
  - `MeshDraw.first_index`
  - `MeshDraw.draw_index_count`
  - `MeshDraw.indirect_args_buffer`
- 三条 mesh raster path 的消费规则现在统一为：
  - 有 `indirect_args_buffer` 时，使用 `draw_indexed_indirect(...)`
  - 没有 `indirect_args_buffer` 时，使用原来的 `draw_indexed(...)`

## Why This Slice Exists

- 上一轮已经把 `cluster_draw_segments` 变成 prepare 对 renderer 的唯一 segment 合同，但最终提交仍然是 CPU 直接 `draw_indexed(...)`。
- 这意味着 `Virtual Geometry` 虽然已经具备 prepare/runtime/gpu-completion/refine 边界，但还没有一个真正的 indirect raster 落点。
- 本轮先落 renderer-local indirect baseline，后续 GPU-generated command compaction、visibility-owned indirect buffer、Nanite-like cluster raster 才有明确的可替换消费点。

## Validation Summary

- `virtual_geometry_prepare_segments_submit_indirect_raster_draws_when_feature_enabled`
  - 证明当 `VirtualGeometry` feature 开启且 prepare segment 存在时，renderer 会提交至少一个 indirect raster draw
- `virtual_geometry_prepare_render`
  - 证明新的 indirect raster 基线没有破坏 prepare-driven filtering、streaming coverage、cluster-id spatial region 与 prepare segment override 行为
- `virtual_geometry`
  - 证明 indirect raster 基线与已有 runtime host、GPU uploader/readback、refine frontier 兼容

## Remaining Route

- GPU-generated indirect command compaction / multi-draw batching
- visibility-owned indirect args buffer，而不是 renderer-local per-draw indirect args
- Nanite-like cluster raster / page indirection / deeper split-merge hierarchy refinement
- 与 occlusion、BVH、RT、Hybrid GI scene representation 的更深层联合路径
