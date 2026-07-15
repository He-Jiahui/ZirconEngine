---
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
  - tools/tests/test_runtime_init_level_naming.py
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/script/vm/host_interface/descriptor.rs
  - zircon_runtime/src/script/vm/host_interface/registry.rs
  - zircon_runtime/src/text/cache/shaped_cache.rs
  - zircon_runtime/src/text/parallel/shape_pool.rs
  - docs/zircon_runtime/structure/module-convention.md
implementation_files:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - python -m unittest tools.tests.test_runtime_init_level_naming -v
  - python .Codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - git diff --check -- .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py tools/tests/test_runtime_init_level_naming.py docs/plans/zircon_runtime/runtime/15/2026-07-16-runtime-editor-naming-owner-classification.md docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md docs/zircon_runtime/structure/module-convention.md
doc_type: milestone-detail
status_anchor: runtime_15_runtime_editor_naming_owner_classification_unclassified_zero_render_debt_pending
---

# Runtime 15 Runtime Editor Naming Owner Classification

## 状态与完成项目

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M2 | Runtime editor token 最低真实 owner 分类 | `runtime_15_runtime_editor_naming_owner_classification_unclassified_zero_render_debt_pending` | 2026-07-16 | editor unclassified 从 73 降为 0；聚焦 Python 回归 4/4 通过。23 处 scene reflection metadata、42 处 script operation descriptor/capability/registration metadata、8 处 `cfg(test)` text product fixture 被精确归类。 |

## Owner 决策

- `zircon_runtime/src/scene/components/scene.rs` 的 23 个新增命中只描述反射可见性提示，归入既有 `scene-reflection-editor-visible-metadata`；序列化数据没有吸收 Editor authoring state。
- `zircon_runtime/src/script/**` 的 42 个命中属于 typed editor-operation contribution descriptor、capability、registration 与反射迁移样例，归入 `script-editor-operation-contribution-descriptor`；命令执行和编辑事务仍由 Editor owner 持有。
- `shaped_cache.rs` 与 `shape_pool.rs` 的 8 个命中只存在于 `#[cfg(test)]` product fixture，归入 `runtime-text-editor-product-fixture`；没有为整个 text 子树开放路径豁免，也没有改变生产 shaping/cache 行为。
- 分类器只接受精确 `editor_hint` token、8 个已审查 script owner 和两个精确 text 文件中的 `cfg(test)` item；未新增兼容 shim、公开 API 或生产分支。

## 验证

- TDD 红态先确认上述路径仍落入 `editor.unclassified_locations`；实现后 `python -m unittest tools.tests.test_runtime_init_level_naming -v` 为 4/4 通过，并以负例锁定 scene/script/text 分类边界。
- 直接 `runtime_naming_boundary_audit` 返回 `gate_status=classified`、`editor.unclassified_location_count=0`、`legacy.unclassified_location_count=0`。
- 分类计数为 scene reflection metadata 35（包含既有 12 与本轮新增 23）、script descriptor 42、text test fixture 8。
- 完整聚合结构审计和输出记录审计作为本切片最终提交门禁执行。

## 未关闭范围

Runtime15 父计划继续 `in_progress`。当前命名未分类位置已经归零，但仍保留 8 处
`legacy-runtime-graphics-debt`，其中 2 处同时触发 graphics hard-cut wording；这些路径属于
Render owner，必须在对应活动范围中硬切，不能通过扩大分类豁免关闭。
