---
handoff_kind: fixed
status: fixed
created_at: 2026-09-08
summary_slug: resumed-reviewer-session-remains-archived
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/optimize/zircon_tooling/06
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/sessions.py
  - tools/session_coordinator/models.py
  - tools/session_coordinator/workflows/failure_closeouts.py
tests:
  - failure closeout-review 843cfed86bb44b29b932bbda3968bcd1 from real reviewer task 01a07063-6f03-7803-a12d-13ea015ca645
  - automatic stale-retention archive followed by resuming the same native Codex task
resolved_at: 2026-09-10
---

# Tooling06: resumed reviewer task retains an unusable archived Session

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Patch attribution scope failure, snapshot 3210 final closeout.
- 修复责任计划：`docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 交接原因：A live native task and its permanently archived coordinator Session
  disagree at the review-admission boundary.

## 失败现象与复现证据

The user requires reuse of independent task `01a07063-6f03-7803-a12d-13ea015ca645`
(coordinator efficiency review). It reviewed snapshot 3210 as C0/I0/M0 and
submitted closeout-review once from its actual native task identity.
Request `59796c5d40d749a1a63d99557cb39c59` was accepted, then terminally rejected
with `failure_closeout_reviewer_not_active`. No review evidence was generated.
Report: `.codex/tmp/coordinator01-patch-admission-3210-closeout-review-20260908-result.txt`.

Read-only SQLite confirms native `codex_sessions` state `idle`, source location
`active`, cwd `E:/Git/ZirconEngine`, and `bound_session_id` equal to that same
thread. The corresponding coordinator Session is `archived`, with reason
`stale retention elapsed`, last heartbeat `2026-09-05T07:33:11.187256+00:00`.
It owns no path in the nine-entry closeout manifest. Source, deleted original,
unique fixed/return artifacts, formal 42/42 validation and live patch155 recovery
were all accepted by independent review; this rejection is a lifecycle gate.

## 最低共享层根因

`FailureCloseoutWorkflowService.record_review` requires the real native task's
coordinator Session to be ACTIVE. Native task discovery can observe a resumed
task while its formerly auto-archived Session remains terminal.
`ALLOWED_STATUS_TRANSITIONS[SessionStatus.ARCHIVED]` is empty and
`SessionService.register/set_status` both enforce it. There is no supported
recovery through those commands. The correct resumed-task lifecycle policy and
audited operation must be defined at the Session owner; changing review counts
or spoofing a second task cannot resolve this mismatch.

## 架构修复验收

- Define and implement an audited lifecycle for a real resumed task whose bound
  Session was archived by stale retention, preserving original ownership and
  unfinished ticket identities.
- Preserve intentional completed/cancelled/archive terminal semantics; require
  current native task provenance and reject unrelated callers or guessed IDs.
- Keep review independent: actual task identity, no reviewed-path ownership,
  unchanged exact source and C0/I0/M0 remain required.
- Replay the original closeout-review through the user-designated task, obtaining
  a real accepted review evidence record before coordinator closeout commit.

## 禁止临时方案

- Do not write SQLite directly, change CODEX_THREAD_ID to another task, remove
  the active/provenance/ownership checks, or fabricate accepted review evidence.
- Do not resubmit the terminally rejected request without repairing its cause.
- Do not create a new reviewer task merely to bypass the user's designated task.

## 修复结果与回传

- 根因：A native Codex task could remain live and bound while stale-retention archived its matching coordinator Session; closeout review rejected the archived reviewer before any evidence was recorded.
- 架构修复：Add an audited native-task reactivation operation at SessionService and invoke it from closeout review only after exact identity, live repository provenance, automatic archive reason, independence, and path checks; preserve terminal archive semantics and existing scopes, leases, and unfinished tickets.
- 验证：Managed Windows Python ticket 27a37d0558d044e19ac25fe1b126c094 passed in immutable copy 4214f2c280194f4ba93cab5bc7220b77 with run 27a37d0558d044e19ac25fe1b126c094, exit code 0; the temporary-repository regression passed automatic recovery and rejected manual archive and outside-cwd provenance.
- 回传：Tooling06 lifecycle repair is source-complete and managed-validated; a fresh closeout under the current fixing Session will exercise the designated reviewer reactivation path.

## 修复与受管验证证据（2026-09-10）

The current owner transferred the exact three production paths from the archived
foundation Session under transfer fingerprint `3ce4d9b8acbc37822683d3bfeac7004e764695d494340602195e0c8d6e65da1b`;
no foreign path was absorbed. The implementation keeps the archived-session
transition out of `set_status`, and adds `SessionService.reactivate_archived_native_task`:
it requires an exact `session_id == thread_id`, the automatic
`stale retention elapsed` reason with no completion timestamp, and a live native
Codex row bound to the same repository and Session. It records an atomic
`session.reactivated` event and preserves the existing scope, leases and
unfinished ticket rows. `FailureCloseoutWorkflowService.record_review` invokes
that owner operation only after reviewer provenance, independence, path and
zero/negative-count checks pass. Manual terminal archives and a native task with
an outside repository cwd remain rejected.

The exact current source hashes are:

- `tools/session_coordinator/models.py`: `6319fced5e56b90e6806d91e6ea6f3ec3430669829a3998851d1ae23cb5a1736`
- `tools/session_coordinator/sessions.py`: `51412b1a10e4fa393d4499b8d1fdbb4c7cd32d3bd441fd387602ec21f55187f1`
- `tools/session_coordinator/workflows/failure_closeouts.py`: `ffe7b1cfc25b22c18a0f3097b0afc6dc359a3a157d646bcd81f001eb51d82ba5`

Managed Windows Python ticket `27a37d0558d044e19ac25fe1b126c094` sealed
exactly those three source paths at baseline epoch 605 and ran in immutable
copy `4214f2c280194f4ba93cab5bc7220b77` with run
`27a37d0558d044e19ac25fe1b126c094`. The regression creates a real temporary
Git repository and exercises all required branches: same-task automatic
stale-retention recovery passes with scope and reason preserved; an operator
archive raises `session_reactivation_terminal`; and a bound task rooted outside
the repository raises `session_reactivation_provenance_invalid`. Worker evidence
records exit code 0 and stdout `native archived-session reactivation regression: PASS`.

The historical closeout replay was attempted once by the designated native
reviewer task `01a07063-6f03-7803-a12d-13ea015ca645`; the old executor Session
was already archived, so coordinator correctly returned
`failure_closeout_session_not_resolving` before reviewer admission. This is
preserved as historical evidence; the current fixing Session will prepare a new
closeout with the same designated reviewer, whose archived Session is now
eligible for audited native-task reactivation.
