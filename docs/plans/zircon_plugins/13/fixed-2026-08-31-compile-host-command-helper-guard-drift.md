---
handoff_kind: fixed
status: fixed
created_at: 2026-08-28
summary_slug: compile-host-command-helper-guard-drift
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_export/pipeline_report_compile_host.py
tests:
  - tools/tests/test_zircon_export_pipeline_report_compile_host_owner_boundaries.py
  - tools/zircon_export/tests/test_pipeline_report_compile_host_stage_schema.py
resolved_at: 2026-08-31
---

# Plugins13 CompileHost command helper guard drift

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：CompileHost final-report command validation ownership
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns export CompileHost report validation and guards.

## 失败现象与复现证据

The CompileHost owner guard required six helper definitions from the retired
direct Cargo invocation contract. Current committed code validates the hard-cut
`zircon_build.py` command through generic string-list, option-value, and target
mode helpers, so the stale inventory failed against correct production source.

## 最低共享层根因

The structural function inventory was not rotated when CompileHost stopped
accepting direct Cargo options and moved to the staged Zircon build command.

## 架构修复验收

- Require the current command parsing and target-mode helper definitions.
- Keep all current helpers in the dedicated CompileHost report owner.
- Explicitly reject definitions for the six retired direct-Cargo helpers.
- Pass the CompileHost owner and stage-schema behavior suites.

## 禁止临时方案

- Do not restore retired helper definitions or direct Cargo semantics.
- Do not remove the dedicated CompileHost owner boundary.
- Do not weaken orchestration size or reverse-import assertions.

## 修复结果与回传

- 根因：The structural owner guard retained six helper names from the retired direct-Cargo CompileHost contract after production moved to the staged zircon_build.py command.
- 架构修复：Commit 71dc4b88d rotates the guard to the current string-list, option-value, and target-mode helpers and explicitly rejects the six retired direct-Cargo helpers without changing production semantics.
- 验证：Fresh focused validation passed 18/18: CompileHost owner and stage-schema suites 10/10; command-semantics and staged-report suites 8/8. All lifecycle and declared owner paths were clean before return.
- 回传：Plugins13 CompileHost owner guard now matches the hard-cut staged-build command contract; the stale helper inventory failure is fixed.
