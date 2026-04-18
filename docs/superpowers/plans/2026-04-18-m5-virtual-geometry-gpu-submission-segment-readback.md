---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_segments.rs
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
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_segments.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
plan_sources:
  - user: 2026-04-18 Virtual Geometry 的 visibility-owned unified indirect authority 继续下沉到真实 GPU submission / cluster raster
  - user: 2026-04-18 继续缺漏内容补充
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-visibility-owned-lineage-segments.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_uses_one_visibility_owned_indirect_segment_across_multi_primitive_model
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect
doc_type: milestone-detail
---

# M5 Virtual Geometry GPU Submission Segment Readback

**Goal:** 把 `Virtual Geometry` 的 visibility-owned unified indirect authority 再下沉一层，不只在 prepare/runtime 里保留 segment contract，还能直接读到“真实提交给 GPU 的 indirect segment buffer”。

**Non-Goal:** 本轮不改写 uploader shader、真正 GPU-driven compaction、cluster raster algorithm 或 render-server façade。

## Delivered Slice

- `build_shared_indirect_args_buffer(...)` 现在会同时保留：
  - 真正提交给 compute pass 的 `segment_buffer`
  - 最终生成 indirect args 的 `output_buffer`
- `SceneRenderer` 的 last-state bookkeeping 现在除了 `last_virtual_geometry_indirect_args_buffer` 之外，也会保留 `last_virtual_geometry_indirect_segments_buffer`。
- 测试侧新增 `read_last_virtual_geometry_indirect_segments()`：
  - 直接回读 GPU submission 的 segment buffer
  - 解码 `cluster_start_ordinal / cluster_span_count / cluster_total_count / page_id / resident_slot / state`
- 这让 unified-indirect 回归现在可以同时证明两件事：
  - 多 primitive draw 仍然共用一条 visibility-owned segment
  - 真正提交给 GPU 的 segment truth 没有在 renderer 里再次 regroup 或降级成 prepare-time projection

## Why This Slice Exists

- 之前的回归只能证明：
  - `cluster_draw_segments`
  - `prepare.unified_indirect_draws()`
  - `read_last_virtual_geometry_indirect_args()`
  三者大致一致。
- 但这还不足以证明真实 GPU submission 没有在中途丢掉 visibility-owned segment contract。
- 这条 readback 把“submission authority”从逻辑投影推进到真实提交缓冲本身，后续继续做 draw-ref ownership、GPU compaction、cluster raster deeper consume 时就有了更直接的回归锚点。

## Validation Summary

- `virtual_geometry_prepare_uses_one_visibility_owned_indirect_segment_across_multi_primitive_model`
  - 现在不仅验证 indirect args 记录数与 segment 计数，还会直接断言 GPU submission segment buffer 中只有一条 visibility-owned segment。
- `virtual_geometry_unified_indirect`
  - 证明新的 submission readback plumbing 没有破坏现有 unified-indirect regressions。

## Remaining Route

- 继续把 submission truth 向下推进到 draw-ref ownership / deeper cluster raster consumption，而不只停在 segment buffer readback。
- 继续把 runtime/readback/stats 一致性往 façade 侧收敛，减少 renderer 私有真值。
