---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-08-31
summary_slug: native-dynamic-directory-consumer-guard-drift
origin_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
plan_link_mode: child_record_only
source_artifact: docs/plans/zircon_plugins/13/failure-2026-08-28-native-dynamic-directory-consumer-guard-drift.md
---

# native-dynamic-directory-consumer-guard-drift 回传摘要

- 状态：`fixed`
- 回传工件：[fixed-2026-08-31-native-dynamic-directory-consumer-guard-drift.md](fixed-2026-08-31-native-dynamic-directory-consumer-guard-drift.md)
- 验证证据：Managed copy `8b72823cf16049bcbd314e09ed064c9c` / run
  `2ad7311c68ae4eaf8094538ec0b6acad` passed 14/14 from immutable input manifest
  `9adc772e5165b6eb1d8bc7eb0e397293a62919f08647a93f07efd21b7847e566`.
- 摘要：Plugins13 directory-helper ownership now follows the committed
  bundle-evidence delegation boundary without modifying the clean production
  owners.
