---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: play-pending-edit-unbounded-queue
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/04-pie-and-simulation.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/04
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/play/edit_protection.rs
  - zircon_editor/src/core/play/pending_edits/intent.rs
  - zircon_editor/src/core/play/pending_edits/queue.rs
---

# Editor04 Play pending edit无界队列

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：`P0 / PERF-MVP-551`
- 修复责任计划：`docs/plans/zircon_editor/editor/04-pie-and-simulation.md`
- 交接原因：Play 延期编辑队列的存储、生命周期、退出决策与重试边界归 Editor04；
  Performance01 只保留审计来源与验收要求。

## 失败现象与复现证据

Play期间允许延期的edit被无上限压入`VecDeque<PendingEditIntent>`；每项拥有完整`EditorOperationInvocation`及JSON参数。长session、remote automation或高频property edit可无限增长entries/bytes，退出时`apply_all`又在一个调用中同步处理全部intent；`snapshot()`深clone整队列。

## 最低共享层根因

Play 延期编辑把完整 invocation 直接按到达顺序保存在单一 `VecDeque`，没有由操作所有者
声明的 retention 语义，也没有 entries、payload bytes、oldest age 或 apply turn 的边界。
因此运行时无法区分必须保留的用户事务、可合并的最新状态和可受限的连续输入；退出决策
同时暴露了全量 payload snapshot 与无预算 apply。

## 架构修复验收

- 按operation/target声明lossless/latest/bounded/coalescing语义，queue同时限制entries、payload bytes和oldest age；不可用统一drop策略。
- invocation payload先共享；decision UI只读compact count/bytes/age/cursor，详情分页。Apply使用count+time/job budget，失败intent保留唯一重试authority。
- 1/1k/100k edits、64B/1MiB payload、0/60min stall记录queue/clone/apply/RSS/p95；内存硬有界、full snapshot clone=0、退出不单帧无界apply。
- 保持running-document lock、target/order、apply/discard/failure/retry与decision receipt；Cargo/F4和独立review通过。

## 禁止临时方案

- 禁止静默丢transaction terminal或用户明确操作；禁止把无界队列搬到UI/notification缓存。
- 禁止只限制entry数量而允许单项JSON无限大。

## 修复结果与回传

Open state: `typed retention、entry+bytes+age 预算、paged decision 与 budgeted apply 正在
Editor04 当前源中实现；当前静态检查不等于性能验收。受管 immutable validation-copy 请求
0fe3733c4fa14ae6b48fc68947e2b9b6 已 accepted，尚未返回 source copy 或 Cargo 终态，故本
failure 仍保持 open。`

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-22 | Performance01 P0 / PERF-MVP-551 | open | 逐文件性能审查登记；要求由 Editor04 处理 retention、队列预算、分页决策和预算化 apply。 |
| 2026-07-27 | Editor04 pending-edit retention recovery | resolving_failure | 已取得 12 个精确源码租约；当前源码以 `Arc` 持有 invocation，显式 lossless/latest/bounded retention，entries/bytes/oldest-age 入队预算，紧凑分页游标、预算化 apply、失败唯一重试 authority 与紧凑 discard 均已实现。补充了两页游标前进、绕过构造器的空 cohort 边界拒绝，以及 oldest-age 拒绝新入队但保留已有 intent 的行为测试；`rustfmt --check`、`git diff --check` 与 `python tools/tests/test_editor04_play_edit_protection_contract.py`（4/4）通过。source-bound `validation-copy materialize-cargo` 使用固定 `zr_vm` commit `503fb72163cd20ddf32a38f8a330083712f5d648`，请求 `0fe3733c4fa14ae6b48fc68947e2b9b6` 已完成但因 1.68 MiB 响应 tombstone 未返回 source-copy job ID；未启动或接受 Cargo 结果。 |
