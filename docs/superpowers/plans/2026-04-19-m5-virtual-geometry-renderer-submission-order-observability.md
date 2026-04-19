---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_order.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_order.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
plan_sources:
  - user: 2026-04-19 deeper visibility-owned unified indirect / cluster-raster submission ownership
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-visibility-owned-submission-execution-order.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_renderer_mesh_draw_submission_order_tracks_visibility_owned_unified_indirect_authority -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture
doc_type: milestone-detail
---

# M5 Virtual Geometry Renderer Submission-Order Observability

## Goal

把 `Virtual Geometry` 的 unified-indirect authority 从“真实执行顺序已经改了”继续推进到“renderer last-state 能稳定读回这条顺序”：

- 前一刀已经让 `MeshDraw -> draw_indexed_indirect(...)` 真实消费 visibility-owned authority
- 但 renderer/test 还缺一条稳定的 observability surface，能直接暴露最终 mesh draw submission order

这轮目标是把 submission key、stats、last-state 和测试 accessor 串起来，让 renderer 自己开始发布这条 execution-order truth。

## Delivered Slice

### 1. MeshDraw 显式携带 virtual geometry submission key

`MeshDraw` 现在会把 `(entity, page_id)` 作为 `virtual_geometry_submission_key` 带进最终 draw record。

这样 renderer 后续不需要回头重新解析 mesh/material/prepare，就能知道哪条 draw 对应哪条 virtual-geometry submission lineage。

### 2. renderer stats/last-state 暴露 submission order

`virtual_geometry_indirect_stats(...)` 会从最终 `mesh_draws` 提取 `draw_submission_order`，随后：

- `render_compiled_scene(...)`
- `render_frame_with_pipeline(...)`
- `store_last_runtime_outputs(...)`

把这条顺序继续写进 `SceneRenderer` last-state。

### 3. 测试 accessor 直接读取 renderer-side submission order

新增 `read_last_virtual_geometry_mesh_draw_submission_order()`，让测试可以直接断言：

- 最终 renderer-side mesh draw order
- 是否跟随 visibility-owned unified indirect authority 翻转

这条 observability surface 后续也为更深的 GPU source/readback closure 提供了过渡锚点。

## Why This Slice Matters

没有 renderer-side observability 时，只看：

- `segment_buffer`
- `draw_ref_buffer`
- `indirect args`

还不足以证明最终 `MeshDraw` 列表自己没有绕开 authority。把 order 真值写进 last-state 之后，execution-order regressions 才能被单测稳定锁住。

## Validation Summary

- 绿灯
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_renderer_mesh_draw_submission_order_tracks_visibility_owned_unified_indirect_authority -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture`

## Remaining Route

- 下一条自然主链仍然是把这条 observability 从 CPU-side recorded order 继续推进到更真实的 GPU-generated args/readback source，而不是只停在 renderer 当场排好的列表。

