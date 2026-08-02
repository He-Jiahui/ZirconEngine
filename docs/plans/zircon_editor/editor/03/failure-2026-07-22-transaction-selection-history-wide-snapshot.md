---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: transaction-selection-history-wide-snapshot
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/03
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editing/context.rs
  - zircon_editor/src/core/editing/engine/command.rs
  - zircon_editor/src/core/editing/engine/history.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
---

# Editor03 transaction selection与history宽快照

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：`zircon_editor/src/core/{context,commands,editing}`逐文件性能静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：typed selection authority、transaction record 与 compact/paged history projection 均由 Editor03 持有；Performance01 只负责发现和度量热路放大，不能建立第二份事务事实源。
- 性能账本：PERF-MVP-549

## 失败现象与复现证据

`CoreEditContext`把scene selection保存为`SelectionSnapshot(serde_json::Value)`：typed `SceneSelectionSnapshot`读取时先clone整Value再`serde_json::from_value`，写入又经`json!`重建。transaction begin/commit和undo/redo recovery均捕获或恢复该payload，大选择集随每个事务复制/遍历。

`HistoryStore::snapshot`还为最多128个record深clone label、participants、selection_before和selection_after；transaction lifecycle events由无预算`Vec`保留。undo/redo过去只为event id+label也调用完整`record.snapshot()`，本轮已改为compact metadata，但完整history/UI状态与event backlog根因仍在。

## 最低共享层根因

Editor03没有typed immutable selection generation/handle，也未区分O(1) history status、paged record detail和lossless lifecycle stream。用JSON作为内部selection authority把ABI/持久化表示带入MVP事务热路；用一个wide snapshot服务按钮enablement和诊断详情，迫使窄查询复制全部history。

## 跨计划依赖

transaction lifecycle event 的 retention、ack/cursor 与锁外 fanout 是 Editor02 消息契约的最低共享责任，已由 [Editor02 event journal/listener 无界保留交接](../02/failure-2026-07-17-editor-event-journal-listener-unbounded-retention.md) 跟踪。Editor03 只负责 typed selection handle 与 compact/paged history projection；不得在 transaction engine 内另造事件队列或以截断 `Vec` 代替 Editor02 的 lossless bounded owner。该 Editor02 handoff 的架构修复、当前源 1k/10k 证据和 fixed return 是本记录 lifecycle-stream 验收的前置条件。

## 架构修复验收

- SelectionModel发布typed immutable ordered selection handle与generation；transaction record只持before/after handle，内部begin/commit/undo/redo不得执行JSON encode/decode或深clone全部selected IDs。
- `HistoryStatus { can_undo, can_redo, dirty, top identity, generation }`为O(1)窄查询；完整record details使用显式page/cursor，且只复制请求窗口。不得删除现有诊断语义。
- transaction lifecycle event接Editor02 lossless有界owner，按entries/bytes/oldest-age观测；terminal/undo/redo事件不可静默丢弃，paused UI内存必须硬有界。
- selection 1/100/10k、records 1/128、events 1/100k记录serde traversals、copied bytes、record visits、queue age、lock wait与p95；事务热路selection JSON=0，compact status record visits/clone=0。
- 保持nested scope、operation-group merge、participants、save baseline、undo/redo/rollback/finalize与selection恢复顺序；current-source Cargo、F4产品交互和独立review完成后回传。

## 禁止临时方案

- 禁止在UI/SelectionModel外再缓存一份history selection或dirty bool。
- 禁止只把`serde_json::Value`包进`Arc`而保留每次typed decode；内部authority必须typed。
- 禁止为限制内存静默截断transaction terminal events，或让producer等待持有editor state锁。

## 修复结果与回传

Open state: `undo/redo compact event metadata、operation-group key与Create redo局部clone止损已完成；等待typed selection handle、compact/paged history projection、bounded lifecycle stream、Cargo/规模/F4与独立复审`。

## 产出记录与时间

- 2026-07-22：状态`open`。静态复核完成并登记PERF-MVP-549；局部源码守卫/rustfmt/diff通过，但不得据此声明Editor03动态验收或迁入performance `review.md`。
