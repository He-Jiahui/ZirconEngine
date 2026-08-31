---
handoff_kind: fixed
status: fixed
created_at: 2026-08-28
summary_slug: distribution-assets-owner-budget
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_plugins/13
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/zircon_export/plugin_validate_distribution_assets.py
  - tools/zircon_export/plugin_validate_distribution_asset_matches.py
tests:
  - tools/tests/test_plugin_validate_distribution_owner_boundaries.py
  - tools/tests/test_plugin_validate_distribution_test_owner_boundaries.py
  - tools/tests/test_plugin_validate_distribution_zui_asset_owner_boundaries.py
resolved_at: 2026-08-31
---

# Plugins13 distribution assets owner budget

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 来源执行切片：plugin distribution asset manifest validation
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugins13 owns the distribution validator and its filesystem leaves.

## 失败现象与复现证据

The distribution-assets validator reached 122 lines while its focused-owner
guard permits 120. The leaf combined manifest syntax and retired-suffix policy
with filesystem glob expansion and plugin-root containment.

## 最低共享层根因

Filesystem match resolution had no named child owner. As a result, the asset
manifest validator also owned Python-version glob normalization, file filtering,
empty-match diagnostics, path resolution, and containment diagnostics.

## 架构修复验收

- Move filesystem glob expansion and plugin-root containment into one named leaf.
- Keep manifest syntax, retired-suffix policy, and ZUI validation dispatch in the
  distribution-assets owner.
- Preserve all existing match and containment diagnostics.
- Keep the assets owner at or below 120 lines and its match leaf at or below 70.
- Pass the complete distribution owner, behavior, and ZUI asset test suites.

## 禁止临时方案

- Do not raise or remove the existing 120-line owner budget.
- Do not trim diagnostics or accepted manifest behavior merely to reduce lines.
- Do not move filesystem matching back into the distribution contract parent.

## 修复结果与回传

- 根因：Filesystem match resolution had no dedicated child owner, so the
  distribution-assets validator mixed manifest policy with path expansion and
  containment diagnostics.
- 架构修复：Commit `62d3728a4` moved glob normalization, file filtering,
  empty-match diagnostics, and plugin-root containment into the focused
  `plugin_validate_distribution_asset_matches.py` leaf. The parent retains
  manifest validation and ZUI dispatch without raising its owner budget.
- 验证：Managed ticket/run `2ca2135b32554babae931e431c5b4e8e`, validation copy
  `54f3ead30246415092bac157b25315e6`, and immutable input manifest
  `2dd6470d0560c94b0261ead5c7bc3ace433e4009d502eba72c5d82b5ef473f6b`
  passed the focused distribution owner, behavior, and ZUI asset suites 14/14.
- 回传：The owner-budget failure is removed from the open graph while the
  original diagnostics and accepted asset-manifest behavior remain intact.

