---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_graphics/src/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_graphics/src/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_graphics/src/tests/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/tests/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
plan_sources:
  - user: 2026-04-19 authority 压进真正的 visibility-owned / GPU-generated args compaction 和更深的 cluster raster consumption
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-prepare-owned-args-source-authority.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - cargo test -p zircon_graphics --offline virtual_geometry_submission_execution_order -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_submission_authority -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_args_source_authority -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_prepare_render -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_gpu -- --nocapture
  - cargo check -p zircon_graphics --lib --offline
doc_type: milestone-detail
---

# M5 Virtual Geometry Visibility-Owned Submission Execution Order

## Goal

把 unified indirect authority 再向真实 raster execution 压一层：不只让 `segment_buffer / draw_ref_buffer / indirect args` 按 visibility-owned truth 排序，还要让最终 `MeshDraw -> draw_indexed_indirect(...)` 的实际提交顺序也消费同一份 authoritative offset，而不是继续停在 CPU `pending_draws` 插入顺序。

## Delivered Slice

### 1. 新回归直接证明真实透明提交顺序会跟随 visibility-owned authority 改变

新增 `virtual_geometry_submission_execution_order.rs`。

测试固定：

- scene mesh 插入顺序始终是 `entity 2 -> entity 3`
- prepare `submission_slot` 在两帧里互换
- 两个 entity 用重叠透明 quad 直接做像素级验证

断言结果现在不再只看 indirect buffers：

- `indirect segment` 顺序会按 authority 翻转
- 中心像素的红/蓝主导关系也会跟着翻转

这证明真实 `draw_indexed_indirect(...)` 执行顺序已经离开 CPU mesh 插入顺序。

### 2. `build_mesh_draws(...)` 现在按 authoritative `indirect_args_offset` 稳定重排真实提交 draws

shared indirect args build 结束后，`build_mesh_draws(...)` 会把每条 pending indirect draw 和它回填到的真实 `indirect_args_offset` 绑定在一起，再按：

- `indirect_args_offset`
- `original_index`

做稳定排序，然后才生成最终 `MeshDraw` 列表。

因此当前 contract 变成：

- `prepare / visibility` 决定 unified indirect order
- `draw_ref_buffer / indirect args` 保留这条 authority
- `MeshDraw` 的实际提交顺序也消费同一份 authority

### 3. base / prepass / deferred 的真实 raster execution 不再绕开这条 truth

`BaseScenePass`、`NormalPrepassPipeline`、`DeferredSceneResources::record_gbuffer_geometry(...)` 仍然只是顺序遍历 `MeshDraw` 并调用 `draw_indexed_indirect(...)`。

本轮关键变化不是改 pass，而是把 pass 消费到的 `MeshDraw` 顺序本身改成 authoritative。

这意味着之前已经落地的：

- fallback slot authority
- prepare-owned indirect ordering
- prepare-owned segment source
- prepare-owned args source

现在终于真正进入 renderer execution，而不只是停在 shared buffer/readback 层。

## Why This Slice Matters

上一刀之后，真实 GPU-submitted `segment_buffer / draw_ref_buffer / indirect args` 已经都是 visibility-owned truth，但 pass 实际遍历的 `MeshDraw` 仍然保持 CPU `pending_draws` 顺序。

这会留下最后一条 execution leak：

- buffers 是 authoritative
- execution order 不是 authoritative

对 opaque draw 这条裂缝不总是可见，但对透明/重叠 draw 它会直接反映成最终像素错误。

本轮补上之后，Virtual Geometry unified indirect authority 才第一次真正贯穿：

- prepare
- buffer build
- indirect args generation
- actual raster submission

## Validation Summary

- red -> green
  - `virtual_geometry_transparent_submission_order_follows_visibility_owned_indirect_authority`
- regressions
  - `virtual_geometry_submission_authority`
  - `virtual_geometry_args_source_authority`
  - `virtual_geometry_unified_indirect`
  - `virtual_geometry_prepare_render`
  - `virtual_geometry_runtime`
  - `virtual_geometry_gpu`
  - `cargo check -p zircon_graphics --lib --offline`

## Remaining Route

- 继续把这条 authority 从 “CPU 按 authoritative offset 排序提交” 推到更真实的 visibility-owned / GPU-generated args source，例如让 draw submission descriptor/compaction source 进一步脱离 CPU-side pending draw bookkeeping
- 继续推进 deeper cluster raster consumption，让更多 cluster/page truth 不只体现在排序和子范围 trim 上，而是进入更真实的 GPU-driven cluster raster execution
- 然后再考虑回到更深的 split-merge frontier residency cascade
