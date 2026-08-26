---
handoff_kind: fixed
status: fixed
created_at: 2026-08-22
summary_slug: cargo-cleanup-deletes-live-validation-copy-parent
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: cross_plan
related_code:
  - tools/session_coordinator/cleanup.py
  - tools/session_coordinator/cleanup_deletion.py
  - tools/session_coordinator/tests/test_cleanup.py
  - tools/session_coordinator/tests/test_workspace_copy.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_cleanup -v
  - python -m unittest tools.session_coordinator.tests.test_workspace_copy -v
resolved_at: 2026-08-26
---

# Coordinator01: Cargo cleanup deletes a live validation-copy parent

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：Frameworks01 current-source managed validation-copy materialization and run admission
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns Cargo cleanup reservations, validation-copy roots, and the destructive filesystem CAS boundary.

## 失败现象与复现证据

Frameworks01 copy `e5c61237dda44d6c9fa9ba227322e3e8` materialized under
`F:\cargo-targets\verify\e5c61237dda44d6c9fa9ba227322e3e8` with immutable input
manifest `139ce46a9ef52dcdebae0f0bde9865a3f57902506ef41c00d624d50f6794111d`.
The durable row remains `status=materialized`, but its physical job root is absent and
`validation_copy.run` was rejected before process launch by
`unmanaged_artifacts_detected`.

A second production reproduction occurred while the review-fixed implementation was
validated but not yet loaded by the daemon. Managed isolated-patch ticket
`419037d5423a45f19dd75f63bf3d3d15` was running from copy
`e9274949e3b74b70a2f4ef0476496121` when old-daemon event `153634` reserved
`F:\cargo-targets\verify` for `retry_failed_cleanup` at
`2026-08-22T15:13:49.493212Z`. Event `153635` then reported a partial-delete failure
inside that exact copy at `target\temporary\tmpd7n7z596\repo`; the next test setup
observed that the just-initialized repository's `.git` directory was gone. The same
focused test passed ten consecutive local repetitions, and the managed run had passed
the preceding 13 cases. This independently proves the parent cleanup can corrupt a
currently running validation copy, not only a materialized copy waiting to start.

Coordinator event evidence identifies the destructive owner. Historical Cargo jobs
registered the shared parent `F:\cargo-targets\verify` as their target. Cleanup event
`153142` reserved that parent at `2026-08-22T12:58:06.001414Z`; event `153143`
completed deletion at `12:58:06.085120Z`. A later explicit-plan pass repeated the
parent deletion in events `153167` through `153169` at `13:07:15Z`. The validation
copy was created at `12:57:40.574746Z`, before the first successful parent deletion.
There is no validation-copy terminal run evidence and no validation-copy cleanup
transition that could justify removing the tree.

## 最低共享层根因

`CleanupService` revalidates active Cargo jobs, retained Cargo pools, and cleanup
reservations before each destructive `shutil.rmtree`, but its immediate, pressure,
and explicit-plan paths never query `validation_copies`. A Cargo cleanup target may
therefore be a parent or child of a planned, materializing, materialized, running, or
cleanup-pending validation copy and still pass the final delete gate. Artifact
governance knows validation-copy paths, but that separate scan cannot protect the
Cargo cleanup service's filesystem mutation.

## 架构修复验收

- Add a shared transactional overlap guard for every non-`removed` validation-copy
  `job_root` and `target_root`, using the same canonical target identity and
  parent/child overlap semantics as Cargo targets.
- Recheck the guard inside the same reservation transaction immediately before every
  Cargo cleanup mutation: prompt/retry cleanup, pressure eviction, explicit-plan
  apply, and interrupted-reservation recovery. Planning-time filtering alone is
  insufficient.
- Preserve and regression-test the existing reverse guard:
  `WorkspaceCopyService._require_cleanup_available()` must continue rejecting a
  job/target root that overlaps an already reserved Cargo cleanup target. The cleanup
  reservation remains the cross-transaction filesystem mutation fence.
- A protected overlap must return a typed durable denial and must not insert a Cargo
  cleanup reservation, delete any validation-copy byte, or change the validation-copy
  durable status.
- Tests must cover a Cargo target that is the parent of a materialized copy, a target
  nested below a protected copy, restart recovery, a validation-copy request racing
  an established Cargo cleanup reservation, and a non-overlapping or `removed` copy
  that does not block otherwise legal cleanup.
- Allow Frameworks01 to request a fresh managed materialization only after the fix is
  committed and loaded by a healthy successor. The destroyed copy remains immutable
  terminal evidence and must not be recreated or rewritten.

## 禁止临时方案

- Do not permanently exempt `F:\cargo-targets\verify` or another path prefix from
  cleanup governance.
- Do not weaken `unmanaged_artifacts_detected`, recreate the Frameworks01 copy, or
  rewrite its durable row to hide the deletion.
- Do not protect only the observed job ID; the invariant is parent/child overlap for
  every live validation-copy lifecycle.

## 修复结果与回传

- 根因：Cargo cleanup revalidated jobs, pools, and cleanup reservations but omitted non-removed validation-copy roots, so a parent or child Cargo target could pass the final destructive gate.
- 架构修复：Added a transactional canonical parent-child overlap guard for every live validation-copy job_root and target_root at prompt cleanup, pressure eviction, explicit-plan apply, and restart recovery; denials are typed and durable while the reverse reservation fence remains intact.
- 验证：Commit 7762880fd is in current HEAD; current-source cleanup module passed 41/41; managed immutable validation previously passed 44/44 and successor copy 77fddbd58f8b4e6fb2089c62f0ff0c43 completed 17/17 before normal removal.
- 回传：Frameworks01 may request a fresh immutable materialization under the loaded guard; destroyed copy e5c61237dda44d6c9fa9ba227322e3e8 remains immutable terminal evidence and is not recreated or rewritten.
