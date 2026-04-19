---
related_code:
  - zircon_framework/src/render/backend_types.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/indirect_counts/indirect_args_count.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/indirect_counts/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/render_framework_bridge.rs
implementation_files:
  - zircon_framework/src/render/backend_types.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/indirect_counts/indirect_args_count.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/indirect_counts/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
plan_sources:
  - user: 2026-04-19 authority 压进真正的 visibility-owned / GPU-generated args compaction
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-visibility-owned-indirect-args-compaction.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/render_framework_bridge.rs
  - cargo test -p zircon_graphics --offline render_framework_stats_follow_actual_virtual_geometry_gpu_submission_for_multi_primitive_model -- --nocapture
  - cargo test -p zircon_graphics --offline render_framework_bridge -- --nocapture
  - cargo check -p zircon_graphics --lib --offline
doc_type: milestone-detail
---

# M5 Virtual Geometry Indirect Args Stats Closure

## Goal

把已经落到 renderer 内部的 visibility-owned indirect args compaction 再继续收口到 `RenderStats`，让 render-server / editor / runtime 外层不只看到 draw/segment 数，还能直接看到真实 compacted `indirect_args_count`。

## Delivered Slice

### 1. `RenderStats` 新增 `last_virtual_geometry_indirect_args_count`

`zircon_framework::render::RenderStats` 现在显式携带：

- `last_virtual_geometry_indirect_draw_count`
- `last_virtual_geometry_indirect_buffer_count`
- `last_virtual_geometry_indirect_args_count`
- `last_virtual_geometry_indirect_segment_count`

这让 façade 能同时区分：

- 真正执行了多少条 draw
- 共享了多少个 indirect buffer
- GPU-generated args source 最终压成了多少条 args record
- visibility-owned segment authority 保留了多少条 segment

### 2. submit/update-stats 主链现在直接消费 renderer last-state args count

`SceneRenderer` 原本已经持有 `last_virtual_geometry_indirect_args_count`，本轮新增显式 accessor，并让 `submit_frame_extract/update_stats/virtual_geometry_stats.rs` 把它写进 `RenderStats`。

这里没有重新回读 GPU buffer，也没有把 readback 逻辑复制到 façade 层；只是把 renderer 已经确认的 last-state count 真值继续向外传播。

### 3. render-server 回归现在能证明 compaction truth 已经出现在 façade 统计面

`render_framework_stats_follow_actual_virtual_geometry_gpu_submission_for_multi_primitive_model` 现在会断言：

- `draw_count == 2`
- `segment_count == 1`
- `indirect_args_count == 1`

因此当前 M5 unified indirect baseline 已经能在 façade 上直接表现出：

- “真实 primitive draw 数”
- “visibility-owned segment 数”
- “compacted args record 数”

三者互相独立。

## Why This Slice Matters

前几刀已经把 compaction authority 推到了 renderer internals，但 façade 统计面仍然只能看到 draw/segment/buffer。

这会让外层只知道“确实用了 indirect”，却看不到：

- compaction 是否真的发生
- args cardinality 是否还在退化回 per-draw

本轮补上之后，M5 Virtual Geometry unified indirect 的 cardinality truth 终于也能离开 renderer 内部，进入稳定的 public stats surface。

## Validation Summary

- red -> green
  - `render_framework_stats_follow_actual_virtual_geometry_gpu_submission_for_multi_primitive_model`
- regressions
  - `render_framework_bridge`
  - `cargo check -p zircon_graphics --lib --offline`

## Remaining Route

- 继续把这条 args cardinality truth 从 façade stats 继续推进到更真实的 visibility-owned / GPU-generated submission source，而不只是在 CPU build step 后汇报结果
- 继续推进 deeper cluster raster / indirect execution，让更多 authority 从 “排序 + 计数” 变成更真实的 GPU-driven execution ownership
