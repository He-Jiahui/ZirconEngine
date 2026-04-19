---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/runtime/virtual_geometry/test_accessors.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/runtime/virtual_geometry/test_accessors.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
plan_sources:
  - user: 2026-04-19 让 confirmed frontier truth 在更深的 recycle / hold / reconnect 级联里继续主导
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_does_not_let_removed_hot_descendant_bias_page_table_reconnect_frontier_merge -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_test_evictions_clear_frontier_truth_before_later_reconnect_prepare -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked page_table -- --nocapture
  - cargo test -p zircon_graphics --offline --locked confirmed_virtual_geometry_completion_deduplicates_replacement_truth_after_page_table_normalization -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Page-Table Reconnect Frontier Merge

## Goal

继续把 `Virtual Geometry` 的 confirmed frontier truth 压进 still-live runtime merge point 本身，避免 authoritative page-table apply 之后仍然残留“已经离场的 hot descendant 继续偏置 reconnect/recycle”的旧热度。

## Problem

上一轮已经把 frontier truth 推进到了：

- `recent_hot_resident_pages`
- `complete_pending_pages(...)`
- `complete_gpu_uploads_with_replacements(...)`
- `apply_gpu_page_table_entries(...)`
- `evict_page(...)`

但 `apply_gpu_page_table_entries(...)` 里还剩一条更深的残余分叉：

- 它会先抓取 `previous_hot_resident_pages`
- 再根据 final GPU page table 驱逐已经消失的 resident page
- 然后把 lineage-based hot inheritance 写回 newly reconnected page

这样一来，如果 hotter descendant 已经不在 final page table 里，但 reconnect 的 ancestor 又在同一帧进入 authoritative resident set，runtime apply 仍然会把那份已经失效的 descendant frontier truth 继续带给 ancestor。

这不是 “replacement keep-hot” 的合法继承，而是 authoritative page-table merge 之后仍然保留了一份不再存活的 lineage residue。

## Delivered Slice

### 1. lineage 继承现在只信 final page-table 里仍然存活的 hot source

`apply_gpu_page_table_entries(...)` 现在把两类 hot 继承源明确拆开：

- 同 slot 替换 hot resident page：仍然允许 carry-forward
- 沿 ancestor/descendant lineage 继承 hot frontier：只允许来自 final page table 中仍然存活的 confirmed hot pages

这样：

- direct replacement 不会被错误打断
- 但已经从 authoritative resident table 消失的 hot descendant，也不会再通过 lineage 路径继续给 reconnect page 残留热度

### 2. 新增红绿回归证明 removed descendant 不再污染 later reconnect

新增回归：

- `virtual_geometry_runtime_state_does_not_let_removed_hot_descendant_bias_page_table_reconnect_frontier_merge`

它覆盖的场景是：

- descendant `800` 在上一拍仍然是 hot frontier
- final GPU page table 已经把 `800` 移出 authoritative resident set
- ancestor `200` 通过 page-table apply 被 reconnect 到新的 resident slot
- 下一拍再请求 unrelated page `700`

修补后，runtime recycle 会正确把 colder `200` 视为优先回收目标，而不再因为 `800` 的 stale frontier residue 改去先踢 unrelated `300`。

### 3. test-side eviction 也收敛到同一条 runtime merge 语义

同一轮顺手补了一个 test-harness 级风险：

- `runtime/virtual_geometry/test_accessors.rs::apply_evictions(...)`

此前它直接改 `resident_slots/free_slots`，会绕过 `evict_page(...)` 对 `current_hot_resident_pages/recent_hot_resident_pages` 的同步清理。

现在它已经直接复用 `evict_page(...)`，因此测试辅助路径与真实 runtime eviction 语义一致，不会再制造“生产代码已修，但测试仍能保留假 hot truth”的伪状态。

## Why This Matters

这条修补把 frontier truth 的 authority 再往下压了一层：

- 不只是 completion 之后 carry-forward
- 不只是 extract/runtime registration 时 prune stale state
- 而是 authoritative final page-table apply 自身也开始区分：
  - 哪些 hot source 仍然存活
  - 哪些只是已经离场的 residue

这样 runtime residency cascade 的 still-live merge point 才真正开始收束到：

- confirmed final resident set
- confirmed replacement truth
- confirmed surviving hot frontier truth

而不是继续让 “上一拍曾经 hot，但这拍已经被 authoritative table 删除”的页，绕过 merge point 再影响下一拍 recycle/hold/reconnect。

## Validation

- `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_does_not_let_removed_hot_descendant_bias_page_table_reconnect_frontier_merge -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_test_evictions_clear_frontier_truth_before_later_reconnect_prepare -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked page_table -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked confirmed_virtual_geometry_completion_deduplicates_replacement_truth_after_page_table_normalization -- --nocapture`
- `cargo check -p zircon_graphics --offline --locked`

## Remaining Route

- 继续审计是否还存在不经过 `evict_page(...)` / `apply_gpu_page_table_entries(...)` / `complete_*` 主链的 residency merge side path。
- 如果没有新的 production 级漏口，这条 M5 VG frontier/residency 收口线就已经从 extract、feedback completion、GPU completion、page-table apply 一直打通到 authoritative runtime merge point。
