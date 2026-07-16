# M6.8 Terminal CPU Reservation Owner Release Starvation Closeout

Plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
Milestone: M6.8
Status: accepted
Files: ["docs/cli-and-tooling/local-session-coordinator.md", "docs/plans/zircon_runtime/frameworks/05/fixed-2026-07-16-terminal-cpu-reservation-owner-release-starvation.md", "docs/plans/zircon_tooling/session_coordinator/01/2026-07-16-m6-8-terminal-cpu-reservation-owner-release-starvation-closeout.md", "docs/plans/zircon_tooling/session_coordinator/01/2026-07-16-terminal-cpu-reservation-owner-release-starvation-return.md", "tools/session_coordinator/cargo_jobs.py", "tools/session_coordinator/cargo_reservations.py", "tools/session_coordinator/tests/test_cargo_reservations.py"]

## Scope delivered

- A managed CPU job release now releases its bound CPU reservation in the same transaction after the process tree is empty.
- Reserve and acquire reconcile a legacy `finished` reservation only when its job is released, no process remains, and the owner Session is non-executable.
- The P0 handoff has been atomically returned as a Frameworks05 `fixed-*` record with the Coordinator01 child-only receipt.

## Fresh testing evidence

- `python -m unittest tools.session_coordinator.tests.test_cargo_reservations`: 15 passed.
- `python -m unittest tools.session_coordinator.tests.test_cargo_jobs`: 45 passed.
- Scoped maintenance and failure-return lease regressions each passed 1/1; `py_compile` passed for the affected coordinator modules.
- Production ledger reproduction confirms the stale Plugins06 reservation `ffe8819e1959439d91b94ddc8decb928` is `released` and no longer holds the CPU FIFO head.

## Review

- Independent Coordinator01 support-scope review found Critical 0 and Important 0 findings: terminal release remains process-tree guarded, legacy reconciliation is deliberately narrower than normal FIFO ownership, and the returned failure artifacts are lease-bound.

## Completion status

- Status: `accepted / managed commit pending`.
- The manifest is deliberately limited to this P0 return slice. Open Coordinator01 failures remain open and are not treated as resolved by this commit.
