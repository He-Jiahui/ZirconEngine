---
handoff_kind: fixed
status: fixed
created_at: 2026-08-14
summary_slug: integration-candidate-stale-index-lock-recovery
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/integration_candidates.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/git_index_lock.py
  - tools/session_coordinator/tests/test_integration_candidates.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_integration_candidates
resolved_at: 2026-08-14
---


# Coordinator01: integration candidate leaves committed recovery state behind a stale index lock

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 触发切片：compile-time resource input closure candidate closeout.
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`.
- 交接原因：`IntegrationCandidateService` owns candidate state transitions, while the shared stale-lock classifier is currently owned by the Coordinator finalize infrastructure.

## 失败现象与复现证据

Candidate `0adafde0387547bf8dc9cb82c6172307` sealed exactly three paths after
compile ticket `1e74e68c24cc45fbb23ddeb76b7d4b77` passed. The ticket used source
manifest `0a148b87a6fc8efbb9a8ce5ef5600341e1e0e69861920362688a5b5b5df34fbb`
and ran the seven `test_validation_copies` cases in a managed Windows copy.

`integration finalize` created and advanced `HEAD` to
`93049504c8e1e5b3d10c7347c22465bf2d37cda3`, but then
`IntegrationCandidateService._align_shared_index` failed with `git update-index`
exit 128. Candidate state therefore remains `integration_ready` with a non-null
`commit_sha`, rather than `integrated_validation_pending`.

The only blocking file is `.git/index.lock`: it is zero bytes and was created at
`2026-08-14 14:16:55 +08:00`, well before the current finalize attempt. The
previous `finalize-readonly-git-index-lock-churn` fix added a fail-closed stale
lock classifier to `GitFinalizeService`, but the candidate-specific finalize
path bypasses that boundary.

## 最低共享层根因

The candidate flow publishes `HEAD` before calling `_align_shared_index`, but it
does not invoke the shared stale-index-lock recovery before its first
`update-index` mutation. A proven abandoned lock can therefore split one atomic
candidate closeout into a committed Git tree and an unadvanced coordinator state.

## 架构修复验收

- Reuse one fail-closed stale-lock recovery boundary for both ordinary scoped
  finalizers and integration candidates; do not duplicate file-owner checks.
- Candidate recovery must preserve the sealed blob IDs and never rebuild from
  mutable worktree files.
- A stale zero-byte, stable, unowned lock may be removed only under the
  coordinator mutex and must emit an audit event with candidate ID and lock
  identity.
- A live, young, non-empty, changed, or uninspectable lock must leave candidate
  state unchanged and refuse the mutation.
- Add a regression that starts with a prepared candidate commit plus an abandoned
  lock, verifies the existing `HEAD` is retained, realigns the index, and records
  `integrated_validation_pending` exactly once.

## 禁止临时方案

- Do not manually delete `.git/index.lock`, reset `HEAD`, amend the existing
  commit, or rerun the validation ticket.
- Do not mark the candidate integrated solely because a commit SHA is present.
- Do not reintroduce a candidate-local lock heuristic or weaken the shared Git
  mutex/lease checks.

## 协调归属

- `tools/session_coordinator/git_finalize.py` and
  `tools/session_coordinator/git_index_lock.py`: current historical owner is
  `coordinator01-benchmark-identity-review-maintenance-20260811`.
- `tools/session_coordinator/integration_candidates.py`: its last owner is the
  archived `coordinator01-control-plane-recovery-20260803-r1`; Coordinator01
  must perform a documented scope rotation before changing it.

## 产出记录与时间

- 2026-08-14 20:43 CST | status: open | 完成：确认 3-path candidate 的受管
  Python compile ticket 已通过，主分支 commit `93049504` 已落地且范围精确；定位
  `integration_ready + commit_sha` 分裂状态由 6 小时前的零字节 `.git/index.lock`
  触发。待完成：Coordinator01 scope rotation 后复用 shared stale-lock recovery，
  完成候选状态回写并补充 candidate-path 回归。

## 修复结果与回传

- 根因：Integration candidate finalize published its prepared commit before index alignment, bypassed the shared stale-index-lock recovery boundary, and treated an advanced descendant HEAD as already accepted instead of recoverable.
- 架构修复：IntegrationCandidateService now invokes the shared fail-closed recover_stale_index_lock primitive under the coordinator Git mutex before any update-index mutation, records candidate-bound lock identity, and recovers a prepared commit only when it is an ancestor whose sealed blobs remain exact.
- 验证：RED reproduced accepted/mutation failure; local unittest integration_candidates + git_index_lock 15/15; managed validation-copy job 3a967d31c2d7450398bc357185703b67 run 36450318cf4e484fbfd5bb0e84245d16 15/15; production candidate 0adafde0387547bf8dc9cb82c6172307 recovered to integrated_validation_pending with commit 93049504c8e1e5b3d10c7347c22465bf2d37cda3 and one git.index_lock_recovered event.
- 回传：Prepared integration candidates now recover stale index locks through the shared identity-checked boundary without moving descendant HEAD or altering foreign staged paths.
