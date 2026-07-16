---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-07-16
summary_slug: global-gpu-lane-exclusivity-and-reservation-fifo
origin_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
plan_link_mode: child_record_only
---

# global-gpu-lane-exclusivity-and-reservation-fifo 回传摘要

- 状态：`fixed`
- 回传工件：[fixed-2026-07-16-global-gpu-lane-exclusivity-and-reservation-fifo.md](../../../zircon_runtime/render/18/fixed-2026-07-16-global-gpu-lane-exclusivity-and-reservation-fifo.md)
- 摘要：Render18 继续仅通过受管 job da6f0c1f7eea49bc8b9707e48124145a 执行当前 RenderDoc capture，并负责 heartbeat/finish/release；不得通用 GPU acquire 或 raw Cargo。
