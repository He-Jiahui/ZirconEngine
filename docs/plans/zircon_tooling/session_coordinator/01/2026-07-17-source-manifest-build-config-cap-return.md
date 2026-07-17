---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-07-17
summary_slug: source-manifest-build-config-cap
origin_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
plan_link_mode: child_record_only
---

# source-manifest-build-config-cap 回传摘要

- 状态：`fixed`
- 回传工件：[fixed-2026-07-17-source-manifest-build-config-cap.md](../../../zircon_runtime/render/01/fixed-2026-07-17-source-manifest-build-config-cap.md)
- 摘要：Coordinator01 returned the source-manifest capacity contract to Render01 without taking its directory lease. Sound focused validation has an independent raw compiler failure; any later broad gate must recompute the complete current manifest and cannot use a partial payload.
