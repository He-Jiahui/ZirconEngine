---
handoff_kind: fixed
status: fixed
created_at: 2026-08-28
summary_slug: native-dynamic-validate-fixture-schema-version
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_export/tests/export_test_support.py
tests:
  - tools/zircon_export/tests/test_pipeline_report_validate_native_dynamic_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_operation_audit_identity_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_operation_audit_platform_schema.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_operation_audit_schema.py
  - tools/zircon_export/tests/test_pipeline_report_stage_location.py
resolved_at: 2026-08-31
---

# Plugins13 NativeDynamic Validate fixture schema version

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：NativeDynamic pipeline report behavior sweep
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns export pipeline fixtures and report schemas.

## 失败现象与复现证据

The shared NativeDynamic Validate fixture omitted the required schema v2
header. NativeDynamic operation-audit and stage-location assertions therefore
failed at Validate before reaching the diagnostic each test intended to prove.

## 最低共享层根因

The failure artifact named a removed production schema owner, but the actual
defect was confined to `_write_validate_report_with_native_dynamic_exports` in
the shared test fixture. Its sibling fixture already emitted the canonical
header.

## 架构修复验收

- Emit `schema_version: 2` from the shared NativeDynamic Validate fixture.
- Preserve profile, plan, package export, and stage identities.
- Prove Validate no longer shadows NativeDynamic diagnostics.
- Keep production report parsing and schema enforcement unchanged.

## 禁止临时方案

- Do not weaken Validate schema enforcement or suppress fatal stages.
- Do not patch individual tests with local schema headers.
- Do not claim the removed `pipeline_report_validate_schema.py` as the owner.

## 修复结果与回传

- 根因：The shared NativeDynamic Validate test writer predated schema v2 and
  emitted no version header.
- 架构修复：Commit `11cac2d08` added the canonical `schema_version: 2` field
  once in `tools/zircon_export/tests/export_test_support.py`; all consuming
  fixtures inherit it without production semantic changes.
- 验证：Managed ticket/run `8a8283d4d79e467b8d858f6c3ab14232`, validation
  copy `d599973a929d4bb0afef75f404df2a72`, and immutable input manifest
  `69000aa406c6e4ebdf8740624a8ac143ea76edc123bb2903e2ce23a0b2bae751`
  passed the focused Validate, operation-audit, and stage-location suites
  54/54.
- 回传：The obsolete open failure is replaced by this corrected owner record.
