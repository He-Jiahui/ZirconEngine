---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-command-reflection
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor/03
related_code:
  - zircon_editor/src/tests/editing/reflected_command/mod.rs
  - tools/tests/test_editor03_scene_transaction_hardcut_contract.py
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib editing --locked
---

# Editor03: reflected-command test owner exceeds the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：reflected command contract 属于 Editor03 command/transaction 领域，不能并入 Editor07 UI asset 测试切片。

## 失败现象与复现证据

准确结构审计在 0 个 test-budget exemption 下报告
`zircon_editor/src/tests/editing/reflected_command.rs` 为 870 行。该 command reflection 覆盖属于
Editor03 command/transaction 领域，必须移出 flat owner 以解除 zero-tolerance gate RED。

## 最低共享层根因

reflected command 的 command shape、fixture 与 execution/assertion 场景在一个 flat 测试文件持续增长，
尚未按具体 command contract 建立 folder-backed 边界。

## 架构修复验收

- 依 command reflection 行为拆成 folder-backed modules，薄 `mod.rs` 只挂载，所有测试文件不超过 800 行。
- 保留 command/transaction 语义、fixture 和断言覆盖；共享 helper 不能被复制。
- 不得留下旧 flat file、`#[path]` compatibility mount、exemption 或无关 domain 迁移。
- 重审计不再报告该路径；全局 owner 清零后受管 structure gate 才能 GREEN。

## 禁止临时方案

- 不得提高预算、删除 reflected-command 覆盖或将其转嫁给 Editor07 UI asset tests。

## 修复结果与回传

Open state: `source_split_complete / managed_behavior_validation_pending`。
稳定 Session 为 `failure-cleanup-editor03-reflection-20260905`；继续使用现有 Editor03
身份，保留上方原始路径和复现行数。

2026-09-06 当前 928 行 flat owner 已物理删除，替换为 6 行 `mod.rs`、5 个行为模块和
唯一 `support.rs`。命令执行、schema 编辑权限、快照投影、Inspector 写入、多选事务分别
保留 2/3/3/4/3 个测试；最大文件为 319 行。与编辑前 HEAD 源码以相同 Rust 2024
rustfmt 归一后比较，15 个测试及 6 个辅助函数的函数体均一致，无测试删除、重复或忽略。
共用的 editing 夹具直接从原 owner 导入，没有复制或兼容挂载。

现有 Python 选集合同直接读取 `reflected_command/multi_selection.rs`，原断言保留。
完整 `tools.tests.test_editor03_scene_transaction_hardcut_contract` 本地运行 13/13 通过
（2.126 秒）；递归 `rustfmt --edition 2024 --check` 和精确 diff-check 通过。
当前结构审计不再报告原始路径，全仓为 12 个超限测试、3 个超限生产文件、2 个重复测试树、
0 个预算豁免；该统计不代表全局结构门通过。

四份现行模块文档的测试入口同步到规范目录。其与 Python 合同的源码归属通过
ownership transfer `aaad11a481db4b9ebb3af05deeb9c73c` 明确交给同一 Editor03 Session，
指纹为 `faaef53de77d49dd0d8a87ae800683c6378d7281d874964871fc9bcc4be6bcaf`。
原始 `cargo test -p zircon_editor --lib editing --locked`、上行结构门、独立审查和
fixed return 仍待完成。遵循用户指令跳过 `zr_vm`；不重复提交已知受该外部工作树阻断的
Cargo 批次，不把格式或 Python 源码合同当作 Rust 行为验收。

快照 `2840` 固定本项 14 个路径（含旧 flat 删除）。受管格式请求
`failure-roll-01a07160-editor03-reflection-format-20260906-r1` 已取得票据
`1340e51177a546b0874528a2b2f8e11c`，源码 manifest hash 为
`f485b0b44bf96a22962a673e5073e026c4e19facf952c558a50b408a56517141`。
回执为 `queued`，由 `navigation-bake-selection-operation-arguments`、
`reflection-write-post-commit-read` 等现有计划依赖返回 `validation_dependency_failed`。
尚未执行 rustfmt；保留原票据，不重复提交同链 Python/Cargo 请求，继续独立 failure。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 command reflection test-budget handoff | `open` | 从准确 48/0 审计隔离 870 行 reflected-command owner。 | 取得源码 lease 后按 command contract folder-backed 拆分，受管 editing 回归和结构审计复验。 |
