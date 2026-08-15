---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-recovery-autosave
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor/17
related_code:
  - zircon_editor/src/core/recovery/tests/autosave_adapter.rs
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib recovery --locked
---

# Editor17: autosave recovery adapter test owner exceeds the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 交接原因：autosave recovery adapter 的恢复、持久化与 retry 覆盖属于 Editor17 services/recovery，不能转移到 host 或 messaging 计划。

## 失败现象与复现证据

当前准确审计在 0 个 test-budget exemption 下报告
`zircon_editor/src/core/recovery/tests/autosave_adapter.rs` 为 1023 行。该 owner 覆盖 autosave recovery adapter
行为，使全局 zero-tolerance structure gate 保持 RED。

## 最低共享层根因

autosave recovery 的 adapter、receipt、failure/retry 和 persistence 场景被追加到单一 flat 测试 owner，
尚未按 recovery 行为域拆成 folder-backed 测试模块。

## 架构修复验收

- 按 autosave/recovery adapter 行为拆分为 folder-backed tests，薄 `mod.rs` 挂载，所有文件不超过 800 行。
- 保留恢复、持久化、失败和 retry 断言语义；共享 fixture 只能拥有一个 owner。
- 不得留下旧 flat file、`#[path]` mount、compatibility shim 或 budget exemption。
- 重审计不再报告该路径；全部 owner 清零后受管 structure gate 才可 GREEN。

## 禁止临时方案

- 不得提高预算、删除 recovery 覆盖，或把该责任转移给 Editor02 messaging/EditorUI08 host。

## 修复结果与回传

Open state: `待修复`。本记录仅完成责任交接，未修改 recovery 源码或测试。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 recovery autosave test-budget handoff | `open` | 从准确 48/0 审计隔离 1023 行 autosave adapter owner。 | 取得源码 lease 后按 recovery 行为 folder-backed 拆分，受管 recovery 回归和结构审计复验。 |
