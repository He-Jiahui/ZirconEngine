---
related_code:
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history/construct.rs
  - zircon_graphics/src/tests/visibility.rs
implementation_files:
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history/construct.rs
plan_sources:
  - user: 2026-04-18 仍然是 Virtual Geometry 更深的 split-merge frontier policy / residency-manager cascade
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-requested-lineage-frontier-budget-hold.md
tests:
  - zircon_graphics/src/tests/visibility.rs
  - cargo test -p zircon_graphics --offline --locked visibility_context_keeps_resident_child_frontier_hot_across_repeated_budget_collapse_without_pending_requests -- --nocapture
  - cargo test -p zircon_graphics --offline --locked visibility_context_only_holds_requested_virtual_geometry_lineage_when_frontier_budget_collapses -- --nocapture
  - cargo test -p zircon_graphics --offline --locked visibility -- --nocapture
doc_type: milestone-detail
---

# M5 Virtual Geometry Repeated Budget-Collapse Frontier Hold

## Goal

把 `Virtual Geometry` 的 merge-back / split-merge frontier hysteresis 再推进一层：当 cluster budget 连续多帧都塌回 coarse parent、且当前没有 pending request 时，系统仍然要继续保热上一条 fully-resident child frontier，而不是只保一拍后就把那批 resident child page 重新放回 `evictable_pages`。

## Non-Goal

- 本轮不放宽已有的 requested-lineage collapse policy。
- 本轮不恢复“shared root ancestor 就保热整片 sibling subtree”的旧行为。
- 本轮不改动 runtime uploader / page-table contract。

## Delivered Slice

### 1. Repeated budget collapse 现在会持续携带 recently-active child frontier

`build_virtual_geometry_plan(...)` 现在在以下条件同时满足时，会把 hidden resident child cluster id 继续带进下一帧 history：

- 当前 `requested_pages` 为空
- 该 cluster 上一帧属于 active visible frontier
- 当前帧它自己已不 visible
- 但它仍然位于当前 visible coarse frontier 之下
- 对应 page 仍然 resident

这使得 repeated collapsed frame 不再只看到当前 coarse parent，而是还能记住“最近活跃过的 resident child frontier”。

### 2. Merge-back child hold 不再只活一帧

旧行为里：

- 第一拍 budget collapse 时
  - `merge_back_child_hold_protected_pages` 会保护 hidden resident children
- 第二拍如果仍然 collapsed
  - previous history 已只剩 coarse parent visible id
  - child pages 会重新掉回 `evictable_pages`

新行为则会在“无 pending request 的纯 budget collapse”场景下继续保留 recently-active child frontier history，因此 repeated collapsed frame 仍然会把这批 resident child page 排除在 `evictable_pages` 外。

### 3. Requested-lineage 收窄策略保持不变

这次 carry-over 是显式 gated 的：

- 只有 `requested_pages.is_empty()` 才会保留这条 repeated-collapse child frontier history

所以之前修复过的行为不会回退：

- 当当前帧真的存在 pending request
- 且 budget collapse 只应持续保热 request 自己那条 lineage

系统仍然只依赖：

- `requested_lineage_targets`
- `streaming_target_lineage_targets`

而不会重新把 unrelated sibling subtree 一起钉住。

## Why This Slice Exists

上一轮已经补齐了两种 collapse policy：

- 有 pending request 时，只保热 requested lineage / current streaming target lineage
- merge-back child hold 的首帧保护

但中间仍有一条空洞：

- 当前帧只是 budget 暂时收紧
- 所有 child page 仍然 resident
- 没有 pending request 可以继续提供 lineage hold
- repeated collapsed frame 却会在第二拍直接把刚刚活跃的 child frontier 放回 `evictable_pages`

这会让 cluster budget 抖动时的 residency path 仍然过于敏感。

本轮补上的就是这条 “no-request repeated collapse” 的更宽 hysteresis。

## Validation Summary

- `visibility_context_keeps_resident_child_frontier_hot_across_repeated_budget_collapse_without_pending_requests`
  - 证明 repeated collapsed frame 不会在第二拍就把最近 fully-resident 的 child frontier 放回 `evictable_pages`
- `visibility_context_only_holds_requested_virtual_geometry_lineage_when_frontier_budget_collapses`
  - 证明这次 widened hysteresis 没有把 earlier requested-lineage 收窄策略重新放宽回 sibling overprotection
- `visibility`
  - 证明整条 visibility-side split/merge / cascade policy 仍然回归通过

## Remaining Route

- 继续把这条更宽的 frontier hysteresis 向 runtime residency-manager cascade 下沉。
- 继续推进更深的 unified indirect / cluster raster / GPU submission authority，让 visibility frontier、recycle truth 和 raster consumption 共享同一套更完整的 hierarchy policy。
