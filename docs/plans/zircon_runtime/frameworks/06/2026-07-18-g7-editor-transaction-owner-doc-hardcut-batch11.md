---
related_code:
  - zircon_editor/src/core/editing/engine/history.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
implementation_files:
  - docs/editor-and-tooling/editor-command-workflow.md
  - docs/editor-and-tooling/index.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/editor-and-tooling/editor-command-workflow.md docs/editor-and-tooling/index.md
---

# Frameworks06 G7 Editor Transaction Owner 文档硬切 Batch 11

Status: completed_focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-18
Session: `frameworks06-g7-editor-transaction-owner-doc-hardcut-batch11-20260718`

## 完成项目

- 将两份 Editor 命令文档中已删除的 `zircon_editor/src/core/editing/history.rs` owner 硬切到唯一现存且已跟踪的 `core/editing/engine/history.rs`。
- 将旧 `EditorHistory` 叙述同步到当前 `EditorTransactionEngine` 提交事务、`HistoryStore` 按 `HistoryContextId` 分区保存 undo/redo 状态的真实边界。
- 明确 gizmo drag 与普通命令共用 transaction engine/history context，不添加旧 owner alias、shim、兼容重导出或第二套历史栈。

## Fresh Testing Evidence

- 修改前 fresh G7：`482` violations / `130` documents。
- 修改后 fresh `python tools/check_conventions.py --only docs --json`：所选两份文档 `0` violations；共享 current-source 全局快照为 `478` violations / `128` documents，G7 继续保持 RED。
- 两份文档内旧机器路径与 `EditorHistory` 术语均为 `0`；current owner 已由 Git 跟踪，并分别存在 `HistoryStore` 与 `EditorTransactionEngine` 声明。
- exact-scope `git diff --check` 通过，staged_total 为 `0`。

## Review

独立只读复审首轮发现 1 个 Important：Gizmo 时序仍把 End 误写为起点/终点比较；修正为每个 preview step 累积 `last→current` 增量、End 补最后增量并以 `MergeMode::Ends` 提交。复审随后指出 1 个 Minor：父计划覆盖规模仍是写入记录前快照；由于并发文档持续改变该规模，父计划已删除易变的 documents/paths 总数。Batch12 继承复审又发现 2 个 Important：普通命令错误声称构造阶段修改 World，以及 `EditorTransactionEngine`/gizmo capture 的真实 owner 未完整列入路径字段；现已按 `engine/transaction.rs` 的 `TransactionScope::push -> EditCommand::apply`、`TransactionScope::commit -> HistoryStore` 时序修正，并补 `engine/transaction.rs` 与 `editor_state_viewport.rs` owner。最终复审为 **Critical 0 / Important 0 / Minor 0**。

## 里程碑判定

本批 focused G7 与独立复审已通过。Frameworks06 M1 和计划 06 仍为 `in_progress`；全局 478 条 missing path 及真实分支 CI 仍待后续批次关闭。
