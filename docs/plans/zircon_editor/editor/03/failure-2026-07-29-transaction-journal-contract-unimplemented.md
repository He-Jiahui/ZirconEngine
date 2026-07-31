---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: transaction-journal-contract-unimplemented
origin_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_workflow_node: M4.1
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_editor/editor/03
fixing_child_dir: docs/plans/zircon_editor/editor/03
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - zircon_editor/src/core/editing/engine/command.rs
  - zircon_editor/src/core/editing/engine/history.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
---

# Editor03 transaction journal contract unimplemented

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 来源执行切片：M4.1 transaction observation and journal implementation audit
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：这是 Editor03 M4.1 内部 transaction-journal 契约缺口；保留本地 failure 用于阻止 M4.2 在 production payload、受管验证和回传完成前被误报通过。
- 跨计划边界：journal storage format belongs to Plan11; crash-recovery consumption belongs to Plan17.

## 失败现象与复现证据

最初审计时，`zircon_editor/src/core/editing/engine/command.rs` 中 `EditCommand::serialize_journal`
仍只有默认实现 `None`。`zircon_editor/src/core/editing/` 内没有任一具体命令覆盖该方法，也没有
transaction-level journal reader, schema contract, or roundtrip test。因此已提交事务无法由 Editor03 提供
可审计的 command journal payload；M4.2 的 journal roundtrip 不能以空 payload 伪装完成。

## 最低共享层根因

事务内核已拥有 transaction identity、history routing、selection before/after 与五态生命周期，但未定义
command journal envelope 的 typed contract，也未规定 command-specific payload 缺失时的 replay/error
semantics。将 `serde_json::Value` 直接作为新的 engine authority 会重新引入未受版本管理的宽快照，并跨越
Plan11 的存储格式职责；仅保留 trait 默认 `None` 则无法为 Plan17 的恢复消费者提供可验证输入。

## 架构修复验收

- 在 Editor03 定义 versioned, typed command-journal envelope，包含 transaction/history identity、label、
  command discriminator 与 payload availability；不得把 untyped JSON 作为 engine 内部 authority。
- 每个可 journal 的 command 明确实现 serialize path；不支持 journal 的 command 必须返回 typed,
  observable unsupported result，不能静默写入空成功记录。
- 建立 transaction journal reader/roundtrip 测试，覆盖 commit payload、schema/version rejection、
  unsupported command、undo/redo identity 与 selection/history metadata preservation。
- Plan11 仅消费稳定 envelope 落盘，Plan17 仅消费 validated envelope 做恢复；不得在任一计划重新创建
  `EditCommand` 私有 history 或旧 operation stack 兼容路径。
- 完成 current-source managed Cargo、independent review 与 Plan03 fixed return 后才可关闭本记录。

## 禁止临时方案

- 禁止把 `serialize_journal() == None` 写成空 JSON 对象并称为可恢复。
- 禁止为兼容旧 journal 恢复 `EditorHistory`、`EditorOperationStack` 或命令内 selection snapshot。
- 禁止让存储层或恢复层反向依赖 concrete editor command 的私有字段。

## 修复结果与回传

The typed contract framework is now implemented in
[transaction journal contract r1](2026-07-29-transaction-journal-contract-r1.md): the default is an explicit
typed unsupported result, a versioned `TransactionJournal` preserves transaction/history/selection metadata,
and tests specify roundtrip, schema rejection, unsupported-command, and undo/redo identity behavior.

This handoff remains `open`. No existing production command has yet supplied a concrete versioned
`journal_payload`; Plan11 persistence, Plan17 recovery, current-source managed Cargo, independent review, and
the fixed return are also outstanding. The framework must not be represented as a complete recoverable journal
until those obligations are met.

The first managed current-source copy `3265199b4cd04fe8afae40dee3fb2970` failed in coordinator
`closure_planning` with `validation_copy_external_source_missing`, before an input manifest, Cargo reservation,
Cargo job, run, or test existed. Its terminal is external validation infrastructure evidence only. The framework
then added the atomic `TransactionJournal::decode` reader boundary, so the r1 source attribution is superseded;
the fixing session must create a new immutable r2 manifest instead of reusing that copy.

## 产出记录与时间

- 2026-07-29：状态`open`。完成 M4.1 journal 实现度审计；已确认 trait 默认 `None` 是当前唯一实现，登记本计划债务，未将空 payload 作为验收结果。
- 2026-07-29 16:10 +08:00：状态`open-框架实现待验证`。完成版本化 typed envelope、history/engine reader、selection metadata projection 与专属 TDD 测试；具体 production command payload migration、Plan11/17 消费边界与受管行为验证仍待完成，故未回传 fixed。
- 2026-07-29 16:27 +08:00：状态`open-外部验证物化失败已记录`。r1 copy `3265199b4cd04fe8afae40dee3fb2970` 在 external-source closure planning 终止，未产生任何 Cargo 结论；随后 reader boundary 修正使 r1 源码过期，转由 r2 current-source manifest 继续，不创建重复 Coordinator01 failure。
