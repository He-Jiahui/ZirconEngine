---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: saved-top-compare-and-mark-save-token
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
origin_workflow_node: M3
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_editor/editor/03
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editing/engine/command.rs
  - zircon_editor/src/core/editing/engine/history.rs
  - zircon_editor/src/core/editing/engine/mod.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/core/editing/engine/transaction/save_token.rs
  - zircon_editor/src/core/editing/engine/transaction/operation_group.rs
tests:
  - tools/tests/test_editor03_saved_top_save_token_contract.py
  - zircon_editor/src/tests/editing/transaction_engine/history.rs
  - zircon_editor/src/core/asset/dirty/tests.rs
  - zircon_editor/src/tests/editing/transaction_engine/operation_group.rs
---

# Editor03 saved_top 原子 compare-and-mark 保存令牌

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行者：`editor09-dirty-registry-saved-top-projection-r1-20260722`
- 来源执行切片：Editor09 M3.1 DirtyRegistry、save/save_all 与关闭询问编排
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：`saved_top` 是 Editor03 唯一事务脏态权威，Editor09 不得在外层复制历史游标或新增保存基线。

## 失败现象与复现证据

现有公开合同只有 `history_snapshot(history)` 与 `mark_saved(history)` 两个独立操作。保存编排若先读取
top、执行写盘、再调用 `mark_saved`，保存期间提交的新事务会被后一次调用错误标成已落盘；即使写盘后
再次比较 `HistorySnapshot.top`，比较与 mark 之间仍存在竞态，而且相同 top 下 redo 分支替换也不能只靠
数组下标识别。Editor09 因此不能安全完成 save/save_all 或在成功后清外部效应位。

## 最低共享层根因

Editor03 的 saved baseline 更新缺少由 transaction identity/generation 约束的单锁 compare-and-mark
原语。把两步协议放到 DirtyRegistry、DocumentToolkit 或 UI host 会复制 history owner，并在并发提交、
undo/redo、branch replacement 与容量淘汰时产生不同真相。

## 架构修复验收

- Editor03 提供 typed save token，至少绑定 `HistoryContextId`、保存开始时的 current transaction
  identity/empty-root identity 与 branch generation；不得只存 top 数组下标。
- `mark_saved_if_unchanged(token)` 在 transaction engine 同一受管操作/锁内验证并更新 `saved_top`；
  history 已变化时返回 typed `ChangedDuringSave`，不得标记当前较新 top。
- token 在 undo/redo、redo 截断重写、容量淘汰、history clear、跨 document 使用和重复 completion 下
  均有确定 typed 结果；不能 panic 或静默接受旧 token。
- 保存写盘失败不调用 compare-and-mark；compare 失败时磁盘结果可记录为 stale save，但内存仍 dirty。
- focused 测试覆盖“保存中提交新事务”“相同 top 分支替换”“保存中 undo/redo”“空 history”与
  多 document save_all；独立 review 0/0/0，并形成受管 commit SHA。
- Editor09 只消费该原语；外部效应 ledger 按自身 revision token 清除，transaction dirty 不缓存。
- token capture/completion 触发的 operation-group flush 不得先移除 group 再执行可失败 commit；初始化
  reservation 必须在 begin 前发布并由 `begin_transaction` 同锁校验 owner，失败清理只作用于匹配 identity。

## 禁止临时方案

- 禁止在 Editor09 保存前禁用全部事务来掩盖缺失原语，或以 UI 单线程假设作为正确性合同。
- 禁止用 `history_snapshot().top == captured_top` 后再调 `mark_saved()` 的两调用方案。
- 禁止在 DirtyRegistry 保存一份平行 `saved_top`、dirty bool 或 transaction generation。

## 修复结果与回传

Open state: `Editor03 exact14 已完成原子 saved_top token、operation-group flush/初始化并发修复、静态合同与独立复审；仍等待受管 current-source Cargo、fixed return 与 managed SHA。Editor09 M3.1 不在这些证据完成前伪造 save completion`。

## 产出记录与时间

- 2026-07-22：状态 `open`。已确认现有 snapshot→I/O→mark_saved 两调用存在新事务误清脏态竞态，
  将原子 token 责任路由至 Editor03；Editor09 只推进安全的 saved_top 实时投影与 external-effect ledger。
- 2026-07-22：状态 `open / source_complete_static_green_review_clean_cargo_blocked`。Editor03 exact14
  已实现 engine-bound `HistorySaveToken` 与原子 `mark_saved_if_unchanged`，删除旧 transaction `mark_saved`，
  并覆盖 commit、same-top branch、undo/redo、empty root、cross-engine/document、active scope、repeat、
  capacity/clear 与 multi-document。为保证 token 两端 flush 安全，同时补齐 pre-begin identity reservation、
  begin 同锁 capability 校验、失败可重试、successor 隔离及首 push rollback 原错保留。Editor03+Editor09
  静态合同 `11/11`、exact rustfmt、旧 API 扫描和 diff-check 通过；两路独立终审均为 `0/0/0`。受管 Cargo/
  fixed return/managed SHA 尚未完成；candidate snapshot936 preview 为 exact14 `14/14` 无漂移，故 failure
  仍保持 open。
