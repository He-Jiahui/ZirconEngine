---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/consume_feedback.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
plan_sources:
  - user: 2026-04-19 切回 deeper residency-manager cascade / split-merge frontier policy
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-hot-frontier-runtime-residency-cascade.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-frontier-priority-and-active-request-lineage.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_hot_farther_descendant_resident_while_reconnecting_missing_ancestor -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
doc_type: milestone-detail
---

# M5 Virtual Geometry Hot-Descendant Frontier Cascade

## Goal

把 `hot_resident_pages` 这条 split-merge frontier truth 继续推进到更深的 lineage eviction ordering：当系统为了 reconnect 缺失 ancestor page 必须在同一 lineage 内回收 resident descendants 时，recent-hot 的更深 descendant 不能只因为 “lineage distance 更远” 就被先踢掉。

## Delivered Slice

### 1. hot frontier protection 提前到 lineage-distance 之前

`ordered_evictable_pages_for_target(...)` 以前的 key 大致是：

- target relation
- farther lineage distance first
- active request lineage priority
- hot resident flag

这意味着在同一个 target relation / active-request group 里，`hot` 只会在最后当 tie-break。  
结果就是：

- colder nearer descendant
- recently-hot farther descendant

这两者冲突时，runtime 仍会先踢更远的 hot page。

本轮把排序改成：

- target relation
- active request lineage priority
- hot frontier group
- lineage distance

这样 `hot_resident_pages` 在同一 split-merge lineage 内终于拥有真实保护权。

### 2. reconnect missing ancestor 时优先保留 hotter deeper branch

新增回归 `virtual_geometry_runtime_state_keeps_hot_farther_descendant_resident_while_reconnecting_missing_ancestor` 锁定了这条更深 cascade：

- resident pages: ancestor root + nearer descendant + farther descendant
- pending request: 中间缺失 ancestor
- evictable set: 两条 descendant
- hot set: 只有 farther descendant

期望行为现在是：

- colder nearer descendant 被回收
- hotter farther descendant 继续保 resident
- 缺失 ancestor 使用释放出来的 slot 重连 lineage

### 3. 已有 frontier/runtime regressions 保持绿色

这次排序调整没有回退现有两类 contract：

- unrelated page 仍优先于 active-request lineage 被回收
- 没有 hot signal 时，ancestor / descendant 仍按 farther lineage distance first 收敛

所以这不是新策略分支，而是把既有 hot-frontier truth 真正推进到更深一层 runtime eviction authority。

## Why This Slice Matters

前一刀已经把 `hot_resident_pages` 写进 runtime state，并让 current-frame completion 与 next-frame prepare recycle 避免先踢 recently-hot frontier。  
但如果 reconnect ancestor 的时候，系统一旦进入 “必须在同一 lineage 内回收 descendant” 的分支，旧排序还是会退回单纯的 farther-distance heuristic。

这会让 split-merge frontier 在最该稳定的时候抖动：

- 当前帧最热的 deeper branch 被先踢掉
- ancestor 刚补回，下一帧又得重新请求同一条 hot branch

本轮把这条更深层 cascade 收口后，`hot frontier` 不再只保护 unrelated-page recycle，而会继续保护同 lineage 的 deeper active branch。

## Validation Summary

- focused red/green
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_hot_farther_descendant_resident_while_reconnecting_missing_ancestor -- --nocapture`
- broader regressions
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture`

