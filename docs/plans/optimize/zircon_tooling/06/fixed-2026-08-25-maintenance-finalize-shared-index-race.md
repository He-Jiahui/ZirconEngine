---
handoff_kind: fixed
status: fixed
created_at: 2026-08-24
summary_slug: maintenance-finalize-shared-index-race
origin_plan: docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/optimize/zircon_tooling/06
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: cross_plan
related_code:
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/maintenance_index.py
  - tools/session_coordinator/tests/test_git_finalize.py
tests:
  - python -u -B -m unittest tools.session_coordinator.tests.test_git_finalize.GitFinalizeTests.test_maintenance_finalize_uses_private_index_while_shared_index_is_locked -v
  - python -u -B -m unittest tools.session_coordinator.tests.test_git_finalize -v
resolved_at: 2026-08-25
---

# Coordinator01: maintenance finalize mutates the shared Git index

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 来源执行切片：PowerShell-wrapped Cargo validation lane repair exact-three maintenance finalize
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the finalizer and its Git transaction boundary.

## 失败现象与复现证据

After baseline Git readers were changed to set `GIT_OPTIONAL_LOCKS=0`, Tooling06
finalize request `1bcf54d1322a4b30ba074ecaec86bae9` still failed before validation with
`Git index lock cannot be recovered safely`. The young lock disappeared after
the concurrent foreign `git status --short -- zircon_editor/...` process exited.

A six-observation process window showed recurring scoped `git status` and
`git grep` commands from active Sessions. At `2026-08-24T20:06:22+08:00`,
`.git/index.lock` existed while the foreign scoped status was live. Cargo and
the coordinator `git_mutex` were otherwise empty. Repeating the Tooling06
finalizer would therefore race an external Git reader again.

## 最低共享层根因

`GitFinalizeService.finalize(..., maintenance=True)` snapshots the shared index,
recovers its lock, resets that same index with `read-tree`, stages the approved
paths, writes the commit tree, then overwrites the shared index with the saved
bytes. The coordinator database mutex cannot serialize independent Git commands
issued by other Sessions. The restore step can also overwrite a legitimate
concurrent index refresh or staged update.

Maintenance commits only need an index image rooted at the accepted baseline;
they do not need to publish staged state into the shared index. The correct
transaction boundary is a private `GIT_INDEX_FILE` used by every stage, scan,
secret check and `write-tree` operation. `update-ref` remains protected by the
existing expected-HEAD compare-and-swap.

## 架构修复验收

- Maintenance finalize creates a private temporary index rooted at the accepted
  baseline HEAD and never recovers, resets, restores or writes the shared index.
- All staged-scope, staged-blob, secret-scan and commit-tree Git operations use
  the private index explicitly; no process-global environment mutation is used.
- A live foreign `.git/index.lock` may coexist with the private staging and
  validation phases. The final short publication window must acquire its own
  shared lock without deleting a live foreign lock.
- Foreign staged paths and their exact binary projection remain unchanged and
  are excluded from the maintenance commit. Only approved entries are aligned
  to the new HEAD. Ordinary non-maintenance finalize behavior remains unchanged.
- Temporary private index files and their private lock files are removed on
  success and failure. Validation failure cannot move HEAD.
- The full Git finalizer suite passes, followed by a controlled rollover and a
  successful replay of the frozen Tooling06 exact-three finalizer.

## 禁止临时方案

- Do not delete or recover a foreign live `.git/index.lock` for maintenance.
- Do not clear, reset, replace or restore the shared index.
- Do not retry around the race, extend timeouts, serialize arbitrary external
  Git commands, or use process-global `GIT_INDEX_FILE`.
- Do not absorb Tooling06 worker paths into this Coordinator01 fix.

## 修复结果与回传

- 根因：Maintenance finalization staged through the shared Git index, so independent repository readers could race index.lock and concurrent staged updates could be overwritten during restore.
- 架构修复：MaintenanceIndexService now builds and validates an accepted-HEAD private GIT_INDEX_FILE, stages only approved entries, publishes through expected-HEAD CAS plus a short shared lock, and leaves the shared index bytes untouched; general validators do not inherit the private index into nested repositories.
- 验证：Commit 514d2127710757e7e991646557934469e771609b passed 82 Git finalizer tests in 780.512 seconds plus the focused 3-test nested-repository regression in 73.053 seconds, then loaded via controlled rollover. Production Tooling06 exact-three request 31eaf00e9b6241218cb8b55d1880ac16 committed 3af73550dd00fe4805f71e96ce199f4ab633687f; the external 19-path staged projection SHA-256 a56e70cfec926da8592151ec15a7c85acba64cd44fecee403d594673f45e3b02 remained exact, index.lock and git_mutex were empty, and schema67 successor is healthy.
- 回传：Maintenance finalization now uses an isolated index end to end, and the frozen Tooling06 consumer committed successfully while preserving all foreign staged state.
