---
handoff_kind: failure
status: open
created_at: 2026-08-28
summary_slug: native-dynamic-file-manifest-normalizer-owner
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_export/native_dynamic_payload.py
  - tools/zircon_export/native_dynamic_payload_file_manifest.py
  - tools/zircon_export/pipeline_report_native_dynamic_payload_platform_bundle.py
  - tools/zircon_export/pipeline_report_native_dynamic_payload_package_report.py
  - tools/zircon_export/pipeline_report_native_dynamic_payload_stage_report.py
  - tools/zircon_export/pipeline_report_native_dynamic_stage_payload.py
tests:
  - tools/tests/test_zircon_export_native_dynamic_payload_file_manifest_owner_boundaries.py
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

The typed file-manifest normalizer now lives in the file-manifest leaf. The
payload summary facade re-exports the same function object for compatibility,
and all internal report consumers import the leaf directly. The focused owner
and compatibility suite passes 4/4, payload schema passes 35/35, PlatformBundle
pipeline payload passes 8/8, and package-report behavior passes 12/12. All seven
changed Python files compile and the scoped diff gate is clean. A separate
stage-payload fixture remains pre-existing RED at its Validate stage and does
not enter this import-only finalizer. The exact-eight coordinator finalizer must
reproduce the 4+12 deterministic gate without foreign worktree inputs.

Open state: `source_validated / failure_return_pending`.
