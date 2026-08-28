---
handoff_kind: failure
status: open
created_at: 2026-08-28
summary_slug: compile-host-link-plan-guard-drift
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_export/pipeline_report_stage_schema.py
tests:
  - tools/tests/test_zircon_export_pipeline_report_stage_metadata_test_owner_boundaries.py
  - tools/zircon_export/tests/test_pipeline_report_compile_host_stage_schema.py
---

# Plugins13 compile host link plan guard drift

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：pipeline report CompileHost schema test ownership
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns export stage report schemas and their static guards.

## 失败现象与复现证据

The stage-metadata owner guard required a removed
`test_report_stage_accepts_compile_host_link_plan` method to remain in the root
test owner. Current committed schema rejects `link_plan` as a legacy unknown
field, with that contract covered in the dedicated CompileHost stage schema
test owner.

## 最低共享层根因

The structural test inventory retained the pre-cutover acceptance contract
after CompileHost release evidence moved to staged build-command fields and the
legacy link-plan payload was retired.

## 架构修复验收

- Keep general CompileHost report evidence schema coverage in the metadata root.
- Track legacy `link_plan` rejection in the dedicated CompileHost stage owner.
- Assert the rejection test mutates the legacy field and checks its diagnostic.
- Keep the dedicated test owner below 120 lines.
- Pass the owner boundary and CompileHost stage schema behavior suites.

## 禁止临时方案

- Do not restore acceptance of the retired `link_plan` field.
- Do not add an obsolete test method solely to satisfy the stale inventory.
- Do not weaken general CompileHost release-evidence schema coverage.

## 修复结果与回传

The owner guard now follows the committed hard cutover and protects the
dedicated legacy-field rejection. The owner-boundary and CompileHost stage
schema suites pass 12/12, the changed test compiles, and the scoped diff gate
is clean. The exact-two coordinator finalizer must reproduce both suites
without foreign worktree inputs.

Open state: `source_validated / failure_return_pending`.
