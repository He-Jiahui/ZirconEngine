---
related_code:
  - zircon_graphics/src/backend/render_backend/read_buffer_u32s.rs
  - zircon_graphics/src/runtime/virtual_geometry/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/update_stats/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/execute_prepare/execute/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
implementation_files:
  - zircon_graphics/src/backend/render_backend/read_buffer_u32s.rs
  - zircon_graphics/src/runtime/virtual_geometry/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/update_stats/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/execute_prepare/execute/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
plan_sources:
  - user: 2026-04-17 Virtual Geometry next step should enter real GPU uploader/readback before cluster hierarchy/refine
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-feedback-streaming.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - cargo test -p zircon_graphics --offline render_server_bridge
  - cargo test -p zircon_graphics --offline virtual_geometry_runtime
  - cargo test -p zircon_graphics --offline virtual_geometry_prepare_render
  - cargo test -p zircon_graphics virtual_geometry_gpu --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare --locked
  - cargo test -p zircon_graphics virtual_geometry --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry GPU Uploader Readback

**Goal:** 用真实 `wgpu` buffer/upload/compute/readback 替换上一阶段只靠 CPU feedback 推进的 request completion source，同时继续保持 capability-gated、renderer-local、可降级的架构边界。

**Non-Goal:** 本轮仍然不实现 cluster hierarchy refine、page streaming residency manager、真实 cluster raster、indirect draw integration，`Virtual Geometry` 仍然只是 opt-in feature 族上的基础 GPU completion baseline。后续 `GPU-assigned slot ownership` 已在 [2026-04-17-m5-virtual-geometry-slot-assignment-ownership.md](./2026-04-17-m5-virtual-geometry-slot-assignment-ownership.md) 继续推进。

## Delivered Slice

- `SceneRenderer` 新增 renderer-local `virtual_geometry/` 子模块：
  - 把 resident page table 上传到真实 storage buffer
  - 把 pending page request 上传到真实 storage buffer
  - 用最小 compute uploader pass 依据 `page_budget + evictable_pages` 计算本帧可完成的 upload set
  - 在提交后通过 readback buffer 拉回 `page_table_entries` 与 `completed_page_ids`
- 当前 `virtual_geometry/gpu_resources/execute_prepare/execute/` 已继续拆成 `collect_inputs / create_buffers / create_bind_group / dispatch / copy_readbacks / execute` 等 helper 子模块，prepare execute 根入口只保留结构 wiring。
- `VirtualGeometryRuntimeState` 新增 `complete_gpu_uploads(...)`，把 GPU 完成的 page id 与 prepare snapshot 里的 evictable page 列表合并消费，复用已有 resident-slot/eviction policy，而不是旁路再造一套 runtime host。
- `WgpuRenderServer::submit_frame_extract(...)` 现在优先消费 renderer 侧的 GPU readback；只有当 GPU readback 不可用时才回退到旧的 `consume_feedback(...)` CPU baseline。

## Renderer Contract

这份文档记录的是“先把 completion source 挪进 GPU”这一步。当前代码已经在后续 slice 中继续推进到 GPU-assigned `page_id -> slot` readback，具体见 [2026-04-17-m5-virtual-geometry-slot-assignment-ownership.md](./2026-04-17-m5-virtual-geometry-slot-assignment-ownership.md)。

- GPU path 仍然不把 `wgpu` 原生类型暴露给 `RenderServer` 或外部 consumer。
- `SceneRenderer` 只额外暴露一个 renderer-local `take_last_virtual_geometry_gpu_readback()` 取回本帧 readback 结果；这仍然停留在 `zircon_graphics` 内部。
- uploader shader 当前只做最小 completion arbitration：
  - `resident_count` 来自 prepare snapshot 的 resident page table
  - `pending_count` 来自 prepare snapshot 的 pending request
  - `page_budget` 来自 extract capability contract
  - `evictable_count` 来自 runtime host 预先给出的 eviction 候选
- 当时这一步的 shader 还不分配 slot，也不直接改 runtime host；当前代码则已经继续把 slot assignment ownership 推到 GPU readback。

## Runtime Contract

- `build_prepare_frame(...)` 继续作为 render 前的唯一 runtime snapshot 入口。
- `runtime/virtual_geometry/prepare_frame/` 与 `submit_frame_extract/update_stats/` 现在都已经拆成 root-only wiring + helper 子模块，prepare snapshot 构造与 submit 后 façade stats 汇总不再混在单个聚合脚本里。
- GPU readback 返回后：
  - runtime host 仅接受仍处于 pending 的 page id
  - resident 数达到 budget 时，只能消费 prepare snapshot 给出的 evictable page
  - 没有可回收 budget 时，请求仍保持 pending
- 这保证 GPU uploader 只替换“completion source”，不会反向污染 runtime host 的 residency/slot policy。

## Validation Summary

- `virtual_geometry_gpu_uploader_readback_reports_completed_page_ids_from_prepare_snapshot`
  - 证明 resident page table 已被真实上传/readback，并且 compute uploader 会在有 evictable slot 时回传可完成 page id
- `virtual_geometry_gpu_uploader_readback_respects_budget_without_evictable_pages`
  - 证明当 resident cache 已满且没有 evictable page 时，GPU completion 会干净地返回空完成集
- `virtual_geometry_runtime_state_applies_gpu_completed_pages_with_evictable_slots`
  - 证明 runtime host 会用 GPU 完成集复用旧 slot，并移除被淘汰 resident page
- `virtual_geometry_prepare_filters_mesh_fallback_to_allowed_entities`
  - 证明 prepare-driven mesh fallback 没有被新的 GPU completion source 破坏

## Remaining Route

- 真正的 page residency manager / async copy / upload queue orchestration
- cluster hierarchy refine / split-merge / SSE-driven selection
- page streaming / page-table indirection 的更完整 GPU ownership
- indirect draw / visibility / BVH / RT 与 `Virtual Geometry` 的深层耦合
