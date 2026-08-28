---
handoff_kind: failure
status: open
created_at: 2026-08-28
summary_slug: native-dynamic-package-report-consumer-guard-drift
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_export/pipeline_report_native_dynamic_payload_platform_bundle.py
  - tools/zircon_export/pipeline_report_native_dynamic_payload_package_path.py
  - tools/zircon_export/pipeline_report_native_dynamic_stage_package_report.py
tests:
  - tools/tests/test_zircon_export_native_dynamic_payload_owner_boundaries.py
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

The guard now follows the committed two-leaf delegation boundary. Focused
package-report, package-path, and stage-package owner checks plus package-report
behavior pass 18/18. The changed test compiles and the scoped diff gate is
clean. The exact-two coordinator finalizer must reproduce these suites without
foreign worktree inputs.

Open state: `source_validated / failure_return_pending`.
