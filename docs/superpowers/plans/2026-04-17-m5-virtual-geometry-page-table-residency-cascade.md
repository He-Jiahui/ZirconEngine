---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/residency_management/evict_page.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/gpu_completion.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/submit/collect_gpu_completions.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/residency_management/evict_page.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/gpu_completion.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/submit/collect_gpu_completions.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
plan_sources:
  - user: 2026-04-17 continue the remaining M5 route without waiting for confirmation
  - user: 2026-04-17 Virtual Geometry still needs visibility-owned unified indirect / deeper cluster raster / residency-manager cascade
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-gpu-uploader-readback.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-slot-assignment-ownership.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_applies_gpu_page_table_snapshot_as_residency_truth
  - cargo test -p zircon_graphics --offline --locked virtual_geometry
  - cargo test -p zircon_graphics --offline --locked render_server_bridge
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Page-Table Residency Cascade

**Goal:** 把 `Virtual Geometry` runtime host 的 residency manager 从“只消费 `completed_page_assignments`”推进到“把 GPU uploader 的 `page_table_entries` 当成 residency truth”，让 slot 迁移、eviction 和下一帧 `available_slots` 级联都跟随 GPU 页表，而不是继续只靠 host 侧推断。

**Non-Goal:** 本轮仍然不实现真正的 Nanite-like cluster raster、GPU-driven occlusion、visibility-owned unified indirect buffer 编码或 split-merge hysteresis。

## Delivered Slice

- `VirtualGeometryGpuCompletion` 现在不再只携带 `completed_page_assignments`，还会保留 renderer readback 的 `page_table_entries`。
- `submit_frame_extract/submit/collect_gpu_completions.rs` 现在会把 GPU uploader 的完整页表快照带回 runtime/server 记录阶段。
- `VirtualGeometryRuntimeState::apply_gpu_page_table_entries(...)` 新增了一条“GPU 页表真值同步”路径：
  - 当前 host resident page 里，不在 GPU snapshot 里的页面会被驱逐
  - snapshot 里的 `(page_id, slot)` 会被提升为当前真实 resident ownership
  - pending request 若已在 snapshot 中出现，会被自动清理
  - `evictable_pages` 会在同步后只保留仍然真实 resident 的页面
- `update_virtual_geometry_runtime(...)` 现在会在消费 `completed_page_assignments` 之前先应用 `page_table_entries`。
  这意味着 runtime host 的 slot/residency 状态优先跟随 GPU uploader 的最终页表，而不是继续把 `completed_page_assignments` 当成唯一真实来源。

## Why This Slice Exists

- 之前的 M5 Virtual Geometry 路线已经把：
  - `available_slots`
  - `completed_page_assignments(page_id, slot)`
  - `page_table_entries`
  接进了 renderer-local compute uploader。
- 但 runtime host 只消费 `completed_page_assignments` 时，仍然有一个缺口：
  - GPU uploader 若在同一帧内改变了已有 resident page 的 slot
  - 或通过页表重写把某些 resident page 移除
  - host 并不会从 `completed_page_assignments` 单独看出整张 page table 的最终形态
- 这会让 `available_slots`、eviction 以及下一帧 prepare snapshot 继续滞后于 GPU 真值。
- 本轮把 page-table snapshot 直接纳入 runtime update 后，residency manager 才真正开始具备“GPU truth -> host state -> next prepare frame” 的级联闭环。

## Behavior Contract

- GPU completion 到达时，page-table snapshot 的优先级高于 host 侧既有 resident slot 记录。
- `completed_page_assignments` 仍然保留并继续消费，但它们现在退化成辅助 completion 信号；真正的 slot/residency authoritative state 来自 GPU page table snapshot。
- 同步后生成的下一帧 `VirtualGeometryPrepareFrame` 会立刻反映：
  - 哪些页仍然 resident
  - 哪些 pending request 已被清理
  - 哪些 slot 重新变成 `available_slots`

## Validation Summary

- `virtual_geometry_runtime_state_applies_gpu_page_table_snapshot_as_residency_truth`
  - 证明 runtime host 会按 GPU 页表快照移除旧 resident page、提升新 resident page，并把剩余 pending request 与下一帧 `available_slots` 一起级联更新。
- `cargo test -p zircon_graphics --offline --locked virtual_geometry`
  - 证明新的 residency truth 路径没有破坏既有的 GPU uploader、prepare render、visibility frontier、slot ownership 与 indirect raster 回归。
- `cargo test -p zircon_graphics --offline --locked render_server_bridge`
  - 证明 render-server 路径在 runtime/server submit 主链上继续保持稳定。
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明 runtime/server completion contract 扩展没有留下 crate 编译缺口。

## Remaining Route

- 把当前 GPU-truth residency cascade 继续推进到更深的 feedback residency manager，而不是继续只同步 slot/page table
- 把 visibility-owned indirect / cluster raster 与 residency truth 连接起来，让 page-table ownership 直接参与更深的 draw submission 和 streaming hysteresis
- 为后续更完整的 Nanite-like cluster streaming / split-merge policy 保留稳定的 runtime-server authority 边界
