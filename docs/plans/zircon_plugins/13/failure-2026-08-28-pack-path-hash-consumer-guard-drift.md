---
handoff_kind: failure
status: open
created_at: 2026-08-28
summary_slug: pack-path-hash-consumer-guard-drift
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_export/pipeline_report_pack_delta_semantics.py
  - tools/zircon_export/pipeline_report_pack_delta_asset_set_semantics.py
tests:
  - tools/tests/test_zircon_export_pack_manifest_schema_helper_owner_boundaries.py
---

# Plugins13 pack path hash consumer guard drift

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：pack manifest path and hash helper ownership
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns pack report schema and semantic validation boundaries.

## 失败现象与复现证据

The path/hash helper owner guard required
`pipeline_report_pack_delta_semantics.py` to import the helper leaf directly.
Current committed source delegates path/hash-aware asset-set semantics to
`pipeline_report_pack_delta_asset_set_semantics.py`, which owns the direct
import and calls.

## 最低共享层根因

The structural consumer inventory was not rotated when delta asset-set
semantics moved into its named owner. It continued to encode the former
higher-level orchestration file as a direct dependency.

## 架构修复验收

- Name the delta asset-set semantics owner as the direct path/hash consumer.
- Assert the higher-level delta semantics owner has no direct helper import.
- Preserve direct-consumer checks for manifest, delta schema, and trim schema.
- Pass the complete pack manifest helper owner-boundary suite.

## 禁止临时方案

- Do not add an unused path/hash import to the higher-level semantics owner.
- Do not weaken checks for the actual direct consumers.
- Do not move asset-set semantics back into orchestration.

## 修复结果与回传

The guard now follows the committed delegation boundary. Focused validation and
adjacent pack owner suites pass 15/15, and the pack-delta schema behavior
suites pass 37/37. The changed test compiles and the scoped diff gate is clean.
The exact-two coordinator finalizer must reproduce both gates without foreign
worktree inputs.

Open state: `source_validated / failure_return_pending`.
