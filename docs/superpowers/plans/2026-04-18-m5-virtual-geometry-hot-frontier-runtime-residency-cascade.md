---
related_code:
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_feedback.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/runtime/virtual_geometry/declarations/virtual_geometry_runtime_state.rs
  - zircon_graphics/src/runtime/virtual_geometry/extract_registration.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/consume_feedback.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/src/tests/visibility.rs
implementation_files:
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_feedback.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/runtime/virtual_geometry/declarations/virtual_geometry_runtime_state.rs
  - zircon_graphics/src/runtime/virtual_geometry/extract_registration.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/consume_feedback.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
plan_sources:
  - user: 2026-04-18 仍然是 Virtual Geometry 更深的 split-merge frontier policy / residency-manager cascade
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-repeated-budget-collapse-frontier-hold.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/src/tests/visibility.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_prefers_evicting_cold_page_before_recent_frontier_hot_page_during_feedback_completion -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_carries_recent_frontier_hot_pages_into_next_prepare_recycle_plan -- --nocapture
  - cargo test -p zircon_graphics --offline --locked visibility -- --nocapture
doc_type: milestone-detail
---

# M5 Virtual Geometry Hot Frontier Runtime Residency Cascade

## Goal

把上一轮 visibility-only 的 repeated budget-collapse frontier hold 继续向 runtime residency-manager cascade 下沉，让 “最近仍然应该保热的 resident frontier” 不只停在 `VisibilityHistorySnapshot` 和 `evictable_pages` 裁剪上，而是继续影响：

- 当前帧 `consume_feedback(...)` 的 resident-slot 回收顺序
- 下一帧 `build_prepare_frame(...)` 的 recycled-slot 规划

## Non-Goal

- 本轮不新增新的 GPU uploader pass。
- 本轮不改动 `VirtualGeometryPrepareRequest` / `frontier_rank` 的 GPU contract。
- 本轮不重写 `build_virtual_geometry_plan(...)` 的 hierarchy refine / request scoring。

## Delivered Slice

### 1. Visibility feedback 现在显式导出 hot resident frontier

`VisibilityVirtualGeometryFeedback` 新增：

- `hot_resident_pages`

它表示：

- 当前仍 resident
- 当前不在 visible frontier 上
- 当前没有进入 `evictable_pages`

的 hidden frontier page 集。

这条集合不只覆盖 repeated budget collapse，也覆盖 merge-back child hold、多级 ancestor request pending 等已经落地的 split-merge hysteresis 分支。

### 2. Runtime host 现在缓存这条 hot frontier truth

`VirtualGeometryRuntimeState` 新增：

- `current_hot_resident_pages`

`consume_feedback(...)` 会先用 feedback 刷新它，再执行当前帧 pending promotion；`register_extract(...)` 也会在场景收缩时裁掉已经离开 extract 的 stale hot page，避免旧 frontier 热度穿过场景边界继续存活。

### 3. Eviction ordering 现在在 runtime 侧也保护 recently-hot frontier

`ordered_evictable_pages_for_target(...)` 先继续保留已有排序层级：

- unrelated / ancestor / descendant
- 更远的 lineage distance 优先回收
- 其他 active request lineage 的保护顺序

在这些更强规则之后，又新增一层 tiebreak：

- colder page 先回收
- `current_hot_resident_pages` 内的页后回收

这使得同属 “unrelated / 无其他更强约束” 的候选页，不会再仅因 page id 或输入顺序靠前，就先回收刚刚还支撑着 split-merge frontier 的 resident page。

### 4. Next-frame prepare recycle plan 也开始吃这条 truth

`pending_page_requests(...)` 的 assigned-slot 规划本来就会复用 `ordered_evictable_pages_for_target(...)`。因此当上一帧 feedback 已经把 hot frontier truth 写回 runtime state 后：

- 下一帧 `build_prepare_frame(...)`
- 下一帧 `VirtualGeometryPrepareRequest.recycled_page_id`

也会优先复用 colder resident page，而不是把最近活跃过的 frontier page 当成普通 unrelated page 回收。

## Why This Slice Exists

上一轮已经补齐了：

- repeated budget collapse 时的 child-frontier history carry-over
- ancestor request pending 时的 deeper descendant hold
- requested-lineage vs streaming-target-lineage 的 collapse 收窄策略

但这些 truth 还主要停在 visibility 侧：

- `evictable_pages` 当帧不会暴露那批 hot resident page
- 可 runtime 一旦重新面对新的 unrelated request 或下一帧 slot recycle，缺少显式 “recently-hot frontier” 真值，仍可能把这批页按普通 unrelated candidate 排到最前面

本轮补上的就是这条 runtime-host 级缺口。

## Validation Summary

- `virtual_geometry_runtime_state_prefers_evicting_cold_page_before_recent_frontier_hot_page_during_feedback_completion`
  - 证明当前帧 feedback completion 会保留 recently-hot frontier page，优先回收 colder unrelated resident page。
- `virtual_geometry_runtime_state_carries_recent_frontier_hot_pages_into_next_prepare_recycle_plan`
  - 证明上一帧 feedback 写回的 hot frontier truth 会继续影响下一帧 prepare 的 recycled-slot 规划。
- `visibility`
  - 证明 visibility-side split/merge / collapse / hysteresis 回归仍然通过，而且新增的 `hot_resident_pages` 导出与已有 frontier hold 结果一致。

## Remaining Route

- 把这条 runtime-host hot-frontier truth 继续向 GPU uploader / page-table completion 下沉，而不只停在 CPU-side recycle order。
- 继续推进更深的 unified indirect / cluster raster / visibility-owned submission authority。
- 继续收敛更深层 page hierarchy 的 split-merge frontier policy，包括更完整的 refine frontier / residency frontier / uploader frontier 统一真值。
