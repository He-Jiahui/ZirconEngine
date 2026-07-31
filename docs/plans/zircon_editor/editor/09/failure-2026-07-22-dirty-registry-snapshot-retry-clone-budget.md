---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: dirty-registry-snapshot-retry-clone-budget
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/09
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/asset/dirty/registry.rs
  - zircon_editor/src/core/editing/engine
related_failures:
  - docs/plans/zircon_editor/editor/03/failure-2026-07-22-history-dirty-batch-generation-contract-missing.md
tests:
  - 1/100/10000 document and effect snapshot scaling
  - concurrent effect mutation retry budget
  - saved-top undo redo and save-token parity
---

# Editor09：DirtyRegistry整批快照复制与重试预算

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-554 DirtyRegistry snapshot retry/clone budget
- 修复责任计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 交接原因：Editor09 拥有 DirtyRegistry effect pairs 与 consumer projection，并消费 Editor03 的 saved-top/dirty generation。

## 失败现象与复现证据

性能计划逐文件审阅确认：单个snapshot过去把effect ID存两份，本轮已低风险改为单份sorted ID与平行revision；但`snapshots/dirty_snapshots`仍先clone所有document effect maps，再逐document调用Editor03 `is_dirty`，任一effect generation变化会让整批最多重做8轮。文档/effect规模或并发保存、导入、UI source-buffer更新可把主线程复制与history锁访问放大。

## 最低共享层根因

DirtyRegistry 缺少跨 Editor03 history 与 external-effect revision 的增量 cursor，只能 clone 全部 effect maps、逐文档查询并在任一 generation 变化后重做整批。

## 架构修复验收

Editor03应提供typed saved-top/dirty batch generation或immutable handle，Editor09只维护compact effect pairs并按changed document增量发布；consumer使用cursor/paged summary。要求stable generation不重建整批、每effect ID唯一owner、changed work近delta、锁持有不随payload bytes增长，且undo/redo/stale revision/save token语义不变。

## 禁止临时方案

禁止增加第四套缓存 dirty bool，禁止以降低重试次数掩盖 false-clean，也不得恢复旧全量 batch API。

## 修复结果与回传

Open：Editor03 已增加 engine-bound `HistoryDirtyCursor`、有界 journal 与
`Unchanged/Delta/Reset` batch；Editor09 已硬切旧 `snapshots/dirty_snapshots`，改为 registry-bound
`changes_since`，stable cursor 返回空，external/transaction change 只发布 changed document，unregister
单独发布 typed removal。1/10,000 文档源码合同已加入，静态 13/13 GREEN。

独立 review 已为 `0/0/0`；current-source Cargo、F4 保存/关闭产品 trace、规模 counter evidence、fixed return 与受管
commit 仍待完成，故继续回链 PERF-MVP-554 且状态保持 `open`。

## 产出记录与时间

- 2026-07-22：完成 Editor03 最低共享层 failure 路由与 Editor09 增量投影源码。旧全量 batch API 已删除，
  无兼容别名、无 transaction dirty 缓存；并发 generation 移动只重试 changed-document projection。
  静态合同 13/13、scoped rustfmt/diff-check 通过；动态验收受 Coordinator01 validation-copy 闭包故障
  阻断，不能改名 fixed。
- 2026-07-22：初审的 retained-journal 全扫描与 counter 假绿已用 generation-indexed
  `VecDeque::range` 及真实 iterator visit counter 关闭；stable=0、single-change=1。undo/redo/clear 与失败
  no-delta 矩阵、当前子计划/模块文档同步完成，两路最终复审均为 `0/0/0`。Cargo/产品 trace/managed SHA
  仍缺，failure 继续 open。
