---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
plan_sources:
  - user: 2026-04-19 切回 deeper residency-manager cascade / split-merge frontier policy
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-frontier-priority-and-active-request-lineage.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-hot-frontier-runtime-residency-cascade.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_hot_later_request_lineage_resident_while_completing_new_target_page -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture
doc_type: milestone-detail
---

# M5 Virtual Geometry Hot-Request-Lineage Frontier Cascade

## Goal

把 `hot_resident_pages` 的保护权继续压进 active-request arbitration：当 runtime 在多个 active request lineages 之间选择回收对象时，recent-hot 的 later lineage 不能只因为它属于“较晚 request”就被优先踢掉。

## Delivered Slice

### 1. hot frontier protection 现在先于 later-request eviction order 生效

`ordered_evictable_pages_for_target(...)` 之前虽然已经会优先回收 unrelated page，并在 active-request lineages 之间偏向 later request，但这条 later-request order 会抢在 hot signal 前面生效。

现在排序改成：

- relation to current target
- active request group
- `hot_resident_pages`
- later-request ordering
- lineage distance

这意味着 recent-hot 的 later lineage 不会再被单纯的 request order 覆盖掉。

### 2. hot later lineage 在 target promotion 时可以继续保 resident

回归 `virtual_geometry_runtime_state_keeps_hot_later_request_lineage_resident_while_completing_new_target_page` 锁定了新的 runtime contract：

- 当前必须为新的 target page 释放一个 resident slot
- 另有一条 later active request lineage 仍然 resident
- 如果那条 later lineage 处于 hot frontier，系统现在会优先回收较冷的 earlier lineage，而不是机械地先踢 later request

## Why This Slice Matters

split-merge frontier 稳定性不只取决于 ancestor/descendant 距离，也取决于“当前哪条 lineage 仍在真实支撑画面”。  
如果 hot signal 不能压过 later-request eviction order，那么 runtime 就会在 reconnect / promote 新 target page 时先踢掉刚刚还在服务当前画面的 hot lineage，下一帧又被迫把它重新请求回来。

这条 cascade 收口后，active-request arbitration 终于开始真正消费 current frontier truth，而不是只消费 request queue 的静态顺序。

## Validation Summary

- focused red/green
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_hot_later_request_lineage_resident_while_completing_new_target_page -- --nocapture`
- broader regressions
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture`

