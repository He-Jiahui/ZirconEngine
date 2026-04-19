---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/declarations/virtual_geometry_runtime_state.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/consume_feedback.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_gpu_uploads_with_slots.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/declarations/virtual_geometry_runtime_state.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/consume_feedback.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_gpu_uploads_with_slots.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
plan_sources:
  - user: 2026-04-19 继续把 normalized page_table / completion / hot frontier 真值压进更深的 residency-manager cascade / split-merge frontier policy
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_carries_recent_hot_frontier_lineage_through_one_cooling_frame_before_next_prepare -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked page_table -- --nocapture
  - cargo test -p zircon_graphics --offline --locked confirmed_virtual_geometry_completion_deduplicates_replacement_truth_after_page_table_normalization -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Recent Hot Frontier Cooling Hysteresis

## Goal

继续把 `Virtual Geometry` 的 normalized `page_table / completion / hot frontier` 真值压进更深的 `residency-manager cascade / split-merge frontier policy`，让 confirmed hot branch 不会因为下一拍 feedback 暂时冷却就立刻失去保护。

## Problem

此前 runtime host 已经能做到：

- 当前帧 feedback completion 优先保护 `current_hot_resident_pages`
- page-table-confirmed completion 会继承 lineage-aware hot frontier truth
- normalized page-table truth 已经统一了 runtime apply 与 completion stats

但还留着一个多帧 hysteresis 缺口：

- 一条 split-merge frontier branch 在 frame `N` 被确认 hot
- frame `N + 1` 的 feedback 因为视角变化或可见性抖动不再显式标热
- frame `N + 2` 的 prepare / reconnect ancestor / recycle plan 就会把这条更深 descendant 当作普通冷页直接回收

这意味着 confirmed `page_table / completion / hot frontier` 还没有真正压进后续 `residency-manager cascade`，而只是停在“当前帧 hot page”层面。

## Delivered Slice

### 1. runtime host 新增 single-frame trailing hot frontier truth

`VirtualGeometryRuntimeState` 现在除了 `current_hot_resident_pages` 之外，还会保留：

- `recent_hot_resident_pages`

`refresh_hot_resident_pages(...)` 会在消费新 feedback 之前，把上一拍的 confirmed hot frontier 先下沉到 `recent_hot_resident_pages`，形成单帧 trailing hysteresis。

### 2. recycle / completion / page-table apply 共用 `current + recent` hot truth

`ordered_evictable_pages_for_target(...)` 新增 exact + lineage 的 shared hot 查询，不再只看当前 feedback 的 hot set。

同一份 `current + recent` frontier truth 现在同时驱动：

- prepare recycle ordering
- feedback completion promotion
- GPU completion replacement inheritance
- page-table apply 之后的 inherited hot carry-forward

因此 split-merge frontier 的 protect/hold 不会只停在 prepare 一处，而是沿着 completion 与 page-table apply 继续往下收束。

### 3. reconnect ancestor 的 cooling-frame 回归已转绿

新增红绿回归：

- `virtual_geometry_runtime_state_carries_recent_hot_frontier_lineage_through_one_cooling_frame_before_next_prepare`

它证明：

- page `800` 在上一拍是 confirmed hot descendant
- 中间一拍 feedback 不再标热
- 下一次 prepare 重新请求缺失 ancestor `200` 时，runtime 仍会回收 colder shallower descendant `400`
- 不会立刻把更深的 `800` 当成冷页踢掉

## Why This Matters

这一刀把 M5 剩余主链从“当前帧 hot frontier”推进到了“confirmed hot frontier 的最小多帧 hysteresis”：

- `page_table` 真值不再只决定 resident slot
- `completion` 真值不再只决定本帧 promotion
- `hot frontier` 真值开始继续影响下一拍 split-merge recycle

这让后续更深的 `residency-manager cascade / frontier policy` 可以围绕同一份 confirmed runtime truth 继续扩展，而不需要重新引入临时 fallback 规则。

## Validation

- `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_carries_recent_hot_frontier_lineage_through_one_cooling_frame_before_next_prepare -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked page_table -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked confirmed_virtual_geometry_completion_deduplicates_replacement_truth_after_page_table_normalization -- --nocapture`
- `cargo check -p zircon_graphics --offline --locked`

## Remaining Route

- 继续把这份 `current + recent` confirmed frontier truth 压进更深的 completion/page-table/residency 归并点，减少 reconnect lineage 在后续多帧里重新退回“只看当前反馈”的残留。
- 继续补更深的 split-merge frontier policy，让 confirmed frontier truth 不只保护单次 ancestor reconnect，还能继续约束后续 recycle / hold / replacement 分支。
