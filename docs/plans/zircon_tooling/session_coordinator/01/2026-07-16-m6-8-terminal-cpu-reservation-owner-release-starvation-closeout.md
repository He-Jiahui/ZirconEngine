# M6.8 Terminal CPU Reservation Owner Release Starvation Closeout

Plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
Milestone: M6.8
Status: accepted
Files: ["docs/plans/zircon_runtime/frameworks/05/fixed-2026-07-16-terminal-cpu-reservation-owner-release-starvation.md", "docs/plans/zircon_tooling/session_coordinator/01/2026-07-16-m6-8-terminal-cpu-reservation-owner-release-starvation-closeout.md", "docs/plans/zircon_tooling/session_coordinator/01/2026-07-16-terminal-cpu-reservation-owner-release-starvation-return.md", "tools/session_coordinator/tests/test_cargo_jobs.py", "tools/session_coordinator/tests/test_cargo_reservations.py"]

## 产出记录与时间

| 状态 | 日期 | 完成项目与当前门禁 |
|---|---|---|
| `FIXED / REVIEW ACCEPTED / VALIDATION PENDING` | 2026-07-16 | 首版 7-file P0 业务提交为 `f303e1f815c95274a3eb23aa6a1f3a58a258dd0e`；本 current-HEAD follow-up 仅包含 5 个 review-remediation delta。新增 runner + bound reservation + stale owner + 重复 release 组合回归，以及 executable owner / 非 released job / 非空 recorded process tree 三组负例；fresh 聚焦回归 `62/62`、独立 re-review `C/I/M=0/0/0`，等待最终哈希受管 validation。 |

## Scope delivered

- A managed CPU job release now releases its bound CPU reservation in the same transaction after the process tree is empty.
- Reserve and acquire reconcile a legacy `finished` reservation only when its job is released, no process remains, and the owner Session is non-executable.
- The P0 handoff has been atomically returned as a Frameworks05 `fixed-*` record with the Coordinator01 child-only receipt.

## Fresh testing evidence

- `python -m unittest tools.session_coordinator.tests.test_cargo_reservations`: 16 passed.
- `python -m unittest tools.session_coordinator.tests.test_cargo_jobs`: 46 passed.
- Scoped maintenance and failure-return lease regressions each passed 1/1; `py_compile` passed for the affected coordinator modules.
- Production ledger reproduction confirms the stale Runtime11 reservation `ffe8819e1959439d91b94ddc8decb928` is `released` and no longer holds the CPU FIFO head.

## Review

- Initial independent review found `Critical 0 / Important 1 / Minor 1`; the implementation predicates were correct, but runner/idempotency and each historical negative predicate lacked focused proof, while two records mislabeled Runtime11 as Plugins06. Fresh re-review after those corrections found `Critical 0 / Important 0 / Minor 0`; the reviewer independently reran `62/62` and scoped diff-check successfully.

## Completion status

- Status: `accepted / managed commit pending`.
- The manifest is deliberately limited to this P0 return slice. Open Coordinator01 failures remain open and are not treated as resolved by this commit.
