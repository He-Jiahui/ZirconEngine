---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_gpu_uploads_with_slots.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_gpu_uploads_with_slots.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
plan_sources:
  - user: 2026-04-19 补更深的 residency-manager cascade / page-table / completion / split-merge frontier policy 收敛
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-hot-frontier-runtime-residency-cascade.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-page-table-confirmed-completion-cascade.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_carries_hot_frontier_truth_into_newly_completed_descendant_before_next_prepare -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu -- --nocapture
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Completed-Descendant Hot-Frontier Cascade

## Goal

继续把 `Virtual Geometry` split-merge / residency cascade 压到更完整的 completion 真值：

- runtime 已经会缓存 `hot_resident_pages`
- final `page_table_entries` 也已经成为 confirmed completion truth
- 但 newly completed descendant page 还不会继承 ancestor frontier 的热度

结果是：

- deeper descendant 明明刚完成 upload 并接住了当前 frontier
- 下一帧 prepare recycle 却仍然把它当成 cold resident page 回收

## Delivered Slice

### 1. 红灯先证明 completion 之后的 hot-frontier truth 仍然会断层

新增 `virtual_geometry_runtime_state_carries_hot_frontier_truth_into_newly_completed_descendant_before_next_prepare`：

- resident hot page `400` 代表当前 frontier
- descendant `800` 在 GPU completion 里刚成为新 resident
- 下一帧请求回连缺失 ancestor `200`

期望：

- recycle 计划应该继续保住更深的 `800`
- 回收更浅的 `400`

实现前实际行为正好相反，说明热度还停在旧 resident page，没有沿 completion 主链继续下沉。

### 2. 热度继承下沉到 completion promotion 本身

修复放在 `complete_gpu_uploads_with_replacements(...)`，而不是只停在 page-table apply：

- 如果 completed page 自己替换了 hot resident page，继承热度
- 如果 completed page 位于当前 hot frontier ancestor chain 下，也继承热度
- promotion 发生后立即把新 page 写进 `current_hot_resident_pages`

这样即使 `apply_gpu_page_table_entries(...)` 随后只做 final truth 对齐，也不会再丢掉 “谁才是刚接住 frontier 的新 resident” 这层信息。

### 3. page-table apply 保留 final truth，但不再承担唯一热度补偿职责

`apply_gpu_page_table_entries(...)` 仍然负责：

- 删除 final page-table 里已消失的 resident page
- 以 final `page_table_entries` 为准重建 resident slot truth
- 只保留当前仍 resident 的 hot page

但热度补偿现在不再完全依赖这一层推断，因为真正的 “newly completed page” 在 runtime path 里已经先被 completion promotion 消化掉了。

## Why This Slice Matters

这一刀把三条 previously split 的 truth 串起来了：

- `VisibilityVirtualGeometryFeedback.hot_resident_pages`
- GPU completion promotion
- next-frame prepare recycle / split-merge eviction ordering

没有这条继承，final page-table truth 虽然是对的，residency policy 却仍然会围绕旧页工作，导致 frontier 刚下沉到 deeper descendant 又被下一帧 recycle 掉。

## Validation Summary

- 红灯
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_carries_hot_frontier_truth_into_newly_completed_descendant_before_next_prepare -- --nocapture`
- 绿灯
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_carries_hot_frontier_truth_into_newly_completed_descendant_before_next_prepare -- --nocapture`
- 回归
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu -- --nocapture`
  - `cargo check -p zircon_graphics --lib --offline --locked`

## Remaining Gaps

- 这轮已经让 newly completed descendant 继承 hot-frontier truth，但更深层 residency-manager cascade 仍然没有完全覆盖 page-table completion 之后的全部 hierarchy hysteresis，例如更宽的 split/merge frontier 级联、visibility-owned unified indirect authority 与 residency manager 的更深耦合。
- Hybrid GI 仍然保留另一条 M5 主链：scene-driven screen-probe hierarchy / RT hybrid lighting continuation。
