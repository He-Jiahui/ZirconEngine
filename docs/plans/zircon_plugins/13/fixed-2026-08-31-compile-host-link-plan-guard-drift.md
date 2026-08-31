---
handoff_kind: fixed
status: fixed
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
resolved_at: 2026-08-31
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

- 根因：The owner inventory encoded the retired root-test location instead of
  the current dedicated legacy-field rejection owner.
- 架构修复：Commit `b7eb49496` keeps general CompileHost schema coverage in the
  root owner and protects legacy `link_plan` rejection in the focused stage
  schema test owner without reintroducing the retired field.
- 验证：Managed validation copy `4d721a1988e34e94a8b634da2ba5fade`, run
  `0d22310219bf475f9e8cfe4f0d2369ff`, and immutable input manifest
  `e24f4c6433806f5c3fa7adbb6534811abbad46a52853a41e26e9ee4e64f77deb`
  passed the owner-boundary and CompileHost schema suites 12/12.
- 回传：The stale owner guard is removed from the open graph and the hard
  cutover remains enforced by the dedicated rejection test.

