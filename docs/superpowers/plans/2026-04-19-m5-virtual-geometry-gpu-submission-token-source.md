---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/virtual_geometry_indirect_args_gpu_resources/new.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_submission_tokens.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/virtual_geometry_indirect_args_gpu_resources/new.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_submission_tokens.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
plan_sources:
  - user: 2026-04-19 deeper visibility-owned unified indirect / cluster-raster submission ownership
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-pending-submission-layout-authority.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_ -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry GPU Submission-Token Source

## Goal

把 `Virtual Geometry` 的 visibility-owned submission authority 再往 GPU source 下沉一层，但不污染真正的 indirect execution 语义：

- 前一刀已经把 `pending_draw_submission_orders` 固定成 direct CPU layout authority
- 但这份 truth 仍主要停在 CPU mesh-build / submission 排序层
- 如果直接把 authority 编进真实 indirect args 字段，又会碰 `wgpu` 的 indirect-first-instance 语义副作用

这轮目标是让同一 compute pass 额外产出一份 renderer-side authoritative GPU submission-token source，让 submission truth 真正进入 GPU-generated source/readback 链，但不改动实际 `draw_indexed_indirect(...)` 的渲染语义。

## Delivered Slice

### 1. indirect args compute pass 新增并行 submission-token buffer

`virtual_geometry_indirect_args.wgsl` 现在除了输出真实 `IndexedIndirectArgs` 之外，还会同步写一份 `submission_debug_buffer`：

- 高 16 位：visibility-owned `submission_index`
- 低 16 位：同一 shared args record 内的 draw-ref compaction rank

这份 token 与真实 indirect args 来自同一条 compute source，不再需要靠 CPU 排序结果事后推断。

### 2. 真正的 indirect args 语义恢复纯渲染字段

为了避免 `wgpu` 的 indirect-first-instance 语义污染真实 draw execution，当前真正提交给 raster 的 `IndexedIndirectArgs.first_instance` 已恢复为 `0`。

换句话说：

- submission authority 继续进入 GPU-generated source
- 但不会再通过真正的 indirect draw 参数字段去冒险改变 draw 语义

这让 renderer 既拿到了更深的 GPU truth，又保住了现有透明排序 / fallback submission 的稳定回归。

### 3. renderer last-state 暴露新的 GPU submission-token readback

`SceneRenderer` 现在会把这份并行 token buffer 一起带进 last-state，并新增：

- `read_last_virtual_geometry_indirect_submission_tokens()`

测试因此可以直接断言：

- 哪条 GPU-generated args record 对应哪一个 visibility-owned submission index
- 这份 truth 是否真的来自 compute-generated source，而不是 CPU mesh ordering sidecar

## Why This Slice Matters

当前 `Virtual Geometry` 的 authority 之前已经收敛到：

- prepare-owned unified indirect order
- prepare-owned segment/draw-ref/args source
- CPU-side pending submission layout authority
- 真实 `draw_indexed_indirect(...)` 的 submission order

但 renderer 还缺一层“GPU source 自己记录 submission authority”的可观测真值。补上 submission-token source 之后：

- visibility-owned ordering 不再只停在 CPU mesh-build 解释层
- 同一 compute pass 生成的 GPU source 现在也能直接暴露 submission truth
- 后续继续推进更深的 GPU-driven compaction / cluster-raster consumption 时，有了一份不会污染真实 indirect execution 的 authoritative readback 基线

这正是继续往更真实的 GPU-generated args source / cluster-raster ownership 深挖前，最稳的一层收口。

## Validation Summary

- 红灯
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu_generated_args_expose_visibility_owned_submission_index_in_first_instance -- --nocapture`
- 绿灯
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture`
- 回归
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_ -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining Gaps

- 当前 submission truth 已经进入 GPU-generated source/readback，但真实 raster 仍然是 CPU-issued `draw_indexed_indirect(...)`，还没进入更完整的 GPU-driven compaction / indirect execution ownership。
- `Virtual Geometry` 下一条最自然的主链仍然是更深的 unified indirect / cluster-raster / GPU-driven compaction，以及与 page-table / completion 真值继续收敛的 residency-manager cascade。
