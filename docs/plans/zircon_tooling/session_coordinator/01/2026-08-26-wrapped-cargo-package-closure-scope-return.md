---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-08-26
summary_slug: wrapped-cargo-package-closure-scope
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
plan_link_mode: child_record_only
source_artifact: docs/plans/zircon_tooling/session_coordinator/01/failure-2026-08-25-wrapped-cargo-package-closure-scope.md
---

# wrapped-cargo-package-closure-scope 回传摘要

- 状态：`fixed`
- 回传工件：[fixed-2026-08-26-wrapped-cargo-package-closure-scope.md](fixed-2026-08-26-wrapped-cargo-package-closure-scope.md)
- 摘要：Package-scoped closure now preserves manifests, ancestor topology and Cargo
  target entrypoints without copying or scanning unrelated source trees. Commit
  `c1d1e76b22915969da8b3e732d4744778c12662e` is loaded by healthy schema-67
  successor `4139b1e4c17a43fc9f9c8f6bcea14c66`; ticket
  `64f834a26e464b529e51427672bb2e9c` materialized once and reached managed Cargo.
  Its explicit exit 101 is an offline registry cache miss for `image`, not the repaired
  closure path, and is not reported as a passing Cargo check.
