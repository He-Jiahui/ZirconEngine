---
status: completed
plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixed: docs/plans/zircon_editor/editor/07/fixed-2026-07-17-ui-root-owner-boundary-migration-debt.md
related_code:
  - zircon_editor/src/ui/component_registry
  - zircon_editor/src/ui/preferences
tests:
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json
  - cargo test -p zircon_editor --lib structure_convention --locked
  - cargo test -p zircon_editor --lib appearance_preferences_ --locked
  - cargo test -p zircon_editor --lib retained_registry_includes_material_text_input_contracts --locked
---

# EditorUI10 M1 UI Root Owner Hard Cut

Plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
Milestone: M1
Status: completed
Files: ["docs/plans/zircon_editor/editor/07/fixed-2026-07-17-ui-root-owner-boundary-migration-debt.md", "docs/plans/zircon_editor/editor_ui/10/failure-2026-07-14-ui-root-owner-boundary-migration-debt.md", "docs/zircon_editor/structure/module-convention.md", "docs/zircon_editor/ui/component_registry.md", "docs/zircon_editor/ui/preferences.md", "zircon_editor/src/ui/component_registry.rs", "zircon_editor/src/ui/component_registry/mod.rs", "zircon_editor/src/ui/component_registry/registry.rs", "zircon_editor/src/ui/component_registry/tests.rs", "zircon_editor/src/ui/preferences.rs", "zircon_editor/src/ui/preferences/appearance.rs", "zircon_editor/src/ui/preferences/mod.rs", "zircon_editor/src/ui/preferences/persistence.rs", "zircon_editor/src/ui/preferences/startup.rs", "zircon_editor/src/ui/preferences/tests/mod.rs", "zircon_editor/src/ui/preferences/tests/persistence.rs", "zircon_editor/src/ui/preferences/tests/startup.rs", "zircon_editor/src/ui/preferences/tests/support.rs", "zircon_editor/src/ui/preferences/tests/tokens.rs"]

## Scope delivered

- 物理删除 `ui/component_registry.rs` 与 `ui/preferences.rs`，不保留 shim、旧路径 re-export 或双实现。
- 组件注册表与 appearance preferences 分别进入 folder-backed responsibility owner；测试按真实责任就近拆分。
- 同步模块文档、EditorUI10 计划状态、Editor07 fixed return 与结构审计镜像。

## Fresh testing evidence

- Python 结构审计：`classified-and-clear`，迁移债、root owner violation、oversized、production dead-code、banned-name、duplicate-test 均为 0。
- 偏好 managed job `7471e8a6feb040cf9f9eddb0fdd9c291` / run `cabceceafbb44cb3b54288c00c06373b`：12 passed / 0 failed / 1 ignored，exit 0；ignored 为显式截图 exporter。
- 组件 managed job `f99cf3b9efec4cf9ba69b4757c60b706` / run `332b3e0237db4bb7809e55ef619f28ed`：1 passed / 0 failed，exit 0。
- 结构 managed job `c7c5a95862824ef6ae04cf08117b076d` / run `ac6e0936c7ad4c509f25c0c20dca5d6a`：3 passed / 0 failed / 3342 filtered，exit 0。

## Review

- 独立只读首轮复审 Critical/Important/Minor = 0/0/0，确认薄 façade、无兼容路径、可见性与 sibling 访问正确，组件 1 项和偏好 12 项行为语义完整保留。
- fixed return 后最终复审先报告 0/1/2：canonical fixed 缺新 owner/行为门明细、结构文档仍有 FIFO pending、child frontmatter 仍指向已删除 failure；三项均已修正，同一审阅者 re-review 为 0/0/0。
- failure graph 对 `ui-root-owner-boundary-migration-debt` 无诊断；schema48 受管 return 已生成规范 fixed record 并删除源 failure。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-07-17 | M1.T1 root owner RED baseline | `verified-red` | 当前源码执行 Editor 结构审计，得到 `m1_gate_status = migration-debt-present`、`migration_debt_count = 2`、`ui_module_owner_boundary_violation_count = 2`；精确旧路径为 `ui/component_registry.rs` 与 `ui/preferences.rs`。 | 禁止修改审计白名单或只用目录 re-export 旧文件；必须物理删除旧 owner。 |
| 2026-07-17 | Component registry owner hard cut | `verified-managed-green` | 旧 root 文件删除；新树为 `component_registry/{mod,registry,tests}.rs`。`mod.rs` 只声明 owner 并精选导出 crate-private 入口，catalog merge 行为仍由 `registry.rs` 单一拥有，原 `retained_registry_includes_material_text_input_contracts` 测试完整保留。 | fresh managed exact 1/1 与独立复审 0/0/0 已完成。 |
| 2026-07-17 | Appearance preferences owner hard cut | `verified-managed-green` | 旧 root 文件删除；`mod.rs` 只导出 startup 入口，appearance DTO、versioned persistence、env/startup resolution 与 typography migration 分属具名 owner；12 项原测试按 tokens/persistence/startup 分组完整保留，无兼容路径。 | fresh managed 12/12 与独立复审 0/0/0 已完成。 |
| 2026-07-17 | Structure audit GREEN 与独立复审 | `verified-script-review-and-cargo-green` | `audit_editor_structure.py --json` 返回 `classified-and-clear`、`migration_debt_count = 0`、`ui_module_owner_boundary_violation_count = 0`，同时 oversized/dead-code/banned-name/duplicate-test 均为 0；旧两个 `.rs` 路径均不存在，文件级 rustfmt 与 scoped `git diff --check` 通过。独立只读复审 Critical/Important/Minor = 0/0/0，确认薄 façade、无旧路径/shim、sibling 可见性正确且 1+12 测试语义完整。 | 受管结构 3/3、偏好 12/12、组件 1/1 与 failure fixed return 已完成；仅 managed milestone commit 待执行。 |
| 2026-07-17 | Preferences 与 component registry managed GREEN | `verified-behavior-green` | 偏好 reservation `35549272507946d484861aa78fbab472`、job `7471e8a6feb040cf9f9eddb0fdd9c291`、run `cabceceafbb44cb3b54288c00c06373b` 执行 `appearance_preferences_`，12 passed / 0 failed / 1 ignored / 3316 filtered，exit 0；ignored 项是显式截图 exporter，不属于 12 项行为合同。组件 reservation `b9859e2edccf439da0ea1abe9ba8c4ef`、job `f99cf3b9efec4cf9ba69b4757c60b706`、run `332b3e0237db4bb7809e55ef619f28ed` 执行 retained registry 精确门，1 passed / 0 failed / 3329 filtered，exit 0。 | 行为门已完成，结构门与 fixed return 见后续记录。 |
| 2026-07-17 | Structure convention managed GREEN 与 fixed return | `verified-3-of-3-return-ready` | reservation `248825f9529748c89684c10d3a26dc1c`、job `c7c5a95862824ef6ae04cf08117b076d`、run `ac6e0936c7ad4c509f25c0c20dca5d6a` 执行 `cargo test -p zircon_editor --lib structure_convention --locked --jobs 1 -- --test-threads=1`，3 passed / 0 failed / 3342 filtered，exit 0。结合 Python audit 0 项迁移债、行为门 12/12 + 1/1 与独立复审 0/0/0，满足 failure 回传条件。 | 创建 Editor07 fixed record，执行 lifecycle return 与 managed milestone commit。 |
| 2026-07-17 | Coordinator schema48 failure return | `fixed-returned` | lifecycle key `Editor07|EditorUI10|ui-root-owner-boundary-migration-debt` 已由受管 `failure return` 原子转换；源 failure 删除，规范回传落在 `editor/07/fixed-2026-07-17-ui-root-owner-boundary-migration-debt.md`。 | 执行最终复审与 immutable managed milestone commit。 |
| 2026-07-17 | M1 protected-plan split 与 exact business closeout | `verified-gates-accepted` | 两份受保护父计划定义已由 maintenance commit `a5e832a8f15e08b5008ea52f2321b02c12568685` 精确提交；successor Session `editorui10-ui-root-owner-hardcut-closeout-r2-20260717` 将其从业务清单移除并绑定剩余 20 路不可变 manifest，run `500cb97d321244dda312163ccf77cff9`。独立 review `965413869b614efc82036b453539f26b` 为 Critical/Important/Minor = 0/0/0；隔离 validation job `52f07b01afe042e1b8470c6bd352f156` 已完成并清理，M1 commit manifest、failure audit、plan output、review 四门均为 accepted，索引为 0。 | 由受管 `milestone commit` 同步 20 路业务 manifest。 |

## 架构边界

- `ui/mod.rs` 的稳定模块名不变；变化只发生在物理 owner 结构，不保留旧文件 shim。
- `component_registry/mod.rs` 和 `preferences/mod.rs` 不承载行为。
- 偏好 persistence 与 startup 不合并，避免 env/IO 策略重新污染 token DTO owner。
- 历史测试结果只作为回归基线；本记录不会把尚未执行的 current-source Cargo 门写成通过。
