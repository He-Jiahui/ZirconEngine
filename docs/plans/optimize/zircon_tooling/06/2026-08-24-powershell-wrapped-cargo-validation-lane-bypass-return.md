---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-08-24
summary_slug: powershell-wrapped-cargo-validation-lane-bypass
origin_plan: docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
fixing_plan: docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
plan_link_mode: child_record_only
source_artifact: docs/plans/optimize/zircon_tooling/06/failure-2026-08-24-powershell-wrapped-cargo-validation-lane-bypass.md
---

# powershell-wrapped-cargo-validation-lane-bypass 回传摘要

- 状态：`fixed`
- 回传工件：[fixed-2026-08-24-powershell-wrapped-cargo-validation-lane-bypass.md](fixed-2026-08-24-powershell-wrapped-cargo-validation-lane-bypass.md)
- 摘要：PowerShell-wrapped declared Cargo validation now materializes an immutable Cargo copy, waits in FIFO, runs only under the Coordinator Cargo process tree, and durably preserves source-copy and selected-manifest identity through terminal evidence.
