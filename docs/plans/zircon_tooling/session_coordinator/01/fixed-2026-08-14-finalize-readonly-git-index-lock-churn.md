---
handoff_kind: fixed
status: fixed
created_at: 2026-08-14
resolved_at: 2026-08-14
summary_slug: finalize-readonly-git-index-lock-churn
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/git_index_lock.py
  - tools/session_coordinator/processes.py
  - tools/session_coordinator/tests/test_git_index_lock.py
tests:
  - python -X dev -W error::ResourceWarning -m unittest tools.session_coordinator.tests.test_git_index_lock -v
---


# Coordinator01: finalize is blocked by stale read-only Git index locks

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：combined Failure closeout commit after accepted managed validation and independent review
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the durable Git mutex, index snapshot, and scoped finalize mutation boundary.

## 失败现象与复现证据

The accepted combined closeout request `f0bf4abdf8d84584b19300578688fbcf`
failed in inner finalize `ca44208fd1c148f59e2b50e1931c47fd`. The finalizer
held the Coordinator `git_mutex`, completed its immutable path preflight, and then
`git read-tree` exited 128 because `.git/index.lock` existed. The lock was a zero-byte
file created at `2026-08-14T06:16:55Z`; it remained unchanged after the failed
finalize and Windows Restart Manager reported no live owner. `HEAD` did not move and the exact shared
index image remained preserved.

This is the second occurrence during a long managed closeout preflight. Codex
Desktop read-only Git inspection may request optional index refresh and can leave a
zero-byte lock after its process exits. The finalizer currently treats every such
lock as an opaque Git command failure, so a fully accepted Failure chain cannot
reach its managed commit without an out-of-band manual deletion.

## 最低共享层根因

`GitFinalizeService` owns the durable Git mutex and the exact index snapshot, but it
has no bounded stale-lock classifier immediately before its first index mutation.
It therefore cannot distinguish an old abandoned zero-byte lock from a live,
non-empty, or changing lock. The missing classifier is Coordinator infrastructure;
no product plan can repair or safely bypass it.

## 架构修复验收

- Run recovery only while the Coordinator Git mutex is held and immediately before
  a managed `read-tree` index mutation.
- Remove only the exact repository `index.lock` when it is zero bytes, older than a
  fixed safety window, stable across two identity observations, and Windows Restart
  Manager reports no process owning that exact file at either observation.
- Refuse recovery for a young, non-empty, changing, uninspectable, or live-owned
  lock without modifying it.
- Persist an event containing the finalize request, Session, relative lock path,
  observed age and stable identity whenever recovery succeeds.
- Apply the same boundary to ordinary scoped commits and maintenance finalization.
- Preserve `HEAD`, the raw shared index snapshot, scoped attribution, lease checks,
  and all existing finalize recovery semantics.

## 禁止临时方案

- Do not delete `.git/index.lock` manually or on Coordinator startup.
- Do not delete an arbitrary Git lock, wildcard path, non-empty lock, or lock while
  Restart Manager reports a live owner.
- Do not weaken `git_mutex`, retry an entire closeout, accept caller-controlled lock
  metadata, or bypass `read-tree` errors.
- Do not launch Cargo or terminate a live validation job to create a recovery window.

## 修复结果与回传

- 根因：Managed finalizer lacked an owner-aware stale index-lock classifier before its first index mutation
- 架构修复：Added a shared fail-closed classifier with stable identity, age, and Restart Manager owner checks plus durable recovery audit
- 验证：Managed snapshot 1744 job 953d769e85c04663ac7350d3aac7aa13 run 26331d8c834c4c63b6291450f55ea083 passed 8 tests with exit 0
- 回传：Scoped and maintenance finalizers now recover only proven abandoned zero-byte index locks
