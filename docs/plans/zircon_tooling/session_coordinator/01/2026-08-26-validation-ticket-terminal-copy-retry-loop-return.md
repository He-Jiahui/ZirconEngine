---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-08-26
summary_slug: validation-ticket-terminal-copy-retry-loop
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
plan_link_mode: child_record_only
source_artifact: docs/plans/zircon_tooling/session_coordinator/01/failure-2026-08-26-validation-ticket-terminal-copy-retry-loop.md
---

# validation-ticket-terminal-copy-retry-loop 回传摘要

- 状态：`fixed`
- 回传工件：[fixed-2026-08-26-validation-ticket-terminal-copy-retry-loop.md](fixed-2026-08-26-validation-ticket-terminal-copy-retry-loop.md)
- 摘要：Durable Cargo materialization kind now survives copy cleanup, so the FIFO worker terminalizes a removed failed copy once instead of rematerializing it indefinitely.
