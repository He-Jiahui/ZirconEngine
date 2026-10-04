---
related_code:
  - zircon_editor/src/ui/retained_host/app/invalidation.rs
  - zircon_editor/src/ui/retained_host/app/invalidation
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/invalidation_bridge
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/invalidation
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/diagnostics.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - inline tests: 6
  - rustfmt check: passed 9/9
  - scoped whitespace check: passed
  - current-source managed Windows Cargo pending
  - pointer-storm and F4 invalidation counter trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained-host invalidation当前源码复核（2026-07-30）

## 范围

`zircon_editor/src/ui/retained_host/app/invalidation.rs`与`invalidation/**`当前源 **9/9** 个Rust文件、**359** 行、**6** 条`#[test]`已逐文件阅读；path+raw-content SHA-256为`7c7e7787b6d3ffb60a91822b62a19cc66ec07ac3920be523db7467b302fa04eb`。范围内无未提交源码差异，本轮未修改Rust。

| 模块 | 文件 | 行 | 测试 | 当前边界 |
|---|---:|---:|---:|---|
| root export/mask | 2/2 | 73 | 0 | module出口、9-bit dirty domain与常数位运算 |
| mask requirements/summary | 2/2 | 73 | 0 | domain依赖判定与Verbose reason格式化 |
| invalidation root | 4/4 | 132 | 0 | pending bitset、request/rebuild counters与diagnostics snapshot |
| root tests | 1/1 | 81 | 6 | paint/render/layout分流、consume/drain与counter合同 |

## 发现

- **正向边界**：pending recompute是一个`u16` bitset，重复请求按位合并，不形成主线程队列、Vec或无界积压；layout/presentation/render/hit/window依赖判定均为常数位运算。paint-only/pointer-hover/viewport-image不进入slow recompute，6条测试锁定这一分流。
- `summary()`的临时Vec/join和`stats_summary()`的format只在`diagnostic_log_allows(Verbose)`之后执行；默认诊断关闭时没有这些String分配。`diagnostics_snapshot()`只复制3个`u64`，没有宽DTO或锁。
- root同时保留pending mask与4个legacy dirty bool，`begin_recompute_invalidation_phase`会union两者。它是PERF-MVP-106迁移到domain generation前的双写兼容层，但每请求仍为O(1)，本轮不把常数状态误报为独立瓶颈。
- **PERF-MVP-601 / 高频相同值写入**：`use_committed_pointer_layout()`的注释要求pointer路由复用last committed frames，但函数唯一动作是`publish_refresh_invalidation_diagnostics()`。当前有 **33** 个pointer/scroll/drag/resize调用点；每个稳定事件都会借用UI state的`RefCell`并重写同一个`HostInvalidationDiagnostics { 3×u64 }`，即使counter未变化且overlay不可见。
- counter变化已经有明确发布点：paint-only request递增后、slow-path rebuild后、render submit前与startup/tick边界都会publish。因而pointer helper中的相同值写入可由EditorUI08删除或用generation/changed guard收敛；不得顺手让pointer事件重建layout，也不得延迟真实counter到present之后。本轮因跨host diagnostics语义需要批准，未直接改Rust。
- 新viewport image的paint-only路径会递增counter并立即publish，再请求region redraw；这是确保下一次present读到新counter的有效边沿。profiling feature关闭时UI perf counter调用编译为no-op。是否需要把每frame3×`u64` publish改为present-time capture，只由WPR/规模counter裁决，不先建立第二缓存。

## 参考与目标

- Godot `dev/godot/scene/main/canvas_item.cpp:143-180,540-551`用一个`pending_update`合并重复redraw，并到draw结束才清除，避免递归更新。Zircon invalidation root的bitset合并方向正确；diagnostics也应随counter generation发布一次，而不是随每个observer入口重复写。
- Bevy `dev/bevy/crates/bevy_winit/src/state.rs:705-732`在一次redraw request广播后清除flag。Zircon保持自己的region damage与counter合同，但稳定pointer observer不应成为重复diagnostics publication源。

EditorUI08让`HostInvalidationRoot`拥有monotonic diagnostics generation，只有slow/render/paint-only counter变化时向host window提交；`use_committed_pointer_layout`继续只表达“读last committed layout”的边界，不产生状态写。若保留setter，则same diagnostics必须no-op且有counter证明；最终只保留一个invalidation authority，随PERF-MVP-106删除legacy bool双写。

## 动态验收

按pointer events `1/1K/1M`、rates `125/500/1000Hz`、dirty mode `none/paint-only/render/presentation/layout`、viewport frames `1/300/10K`运行stable与1% changed。记录invalidate requests、pending mask、diagnostics generations/setter calls/`RefCell` writes、counter publishes、slow/render rebuild、String alloc、UI thread p50/p95和F4 overlay值。

验收要求：稳定pointer的diagnostics setter/write=0；重复dirty仍只有一个pending bit且无队列内存；Verbose off的summary/stats String alloc=0；每次slow/render/paint-only counter变化在下一present前发布一次且不丢、不重复；paint-only不升级slow path。managed Cargo、规模counter、independent review和F4产品trace完成前保留在`pending.md`，不进入`review.md`。
