---
related_code:
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_graphics/src/runtime/virtual_geometry/snapshot.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_gpu_uploads_with_slots.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
implementation_files:
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_graphics/src/runtime/virtual_geometry/snapshot.rs
plan_sources:
  - user: 2026-04-18 把同一套 submission_slot / page-table / completion 真值继续推进到更深的 residency-manager cascade
  - user: 2026-04-18 把 fallback slot authority 继续下沉到 unified indirect / draw-ref / submission ordering
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-explicit-replacement-runtime-host-and-stats-closure.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-stale-explicit-recycle-slot-contract-guard.md
tests:
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics --offline --locked gpu_completion_path_ignores_reported_replacement_when_previous_slot_owner_stays_resident -- --nocapture
  - cargo test -p zircon_graphics --offline --locked confirmed_virtual_geometry_completion_uses_previous_slot_owner_when_reported_replacement_is_stale -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_authority -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render -- --nocapture
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Page-Table Confirmed Completion Cascade

## Goal

把 `submission_slot / page_table / completion` 这条真值链继续从 GPU readback 表面推进到 runtime host 的最终确认阶段。

这一轮要收口的是一个更深但更关键的断层：

- GPU uploader 已经会回读 `page_table_entries`
- GPU readback 也已经会带回 `completed_page_assignments` 与 `completed_page_replacements`
- 但 runtime host 之前仍然可能过早信任“raw completed / raw replacement id”，而不是只信最终页表和旧 slot owner 的最终去留

结果就是，stale replacement id 仍可能继续抬高 replacement pressure，甚至让 runtime stats 看起来像是发生了真实替换，即使旧 owner 其实还活着。

## Non-Goal

- 本轮不改 GPU uploader shader 的 replacement 生成逻辑。
- 本轮不重写 runtime residency eviction policy。
- 本轮不直接推进新的 cluster raster draw path 或 page-table shader 数据结构。

## Delivered Slice

### 1. Completion 只有在 final page table 真正保留新页时才被确认

`update_virtual_geometry_runtime(...)` 现在在 record 阶段先做一轮 `confirmed_virtual_geometry_completion(...)`：

- `completed_page_assignments` 只保留那些最终仍然出现在 `page_table_entries` 里的 page
- 如果 GPU 报告“完成上传”，但最终页表没有保留它，runtime host 就不会清掉 pending request

这让 page-table snapshot 成为 completion 主链的最终 truth，而不是只把它当辅助 readback。

### 2. Replacement 只按 confirmed slot 的 previous owner 计算

新增的关键变化不是“继续信 raw `completed_page_replacements`”，而是改成：

- 先读取 runtime host 当前 resident slot owner snapshot
- 对每个被 final page table 确认的 `(page_id, slot)`，查看这个 slot 之前真正是谁
- 只有当这个 previous owner 在 final page table 里已经消失时，才把它计作 confirmed replacement

因此：

- raw replacement id 如果来自别的 slot，或者已经 stale，不再继续污染 host
- previous slot owner 如果只是被挪到别的 slot、最终仍然 resident，也不会再被错误计成 replacement

### 3. Runtime stats 和 pending clear 现在都跟随同一条 truth

这层确认逻辑让三个 runtime/host 行为开始共享同一条闭环：

- `pending_requests` 只在 final page-table-confirmed completion 下清掉
- `completed_page_count` 只统计真的落地到 final page table 的完成项
- `replaced_page_count` 只统计真正从 confirmed slot 中消失的 previous owner

这样 `RenderStats` 与 runtime residency host 不再分别信不同层级的 completion 信号。

### 4. 新增回归覆盖 stale replacement 与 previous slot owner 两种失真

本轮补了两条关键测试：

- `gpu_completion_path_ignores_reported_replacement_when_previous_slot_owner_stays_resident`
  - 证明 reported recycled-page id 即使命中了旧 owner，只要旧 owner 最终还留在 page table 里，就不能继续计入 replacement pressure
- `confirmed_virtual_geometry_completion_uses_previous_slot_owner_when_reported_replacement_is_stale`
  - 证明 confirmed replacement 会回到 “final resident slot 的 previous owner” 这条 truth，而不是继续信另一个 slot 带来的 stale replacement id

## Why This Slice Exists

前几轮已经把 Virtual Geometry 推到了：

- fallback recycle preference
- fallback slot submission authority
- implicit replacement readback
- explicit replacement runtime/stats closure

但这些切片还没有回答最后一个问题：

“当 GPU readback 和最终 page table 在语义上不完全对齐时，runtime host 到底信谁？”

如果这个问题不收口，runtime stats 与 pending clear 仍然会在 residency-manager cascade 的最下游出现“看起来上传成功/替换成功，但最终 resident truth 并不支持”的假阳性。

本轮补上的就是这条 host-side final confirmation gate。

## Validation Summary

- 目标回归：
  - `gpu_completion_path_ignores_reported_replacement_when_previous_slot_owner_stays_resident`
  - `confirmed_virtual_geometry_completion_uses_previous_slot_owner_when_reported_replacement_is_stale`
- Virtual Geometry runtime/gpu/unified-indirect/submission/render suites：
  - 保证 completion/replacement gate 下沉后，没有把 uploader、unified indirect、draw-ref、cluster raster 或 runtime prepare 路径打回旧状态
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 保证新的 slot-owner accessor 与 record-stage confirmation 没有破坏 crate compile closure

## Remaining Route

- 下一条更自然的 `Virtual Geometry` 主链，仍然是把这套 `submission_slot / page-table / completion` truth 继续压进更深的 residency-manager cascade，而不只停在 runtime host stats closure。
- 另一条并行价值更高的下沉方向，是把同一套 confirmed slot/page truth 继续推进到更真实的 GPU-driven cluster raster / indirect execution ownership。
