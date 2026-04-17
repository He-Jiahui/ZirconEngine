---
related_code:
  - zircon_graphics/src/types/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion.rs
  - zircon_graphics/src/runtime/virtual_geometry/residency_management.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/gpu_completion.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/record_submission.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/submit.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/virtual_geometry_uploader_params.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
implementation_files:
  - zircon_graphics/src/types/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion.rs
  - zircon_graphics/src/runtime/virtual_geometry/residency_management.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/gpu_completion.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/record_submission.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/submit.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/virtual_geometry_uploader_params.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
plan_sources:
  - user: 2026-04-17 continue the remaining M5 milestones without waiting for confirmation
  - user: 2026-04-17 Virtual Geometry should continue from the uploader/readback baseline into GPU-assigned slot ownership
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-gpu-uploader-readback.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-size-aware-streaming-uploader.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics virtual_geometry_gpu_uploader_readback_assigns_free_slots_before_recycling_evictable_slots --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_applies_gpu_assigned_free_slots_before_evictable_recycling --locked
  - cargo test -p zircon_graphics virtual_geometry --locked
  - cargo test -p zircon_graphics render_server_bridge --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Virtual Geometry Slot Assignment Ownership

**Goal:** 把 `Virtual Geometry` page-slot ownership 从 runtime host 的“完成后再分配 slot”推进到 renderer-local GPU uploader 的真实 `page_id -> slot` assignment readback，同时继续保持 capability-gated、prepare-driven、可降级的 SRP/RHI 边界。

**Non-Goal:** 本轮仍然不实现真实 async copy/upload queue、cluster streaming residency manager、GPU-generated indirect command compaction 或 Nanite-like cluster raster。

## Delivered Slice

- `VirtualGeometryPrepareFrame` 新增显式 `available_slots`，把 runtime host 当前可直接分配的 free slot 与 future sequential slot 一起导出给 renderer。
- `VirtualGeometryRuntimeState::build_prepare_frame(...)` 现在不只导出 `resident_pages / pending_page_requests / evictable_pages`，还会稳定导出：
  - 先按升序列出 `free_slots`
  - 再按 `next_slot ..` 补足在当前 `page_budget` 下仍可新开的 sequential slot
- renderer uploader 现在不再只回传 `completed_page_ids`；`VirtualGeometryGpuReadback` 新增 `completed_page_assignments: Vec<(page_id, slot)>`。
- `uploader.wgsl` 不再只按“能完成几个 request”做 count-based 裁剪，而是明确执行两段 slot arbitration：
  - 优先消费 prepare 显式给出的 `available_slots`
  - 用完后才开始复用 `evictable_pages[*].slot`
- `VirtualGeometryRuntimeState` 新增 `complete_gpu_uploads_with_slots(...)`，会严格 honor GPU 分配的 slot，而不是重新套用本地 slot policy。
- `WgpuRenderServer::submit_frame_extract(...)` 现在把 renderer readback 的 `completed_page_assignments` 穿过 submit/record 路径，再交给 runtime host 完成 resident page table 更新。

## Runtime Contract

- runtime host 仍然保有 residency policy，但不再保有“GPU 完成页最终落在哪个 slot”这个决定权。
- `promote_to_resident_in_slot(...)` 负责三件事：
  - 清理 pending request
  - 占用 GPU 指定的 slot，并在需要时释放中间 future slot 为后续帧继续使用
  - 当 assigned slot 仍被 resident page 占用时，只在该页属于 `evictable_pages` 时回收它
- 旧的 `consume_feedback(...)` page-id 路径仍然保留，作为没有 GPU readback 时的兼容回退；但正常 renderer path 现在优先使用带 slot 的 completion source。

## Renderer Contract

- renderer 现在会把三类 GPU 输入显式拆开上传：
  - `pending_requests`
  - `available_slots`
  - `evictable_slots`
- `completed_buffer` 布局从 `[count, page_id...]` 升级为 `[count, page_id, slot, page_id, slot, ...]`。
- size-aware streaming byte arbitration 继续生效；slot ownership 迁移只是把“完成了哪些页”进一步升级成“完成后这些页进入哪个 slot”。

## Why This Slice Exists

- 前一轮 size-aware uploader 已经把 completion arbitration 移到 GPU，但 slot assignment 仍然留在 host，意味着 renderer 对 page-table slot 只有“是否足够”的判断权，没有“具体落点”的判断权。
- 这会让后续 cluster streaming、page-table indirection、GPU-driven residency manager 很难继续把 ownership 往 renderer/GPU 推进。
- 本轮把 `available_slots + evictable_slots -> completed_page_assignments` 合同固定下来后，后续 async copy、resident page table、cluster streaming 才有真正稳定的 GPU-side slot 边界。

## Validation Summary

- `virtual_geometry_gpu_uploader_readback_assigns_free_slots_before_recycling_evictable_slots`
  - 证明 uploader 会先消耗显式 `available_slots`，再复用 evictable resident slot。
- `virtual_geometry_runtime_state_applies_gpu_completed_pages_with_evictable_slots`
  - 证明 runtime host 会按 GPU 返回的 assigned slot 推进 resident page，并淘汰对应的 evictable resident page。
- `virtual_geometry_runtime_state_applies_gpu_assigned_free_slots_before_evictable_recycling`
  - 证明 runtime host 不会重排 GPU assignment，而是严格按 `(page_id, slot)` 应用。
- `cargo test -p zircon_graphics --lib --locked`
  - 证明新的 slot ownership 合同没有破坏 M4 行为层、Hybrid GI、render server bridge 与现有 Virtual Geometry fallback/indirect raster 路径。

## Remaining Route

- 真正的 async copy/upload queue/readback fence orchestration
- cluster streaming / residency manager / page table indirection 的更深层 GPU ownership
- visibility-owned indirect command compaction 与 Nanite-like cluster raster
- 与 occlusion、BVH、Hybrid GI scene representation 的更深层联合路径
