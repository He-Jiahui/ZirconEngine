---
handoff_kind: fixed
status: fixed
created_at: 2026-08-23
summary_slug: cross-plan-fixed-artifact-closeout-attribution
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: cross_plan
related_code:
  - tools/session_coordinator/failure_return_delegations.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/ownership_transfers.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/workflows/failure_closeouts.py
  - tools/session_coordinator/tests/test_database.py
  - tools/session_coordinator/tests/test_server.py
  - tools/session_coordinator/tests/test_ownership_transfers.py
  - tools/session_coordinator/tests/test_failure_closeout.py
  - tools/session_coordinator/tests/test_failure_return_delegations.py
  - tools/session_coordinator/tests/test_migrations.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_failure_return_delegations tools.session_coordinator.tests.test_migrations tools.session_coordinator.tests.test_ownership_transfers -v
  - python -B -m unittest tools.session_coordinator.tests.test_database.DatabaseTests.test_latest_schema_persists_delegated_failure_return_proofs tools.session_coordinator.tests.test_server.ServerTests.test_failure_return_seals_origin_destination_authorization tools.session_coordinator.tests.test_failure_closeout.FailureCloseoutWorkflowTests.test_active_origin_owner_delegation_commits_exact_fixed_artifact tools.session_coordinator.tests.test_failure_closeout.FailureCloseoutWorkflowTests.test_origin_owned_fixed_artifact_without_authorization_stays_rejected tools.session_coordinator.tests.test_failure_closeout.FailureCloseoutWorkflowTests.test_commit_is_exact_notifies_and_keeps_resolving_failure_open -v
resolved_at: 2026-08-26
---

# Coordinator01: cross-plan fixed artifact cannot reach Failure closeout

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：Runtime15 non-network server classification Failure return and exact closeout
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns Failure return authorization, path attribution, lease transfer and scoped Git finalize.

## 失败现象与复现证据

Runtime15 Session `runtime15-server-context-classification-r1-a9220896-20260822`
completed implementation, managed validation and independent review for lifecycle
`non-network-server-context-classification-drift`. Closeout
`1b8bbf870c49475180fef39617171810` has accepted validation evidence
`987980824c454ed388cb81bd4e426217` and accepted review evidence
`dcd0a96d3f414d408973a73fddae1dec`; its exact manifest remains byte-stable on
HEAD `f1614c5e601d0879cfa3ac1e5d4886f0d8734d97`.

The final commit is rejected with `finalize_unattributed_path` for
`docs/plans/zircon_runtime/frameworks/01/fixed-2026-08-23-non-network-server-context-classification-drift.md`.
That file has hash
`c9a07bfa1efa0b95ab87227eaa14d01dadb57b27410bafd65fcdeef059ad1bde`
and is attributed to active origin Session
`frameworks01-open-failure-convergence-r9-a9220896-20260822`, while the closeout
executor is Runtime15. No Git commit was created.

The state is not recoverable through the public ownership-transfer protocol.
`ownership.transfer.preview` correctly rejects the path with
`source_owner_executable` while Frameworks01 remains `resolving_failure`.
Cancelling, completing or staling that active Session solely to make the transfer
eligible would falsify lifecycle state. Reclaiming and re-attributing the path through
the generic lease API would also bypass both immutable write scope and the reviewed
ownership-transfer contract.

## 最低共享层根因

The child-record-only return path implements two individually reasonable but
incompatible contracts:

- `server.py::_require_origin_destination_lease()` allows an active origin-plan
  Session to authorize the generated `fixed-*` destination when the fixing Session
  does not own that lease. Its docstring calls this a narrow lifecycle transfer and
  records `failure.return_origin_destination_authorized`.
- `GitFinalizeService.commit_failure_closeouts()` still delegates to the ordinary
  single-owner scoped finalizer. `_require_attribution()` and the live-lease gates
  require every material path to belong to the fixing Session, with no interpretation
  of the recorded origin-destination authorization.
- Generic ownership transfer deliberately accepts only abandoned paths and therefore
  cannot bridge the expected case where the origin Session is still active.

The coordinator consequently permits creation of a valid cross-plan fixed artifact
but provides no legal transition that can commit the same lifecycle. This is a
state-machine reachability defect, not an engine-source failure or stale evidence.

## 架构修复验收

- Define one explicit, durable delegated-ownership proof for the generated fixed
  destination when `failure.return` uses an active origin-plan lease. Bind it to the
  lifecycle key, origin and fixing Session IDs, exact normalized path, content hash,
  baseline epoch and authorization event.
- Make `failure.closeout_prepare` and `failure.closeout_commit` consume that proof
  atomically. The finalizer may use a narrow delegated lease/attribution grant or an
  exact lifecycle transfer, but it must not require the origin Session to become
  non-executable and must not enable unrelated cross-session paths.
- Revalidate under the Git mutex that the delegated path, hash, lifecycle, origin
  Session and closeout fingerprint still match. Any byte, HEAD, baseline, graph,
  snapshot, lease or authorization drift must fail closed and stale prior evidence.
- Preserve the ordinary rule that every non-delegated material path is attributed to
  and leased by the executor. Generic `lease.claim`, `baseline.attribute` and
  `ownership.transfer` must not become an out-of-scope bypass.
- Add an end-to-end test with an active origin owner and a distinct resolving fixer:
  child-only return, accepted exact closeout evidence and scoped commit must succeed
  without cancelling either Session. Add negative tests for missing authorization,
  unrelated path injection, hash drift, foreign lease reacquisition and replay.
- After the fix is committed and loaded by a healthy successor, re-prepare the
  Runtime15 closeout on current HEAD. Do not reuse an evidence fingerprint if the
  coordinator fix changes HEAD, baseline or Failure graph.

## 禁止临时方案

- Do not mark Frameworks01 completed, cancelled, stale or archived merely to satisfy
  `ownership.transfer.preview`.
- Do not manually rewrite SQLite attribution, impersonate either Session, widen the
  Runtime15 immutable scope through the unguarded lease API, or commit with a
  maintenance attribution bypass.
- Do not move the fixed record to Runtime15, omit it from the exact lifecycle manifest
  or commit it separately; the child-record-only Failure graph must remain canonical.
- Do not weaken independent review, managed validation, exact snapshot or Git mutex
  guards to make this one closeout pass.

## 修复结果与回传

- 根因：Child-record-only failure return authorized an active origin-owned fixed artifact, but ordinary scoped finalization still required every path to be attributed to the fixing Session, leaving the valid lifecycle unreachable.
- 架构修复：Schema 66 added an exact durable delegated-return proof bound to lifecycle, origin/fixing Sessions, normalized path, content hash, baseline epoch and authorization event; closeout prepare/commit revalidates and consumes it atomically under the Git mutex without weakening ordinary attribution.
- 验证：Commit 753cbf527 is in current HEAD; current-source delegation/transfer/migration tests passed 18/18 and end-to-end closeout tests passed 5/5. Production proof ff8bba22c2f849eba55d9a683dd2dca8 was consumed once by closeout 71b52f29c63c4a1fa89e01b988e47eb7 in commit c4761b14c6748c4fb0ac7edef67183d8d5afb5eb.
- 回传：Frameworks01 and distinct fixing Sessions can now complete exact child-record-only returns while origin ownership remains active; unrelated paths, drift, and proof replay remain rejected.
