---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-15
summary_slug: finalize-recovery-index-lock-and-baseline-archive
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/tests/test_git_finalize.py
  - tools/session_coordinator/baselines.py
  - tools/session_coordinator/tests/test_baselines.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_git_finalize
  - python -m unittest tools.session_coordinator.tests.test_baselines
resolved_at: 2026-08-15
---


# Coordinator01: published finalize cannot recover a recreated index lock and large archive pipe

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：planless Session registration failure closeout
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the durable Git mutex, index snapshot, baseline epoch, and daemon startup recovery boundary.

## 失败现象与复现证据

- Maintenance finalize request `9b75532385084c229c047967794df121` published HEAD `b18f23ab21b9bcaffb7c234a2ffecd7eea7768ed`, then failed while restoring the shared index because a second `.git/index.lock` appeared after its first lock recovery.
- Controlled rollover action `c580bd8672c0486989ba8294ff3711b3` reached `awaiting_restart`, but five automatic successor starts exited while `recover_stale_mutex()` attempted `git reset` without revalidating that recreated lock.
- After the lock was safely recovered, successor startup failed again with `baseline_commit_archive_failed`. The same pinned HEAD succeeds under direct `git archive`, while `BaselineService._commit_manifest()` returned nonzero with empty stderr.
- The interrupted transaction retained the 3,954,110-byte index snapshot and Git mutex throughout both failures; no direct index mutation or lock deletion was used.

## 最低共享层根因

`recover_stale_mutex()` assumed the finalize-time index-lock recovery remained valid across HEAD publication and shared-index restoration. It restored the durable snapshot and immediately ran `git reset`, allowing a recreated stale lock to prevent every successor from starting.

The baseline archive reader separately stopped consuming stdout at the tar end marker and closed the pipe before draining to producer EOF. On the current 26,642-path tree, Git could still be flushing its pipe and exited as a broken producer even though the archive itself was valid.

## 架构修复验收

- Startup recovery must run the same two-observation, owner-aware stale-lock recovery immediately before any durable recovery reset.
- An active, changed, young, nonzero, or otherwise unproven lock remains fail-closed; an ambiguous HEAD keeps the mutex and snapshot.
- Recovery preserves every unrelated staged path and staged blob while aligning only the committed finalize paths to current HEAD.
- Baseline tar streaming must drain Git stdout to EOF before closing the pipe and checking the child exit code.
- Git archive stderr and nonzero exits remain typed failures; filters and incremental baseline refresh behavior remain unchanged.
- Production recovery must leave HEAD at `b18f23a`, clear the mutex/snapshot and `index.lock`, restore the 283-path staged projection, and publish a healthy schema-63 successor.

## 禁止临时方案

- Do not manually delete `.git/index.lock`, reset the shared index, clear `git_mutex`, or discard the durable snapshot.
- Do not ignore a nonzero Git exit, disable worktree filters, extend an arbitrary timeout, or rebuild a baseline from live worktree bytes.

## 修复结果与回传

- 根因：Finalize recovery reused an earlier index-lock observation before git reset, while baseline archive parsing closed the producer pipe before EOF.
- 架构修复：Revalidate and recover stale index locks immediately before durable reset; drain archive stdout through producer EOF before checking Git exit status.
- 验证：Focused recovery 4/4 and baseline 3/3 passed; real 26,642-path manifest succeeded; production requests 9b755323 and e3f4bbd2 recovered committed HEADs, cleared git_mutex/index snapshots, and restored the 283-path staged projection.
- 回传：Committed 480985ae6; schema63 successors recovered both the original and a second post-commit interrupted finalize with baseline healthy, index.lock absent, git_mutex empty, and staged count 283.
