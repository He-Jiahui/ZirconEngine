---
handoff_kind: fixed
status: fixed
created_at: 2026-08-28
summary_slug: native-dynamic-directory-consumer-guard-drift
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/tests/test_zircon_export_native_dynamic_payload_directory_owner_boundaries.py
tests:
  - tools/tests/test_zircon_export_native_dynamic_payload_directory_owner_boundaries.py
  - tools/tests/test_zircon_export_native_dynamic_payload_bundle_evidence_owner_boundaries.py
  - tools/tests/test_zircon_export_native_dynamic_payload_platform_bundle_handoff_owner_boundaries.py
  - tools/zircon_export/tests/test_platform_bundle_native_dynamic_pipeline_payload.py
resolved_at: 2026-08-31
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

- 根因：The static guard described the former orchestration edge instead of
  the current bundle-evidence delegation boundary.
- 架构修复：Commit `d5ffe84d0` names the bundle-evidence leaf and stage payload
  as direct directory-helper consumers while explicitly rejecting a direct
  PlatformBundle orchestrator import.
- 验证：Managed ticket/run `2ad7311c68ae4eaf8094538ec0b6acad`, validation
  copy `8b72823cf16049bcbd314e09ed064c9c`, and immutable input manifest
  `9adc772e5165b6eb1d8bc7eb0e397293a62919f08647a93f07efd21b7847e566`
  passed the focused directory, bundle-evidence, PlatformBundle handoff, and
  payload behavior suites 14/14.
- 回传：The stale consumer inventory is removed from the open graph without
  changing either clean production owner.
