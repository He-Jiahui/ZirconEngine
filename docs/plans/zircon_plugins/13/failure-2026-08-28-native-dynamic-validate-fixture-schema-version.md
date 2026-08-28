---
handoff_kind: failure
status: open
created_at: 2026-08-28
summary_slug: native-dynamic-validate-fixture-schema-version
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_export/pipeline_report_validate_schema.py
tests:
  - tools/zircon_export/tests/export_test_support.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_operation_audit_identity_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_operation_audit_platform_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_operation_audit_schema.py
  - tools/zircon_export/tests/test_pipeline_report_stage_location.py
---

# Plugins13 NativeDynamic Validate fixture schema version

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：NativeDynamic pipeline report behavior sweep
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns export pipeline fixtures and report schemas.

## 失败现象与复现证据

Sixty-two NativeDynamic pipeline assertions fail before reaching their target
stage. Every report has `fatal_stages=['Validate']`; direct reproduction shows
the sole source diagnostic is `non-fatal validate report schema_version must
be 2`, followed by the pipeline fatal-stage summary.

## 最低共享层根因

`_write_validate_report_with_native_dynamic_exports` predates Validate schema
v2 and omits `schema_version`, while the sibling strategies writer already
emits the canonical header. All affected NativeDynamic fixtures share this
single writer.

## 架构修复验收

- Emit `schema_version: 2` from the shared NativeDynamic Validate writer.
- Preserve all existing profile, plan, package export, and stage identities.
- Prove Validate no longer shadows NativeDynamic operation-audit/location
  diagnostics.
- Pass the original 43 operation-audit and 19 stage-location assertions.

## 禁止临时方案

- Do not weaken Validate schema enforcement or suppress fatal stages.
- Do not patch individual report tests with local schema headers.
- Do not change NativeDynamic production report semantics.

## 修复结果与回传

The shared NativeDynamic Validate writer now emits the required schema v2
header. The original 43 operation-audit subcases and 19 stage-location
subcases reach their intended NativeDynamic diagnostics, and all 24 related
pipeline modules pass 389/389 test methods. The shared support owner guard
passes 4/4, the helper compiles, and the scoped diff gate is clean. The
exact-two coordinator finalizer must reproduce the original failure modules
without foreign worktree inputs.

Open state: `source_validated / failure_return_pending`.
