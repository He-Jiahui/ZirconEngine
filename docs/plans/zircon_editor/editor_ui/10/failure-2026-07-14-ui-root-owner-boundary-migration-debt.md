---
handoff_kind: failure
status: open
created_at: 2026-07-14
summary_slug: ui-root-owner-boundary-migration-debt
origin_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
fixing_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_editor/editor/07
fixing_child_dir: docs/plans/zircon_editor/editor_ui/10
related_code:
  - zircon_editor/src/ui/component_registry.rs
  - zircon_editor/src/ui/preferences.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/editor_structure_audits/module_convention_boundary.py
tests:
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json
  - cargo test -p zircon_editor --lib structure_convention --locked
---

# EditorUI10：UI 根层仍有两个单文件 owner 边界债务

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 来源执行切片：Editor07 优先失败关闭前的 `engine-code-structure-convention` 结构门复验
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 交接原因：Editor07 只消费 typed document/animation toolkit owner；`ui/` 根层的模块形态与 folder-backed owner 收敛属于 EditorUI10 M1，不能由领域编辑器切片顺手移动。

## 失败现象与复现证据

2026-07-14 在当前源码运行 `audit_editor_structure.py --json`，得到
`m1_gate_status = migration-debt-present`、`migration_debt_count = 2`、
`ui_module_owner_boundary_violation_count = 2`。精确违规为：

- `zircon_editor/src/ui/component_registry.rs`
- `zircon_editor/src/ui/preferences.rs`

同次审计的 `oversized_production_file_count` 与
`production_dead_code_suppression_count` 均为 `0`，因此当前剩余信号不是大文件或
dead-code 噪声，而是 UI 根层 owner 形态没有完成结构硬切。

## 最低共享层根因

两个 owner 仍以 `ui/*.rs` 顶层单文件存在，没有进入可继续按责任拆分的 folder-backed
owner 树。`component_registry` 的行为归组件注册表，`preferences` 的行为归 Editor17
settings/appearance 迁移；但二者在 `ui/` 根层的物理模块边界由 EditorUI10 M1 统一治理。

## 架构修复验收

- 将 `component_registry.rs` 硬切为 `ui/component_registry/` 下的薄 `mod.rs` 与具名责任叶子，注册、合并和查询保持单一真源。
- 将 `preferences.rs` 按 Editor17 settings owner 裁决后硬切到 folder-backed owner；若由 Editor17 吸收，则删除旧 `ui/preferences.rs`，不得保留路径 shim。
- 更新全部调用点与结构镜像；旧物理文件为不存在，不保留双路径、兼容 re-export 或 façade 包装旧实现。
- `audit_editor_structure.py --json` 恢复 `classified-and-clear`，`migration_debt_count = 0` 且 `ui_module_owner_boundary_violation_count = 0`。
- 回跑 `cargo test -p zircon_editor --lib structure_convention --locked`，再由功能 owner 补相应组件注册表与偏好持久化行为门。

## 禁止临时方案

- 禁止放宽审计白名单、改名规避分类器或仅增加目录后继续从旧单文件 re-export。
- 禁止把组件注册表和 appearance preferences 合并成一个无主的 `ui/common`/`utils` 模块。
- 禁止由 Editor07 为通过自身门代改这两个跨功能 owner。

## 产出记录与时间

| 日期 | 里程碑/切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-14 | EditorUI10 M1 `ui/` owner 边界复验 | `open-待功能 owner 处理` | 当前结构审计精确报告 `component_registry.rs` 与 `preferences.rs` 两项 root owner 违规；其余大文件与 production dead-code 计数为 0。本条只登记失败与验收边界，不声明结构门完成。 |

## 修复结果与回传

Open state: `待修复`; no pass is claimed. 完成后由 EditorUI10 联合组件/设置功能 owner 回传 Editor07 的全局结构门结果；Editor07 的两个既有失败关闭不以本项为旁路前提。
