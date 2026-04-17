---
related_code:
  - zircon_graphics/src/types/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/buffer_helpers.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/gpu_pending_request_input.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/virtual_geometry_uploader_params.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/buffer_helpers.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/gpu_pending_request_input.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/virtual_geometry_uploader_params.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
plan_sources:
  - user: 2026-04-17 Virtual Geometry should continue from refine and uploader baseline into a more real cluster streaming path
  - user: 2026-04-17 continue the remaining M5 Virtual Geometry route without waiting for confirmation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-gpu-uploader-readback.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-indirect-raster-consumption.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - cargo test -p zircon_graphics virtual_geometry_gpu_uploader_readback_reports_completed_page_ids_from_prepare_snapshot --locked
  - cargo test -p zircon_graphics virtual_geometry_gpu_uploader_readback_respects_budget_without_evictable_pages --locked
  - cargo test -p zircon_graphics virtual_geometry_gpu_uploader_readback_respects_streaming_bytes_even_with_evictable_pages --locked
  - cargo test -p zircon_graphics virtual_geometry_gpu_uploader_readback_skips_oversized_requests_and_completes_ones_that_fit --locked
  - cargo test -p zircon_graphics virtual_geometry_gpu --locked
  - cargo test -p zircon_graphics virtual_geometry --locked
  - cargo test -p zircon_graphics --lib --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Size-Aware Streaming Uploader

**Goal:** 把 `Virtual Geometry` GPU uploader 从“只按 pending page 数量和 evictable slot 数量做 completion arbitration”推进到真正会消费 `size_bytes`、并在 renderer-local compute pass 内执行 size-aware streaming 调度的基线。

**Non-Goal:** 本轮仍然不实现真实 async copy queue、磁盘/page I/O、GPU-driven indirect compaction 或 Nanite-like cluster raster。后续 `page-table slot assignment ownership` 已在 [2026-04-17-m5-virtual-geometry-slot-assignment-ownership.md](./2026-04-17-m5-virtual-geometry-slot-assignment-ownership.md) 继续推进。

## Delivered Slice

- `VirtualGeometryPrepareRequest.size_bytes` 现在不再只停留在 runtime snapshot；`execute_prepare(...)` 会把 pending request 上传成 `GpuPendingRequestInput { page_id, size_bytes }`。
- uploader params 新增：
  - `streaming_budget_bytes`
  - `reclaimable_bytes`
- current baseline 把 `page_budget` 映射成 renderer-local streaming granularity budget，并把 `prepare.evictable_pages[*].size_bytes` 汇总成可回收字节预算。
- `uploader.wgsl` 不再并行地按 request ordinal 直接截断，而是在 `global_id.x == 0` 的顺序循环里做 greedy streaming arbitration：
  - 既检查可用 completion slots
  - 也检查剩余 streaming bytes
  - oversized request 会被跳过，后续更小且能装入预算的 request 仍可完成

## Renderer Contract

这份文档记录的是 size-aware uploader 这一步的基线切割点。当前代码已经在后续 slice 中继续前进到 GPU-assigned slot ownership，具体见 [2026-04-17-m5-virtual-geometry-slot-assignment-ownership.md](./2026-04-17-m5-virtual-geometry-slot-assignment-ownership.md)。

- `RenderServer` 与 runtime host 的外部合同没有变化：
  - readback 仍然只返回 `page_table_entries` 与 `completed_page_ids`
  - slot 分配/eviction policy 仍然留在 `VirtualGeometryRuntimeState`
- size-aware 行为仍然是 renderer-local 细节：
  - prepare snapshot 提供 `resident_pages / pending_page_requests / evictable_pages`
  - renderer uploader 决定本帧哪些 request 真正完成
  - runtime host 只消费完成结果，不接触 `wgpu` 或 uploader 私有 params

## Why This Slice Exists

- 之前的 GPU uploader 虽然已经是真实 compute/readback，但仍然完全忽略 `size_bytes`，本质上只是把 CPU count-budget 裁剪搬进 shader。
- 这会让超大 page request 在只剩一个 evictable slot 时也被错误地“完成”，与后续 page streaming/residency manager 路线不兼容。
- 本轮先把 size-aware arbitration 固定在 renderer-local uploader 内部，后续 async copy、page-table ownership、streaming queue orchestration 才有一个可信的替换点。

## Validation Summary

- `virtual_geometry_gpu_uploader_readback_reports_completed_page_ids_from_prepare_snapshot`
  - 证明 size-aware uploader 没有破坏原有正常 completion 基线
- `virtual_geometry_gpu_uploader_readback_respects_budget_without_evictable_pages`
  - 证明没有可回收 slot 时，request 仍然会保持 pending
- `virtual_geometry_gpu_uploader_readback_respects_streaming_bytes_even_with_evictable_pages`
  - 证明即使存在 evictable slot，超出 streaming byte budget 的大页也不会被误判为 completed
- `virtual_geometry_gpu_uploader_readback_skips_oversized_requests_and_completes_ones_that_fit`
  - 证明 uploader 会跳过当前装不下的大页，并继续完成后面能装入预算的小页

## Remaining Route

- async copy / upload queue / readback fence 的真正调度
- 与 prepare segment、occlusion、indirect compaction、cluster raster 的更深层联合路径
