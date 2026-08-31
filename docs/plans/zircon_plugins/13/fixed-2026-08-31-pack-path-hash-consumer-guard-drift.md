---
handoff_kind: fixed
status: fixed
created_at: 2026-08-28
summary_slug: pack-path-hash-consumer-guard-drift
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/tests/test_zircon_export_pack_manifest_schema_helper_owner_boundaries.py
tests:
  - tools/tests/test_zircon_export_pack_manifest_schema_helper_owner_boundaries.py
  - tools/tests/test_zircon_export_pack_delta_asset_set_semantics_owner_boundaries.py
  - tools/tests/test_zircon_export_pack_delta_semantics_owner_boundaries.py
  - tools/zircon_export/tests/test_pipeline_report_pack_delta_schema.py
  - tools/zircon_export/tests/test_pipeline_report_pack_delta_schema_clean.py
  - tools/zircon_export/tests/test_pipeline_report_pack_delta_top_level_schema.py
resolved_at: 2026-08-31
---

# Plugins13 pack path hash consumer guard drift

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：pack manifest path and hash helper ownership
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns pack report schema and semantic validation boundaries.

## 失败现象与复现证据

The path/hash helper owner guard required the higher-level pack-delta semantics
orchestrator to import the helper leaf directly. Current committed source
delegates path/hash-aware asset-set semantics to a named leaf, which owns the
direct import and calls.

## 最低共享层根因

The structural consumer inventory was not rotated when delta asset-set
semantics moved into its named owner. It continued to encode the former
higher-level orchestration file as a direct dependency.

## 架构修复验收

- Name the delta asset-set semantics owner as the direct path/hash consumer.
- Assert the higher-level delta semantics owner has no direct helper import.
- Preserve direct-consumer checks for manifest, delta schema, and trim schema.
- Pass the complete pack manifest helper owner-boundary and schema suites.

## 禁止临时方案

- Do not add an unused path/hash import to the higher-level semantics owner.
- Do not weaken checks for the actual direct consumers.
- Do not move asset-set semantics back into orchestration.

## 修复结果与回传

- 根因：The owner guard retained a stale direct-consumer edge after path/hash
  validation moved into the pack-delta asset-set semantics leaf.
- 架构修复：Commit `67e91cc6e` names the leaf as a direct consumer and
  explicitly rejects a direct path/hash helper import in the higher-level
  delta semantics orchestrator.
- 验证：Managed ticket/run `c4cd669b81a24246a0810705511ff32d`, validation
  copy `a9945e9afafe42d1b3bc7ad18f79e566`, and immutable input manifest
  `b4ab6c9795ac2425254fb2bbd9a18cdf840f99d2935a93ef4522ae9d5a6c3591`
  passed the focused owner-boundary and pack-delta schema suites 52/52.
- 回传：The guard now follows the committed delegation boundary without
  modifying or absorbing either clean production semantics owner.
