---
handoff_kind: fixed
status: fixed
created_at: 2026-08-28
summary_slug: feature-provider-parent-owner-budget
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_export/plugin_validate_feature_provider.py
  - tools/zircon_export/plugin_validate_feature_provider_manifest_parse.py
tests:
  - tools/tests/test_plugin_validate_feature_provider_owner_boundaries.py
  - tools/zircon_export/tests/test_plugin_validate_feature_provider.py
resolved_at: 2026-08-31
---

# Plugins13 feature-provider parent owner budget

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：feature-provider package projection validation
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns both the validator entry and its generated-manifest leaves.

## 失败现象与复现证据

The feature-provider projection parent reached 93 lines while its focused-owner
guard permits 90. It still parsed generated package TOML locally after metadata
schema and projection checks had moved to named child owners.

## 最低共享层根因

Generated-manifest decoding remained coupled to package projection
orchestration. The parent therefore owned input decoding in addition to known
field checks, schema dispatch, distribution checks, and extension dispatch.

## 架构修复验收

- Move generated package TOML decoding and its typed diagnostic into one
  manifest-parse leaf.
- Keep package projection and known-field routing in the parent.
- Preserve invalid TOML and non-table diagnostics exactly.
- Keep the parent at or below 90 lines and the parse leaf at or below 45 lines.
- Pass the focused owner boundary and complete feature-provider behavior suite.

## 禁止临时方案

- Do not raise the parent budget or remove its structural assertion.
- Do not duplicate TOML parsing in the schema, extension, or entry owners.
- Do not change accepted manifest values or diagnostics to satisfy the split.

## 修复结果与回传

- 根因：Generated-manifest decoding was coupled to projection orchestration,
  leaving the parent responsible for parsing as well as routing.
- 架构修复：Commit `e2d29a4a9` moved TOML decoding and its typed diagnostics
  into the 30-line `plugin_validate_feature_provider_manifest_parse.py` leaf.
  The parent now remains at 72 lines and retains projection and known-field
  routing without weakening diagnostics or raising the 90-line budget.
- 验证：Managed ticket/run `d97a67484db24bbcbaea27a5a74e230c`, validation copy
  `ced8d81a7b2847799835ac087479598c`, and immutable input manifest
  `4ce12353bbf274892c2ef7bb35b6186bcd5b68b717e2abb7591e715519829ab3`
  passed the focused owner-boundary and feature-provider behavior suites 19/19.
- 回传：The owner-budget failure is removed from the open graph while invalid
  TOML and non-table diagnostics remain unchanged.
