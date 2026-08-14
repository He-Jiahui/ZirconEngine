---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-08-13
summary_slug: host-api-abi-decode-target-cache-rmeta-missing
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
plan_link_mode: child_record_only
source_artifact: docs/plans/zircon_tooling/session_coordinator/01/failure-2026-08-13-host-api-abi-decode-target-cache-rmeta-missing.md
---

# host-api-abi-decode-target-cache-rmeta-missing 回传摘要

- 状态：`fixed`
- 回传工件：[fixed-2026-08-13-host-api-abi-decode-target-cache-rmeta-missing.md](../../../zircon_plugins/01/fixed-2026-08-13-host-api-abi-decode-target-cache-rmeta-missing.md)
- 摘要：Coordinator cleanup now preserves active Cargo targets and emits durable, crash-recoverable deletion provenance; Plugins01 may rerun its focused gate only on a new managed target after FIFO admission, while failed D/E targets and validation copy 5945e3ef29d74bd69602adca02e243b5 remain untouched.
