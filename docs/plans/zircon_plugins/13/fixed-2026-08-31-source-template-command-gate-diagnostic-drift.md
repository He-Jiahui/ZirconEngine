---
handoff_kind: fixed
status: fixed
created_at: 2026-08-28
summary_slug: source-template-command-gate-diagnostic-drift
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_export/tests/test_source_template_command_gate.py
tests:
  - tools/zircon_export/tests/test_source_template_command_gate.py
  - tools/zircon_export/tests/test_source_template_build_plan_schema_gate.py
resolved_at: 2026-08-31
---

# Plugins13 SourceTemplate command gate diagnostic drift

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：SourceTemplate and CompileHost behavior sweep
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns SourceTemplate plan and stage validation contracts.

## 失败现象与复现证据

The SourceTemplate/CompileHost sweep had one assertion mismatch. A blank
command entry is correctly rejected with the canonical SourceTemplate Validate
handoff-schema diagnostic, while the downstream command-gate test expected a
retired build-plan diagnostic label.

## 最低共享层根因

The duplicate command-gate assertion was not rotated when malformed plan rows
moved to the SourceTemplate Validate handoff schema gate. That gate now stops
before downstream command semantics and intentionally avoids a duplicate
diagnostic.

## 架构修复验收

- Expect the canonical SourceTemplate Validate handoff-schema diagnostic.
- Preserve fail-closed exit, empty command, and absent project output checks.
- Keep downstream Cargo command semantics diagnostics for schema-valid plans.
- Pass focused command/schema gates and the SourceTemplate/CompileHost sweep.

## 禁止临时方案

- Do not emit a duplicate downstream diagnostic after schema failure.
- Do not weaken blank-entry rejection or accept whitespace commands.
- Do not change production diagnostics only to satisfy the stale assertion.

## 修复结果与回传

- 根因：The test retained a retired downstream diagnostic after malformed plan
  validation moved to the canonical SourceTemplate Validate schema gate.
- 架构修复：Commit `668144d01` updates the assertion to the canonical
  handoff-schema diagnostic while retaining all fail-closed output checks.
- 验证：Focused command/schema gates pass 11/11 and the complete SourceTemplate
  and CompileHost sweep passes 94/94. Managed immutable-copy ticket
  `b7089470d5e1404bb47072f83e942eef` passed with source manifest
  `efa7c4b7f0b26f6c3b00873e5a947da798190397e6974afe84e892cf83743a01`.
- 回传：The test follows the production diagnostic boundary without changing
  production diagnostics or absorbing clean implementation files.
