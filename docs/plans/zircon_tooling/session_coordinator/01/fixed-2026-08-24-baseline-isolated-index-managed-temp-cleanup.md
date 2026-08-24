---
handoff_kind: fixed
status: fixed
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
resolved_at: 2026-08-24
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

- 根因：Baseline private index and finalizer validation children inherited managed Cargo TEMP/TMP paths whose cleanup could remove live Coordinator temporary state.
- 架构修复：Route baseline index, pathspec, secret-scan, and validation child temporary files through the stable Coordinator-owned state temp root while preserving the ordinary child environment and scoped finalizer invariants.
- 验证：Baseline suite 17/17; focused environment, pathspec, stale-lock, and scoped finalizer regressions 4/4 plus 3/3; commits dd1eccb9cdef9824754664d673db2e2b6f9073f0 and 5db712ffabd6e8d5fd23141d956ebde55c20f0de; schema66 successor e9670dd6f626422dbb70553b0b5ed340 recovered the post-CAS request without replay, kept the 19-path staged fingerprint d221e55776b1167410e35fbb3a6a0c0bdb754df8, and left no mutex/index lock.
- 回传：Baseline capture and maintenance finalization now survive managed TEMP cleanup; the recovered commit and external staged index remain intact, so scoped failure finalizers may continue.
