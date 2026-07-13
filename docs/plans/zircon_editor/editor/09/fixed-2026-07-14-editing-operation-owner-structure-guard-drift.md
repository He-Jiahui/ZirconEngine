---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: editing-operation-owner-structure-guard-drift
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_editor/editor/03
related_code:
  - zircon_editor/src/core/editing/
  - zircon_editor/src/tests/ui/boundary/editor_event_cutover.rs
tests:
  - cargo test -p zircon_editor --lib --locked tests::ui::boundary::editor_event_cutover::editor_event_owners_are_split_without_the_legacy_aggregate -- --exact --test-threads=1
  - cargo test -p zircon_editor --lib --locked tests::ui::boundary::editor_event_cutover::editor_transaction_context_hard_cuts_the_operation_stack_shape -- --exact --test-threads=1
resolved_at: 2026-07-14
---


# Editor03：Editing operation owner 结构守卫仍要求已移除文件

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-13 | Editor09 M1 当前完整门复现两项 editing/event hard-cut 结构守卫失败；两项均直接要求不存在的 `core/editing/operation_state.rs`，而当前 owner 已拆为 `context.rs`、`history.rs` 与 `engine/{history,transaction,...}`。已交接 Editor03 裁决当前唯一 transaction/history owner 并迁移守卫，不恢复退役聚合文件。 |
| `FIXED / 已回传` | 2026-07-14 | 守卫已硬切到 `editing/context.rs` 与 `editing/engine/{history,transaction}.rs`，并反向断言退役 `operation_state.rs` 不存在。独立 harness 直接编译当前原始测试源，Editor03/Render01 合并结果 6 passed / 0 failed；未恢复 facade、alias 或 compatibility re-export。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：M1 完整 Windows lib-test acceptance 的 editing boundary 聚类
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：Editor03 拥有 command/transaction/undo 与 operation history 结构；Editor09 不拥有该文件布局。

## 失败现象与复现证据

两个 fully-qualified exact 均为 0/1：

- `editor_event_owners_are_split_without_the_legacy_aggregate`：
  `new editor event owner missing: .../core/editing/operation_state.rs`。
- `editor_transaction_context_hard_cuts_the_operation_stack_shape`：读取同一路径时报 Windows `Os code 2`。

当前 `core/editing/` 实际已有 `context.rs`、`history.rs`、`command.rs`，并将执行体拆到
`engine/history.rs` 与 `engine/transaction.rs`；守卫没有跟随 folder-backed owner 演进。完整门日志：
`.codex/tmp/editor09-m1-full-lib-test-r2-20260713.log`。

## 最低共享层根因

Editor03 的 structure test 仍把旧单文件 `operation_state.rs` 当成新架构必需 owner，与当前已拆分的
editing/engine 结构冲突。需要先由 Editor03 确认 canonical transaction/history owner，再让守卫锁定当前
模块边界与退役符号不存在；不能为了源码字符串测试重新放回空壳文件。

## 架构修复验收

- 两个原始 exact tests 基于当前 folder-backed editing owner 验证 transaction/history/context 分层。
- 守卫继续验证旧 operation stack API 与 UI dependency 为零，并覆盖当前 engine module exports。
- 全仓不新增 `operation_state.rs` facade、compat re-export 或只为 `include_str!` 存活的占位文件。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止恢复 `operation_state.rs` 空壳、旧 `EditorOperationStack` 或从新模块 re-export 旧结构。

## 修复结果与回传

- 根因：Editor03 had physically deleted the retired core/editing/operation_state.rs aggregate and split the canonical owner into editing/context plus editing/engine history and transaction modules, but the hard-cut source guard still required and read the deleted file.
- 架构修复：The boundary guard now asserts operation_state.rs remains absent, requires the current folder-backed editing/context and editing/engine owner files, verifies the current engine exports HistoryStore and EditorTransactionEngine, and preserves the zero-legacy-stack and zero-UI-dependency checks. No facade or compatibility file was restored.
- 验证：rustfmt --check and scoped diff check passed. A standalone rustc --test harness directly included the current original editor_event_cutover.rs and support.rs; all three boundary tests passed, including editor_event_owners_are_split_without_the_legacy_aggregate and editor_transaction_context_hard_cuts_the_operation_stack_shape. Combined Editor03/Render01 source-guard run: 6 passed, 0 failed, 9.72s; log .codex/tmp/editor03-render01-guard-standalone-20260714.log.
- 回传：Editor03 operation owner structure guard now follows the canonical folder-backed transaction/history architecture and the retired aggregate remains deleted.
