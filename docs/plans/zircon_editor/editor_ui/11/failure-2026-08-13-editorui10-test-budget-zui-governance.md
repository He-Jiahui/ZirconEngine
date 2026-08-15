---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-zui-governance
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor_ui/11
related_code:
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/workbench_primitives.rs
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib ui --locked
---

# EditorUI11: ZUI governance test owners exceed the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md`
- 交接原因：这两项覆盖 ZUI asset governance 和 workbench primitive retirement/convergence，属于 EditorUI11 的
  suffix/convergence 责任，不属于 MUI 或通用 audit owner。

## 失败现象与复现证据

准确结构审计报告：`tests/ui/boundary/zui_asset_governance.rs` 为 981 行，
`tests/ui/boundary/zui_asset_governance/workbench_primitives.rs` 为 1174 行；test-budget exemptions 为 0。

## 最低共享层根因

ZUI governance、asset policy 和 workbench primitive retirement 回归累积在两个 flat owner 文件，未按治理规则、
asset fixture 与 primitive cutover 行为拆分为 folder-backed 模块。

## 架构修复验收

- 将两项按独立 ZUI governance/cutover 行为拆为 folder-backed 模块，薄 `mod.rs` 仅挂载，每个 Rust 测试文件
  不超过 800 行。
- 保留 suffix convergence、UI TOML retirement、governance 与 workbench primitive 的覆盖语义，不复制 fixture。
- 不得保留 flat compatibility test、`#[path]` mount 或以 exemption 回避预算。
- 重审计不再报告这两项；全局计数清零后，受管 structure gate 才可 GREEN。

## 禁止临时方案

- 不得把 ZUI 责任回退到旧 suffix/compat 路径，不得提高预算或使用 blanket exemption。
- 不得将无关 MUI/component 或 retained-host 覆盖混入本切片。

## 修复结果与回传

Open state: `待修复`。本 handoff 未修改业务测试，不能作为 EditorUI11 或根 failure 的 fixed evidence。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 ZUI governance test-budget handoff | `open` | 从准确 48/0 审计中隔离 2 个 ZUI governance owner，最大项为 1174 行。 | 取得源码 exact lease 后按 cutover 行为 folder-backed 拆分，重跑受管回归和结构审计。 |
