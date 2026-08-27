---
handoff_kind: fixed
status: fixed
created_at: 2026-08-24
summary_slug: baseline-untracked-scan-index-lock-race
origin_plan: docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/optimize/zircon_tooling/06
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: cross_plan
related_code:
  - tools/session_coordinator/baselines.py
  - tools/session_coordinator/tests/test_baselines.py
tests:
  - python -u -B -m unittest tools.session_coordinator.tests.test_baselines.BaselineTests.test_shared_index_readers_disable_git_optional_locks -v
  - python -u -B -m unittest tools.session_coordinator.tests.test_baselines -v
resolved_at: 2026-08-25
---

# Coordinator01: baseline untracked scan races the managed finalizer index

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 来源执行切片：PowerShell-wrapped Cargo validation lane repair exact-three maintenance finalize
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns baseline observation and the shared Git index transaction.

## 失败现象与复现证据

Tooling06 finalizer requests `5efe01d226904295ad110b297c7f4193` and
`a8777828fc77473bbf4683e5615586ac` each passed the full 28-test validation
command, then failed at `git write-tree` with `finalize_git_command_failed`
because `.git/index.lock` had appeared after the finalizer's initial stale-lock
recovery. Both requests restored the pre-existing shared index, left HEAD at
`79f64878f3b9526517644c055ad3bf5cadfccd0f`, and released `git_mutex`.

During both failures, daemon instance
`78d0970de909432ba90533dee4a60237` retained the same long-running child tree:

```text
PID 37652 -> PID 32712
git ls-files --others --exclude-standard -z
started 2026-08-24T19:03:04+08:00
```

The child continued accumulating CPU after both finalizers. Its zero-byte
`.git/index.lock` was created again while each finalizer validation ran and
disappeared only after the failed transaction restored the shared index. The
Cargo lane and database `git_mutex` were empty before each request; an unrelated
FIFO Cargo job that started during the second validation released normally and
did not own the repository index lock.

## 最低共享层根因

`BaselineService._workspace_manifest_from_baseline()` enumerates untracked paths
through `_git_path_set("ls-files", "--others", ...)`. `build_manifest()`,
`_git_path_set()`, `_git_output()` and the other read-only Git helpers inherit the
daemon environment without `GIT_OPTIONAL_LOCKS=0`. With Git's untracked cache,
an otherwise read-only `ls-files --others` is allowed to refresh the shared index
and create `index.lock`. Baseline scans do not hold the finalizer's database
`git_mutex`, so that optional index write can begin while a managed finalizer is
validating and deterministically block its later `write-tree`.

The existing isolated-index repair protects baseline `write-tree` by using a
private `GIT_INDEX_FILE`; it does not constrain separate read-only worktree
enumeration commands. Stale-lock recovery cannot solve this race because the
scan is still active and can recreate the lock after recovery.

## 架构修复验收

- Every Git subprocess in `BaselineService` that observes repository state must
  inherit the normal process environment plus exact `GIT_OPTIONAL_LOCKS=0`.
- The isolated baseline tree path must retain both its private
  `GIT_INDEX_FILE` and `GIT_OPTIONAL_LOCKS=0`.
- A regression must exercise baseline initialization and normal workspace scan
  and prove that `ls-files`, `diff`, and `ls-tree` receive the read-only
  environment.
- The full baseline suite must pass without changing accepted manifest hashes,
  shared index bytes, stale-lock preservation, worktree filter behavior, or
  archive stream handling.
- After commit and controlled rollover, the same frozen Tooling06 exact-three
  snapshot must finalize successfully without manual lock deletion. This real
  consumer is the production concurrency acceptance proof.

## 禁止临时方案

- Do not terminate the live Git process, manually delete `.git/index.lock`,
  clear the shared index, or mutate external staged paths.
- Do not serialize every baseline filesystem scan under the long-lived database
  `git_mutex`; read-only scans must remain concurrent and truly read-only.
- Do not extend finalizer timeouts, retry failed finalizers blindly, disable the
  untracked-path scan, or weaken stale-lock recovery.
- Do not absorb Tooling06 validation worker paths into this Coordinator01 fix.

## 修复结果与回传

- 根因：BaselineService read-only Git observers inherited optional index locking, allowing ls-files untracked-cache refreshes to create the shared index.lock during maintenance finalization.
- 架构修复：Run every baseline Git observer with GIT_OPTIONAL_LOCKS=0 while preserving the private GIT_INDEX_FILE for isolated tree construction, keeping read scans concurrent without permitting shared-index writes.
- 验证：Commit ed543173c passed the focused optional-lock regression and full baseline suite; controlled rollover loaded it. Production Tooling06 exact-three finalizer request 31eaf00e9b6241218cb8b55d1880ac16 then committed 3af73550dd00fe4805f71e96ce199f4ab633687f without manual lock deletion, with external 19-path staged projection SHA-256 a56e70cfec926da8592151ec15a7c85acba64cd44fecee403d594673f45e3b02 unchanged and schema67 successor healthy.
- 回传：Baseline Git observation is now truly read-only, and the frozen Tooling06 maintenance consumer finalized successfully without racing the shared index.
