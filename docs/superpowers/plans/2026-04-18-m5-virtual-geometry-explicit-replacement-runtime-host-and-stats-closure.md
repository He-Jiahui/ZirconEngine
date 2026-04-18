---
related_code:
  - zircon_framework/src/render/backend_types.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/gpu_completion.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/submit/collect_gpu_completions.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_gpu_uploads_with_slots.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/render_framework_bridge.rs
implementation_files:
  - zircon_framework/src/render/backend_types.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/gpu_completion.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/submit/collect_gpu_completions.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_gpu_uploads_with_slots.rs
plan_sources:
  - user: 2026-04-18 下一步是更深的 unified-indirect / residency-manager cascade，把同一套 frontier truth 继续推进到真实 GPU uploader / page-table / split-merge frontier policy
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-explicit-frontier-rank-uploader-and-page-table-cascade.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-runtime-owned-stats-closure.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/render_framework_bridge.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_consumes_explicit_gpu_replacement_truth_before_slot_fallbacks -- --nocapture
  - cargo test -p zircon_graphics --offline --locked headless_wgpu_server_exposes_current_m5_flagship_baselines_without_rt_capabilities -- --nocapture
  - cargo test -p zircon_graphics --offline --locked render_framework_bridge -- --nocapture
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Explicit Replacement Runtime Host And Stats Closure

## Goal

把 `Virtual Geometry` 已经存在于 GPU readback 里的 `completed_page_replacements(page_id, recycled_page_id)` 继续推进到 runtime host 和 façade stats，让 replacement truth 不再只停在 uploader/page-table decode 结构体里。

## Non-Goal

- 本轮不改写 GPU uploader shader contract。
- 本轮不引入新的 residency manager scoring tree。
- 本轮不让 stats 暴露页级明细列表；仍然只暴露稳定计数。

## Delivered Slice

### 1. Explicit replacement truth 现在穿过 render-framework host

- `VirtualGeometryGpuCompletion` 新增 `completed_page_replacements`
- `collect_gpu_completions(...)` 会把 renderer readback 的 replacement truth 带进 `submit_frame_extract` 主链
- `VirtualGeometryStatSnapshot` 也同步新增 `replaced_page_count`

这意味着 runtime host 不再只有：

- `page_table_entries`
- `completed_page_assignments`

而是开始直接拿到“这次完成到底替掉了谁”的显式事实。

### 2. Runtime completion 现在先消费 explicit replacement，再回灌 page table snapshot

- `VirtualGeometryRuntimeState` 新增 `complete_gpu_uploads_with_replacements(...)`
- 如果 GPU completion 显式报告：
  - `page 200 -> slot 2`
  - `recycled_page_id = 800`
- runtime 会先按这条 replacement truth 回收 `800`
- 然后再把 `page 200` 提升进同一 slot
- 最后仍由 `apply_gpu_page_table_entries(...)` 做整张页表快照对齐

这样 runtime 已经不再只靠：

- 当前帧 `evictable_pages`
- slot aliasing 推断
- page-table diff 间接回推

来理解“哪一页真的被替掉了”。

### 3. RenderStats 现在能看见 real replacement pressure

- `RenderStats` 新增：
  - `last_virtual_geometry_replaced_page_count`
- `update_virtual_geometry_stats(...)` 现在会把本帧 GPU completion 的 replacement 规模写回 façade

于是 façade 现在能同时看见：

- `completed_page_count`
- `replaced_page_count`
- prepare-owned `indirect_draw_count / indirect_segment_count`

这让上层可以区分：

- 只是完成了 free-slot upload
- 还是已经开始发生真实 resident-page recycling

## Why This Slice Exists

上一轮已经把 frontier truth 推到：

- explicit `frontier_rank`
- explicit `assigned_slot`
- explicit `recycled_page_id`
- GPU uploader / page-table completion readback

但 runtime host 仍然缺最后一段：

- completion 回写阶段并不直接消费 `recycled_page_id`
- façade stats 也看不到 replacement pressure

这会导致两类问题：

1. runtime 仍然偏向依赖 slot/page-table 的间接推断，而不是显式 GPU truth
2. 上层观测不到 residency cascade 什么时候已经从“补空槽”进入“真实 recycle resident page”

本轮补上的就是这条 readback -> runtime host -> RenderStats 的闭环。

## Validation Summary

- `virtual_geometry_runtime_state_consumes_explicit_gpu_replacement_truth_before_slot_fallbacks`
  - 证明 runtime completion 会信任 explicit replacement truth，把 GPU 指定的 recycled page 先回收掉，而不是继续只看当前 `evictable_pages`
- `headless_wgpu_server_exposes_current_m5_flagship_baselines_without_rt_capabilities`
  - 证明 façade stats 现在会真实暴露 `last_virtual_geometry_replaced_page_count`
- `render_framework_bridge`
  - 证明 submit/runtime/stats 主链整体没有回归
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明 replacement truth 下沉没有破坏 graphics crate 编译闭环

## Remaining Route

- 继续把 replacement / frontier truth 推向更深的 split-merge frontier policy，而不只停在 completion 回写和 stats 观察面。
- 继续把 unified indirect / cluster raster / residency-manager cascade 收敛成同一套 visibility-owned authority。
