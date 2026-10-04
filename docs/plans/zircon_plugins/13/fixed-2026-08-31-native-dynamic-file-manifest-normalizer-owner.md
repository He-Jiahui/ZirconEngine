---
handoff_kind: fixed
status: fixed
created_at: 2026-08-28
summary_slug: native-dynamic-file-manifest-normalizer-owner
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/export/native_dynamic_payload.py
  - tools/export/native_dynamic_payload_file_manifest.py
  - tools/export/pipeline_report_native_dynamic_payload_platform_bundle.py
  - tools/export/pipeline_report_native_dynamic_payload_package_report.py
  - tools/export/pipeline_report_native_dynamic_payload_stage_report.py
  - tools/export/pipeline_report_native_dynamic_stage_payload.py
tests:
  - tools/tests/test_zircon_export_native_dynamic_payload_file_manifest_owner_boundaries.py
resolved_at: 2026-08-31
---

# Plugins13 NativeDynamic file manifest normalizer owner

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：NativeDynamic payload file-manifest reporting
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns NativeDynamic export payload generation and reports.

## 失败现象与复现证据

The file-manifest owner guard found a report consumer reaching manifest
normalization through `native_dynamic_payload.py`. The normalizer itself still
lived in that summary facade even though file enumeration, hashing, and path
resolution had already moved to the file-manifest leaf.

## 最低共享层根因

The prior file-manifest split omitted the typed row normalizer and did not
rotate every internal report consumer to the direct owner. This left an
indirect dependency through the payload summary facade.

## 架构修复验收

- Move `normalized_file_manifest` into the file-manifest leaf.
- Preserve the facade import for compatibility with external callers.
- Rotate all internal report consumers to import the leaf directly.
- Expand the consumer inventory to include the stage-report owner.
- Pass the focused owner boundary and NativeDynamic payload report suites.

## 禁止临时方案

- Do not add unused imports merely to satisfy the structural guard.
- Do not duplicate the normalizer in the facade and leaf.
- Do not break the existing facade-level symbol during this ownership move.

## 修复结果与回传

- 根因：The typed NativeDynamic file-manifest normalizer remained in the payload summary facade after file enumeration, hashing, and path ownership moved to the file-manifest leaf, forcing internal report consumers through the wrong owner.
- 架构修复：Move normalized_file_manifest to native_dynamic_payload_file_manifest, keep one facade re-export for external compatibility, and rotate every internal report consumer plus the stage-report inventory to the leaf owner without duplicating normalization.
- 验证：Coordinator validation ticket 85779c43cc01421480e84c5166e342eb passed the exact owner-boundary and package-report modules against source manifest c7bb049b48af06277e5760e774daf4768ebf27472119b4f51142cb3d30aa8443; current HEAD 5798051603e7 rerun passed 17/17; independent review task 01a05b24-f065-79d1-8b7b-fc8f02c9d7ca reported 0/0/0/0 findings.
- 回传：NativeDynamic file-manifest normalization now has one leaf owner; internal consumers use it directly, the compatibility facade re-exports the same function object, and exact source-bound validation plus independent review are green.
