---
handoff_kind: failure
status: open
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
  - tools/zircon_export/tests/test_plugin_validate_modules.py
---

# Plugins13 module row owner budget

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：standalone plugin manifest module validation ownership
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns plugin manifest validation and its owner boundaries.

## 失败现象与复现证据

The focused PluginValidate owner sweep fails only
`test_modules_lives_in_modules_owner`: `plugin_validate_modules.py` is 370
lines while its enforced owner budget is 300 lines.

## 最低共享层根因

Manifest-level module orchestration and row-level module schema semantics share
one owner. Event consumers, target modes, capabilities, crate identity, and
namespace rules make the row responsibility independently substantial.

## 架构修复验收

- Keep manifest and optional-feature traversal in `plugin_validate_modules.py`.
- Move row schema and value semantics into `plugin_validate_module_rows.py`.
- Rotate generated feature-provider schema validation to the row leaf.
- Preserve the public `validate_plugin_modules` entry contract and diagnostics.
- Keep crate and system validators as lower leaf owners.
- Pass focused owner and module behavior suites.

## 禁止临时方案

- Do not raise the production owner budget to conceal mixed responsibilities.
- Do not duplicate module validation or diagnostics across owners.
- Do not change accepted plugin manifest behavior.

## 修复结果与回传

Module manifest traversal remains in the 86-line orchestration owner, while
row schema and value semantics now live in a 299-line leaf. The generated
feature-provider schema consumer imports that leaf directly. Focused module,
feature-provider, and adjacent owner suites pass 50/50; changed Python files
compile and the scoped diff gate is clean. The exact-five coordinator
finalizer must reproduce these checks without foreign worktree inputs.

Open state: `source_validated / failure_return_pending`.
