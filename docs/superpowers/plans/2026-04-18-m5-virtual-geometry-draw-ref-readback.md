---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_draw_refs.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_draw_refs.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
plan_sources:
  - user: 2026-04-18 Virtual Geometry 的 visibility-owned unified indirect authority 继续下沉到真实 GPU submission / cluster raster
  - user: 2026-04-18 继续任务
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-gpu-submission-segment-readback.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_uses_one_visibility_owned_indirect_segment_across_multi_primitive_model
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect
doc_type: milestone-detail
---

# M5 Virtual Geometry Draw-Ref Readback

**Goal:** 把 `Virtual Geometry` 的 unified indirect authority 再从 segment truth 向下推进一层，直接回读真实 GPU submission 的 draw-ref buffer，证明每条 indirect draw 最终引用了哪条 segment。

**Non-Goal:** 本轮不改写 GPU compaction policy、draw ordering 或 cluster-raster shader。

## Delivered Slice

- `build_shared_indirect_args_buffer(...)` 现在除了保留 `segment_buffer` 与 `output_buffer`，还会保留真实提交给 compute pass 的 `draw_ref_buffer`
- draw-ref buffer 已补上 `COPY_SRC`，允许测试从 GPU 回读真实 draw-ref truth
- compiled-scene stats / renderer last-state plumbing 现在会一路保留这份 buffer：
  - `build_compiled_scene_draws(...)`
  - `virtual_geometry_indirect_stats(...)`
  - `render_compiled_scene(...)`
  - `render_frame_with_pipeline(...)`
  - `store_last_runtime_outputs(...)`
  - `SceneRenderer`
- 测试侧新增 `read_last_virtual_geometry_indirect_draw_refs()`，直接读取 `(mesh_index_count, segment_index)` 对

## Why This Slice Exists

- 上一轮已经能回读真实 GPU submission 的 `segment_buffer`。
- 但 segment truth 还不足以证明“每条 draw 如何映射到 segment”这层真实提交关系。
- draw-ref readback 让 unified indirect authority 的回归锚点继续推进到真正的 per-draw submission mapping。

## Validation Summary

- `virtual_geometry_prepare_uses_one_visibility_owned_indirect_segment_across_multi_primitive_model`
  - 现在会额外断言两条 primitive draw-ref 都映射到同一个 `segment_index = 0`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect`

## Remaining Route

- 继续把 submission truth 推向更深的 cluster raster consumption，而不只停在 draw-ref readback。
- 如果后续进入真正 GPU-driven indirect compaction，这份 readback 可以直接作为 submit-time regroup 的回归锚点。
