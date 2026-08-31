---
handoff_kind: fixed
status: fixed
created_at: 2026-08-28
summary_slug: plugin-module-row-owner-budget
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_export/plugin_validate_modules.py
  - tools/zircon_export/plugin_validate_module_rows.py
  - tools/zircon_export/plugin_validate_feature_provider_module_schema.py
tests:
  - tools/tests/test_plugin_validate_event_component_module_owner_boundaries.py
  - tools/tests/test_plugin_validate_feature_provider_module_owner_boundaries.py
  - tools/tests/test_plugin_validate_owner_boundaries.py
  - tools/zircon_export/tests/test_plugin_validate_modules.py
  - tools/zircon_export/tests/test_plugin_validate_feature_provider.py
resolved_at: 2026-08-31
---

# Plugins13 module row owner budget

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：standalone plugin manifest module validation ownership
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns plugin manifest validation and its owner boundaries.

## 失败现象与复现证据

`plugin_validate_modules.py` exceeded its 300-line owner budget because it
combined manifest traversal with row schema, namespace, capability, crate, and
system semantics.

## 最低共享层根因

The public module-list orchestrator also owned each module row's independent
validation domain. Generated feature-provider validation therefore depended on
the oversized parent instead of a stable row-schema leaf.

## 架构修复验收

- Keep manifest and optional-feature traversal in the parent.
- Keep row schema and value semantics in `plugin_validate_module_rows.py`.
- Route generated feature-provider module validation to the row leaf.
- Preserve public entry contracts and diagnostics.
- Pass owner-boundary, module, and feature-provider behavior suites.

## 禁止临时方案

- Do not raise the owner budget or weaken structural assertions.
- Do not duplicate row validation between parent and generated schema owners.
- Do not change accepted manifest behavior.

## 修复结果与回传

- 根因：Manifest traversal and row semantics shared one owner.
- 架构修复：Commit `84d8d94e4` split row validation into the dedicated leaf
  and routed the generated feature-provider schema consumer directly to it.
  Current source keeps the orchestration parent at 86 lines and row leaf at
  299 lines, both below their enforced budgets.
- 验证：Managed ticket/run `4f48a11b69734919bcd8b756d0bc00fd`, validation
  copy `9ae077c87c9d424d96a57b9e97b1fb09`, and immutable input manifest
  `ae5cdd98876c5c26907874eb199bc752c7e52007d5953f97d9b0f2275f66fa24`
  passed the focused owner-boundary, module, and feature-provider suites 36/36.
- 回传：The open owner-budget failure is removed without widening budgets or
  changing plugin manifest diagnostics.
