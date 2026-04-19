---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_order.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_order.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
plan_sources:
  - user: 2026-04-19 Virtual Geometry deeper unified indirect / GPU-generated args / cluster-raster submission ownership
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-draw-level-submission-records.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_renderer_submission_records_keep_draw_level_tokens_without_gpu_submission_buffer -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Direct Submission Token Records

## Goal

继续把 `Virtual Geometry` 的 unified-indirect authority 从 GPU side-channel 再往 renderer 自身压一层。

上一刀已经让 renderer last-state 可以读出：

- `(entity, page_id, submission_index, draw_ref_rank)`

但那条 helper 仍然依赖真实 GPU `submission_buffer` readback。

这意味着 draw-level submission truth 仍然主要住在 buffer/readback 侧，而不是 renderer 自己的 last-state。只要 `submission_buffer` 不在，renderer observability 还是会掉回空值或 coarse fallback。

## Delivered Slice

### 1. 红灯锁定 “draw-level token records 仍然依赖 GPU submission buffer”

新增回归：

- `virtual_geometry_renderer_submission_records_keep_draw_level_tokens_without_gpu_submission_buffer`

测试构造的是：

- repeated primitive
- same visibility-owned segment
- renderer 先正常完成一帧
- 随后主动丢掉 `last_virtual_geometry_indirect_submission_buffer`

要求是 renderer 仍然能继续返回：

- `(2, 300, 0, 0)`
- `(2, 300, 0, 1)`

实现前做不到，因为 draw-level helper 仍然要回头读 GPU submission buffer。

### 2. draw-level token/rank 现在直接沉到 `MeshDraw`

新增：

- `VirtualGeometrySubmissionDetail`

并把它直接挂到 `MeshDraw`：

- `entity`
- `page_id`
- `submission_index`
- `draw_ref_rank`

这让 repeated primitive / same-segment compaction 的 draw-level truth 第一次真正进入 renderer 自己的 draw record，而不再只停在 GPU args / submission buffer。

### 3. `build_mesh_draws(...)` 直接把 token/rank 写进 renderer submission truth

`build_mesh_draws(...)` 当前已经拥有：

- `pending_draw_submission_tokens`

这轮不再只拿它排序 CPU `pending_draws`。

它现在还会把 token 解包成：

- `submission_index`
- `draw_ref_rank`

并连同 `(entity, page_id)` 一起写进 `VirtualGeometrySubmissionDetail`。

### 4. stats / runtime-output / last-state 不再只靠 GPU readback 补回 truth

`virtual_geometry_indirect_stats(...)` 现在会直接收集：

- `draw_submission_token_records`

随后这条 direct token truth 会继续经过：

- `render_compiled_scene(...)`
- `store_last_runtime_outputs(...)`
- `SceneRenderer.last_virtual_geometry_mesh_draw_submission_token_records`

最终 `read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()` 会优先消费这份 renderer-owned record；只有 direct record 不存在时才回退到旧的 GPU submission buffer readback。

### 5. coarse submission order 也切到同一份 direct truth

`read_last_virtual_geometry_mesh_draw_submission_order()` 现在不再单独维护另一套基于 buffer readback 的排序逻辑，而是直接从 draw-level token records 收口。

这样 coarse order 与 draw-level order 共享同一份 renderer-owned truth，不再继续分叉。

## Why This Slice Matters

如果 draw-level submission truth 只能存在于：

- `submission_buffer`
- indirect args
- readback helper

那 unified-indirect authority 仍然没有真正进入 renderer 自己的 state。

补上这层之后，renderer last-state 自己就开始拥有：

- repeated primitive compaction 的 draw-level token/rank
- coarse order 与 draw-level order 的同源 truth
- 不依赖 GPU submission buffer 存活的 submission record

这让后续继续推进更深的 cluster-raster submission ownership、stats closure、runtime-host bridge 时，不必再从 GPU side-channel 反推 renderer 自己刚刚提交过什么。

## Validation Summary

- 红灯
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_renderer_submission_records_keep_draw_level_tokens_without_gpu_submission_buffer -- --nocapture`
- 绿灯
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_renderer_submission_records_keep_draw_level_tokens_without_gpu_submission_buffer -- --nocapture`
- 回归
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining Route

- `Virtual Geometry` 仍然还有更行为层的一刀值得继续：
  - repeated draw / compaction / token truth 是否已经被单独钉到真实 cluster-raster output，而不是只停在 args / renderer records
  - 或继续切回更深的 residency-manager cascade / split-merge frontier policy
