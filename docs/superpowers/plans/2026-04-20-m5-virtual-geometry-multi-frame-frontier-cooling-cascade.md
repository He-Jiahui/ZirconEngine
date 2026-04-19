---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/declarations/virtual_geometry_runtime_state.rs
  - zircon_graphics/src/runtime/virtual_geometry/declarations/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/extract_registration.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/consume_feedback.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_pending_pages.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_gpu_uploads_with_slots.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/runtime/virtual_geometry/residency_management/evict_page.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - docs/assets-and-rendering/render-framework-architecture.md
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/declarations/virtual_geometry_runtime_state.rs
  - zircon_graphics/src/runtime/virtual_geometry/declarations/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/extract_registration.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/consume_feedback.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
plan_sources:
  - user: 2026-04-20 M5 总路线继续回到更深的 unified indirect / cluster-raster / residency-manager cascade，confirmed frontier truth 不能只停在 single-frame cooling hysteresis
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_carries_confirmed_hot_frontier_lineage_through_two_cooling_frames_before_next_prepare -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_drops_confirmed_hot_frontier_lineage_after_cooling_budget_expires -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked page_table -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Multi-Frame Frontier Cooling Cascade

## Goal

继续把 `Virtual Geometry` 的 confirmed `page_table / completion / frontier` 真值压进更深的 `residency-manager cascade`，让 split-merge frontier 的保护不只停在 “current frame + 上一拍 recent”，而是能跨过更真实的 reconnect / recycle / completion merge 点。

## Problem

上一轮已经把 hot-frontier truth 推进到：

- `current_hot_resident_pages`
- 单帧 `recent_hot_resident_pages`
- feedback completion / GPU completion / page-table apply 共用的 lineage-aware hot query

但这仍然只提供单帧 cooling hysteresis：

- frame `N` 确认 descendant `800` 为 hot
- frame `N + 1` feedback 暂时不再标热，runtime 仍能保住这条 branch
- frame `N + 2` 如果 ancestor reconnect 直到这时才发生，runtime 已经丢掉这份 confirmed hot truth，又会退回较浅的冷页启发式

这说明 confirmed frontier truth 还没有真正收口到更深的 recycle / reconnect 归并点。

## Delivered Slice

### 1. `recent_hot_resident_pages` 从单帧 set 升级成 bounded cooling window

`VirtualGeometryRuntimeState` 现在把 `recent_hot_resident_pages` 改成 `BTreeMap<page_id, frames_remaining>`，并定义固定的 `HOT_FRONTIER_COOLING_FRAME_COUNT = 2`。

`refresh_hot_resident_pages(...)` 的行为改成：

- 先衰减已有 cooling entries
- 再把上一拍 `current_hot_resident_pages` 重新注入 cooling window
- 最后再写入当前 feedback 的 `hot_resident_pages`

因此 confirmed hot frontier 可以跨两次 cooling feedback frame 继续影响后续 recycle / reconnect，而不是只存活一拍。

### 2. recycle / completion / page-table apply 继续消费同一份 cooling truth

以下路径不需要新增私有分支，直接复用升级后的 `page_is_frontier_hot(...) / frontier_hot_resident_pages(...)`：

- `ordered_evictable_pages_for_target(...)`
- `complete_pending_pages(...)`
- `complete_gpu_uploads_with_replacements(...)`
- `apply_gpu_page_table_entries(...)`

同时 `register_extract(...)` 和 `evict_page(...)` 也会同步修剪这份 cooling window，确保 live-scene 边界与 resident ownership 仍然是 authoritative source。

### 3. cooling cascade 现在既更深也仍然有界

新增两条回归把语义钉死：

- `virtual_geometry_runtime_state_carries_confirmed_hot_frontier_lineage_through_two_cooling_frames_before_next_prepare`
  - 证明 confirmed hot descendant 在两次 cooling frame 之后，仍能让 ancestor reconnect recycle colder `400`，而不是更深的 `800`
- `virtual_geometry_runtime_state_drops_confirmed_hot_frontier_lineage_after_cooling_budget_expires`
  - 证明 cooling window 不是无限热偏置；预算耗尽后，系统会干净退回 colder-depth ordering

这让 runtime host 既能继续保住 still-live frontier truth，又不会把旧 branch 永久粘住。

## Why This Matters

这一刀把当前 M5 主链从 “single-frame recent hysteresis” 推进到 “bounded multi-frame confirmed frontier cooling cascade”：

- confirmed hot descendant truth 现在能穿过更深的 reconnect 延迟
- feedback completion / GPU completion / page-table apply 不再只共享一拍 recent
- extract prune / eviction 仍然能把这条 truth 在 live-scene 边界上及时清掉

它不是 Nanite-like residency manager 的终点，但已经把 still-live runtime merge point 从单拍缓存推进到了更稳定、可验证的多拍冷却级联。

## Validation

- `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_carries_confirmed_hot_frontier_lineage_through_two_cooling_frames_before_next_prepare -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_drops_confirmed_hot_frontier_lineage_after_cooling_budget_expires -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked page_table -- --nocapture`
- `cargo check -p zircon_graphics --offline --locked`

## Remaining Route

- 把这份更深的 confirmed frontier truth 继续压回 `Virtual Geometry` 的 unified indirect / cluster-raster / GPU-generated args authority，减少 renderer 末端仍然残留的 CPU submission 排序痕迹。
- 继续推进更深的 residency-manager cascade，让 page-table / completion / frontier truth 不只支配 recycle 选择，还能继续主导更真实的 cluster-raster submission ownership。
