---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-08-31
summary_slug: native-dynamic-validate-fixture-schema-version
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
plan_link_mode: child_record_only
source_artifact: docs/plans/zircon_plugins/13/failure-2026-08-28-native-dynamic-validate-fixture-schema-version.md
---

# native-dynamic-validate-fixture-schema-version 回传摘要

- 状态：`fixed`
- 回传工件：[fixed-2026-08-31-native-dynamic-validate-fixture-schema-version.md](fixed-2026-08-31-native-dynamic-validate-fixture-schema-version.md)
- 验证证据：Managed ticket/run `8a8283d4d79e467b8d858f6c3ab14232`, copy
  `d599973a929d4bb0afef75f404df2a72`, and immutable input manifest
  `69000aa406c6e4ebdf8740624a8ac143ea76edc123bb2903e2ce23a0b2bae751`
  passed the focused suites 54/54.
- 摘要：The shared NativeDynamic Validate fixture now emits schema v2 once,
  allowing downstream audit and location tests to exercise their real contracts.
