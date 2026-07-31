Plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
Milestone: M4
Status: validation_external_failure
Files: ["docs/plans/zircon_editor/editor/03/2026-07-29-scene-command-journal-payloads-r1.md", "zircon_editor/src/core/editing/command.rs", "zircon_editor/src/core/editing/selection.rs", "zircon_editor/src/core/editing/engine/command.rs", "zircon_editor/src/core/editing/engine/history.rs", "zircon_editor/src/core/editing/engine/journal.rs", "zircon_editor/src/core/editing/engine/mod.rs", "zircon_editor/src/core/editing/engine/transaction.rs", "zircon_editor/src/tests/editing/transaction_engine/mod.rs", "zircon_editor/src/tests/editing/transaction_engine/journal_scene_commands.rs"]

# Editor03 M4 scene command journal payloads r1

## Scope Delivered

The only production `EditCommand` family, `EditorCommand`, now emits versioned command payloads
instead of the journal contract's typed unsupported result.

- `create_node` serializes its creation intent and the retained `NodeRecord` required for a stable
  completed-command journal.
- `delete_node` serializes the recursive records plus camera and fallback-selection routing facts.
- `update_node` serializes only node identity and typed before/after edit state; its transient
  `already_applied` flag is deliberately excluded.
- `set_reflected_field` serializes node identity, reflected field address, and typed before/after
  reflected values.

Each payload has a stable `zircon.editor.scene.*` discriminator and schema version `1`. No old
history stack, empty JSON success value, or recovery compatibility path is introduced.

## Validation State

- New real-context tests commit all four scene command kinds and assert the resulting journal
  payload type, schema version, and business fields.
- `rustfmt --check`, scoped `git diff --check`, and production `serialize_journal` static scan pass.
- Current-source copy `f49cc0adb755482eafda11692d548c1e` failed in coordinator external-source
  closure planning before an input manifest, Cargo reservation, job, run, or test existed. This is
  not a compile result. Managed Cargo and independent review remain required before this
  implementation can close the journal failure or be included in an M4 commit.

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-07-29 16:47 +08:00 | `实现完成-静态门通过-受管行为门待办` | 为 create/delete/update/reflected 四类 `EditorCommand` 实现版本化 journal payload，并新增真实 CoreEditContext 的 committed-command payload 测试。 | `rustfmt --check`、scoped `git diff --check` 通过；`serialize_journal(` 生产命中为 0。受管 Cargo 与独立 review 尚未完成，未回传 fixed。 |
| 2026-07-29 16:52 +08:00 | `validation_external_failure` | immutable copy `f49cc0adb755482eafda11692d548c1e` 在 external-source closure planning 终止。 | `validation_copy_external_source_missing`；无 input manifest、Cargo reservation/job/run/test。归入 Coordinator01 既有 materialization failure，不作为 Editor03 行为失败或通过。 |
