---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-07-17
summary_slug: screen-space-ui-text-font-id-report-mount-drift
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
plan_link_mode: child_record_only
---

# screen-space-ui-text-font-id-report-mount-drift 回传摘要

- 状态：`fixed`
- 回传工件：[fixed-2026-07-17-screen-space-ui-text-font-id-report-mount-drift.md](../../text/01/fixed-2026-07-17-screen-space-ui-text-font-id-report-mount-drift.md)
- 摘要：Runtime15 restored the real production mount and converged font-id reporting to a single shaping-query/actual-glyph owner without aliases or shims; Text01 may resume its upper gateway gate.
