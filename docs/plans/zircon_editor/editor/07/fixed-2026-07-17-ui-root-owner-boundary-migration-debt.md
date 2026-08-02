---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: ui-root-owner-boundary-migration-debt
origin_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
fixing_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_editor/editor/07
fixing_child_dir: docs/plans/zircon_editor/editor_ui/10
related_code:
  - zircon_editor/src/ui/component_registry/mod.rs
  - zircon_editor/src/core/settings/mod.rs
  - zircon_editor/src/core/settings/defaults.rs
  - zircon_editor/src/core/settings/io.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/editor_structure_audits/module_convention_boundary.py
tests:
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json
  - cargo test -p zircon_editor --lib appearance_preferences_ --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib retained_registry_includes_material_text_input_contracts --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib structure_convention --locked
resolved_at: 2026-07-17
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
| 2026-07-17 | EditorUI10 M1 root owner hard cut | `code-complete-review-accepted-audit-green-cargo-pending` | 已删除两个旧 root 文件；组件注册表拆为薄 façade、registry owner 与 focused test，偏好拆为 appearance/persistence/startup/migration owners 和 tokens/persistence/startup 三族 tests，原 1+12 测试名全部保留。Python audit 从 2 项恢复为 `classified-and-clear`、`migration_debt_count = 0`、root violation = 0；文件级 rustfmt/scoped diff 通过；独立只读复审 Critical/Important/Minor = 0/0/0。受管 Rust 门、fixed return 与 managed commit 尚未完成。 |
| 2026-07-17 | EditorUI10 M1 behavior gates | `behavior-green-structure-gate-pending` | 受管偏好门 job `7471e8a6feb040cf9f9eddb0fdd9c291` / run `cabceceafbb44cb3b54288c00c06373b` 为 12 passed / 0 failed / 1 ignored；受管组件门 job `f99cf3b9efec4cf9ba69b4757c60b706` / run `332b3e0237db4bb7809e55ef619f28ed` 为 1 passed / 0 failed，二者 exit 0。`structure_convention` 仍 pending，因此 failure 保持 open，不提前 fixed。 |
| 2026-07-17 | EditorUI10 M1 structure gate 与回传 | `fixed-return-verified` | 受管结构门 reservation `248825f9529748c89684c10d3a26dc1c`、job `c7c5a95862824ef6ae04cf08117b076d`、run `ac6e0936c7ad4c509f25c0c20dca5d6a` 为 3 passed / 0 failed / 3342 filtered，exit 0。Python audit、1+12 行为合同、结构门与独立复审全部为绿，已具备向 Editor07 回传 fixed 的完整证据。 |

## 修复结果与回传

- 根因：component_registry and preferences remained multi-responsibility ui root single-file owners
- 架构修复：hard-cut both roots into folder-backed responsibility owners and deleted the legacy physical files without shims
- 验证：结构审计为 `classified-and-clear` 且迁移债/root owner violations 均为 0；偏好 job `7471e8a6feb040cf9f9eddb0fdd9c291` / run `cabceceafbb44cb3b54288c00c06373b` 执行 `appearance_preferences_` 为 12 passed / 0 failed / 1 ignored、exit 0（ignored 为显式截图 exporter）；组件 job `f99cf3b9efec4cf9ba69b4757c60b706` / run `332b3e0237db4bb7809e55ef619f28ed` 执行 retained registry exact 为 1 passed / 0 failed、exit 0；结构 job `c7c5a95862824ef6ae04cf08117b076d` / run `ac6e0936c7ad4c509f25c0c20dca5d6a` 为 3 passed / 0 failed / 3342 filtered、exit 0；独立复审 0/0/0。
- 回传：EditorUI10 root-owner debt is fixed and returned to Editor07 with current-source managed evidence
