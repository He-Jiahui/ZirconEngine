---
handoff_kind: fixed
status: fixed
created_at: 2026-08-28
summary_slug: native-dynamic-package-report-consumer-guard-drift
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/tests/test_zircon_export_native_dynamic_payload_owner_boundaries.py
tests:
  - tools/tests/test_zircon_export_native_dynamic_payload_owner_boundaries.py
  - tools/tests/test_zircon_export_native_dynamic_payload_package_path_owner_boundaries.py
  - tools/tests/test_zircon_export_native_dynamic_stage_package_report_owner_boundaries.py
  - tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_package_report.py
resolved_at: 2026-08-31
---

# Plugins13 NativeDynamic package report consumer guard drift

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：NativeDynamic package-report diagnostics ownership
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns NativeDynamic package report and handoff boundaries.

## 失败现象与复现证据

The package-report owner guard required the PlatformBundle payload orchestrator
to import package-report diagnostics directly. Current committed source routes
package paths and stage package reports through two named leaves, and those
leaves directly import and call the package-report owner.

## 最低共享层根因

The direct-consumer inventory was not rotated when package path and stage
package-report validation moved out of PlatformBundle payload orchestration.

## 架构修复验收

- Name payload package-path and stage package-report leaves as direct consumers.
- Assert the PlatformBundle payload orchestrator remains an indirect consumer.
- Preserve ownership assertions for all package-report diagnostic definitions.
- Pass the focused package-report and adjacent owner/behavior suites.

## 禁止临时方案

- Do not add an unused package-report import to payload orchestration.
- Do not weaken definition ownership or reverse-import checks.
- Do not move package path or stage report validation back into orchestration.

## 修复结果与回传

- 根因：The static guard retained the former orchestration edge after package
  path and stage package-report validation moved to named leaves.
- 架构修复：Commit `049721890` names both leaves as direct consumers and
  explicitly rejects a direct package-report import in the PlatformBundle
  payload orchestrator.
- 验证：Managed ticket/run `b864223dae944cd1a9fec050549fb90d`, validation
  copy `15de9137587441c08e41968620e292fd`, and immutable input manifest
  `8472aa0f298a699e1f8619bc6b0174bc81c5c618a6d0ca639f73df895d8c8c70`
  passed the focused package-report, package-path, stage-package owner, and
  package-report behavior suites 18/18.
- 回传：The stale consumer inventory is removed without modifying or absorbing
  the three clean production owners.
