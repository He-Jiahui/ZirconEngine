---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_draw_refs.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_draw_refs.rs
plan_sources:
  - user: 2026-04-19 Virtual Geometry 更深的 unified indirect / cluster-raster submission ownership
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Explicit Draw-Ref Authority And Cluster-Raster Submission

## Goal

把 `Virtual Geometry` 的 unified-indirect authority 再往下压一层，不再让 renderer 和 compute shader 分别从 buffer 顺序、`pending_draw.indirect_draw_ref` 或 debug token side-channel 重新推断 draw-ref rank / submission detail。

## Delivered Slice

### 1. shared layout 现在一次性产出 draw submission 真值

`build_shared_indirect_args_layout(...)` 现在不只生成：

- `indirect_args_offsets`
- `pending_draw_submission_orders`
- `pending_draw_submission_tokens`

还会继续生成：

- `pending_draw_submission_details`
- 每条 draw-ref 的 `segment_draw_ref_count`
- 每条 draw-ref 的显式 `submission_token`

因此 shared layout 自己已经成为 renderer draw-level submission detail 与 GPU args compaction 的共同真值，而不再只负责“先分配 offset，再让上下游自己补语义”。

### 2. GPU draw-ref input 现在显式携带 compaction metadata

`VirtualGeometryIndirectDrawRefInput` 新增：

- `segment_draw_ref_count`
- `submission_token`

`virtual_geometry_indirect_args.wgsl` 不再扫描整个 `draw_ref_buffer` 来重新计算同-segment draw-ref rank/count，而是直接消费 shared layout 写入的显式 metadata。

这意味着 cluster-raster compaction 的 authority 已经从：

- buffer slot 顺序隐式推断

继续下沉到：

- draw-ref record 自己显式携带的 authority

### 3. renderer-side submission detail 也切回 shared layout

`build_mesh_draws(...)` 现在优先从 shared layout 回填 `VirtualGeometrySubmissionDetail`，只在没有 shared layout detail 时才回退到旧的 `pending_draw.indirect_draw_ref` 路径。

因此 renderer last-state / stats / draw submission record 已经开始直接消费 unified-indirect 真值，而不再主要依赖 renderer 私有 draw construction residue。

## Why This Matters

之前虽然 `segment_buffer / draw_ref_buffer / indirect args` 已经进入 prepare-owned / visibility-owned 主链，但还剩两条重复推断路径：

- compute shader 通过扫描 draw-ref buffer 重建 draw-ref rank/count
- renderer 通过 `pending_draw.indirect_draw_ref` 重建 draw-level submission detail

这会留下两份 “晚于 shared layout 才出现” 的 authority 副本。

本轮之后：

- cluster-raster compaction 直接吃 shared layout 真值
- renderer draw submission detail 直接吃 shared layout 真值
- `first_instance` / submission debug token 也和同一份 token 对齐

于是 unified-indirect authority 已经不再只是“shared buffer 长什么样”，而是真正成为 CPU/GPU 两侧共同消费的 execution contract。

## Validation Summary

- `virtual_geometry_unified_indirect`
- `virtual_geometry_submission_execution_order`
- `virtual_geometry_prepare_render`
- `cargo check -p zircon_graphics --offline --locked`

这些回归共同证明：

- repeated primitive compaction 没被打穿
- visibility-owned submission order 仍然稳定控制透明叠加次序
- cluster-raster output 仍然按真实 indirect args / segment truth 变化

## Remaining Route

- 把这份 shared-layout truth 再继续压进更真实的 GPU-generated args compaction source，减少 CPU-side per-draw submission ordering 的最后残留
- 继续推进 deeper cluster-raster / indirect execution ownership，让 actual submission 更接近 visibility-owned unified indirect buffer 主导
- 再往下切回 residency-manager cascade / split-merge frontier policy，把 page-table / completion 真值进一步收口
