---
handoff_kind: failure
status: open
created_at: 2026-08-28
summary_slug: native-dynamic-directory-consumer-guard-drift
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_export/pipeline_report_native_dynamic_payload_platform_bundle.py
  - tools/zircon_export/pipeline_report_native_dynamic_payload_bundle_evidence.py
tests:
  - tools/tests/test_zircon_export_native_dynamic_payload_directory_owner_boundaries.py
---

# Plugins13 NativeDynamic directory consumer guard drift

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：NativeDynamic directory payload helper ownership
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns NativeDynamic export report evidence boundaries.

## 失败现象与复现证据

The directory-payload owner guard required the PlatformBundle payload
orchestrator to import directory helpers directly. Current committed source
delegates current-bundle verification to its bundle-evidence leaf, which
directly imports and calls the directory helper owner.

## 最低共享层根因

The static consumer inventory was not rotated when current-bundle evidence
moved out of PlatformBundle payload orchestration. It encoded the former file
location instead of the current direct dependency edge.

## 架构修复验收

- Name the bundle-evidence leaf as the direct directory-helper consumer.
- Keep the NativeDynamic stage-payload owner in the direct consumer inventory.
- Assert the PlatformBundle orchestrator has no direct directory-helper import.
- Pass the focused directory owner and adjacent payload owner suites.

## 禁止临时方案

- Do not add an unused directory-helper import to the orchestrator.
- Do not weaken the direct-consumer check for the actual evidence leaf.
- Do not move current-bundle evidence back into orchestration.

## 修复结果与回传

The guard now follows the committed evidence delegation boundary. Focused
directory, bundle-evidence, and PlatformBundle handoff owner checks plus payload
behavior pass 14/14. The changed test compiles and the scoped diff gate is
clean. The exact-two coordinator finalizer must reproduce these suites without
foreign worktree inputs.

Open state: `source_validated / failure_return_pending`.
