# Cargo Target Single-Pool Reuse and Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restrict all Cargo output to nine D/E/F drive-root build directories, reuse one exclusive target pool per complete compatibility key across Sessions, immediately delete non-reusable targets, and reclaim idle pools before disks fill.

**Architecture:** The Session coordinator remains the sole authority for managed target ownership and deletion. A validated compatibility document maps to one retained primary directory; missing compatibility produces an ephemeral directory, while the cleanup service owns immediate deletion, retry, stale cleanup, and low-disk LRU eviction. Repository-local skills and PowerShell validators enforce the same nine-root allowlist, and the standalone wrapper handles stale unmanaged directories without bypassing managed-lane denials.

**Tech Stack:** Python 3 standard library and SQLite, Windows PowerShell/Pester, repository-local Codex skills.

**Approved design:** `docs/superpowers/specs/2026-07-12-expand-stale-target-cleanup-design.md`

---

## Milestone M1: Authoritative compatibility key and single primary pool

**Goal:** Make one retained, exclusive target directory the only reusable representation of a complete compatibility key.

**In-scope behaviors:** schema 22 lifecycle columns and historical v21 repair; nine-root policy; canonical key validation; repository/platform/toolchain/architecture/workspace/build-config separation; cross-Session reuse; busy denial instead of fallback allocation; ephemeral-by-default behavior when compatibility is absent.

**Dependencies:** Existing schema 20 supervision work must stabilize first; preserve all concurrent M5 edits and append rather than rewrite released migrations.

### Implementation slices

- [x] **M1.1 Complete schema 22 and focused migration tests.** Modify `tools/session_coordinator/migrations.py` and `tools/session_coordinator/tests/test_database.py`. Keep `reuse_key`, `cleanup_policy`, `cleanup_status`, `reused_from_job_id`, and `cleanup_error`; add a partial unique index that permits at most one active owner per non-null key:

  ```sql
  CREATE UNIQUE INDEX cargo_jobs_active_reuse_key
  ON cargo_jobs(reuse_key)
  WHERE reuse_key IS NOT NULL AND status IN ('leased', 'running');
  ```

  Tests cover fresh schema creation, repair of the historical production database whose v21 marker predated these columns, invalid cleanup enums, and active-key uniqueness.

- [x] **M1.2 Define and validate the compatibility document.** Modify `tools/session_coordinator/cargo_jobs.py` and `tools/session_coordinator/tests/test_cargo_jobs.py`. Add a frozen `CargoCompatibility` value with `platform`, `toolchain`, `target_architecture`, `workspace`, and `build_config`; accept only `windows`/`wsl`, non-empty normalized fields, repository-relative workspace identities without traversal, and canonical JSON serialization. Hash canonical JSON plus the coordinator-owned repository identity:

  ```python
  payload = {
      "repository": target_identity(self.repo_root),
      "platform": compatibility.platform,
      "toolchain": compatibility.toolchain,
      "target_architecture": compatibility.target_architecture,
      "workspace": compatibility.workspace,
      "build_config": compatibility.build_config,
  }
  reuse_key = hashlib.sha256(
      json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
  ).hexdigest()
  ```

  Remove lockfile/content hashing and lane-kind fallback keys. A reusable acquisition requires the complete document; no document means `delete_on_release`. Reject `ephemeral=True` combined with compatibility.

- [x] **M1.3 Enforce one primary directory and exclusive acquisition.** In `CargoJobService.acquire`, query all records for the key inside the writer transaction. If an overlapping leased/running row exists, raise `cargo_reuse_pool_busy`. Reuse the newest existing retained directory; if it was deleted or is missing, mark stale retained rows deleted and allocate exactly one replacement below `<chosen-root>/zircon-engine/pool/<reuse-key>`. Never allocate a second fallback directory for a busy key. Keep target-overlap and cleanup-reservation checks.

- [x] **M1.4 Expand the managed root policy.** Modify `tools/session_coordinator/config.py`, `cargo_jobs.py`, `audit.py`, and focused tests so enabled roots are the nine D/E/F combinations of `cargo-targets`, `targets`, and `ZirconBuilds`. `TargetPathPolicy` accepts only configured roots with those names and validates descendants after realpath resolution; it rejects each root itself, repo-local `target`, symlink escapes, user/temp paths, and other drives.

### Testing stage M1-T

Run after all M1 slices:

```powershell
python -m unittest tools.session_coordinator.tests.test_database tools.session_coordinator.tests.test_cargo_jobs tools.session_coordinator.tests.test_rollout_audit -v
python -m compileall -q tools/session_coordinator
```

Expected: schema upgrade and all key/pool/path tests pass. Debug in order: migration → compatibility normalization → target policy → acquisition transaction. Exit evidence names the passing tests for cross-Session reuse, incompatible-key separation, busy denial, and nine-root rejection.

## Milestone M2: Immediate deletion, retry, and pressure eviction

**Goal:** Delete non-reusable targets as soon as ownership ends and keep reusable pools only while the selected drive has adequate reserve.

**In-scope behaviors:** release-time ephemeral deletion; orphan cleanup; failed-delete retry; LRU idle-pool eviction at 50 GiB; active/reserved path protection; server/maintenance integration.

**Dependencies:** M1 single-pool identity and exclusivity.

### Implementation slices

- [x] **M2.1 Finalize immediate cleanup.** Modify `tools/session_coordinator/cleanup.py`, `server.py`, and `tests/test_cleanup.py`. `cargo.release` first commits release state, then requests the prompt cleanup worker; `cleanup_job_now` reserves and revalidates the exact target, deletes outside the writer transaction, and records deleted/failed. The worker drains release requests that arrive during an active pass. `retry_pending_jobs` retries both pending and failed released/orphaned ephemeral jobs. Orphan reconciliation leaves ephemeral rows pending so maintenance can delete them once the PID is dead.

- [x] **M2.2 Add pressure-driven LRU eviction.** Add `CleanupService.evict_idle_pools_under_pressure()`. Group retained reusable jobs by canonical target, order idle groups by their latest release/reference time ascending, reserve one target, delete it, mark every row for that target deleted, and re-read free space after each deletion. Stop when free space exceeds `50 * 1024**3` or no safe idle target remains. Active leases, live PIDs, and cleanup reservations are denials, never eviction candidates.

- [x] **M2.3 Wire release and maintenance.** After release in `server.py`, request one prompt worker that drains pending deletion and pressure eviction; call both retry and eviction from the periodic maintenance loop. Return whether cleanup was newly scheduled without changing the job ownership contract. Update `docs/cli-and-tooling/local-session-coordinator.md` with reusable/ephemeral states, busy handling, retries, and pressure behavior.

### Testing stage M2-T

```powershell
python -m unittest tools.session_coordinator.tests.test_cleanup tools.session_coordinator.tests.test_cargo_jobs tools.session_coordinator.tests.test_server -v
python -m compileall -q tools/session_coordinator
```

Expected: immediate delete, injected delete failure/retry, orphan cleanup, oldest-idle-first eviction, reserve-restored stop, and active-pool protection tests all pass. Debug in order: job terminal state → reservation → deletion result persistence → pressure ordering → server composition.

## Milestone M3: Skills, validator, and unmanaged cleanup wrapper

**Goal:** Make every documented and automated Cargo path obey the same allowlist while providing safe cleanup for unmanaged stale output.

**In-scope behaviors:** nine-root skill policy for Windows and WSL; validator rejection outside allowlist; compatibility-key generation; stale unmanaged direct-child preview/apply; managed candidate/denial protection.

**Dependencies:** M1/M2 coordinator contracts.

### Implementation slices

- [x] **M3.1 Update repository-local Cargo skills.** Modify `.codex/skills/zircon-dev/SKILL.md`, `.codex/skills/zircon-dev/references/cargo-target-disk-policy.md`, `.codex/skills/zircon-dev/validation/SKILL.md`, `.codex/skills/zircon-dev/validation/manual-commands.md`, `.codex/skills/zircon-project-skills/prefer-windows-validation/SKILL.md`, and related indexed summaries. State that every Cargo invocation must use a descendant of one of the nine roots; WSL may use only mounted equivalents; no repository/user/temp/other-drive fallback is allowed. Document the exact compatibility fields and ephemeral fallback.

- [x] **M3.2 Enforce the validator contract.** Modify `.codex/skills/zircon-dev/scripts/validate-matrix.ps1` and `.codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1`. Validate `-TargetDir`/`CARGO_TARGET_DIR` against the nine canonical roots. When acquiring from the coordinator, generate the canonical platform, toolchain, target architecture, workspace, and build-configuration document and pass it to `cargo acquire`; a deliberately non-reusable run passes `--ephemeral`.

- [x] **M3.3 Expand `tools/cleanup-stale-targets.ps1`.** Add a test-only root override, pure canonical-path/classification helpers, and dot-source-safe invocation. Build the managed set from both `plan.candidates` and `plan.denied[].path`; preview stale unmanaged direct children; under `-Apply`, retain coordinator apply then independently revalidate and `ShouldProcess` each local deletion. Never select a root, file, nested path, link, junction, or fresh directory.

- [x] **M3.4 Add `tools/cleanup-stale-targets.Tests.ps1`.** Use Pester temporary roots to cover stale/fresh filtering, managed candidate and denial exclusion, missing roots, direct-child depth, reparse-point rejection, apply-time freshness revalidation, actual removal, root protection, and `-WhatIf`.

### Testing stage M3-T

```powershell
$matrix = Invoke-Pester .\.codex\skills\zircon-dev\scripts\validate-matrix.Tests.ps1 -PassThru
$cleanup = Invoke-Pester .\tools\cleanup-stale-targets.Tests.ps1 -PassThru
if ($matrix.FailedCount -ne 0 -or $cleanup.FailedCount -ne 0) { exit 1 }
python -m unittest tools.session_coordinator.tests.test_cargo_jobs tools.session_coordinator.tests.test_cleanup -v
git diff --check -- tools/session_coordinator tools/cleanup-stale-targets.ps1 tools/cleanup-stale-targets.Tests.ps1 .codex/skills/zircon-dev .codex/skills/zircon-project-skills/prefer-windows-validation
```

Expected: both Pester suites have zero failures, coordinator regressions pass, and the scoped diff has no whitespace errors. Debug the lowest shared policy/helper first, then rerun upward.

## 状态与产出记录

执行时逐切片填写；完成一个切片更新一行，不许批量补记。

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1 | M1.1 | completed | 2026-07-12 | schema 22 adds/repairs `compatibility_json` and active-key uniqueness; fresh and historical-v21 repair tests passed |
| M1 | M1.2 | completed | 2026-07-12 | `CargoCompatibility`; invalid/missing/conflicting compatibility tests passed |
| M1 | M1.3 | completed | 2026-07-12 | cross-Session reuse, busy denial, missing-primary replacement, and duplicate-retained-pool demotion tests passed |
| M1 | M1.4 | completed | 2026-07-12 | all three root-name policy and rollout-audit tests passed; production config emits nine D/E/F roots |
| M2 | M2.1 | completed | 2026-07-12 | prompt release cleanup, in-flight scheduling drain, injected failure retry, and orphan retry tests passed |
| M2 | M2.2 | completed | 2026-07-12 | oldest-idle eviction, reserve restoration, active-lease and live-PID protection tests passed |
| M2 | M2.3 | completed | 2026-07-12 | release/maintenance scheduling implemented; coordinator documentation updated |
| M3 | M3.1 | completed | 2026-07-12 | five Cargo skill documents now state the nine-root hard allowlist and single-pool lifecycle |
| M3 | M3.2 | completed | 2026-07-12 | validator emits complete compatibility JSON and rejects paths outside all nine roots; `validate-matrix.Tests.ps1`: 70 passed |
| M3 | M3.3 | completed | 2026-07-12 | wrapper scans nine exact roots, protects all coordinator job paths and refreshes before delete |
| M3 | M3.4 | completed | 2026-07-12 | `cleanup-stale-targets.Tests.ps1`: 10 passed, 0 failed |
