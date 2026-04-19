---
related_code:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_order.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_submission_tokens.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_order.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
plan_sources:
  - user: 2026-04-19 visibility-owned / GPU-generated args source / compaction ownership
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-renderer-submission-order-observability.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_ -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry GPU-Generated Submission-Order Readback

## Goal

把 `Virtual Geometry` 的 renderer-side submission order 观测再下沉一层：

- renderer last-state 已经会记录最终 `MeshDraw` 顺序
- 但这条顺序仍然来自 CPU 当场排好的 draw list
- 还没有优先锚定到真实的 GPU-generated submission token source

这轮目标是让 last-state 读取在可能时优先从 `submission_debug_buffer` 反推出最终 draw order，减少测试和观测对 CPU reconstruction 的依赖。

## Delivered Slice

### 1. indirect stats 现在额外保留 draw submission records

`virtual_geometry_indirect_stats(...)` 现在除了 `draw_submission_order` 之外，还会保留：

- `(entity, page_id, indirect_args_offset, original_index)`

也就是每条 virtual-geometry draw 对应的 shared args record 位置和稳定 tie-break。

### 2. renderer last-state 新增 records 持久化

`render_compiled_scene(...)`、`render_frame_with_pipeline(...)`、`store_last_runtime_outputs(...)` 和 `reset_last_runtime_outputs(...)` 现在都会一起维护这批 submission records。

这样测试 accessor 不需要重新遍历 `MeshDraw` 构建上下文，也能在帧后恢复“哪条 draw 对应哪个 shared args record”。

### 3. read accessor 优先从 GPU-generated submission tokens 还原顺序

`read_last_virtual_geometry_mesh_draw_submission_order()` 现在会优先：

1. 调 `read_last_virtual_geometry_indirect_submission_tokens()`
2. 用 `indirect_args_offset / stride` 找到每条 draw 对应的 GPU-generated args record
3. 按 `(submission_token, indirect_args_offset, original_index)` 排序恢复最终 order

只有在没有 GPU token/readback 可用时，才回退到原来记录下来的 CPU order。

## Why This Slice Matters

这轮没有把正式 raster execution 改成真正的 GPU-driven multi-draw submission，但它已经让：

- renderer last-state
- 测试断言
- execution-order 观测

开始优先依赖真实 GPU-generated args source，而不是完全相信 CPU 当场怎么排。

这一步能显著减少 “CPU 排序和 GPU source 偶然一致，所以测试看不出裂缝” 的残留风险，也为下一条更深的 GPU-generated args compaction / cluster-raster submission ownership 留下更稳的观测基线。

## Validation Summary

- 绿灯
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_ -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining Route

- 当前还只是 GPU-generated order readback / observability 收口，真正的 raster execution 仍是 CPU 逐 draw 提交。
- 下一条最自然的主链仍然是更深的 visibility-owned / GPU-generated args compaction、deeper cluster-raster execution ownership，以及更完整的 residency-manager cascade / split-merge frontier policy。

