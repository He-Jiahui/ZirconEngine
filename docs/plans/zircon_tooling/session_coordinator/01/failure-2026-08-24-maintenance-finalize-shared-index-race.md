---
handoff_kind: failure
status: open
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

Implementation GREEN, managed finalizer pending:

- `MaintenanceIndexService` owns private staging-index creation, shared staged
  binary-projection comparison, approved-entry alignment and the short
  publication lock. Temporary index and private lock cleanup is context-bound.
- `GitFinalizeService` now persists a shared index snapshot only for the short
  post-CAS publication/recovery window. Successful publication clears it before
  baseline reconciliation, so successor recovery does not touch a later foreign
  `.git/index.lock`.
- Maintenance add, staged identity scan, credential scan, validation and
  `write-tree` receive one explicit private `GIT_INDEX_FILE`. Direct Git
  validation commands receive the same index, while general validation tools do
  not inherit it into nested repositories; ordinary finalize retains its
  existing shared-index recovery behavior.
- RED proof failed at `_recover_index_lock` with a nonzero foreign lock before
  the implementation. The modularized implementation passes the final complete
  finalizer suite, 81/81 in 1105.182 seconds, including private-index identity,
  external staged projection preservation, official stale-lock recovery,
  post-CAS recovery and module-level stderr redaction. The real managed
  maintenance consumer remains required before closeout.
- Production finalize request `65d1676ca032434499b28dd12793795e`
  reached the new private validation phase but failed because the parent
  `GIT_INDEX_FILE` leaked into the suite's temporary repositories. An equivalent
  one-test RED reproduced the invalid cross-repository object lookup. The
  validation boundary now exposes the private index only to a direct `git`
  executable and proves a general Python validator can create and commit an
  independent nested repository. Focused regression passes 3/3 in 73.053
  seconds, and the updated complete finalizer suite passes 82/82 in 780.512
  seconds. A replacement managed finalizer consumer remains required.
