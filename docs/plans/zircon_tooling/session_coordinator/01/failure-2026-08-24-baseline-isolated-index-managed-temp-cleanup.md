---
handoff_kind: failure
status: open
created_at: 2026-08-24
summary_slug: baseline-isolated-index-managed-temp-cleanup
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/baselines.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/tests/test_baselines.py
  - tools/session_coordinator/tests/test_git_finalize.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_baselines.BaselineTests.test_isolated_index_tree_survives_managed_process_temp_cleanup -v
  - python -B -m unittest tools.session_coordinator.tests.test_git_finalize.GitFinalizeTests.test_validation_command_uses_stable_coordinator_temp_environment -v
  - python -B -m unittest tools.session_coordinator.tests.test_git_finalize.GitFinalizeTests.test_git_add_pathspec_survives_managed_process_temp_cleanup tools.session_coordinator.tests.test_git_finalize.GitFinalizeTests.test_milestone_commit_is_scoped_atomic_and_keeps_session_active -v
---

# Coordinator01: baseline isolated index is deleted with managed process TEMP

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Render02 maintenance finalizer recovery after stable pathspec temp-root repair
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns baseline capture, scoped finalization, post-CAS recovery, and daemon rollover.

## 失败现象与复现证据

Maintenance finalize request `3fc7b6961c3a41ad9f212b18ed4f842f` validated and
published exact-one commit `8dc299a8b65813f692e222a709f951e6ace90be6` from
start HEAD `f73dd740892f9ecc86e0783b31e4cb8660ef0e75`. The target file is
clean and the unrelated 19-path staged fingerprint remains
`d221e55776b1167410e35fbb3a6a0c0bdb754df8`.

After the compare-and-swap ref update, baseline acceptance raised
`Command '['git', 'write-tree']' returned non-zero exit status 128`. The durable
request therefore remains `finalizing` with `ref_updated_sha=8dc299a8...`; replay is
forbidden because the commit already exists on HEAD.

## 最低共享层根因

`GitFinalizeService` now creates pathspec and secret-scan temporary files under the
Coordinator state root, but `BaselineService._isolated_index_tree()` still uses the
process-global `tempfile.TemporaryDirectory`. A successor inherited `TEMP` under a
managed Cargo target. Cleanup can delete that directory after the shared index is
copied and before `git write-tree` opens `GIT_INDEX_FILE`, leaving a post-CAS
finalizer that cannot finish baseline acceptance.

## 架构修复验收

- Create the baseline private index only under a stable Coordinator-owned temporary
  root derived from the database state directory.
- Reproduce managed process TEMP cleanup immediately before `git write-tree`; the
  isolated index must remain readable and the shared index bytes must not change.
- Preserve existing stale-index-lock behavior and exact scoped finalizer semantics.
- Controlled rollover must recover request `3fc7b696...` as committed without replay,
  clear its durable index snapshot, and keep the external staged fingerprint exact.
- Commit the production fix and focused regressions through the maintenance finalizer,
  then load a healthy successor from that commit before more failure finalizers run.

## 禁止临时方案

- Do not replay the Render02 finalize, reset HEAD, restore or restage the shared index,
  delete coordinator state, or edit SQLite directly.
- Do not disable baseline capture, suppress `write-tree` failures, use the live shared
  index without a private copy, or exempt managed Cargo cleanup.

## 修复结果与回传

The current worktree creates the private baseline index under
`.codex/state/session-coordinator/temporary`, derived only from the Coordinator
database path. The RED regression deleted the process-global managed target TEMP
immediately before `git write-tree` and reproduced the production exit 128. The same
test is GREEN after the stable-root change.

Focused validation passed the new regression, the existing shared-index/stale-lock
case, the finalizer pathspec cleanup regression, and the scoped atomic milestone case
(`4/4`). The complete baseline suite passed `17/17`; Python compilation, diff checks,
and two Failure schema regressions also passed.

The first exact-three maintenance request `c9bcf21cc287464a9e15804f325e6dca`
then exposed the remaining finalizer-context boundary before CAS. The same `19/19`
command passed in the caller environment, while the daemon-launched validation child
inherited its managed Cargo `TEMP/TMP`. `GitFinalizeService` now copies the existing
environment, redirects `TEMP`, `TMP`, and `TMPDIR` to the stable Coordinator root, and
preserves `SystemRoot` plus ordinary variables. Both ordinary finalize and scoped
milestone validation use the same builder while retaining their existing typed error
codes. The environment RED/GREEN and both error-contract regressions pass `3/3`.

Controlled rollover action `ca672196a6e24810bd90941e58576f1e` loaded healthy
read-write schema-66 successor `b3f8c5397181475f81ec6957b12ea6f6`. Startup recovery
completed request `3fc7b6961c3a41ad9f212b18ed4f842f` without replay:
`status=committed`, `commit_sha=ref_updated_sha=8dc299a8...`, `index_snapshot=NULL`.
Baseline epoch 394 is healthy on that commit; the shared 19-path staged fingerprint
remains `d221e55776b1167410e35fbb3a6a0c0bdb754df8` and no index lock or Git mutex
remains.

The exact five Coordinator source/test/evidence paths were committed by maintenance
finalize request `9730fbeb998541698e20e0d20597d9ec` as
`dd1eccb9cdef9824754664d673db2e2b6f9073f0`. Post-commit controlled rollover action
`65dcd2fafe6c42ca9ebb7b528178d151` loaded healthy read-write schema-66 successor
`e9670dd6f626422dbb70553b0b5ed340` from that source. Fresh child-environment and
baseline-index focused regressions passed `2/2`; the Coordinator temporary root was
empty, no finalizing request, active Cargo job, Git mutex, artifact reservation, or
index lock remained, and the unrelated 19-path staged fingerprint was still exactly
`d221e55776b1167410e35fbb3a6a0c0bdb754df8`.

Open state: `implementation, maintenance commit, startup recovery, and post-commit
daemon alignment accepted / formal local failure closeout pending`.
