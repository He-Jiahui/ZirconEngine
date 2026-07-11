---
related_code:
  - tools/session_coordinator/__main__.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/client.py
  - tools/session_coordinator/config.py
  - tools/session_coordinator/database.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/models.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/sessions.py
  - tools/session_coordinator/baselines.py
  - tools/session_coordinator/snapshots.py
  - tools/session_coordinator/leases.py
  - tools/session_coordinator/patches.py
  - tools/session_coordinator/watch.py
  - tools/session_coordinator/plans.py
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cleanup.py
  - tools/session_coordinator/processes.py
  - tools/zircon-session.ps1
  - tools/cleanup-stale-targets.ps1
  - tools/install-session-coordinator-task.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
implementation_files:
  - tools/session_coordinator/__main__.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/client.py
  - tools/session_coordinator/config.py
  - tools/session_coordinator/database.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/models.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/sessions.py
  - tools/session_coordinator/baselines.py
  - tools/session_coordinator/snapshots.py
  - tools/session_coordinator/leases.py
  - tools/session_coordinator/patches.py
  - tools/session_coordinator/watch.py
  - tools/session_coordinator/plans.py
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cleanup.py
  - tools/session_coordinator/processes.py
  - tools/zircon-session.ps1
  - tools/cleanup-stale-targets.ps1
  - tools/install-session-coordinator-task.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
plan_sources:
  - user: 2026-07-11 implement local multi-Session coordination on shared main
  - docs/superpowers/specs/2026-07-11-local-session-coordinator-design.md
  - docs/superpowers/plans/2026-07-11-local-session-coordinator.md
tests:
  - tools/session_coordinator/tests/test_database.py
  - tools/session_coordinator/tests/test_server.py
  - tools/session_coordinator/tests/test_sessions.py
  - tools/session_coordinator/tests/test_baselines.py
  - tools/session_coordinator/tests/test_snapshots.py
  - tools/session_coordinator/tests/test_leases.py
  - tools/session_coordinator/tests/test_patches.py
  - tools/session_coordinator/tests/test_watch.py
  - tools/session_coordinator/tests/test_concurrent_writers.py
  - tools/session_coordinator/tests/test_plans.py
  - tools/session_coordinator/tests/test_failures.py
  - tools/session_coordinator/tests/test_cargo_jobs.py
  - tools/session_coordinator/tests/test_cleanup.py
  - tools/tests/session-coordinator-smoke.Tests.ps1
doc_type: workflow-detail
---

# Local Session Coordinator

## Purpose

The local Session coordinator is the shared-`main` control plane for ZirconEngine development. It gives each Session a typed lifecycle, records a hash-based workspace baseline, stores intermediate file contents outside Git, serializes concrete file writes, governs plan/failure records, and owns isolated Cargo validation lanes.

Business Session intermediate versions remain service-managed rather than Git commits. Git finalization stays explicit. The service protects unrelated active Sessions and their dirty files without creating branches or worktrees.

## Runtime and State

Run the Windows entrypoint from the repository root:

```powershell
.\tools\zircon-session.ps1 start -Json
.\tools\zircon-session.ps1 status -Json
```

The wrapper starts Python in a hidden window only when the health endpoint is unavailable. The daemon binds an operating-system-assigned port on `127.0.0.1` and writes its port, PID and random bearer token to `.codex/state/session-coordinator/runtime.json`.

All mutable coordinator data remains under `.codex/state/session-coordinator/`:

- `coordinator.sqlite3`: WAL database for Sessions, events, baseline epochs, object indexes, snapshots, attributions, leases and patches;
- `objects/`: zlib-compressed SHA-256 objects;
- `runtime.json`: local connection descriptor;
- `coordinator.lock`: single-instance ownership.

The service validates the active Git branch. A checkout that is not on `main` is diagnostic/read-only: health, Session list and Session show remain available, while mutations fail with `not_on_main`.

## Session Lifecycle

Register the current Codex thread and activate it:

```powershell
.\tools\zircon-session.ps1 session register `
  --display-name "runtime plan 02" `
  --plan-path "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md" `
  --write-scope "tools/session_coordinator"
.\tools\zircon-session.ps1 session set-status active
```

`CODEX_THREAD_ID` is used when `--session-id` is omitted. Manual shells receive a generated UUID if neither is available.

The only persisted status values are `registered`, `active`, `waiting_lease`, `resolving_failure`, `waiting_validation`, `finalizing`, `completed`, `stale`, `archived`, and `cancelled`. The transition table lives in `models.py`; invalid transitions fail without changing the database. Explanatory text belongs in `status_reason` rather than inventing another status string.

## Baseline Epochs

Initialize and inspect the workspace baseline:

```powershell
.\tools\zircon-session.ps1 baseline init
.\tools\zircon-session.ps1 baseline diff
.\tools\zircon-session.ps1 baseline scan
```

An epoch records HEAD, the Git index tree, and SHA-256 hashes for tracked and non-ignored files. Coordinator state is excluded. `baseline scan` compares current content to the epoch. A change does not get reverted; the baseline becomes `degraded` and the path remains on disk.

Attribute a known change and explicitly establish the next epoch:

```powershell
.\tools\zircon-session.ps1 baseline attribute README.md
.\tools\zircon-session.ps1 baseline accept --reason "attribute current Session changes"
```

Acceptance is an explicit reconciliation action. M1-M2 never convert it into a Git commit.

## File Leases

Claim concrete files before writing:

```powershell
.\tools\zircon-session.ps1 lease claim tools/session_coordinator/leases.py
.\tools\zircon-session.ps1 lease heartbeat
.\tools\zircon-session.ps1 lease release tools/session_coordinator/leases.py
```

Paths are resolved under the repository, normalized to case-insensitive keys, sorted, and acquired in one `BEGIN IMMEDIATE` transaction. A multi-file request is all-or-nothing. `.git`, coordinator state and the repository root cannot be leased.

The default lease is five minutes with a two-minute recovery grace. The same Session may renew or reacquire its lease. Another Session receives the conflicting display paths and no partial ownership.

## Snapshots and Delayed Patches

Create a recoverable intermediate snapshot and preview a restoration:

```powershell
.\tools\zircon-session.ps1 snapshot create README.md --purpose "before refactor"
.\tools\zircon-session.ps1 snapshot preview 1
```

Objects are deduplicated by SHA-256 and verified when read. Preview compares object hashes and does not write the workspace.

Queue a unified Git patch with explicit target files:

```powershell
.\tools\zircon-session.ps1 patch enqueue `
  --file E:\temp\change.patch `
  --target README.md
.\tools\zircon-session.ps1 patch status 1
```

If the Session obtains every target lease immediately, the service snapshots the targets, runs `git apply --check`, applies the patch, snapshots the result and records file attribution. If another Session owns a target, the patch is stored as `queued` and the requesting Session moves to `waiting_lease`.

Releasing a lease processes queued patches in creation order. The service recomputes all target hashes first:

- unchanged hashes allow the queued patch to apply;
- changed hashes produce `needs_rebase`, capture the current objects, retain the original base objects and patch object, and leave the workspace content untouched.

This is the overwrite-prevention invariant: queue release never treats a later write as permission to discard an earlier Session's content.

## Failure and Recovery Semantics

- Missing or stale runtime descriptors produce a structured `offline` result and exit code `3`.
- Invalid requests and state transitions produce exit code `2`.
- SQLite transactions roll back as a unit on error.
- Object writes use an atomic temporary-file replacement and verify SHA-256 on read.
- Patch application failures retain snapshots and return `failed`; leases are released in `finally`.
- External workspace edits are preserved and mark the baseline degraded.
- `stop` asks the authenticated service to shut down, then removes only runtime/lock files owned by its PID.

## Test Coverage

M1-T passed eight Python tests plus the kernel PowerShell smoke. After strengthening the single-instance, non-main read-only, background watcher and immediate-apply race coverage, M2-T passed the complete 21-test Python suite, including 20 repeated two-thread lease races, plus the delayed-patch PowerShell smoke. Resource warnings were promoted to errors during the Python suite.

The accepted M1-M2 commands were:

```powershell
python -m compileall -q tools/session_coordinator
python -W error::ResourceWarning -m unittest discover -s tools/session_coordinator/tests -p "test_*.py" -v
powershell -NoProfile -ExecutionPolicy Bypass -File tools/tests/session-coordinator-smoke.Tests.ps1 -KernelOnly
powershell -NoProfile -ExecutionPolicy Bypass -File tools/tests/session-coordinator-smoke.Tests.ps1 -LeaseAndPatch
git diff --check -- tools/session_coordinator tools/zircon-session.ps1 tools/tests/session-coordinator-smoke.Tests.ps1
```

## Plan Ownership and Write Guards

M3 adds recursive plan discovery and write authorization:

```powershell
.\tools\zircon-session.ps1 plan audit -Json
.\tools\zircon-session.ps1 plan owner docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
.\tools\zircon-session.ps1 plan authorize docs/plans/zircon_runtime/frameworks/02/2026-07-11-output.md
```

`docs/plans` is the formal recursive root. `.codex/plans` remains a read-only legacy inventory. A Session registered to a numbered plan may write only below the matching numbered child directory. Ordinary business Sessions are denied for every `index.md`, `engine-code-*.md`, numbered plan-definition Markdown, sibling child directory, repository-external path and non-plan path.

Maintenance is an explicit authorization flag, not an inferred role. It may update protected plan files but cannot escape `docs/plans` or the repository realpath boundary.

## Failure Graph

The existing handoff validator now exports structured `HandoffRecord` values. `failures.py` imports those records into SQLite schema v3 without replacing the Markdown artifacts as canonical truth.

```powershell
.\tools\zircon-session.ps1 failure import -Json
.\tools\zircon-session.ps1 failure audit -Json
.\tools\zircon-session.ps1 failure open docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
```

Graph diagnostics cover schema errors, duplicate lifecycles, self-edges, cycles and excessive dependency depth. Open failures sort before fixed records and then by creation date/slug. Registering a Session with a fixing plan imports current Markdown; applicable failures are returned in `open_failures` and the Session enters `resolving_failure` instead of an untyped blocked state.

After architectural repair and upward validation, `failure return` requires the lifecycle key, accepted-fix date, root cause, architecture repair, validation and return summary. The service rewrites the artifact as `fixed-*`, moves it into the origin child directory, replaces both plan links with concise `fixed 已修复` relative summaries, reruns the validator, and rolls back all file changes if any write or import fails.

## M3 Validation

M3 tests use generated temporary plan trees. They never move or rewrite live business handoff artifacts. The real-repository M3 audit is read-only and reports concurrent invalid/in-progress artifacts as diagnostics rather than treating them as coordinator-owned fixes.

The coordination context script now queries service health, indexed Session count and Failure graph first. Its offline fallback recursively scans both `docs/plans` and `.codex/plans`, correcting the previous formal-root omission.

## Managed Cargo Jobs

Schema v4-v7 records Cargo jobs, cleanup reservations and persisted cleanup plans. Jobs use `check`, `test`, `workspace`, and `gpu` lanes with `leased`, `running`, `succeeded`, `failed`, `released`, and `orphaned` states. Targets must be direct children of an available `D:\targets\zircon-engine\lanes`, `E:\targets\zircon-engine\lanes`, or `F:\targets\zircon-engine\lanes` root. A case- and separator-normalized identity plus ancestor/descendant overlap checks prevent Windows path aliases or nested lanes from becoming simultaneous writers. Repo-local targets, nested lanes, symlink/junction escapes and arbitrary paths fail with `cargo_target_not_managed`.

```powershell
.\tools\zircon-session.ps1 cargo acquire workspace
.\tools\zircon-session.ps1 cargo list
```

`validate-matrix.ps1` performs the lifecycle automatically: register the caller, acquire a unique lane with the wrapper PID, immediately enter `try/finally`, record the process command line at start, run validation, record the exit code, and owner-checked release. Explicit `-TargetDir` and inherited `CARGO_TARGET_DIR` are normalized through the same policy; released explicit lanes may be reused. Dry-run jobs are audited but their directories are not created. The daemon converts dead running jobs and dead/timed-out pre-start leases to `orphaned`.

## Cleanup and Scheduled Maintenance

Cleanup is deliberately two-phase:

```powershell
.\tools\cleanup-stale-targets.ps1
.\tools\cleanup-stale-targets.ps1 -Apply -WhatIf
.\tools\cleanup-stale-targets.ps1 -Apply
```

Planning persists an immutable, expiring `plan_id` with its candidate snapshot, retention and status. Apply accepts only that server-stored plan, can run once, and may shrink it after revalidating job history, direct-child realpath, overlapping live PID, active lease and positive retention. An untracked directory is never deletable even if a plan row is corrupted. A short SQLite transaction writes a cleanup reservation; deletion runs outside the global writer lock; a final short transaction records success/failure and clears the reservation. New Cargo acquisition observes the reservation, and daemon restart recovers abandoned reservations. The script never enumerates fuzzy drive-root names and never deletes directly.

The user-level scheduler definition is idempotent and repository-specific:

```powershell
.\tools\install-session-coordinator-task.ps1 -Action Install -DryRun
.\tools\install-session-coordinator-task.ps1 -Action Update -DryRun
.\tools\install-session-coordinator-task.ps1 -Action Query -DryRun
.\tools\install-session-coordinator-task.ps1 -Action Remove -DryRun
```

Installation creates a hidden at-logon daemon task and a 15-minute maintenance task with limited user privileges. Dry-run prints exact commands without changing Task Scheduler. No webhook address, credential or machine secret is part of this service configuration.

## M4 Validation

M4 adds unit coverage for direct-child and junction/symlink escapes, unavailable roots, case aliases, nested legacy overlap, explicit reuse, foreign-session mutation, running/pre-start orphan reconciliation, positive retention, reviewed-plan non-expansion, reservation/acquire concurrency and transaction-free deletion. The PowerShell smoke also validates the scheduled-task plan, while validator tests assert that command-line/environment overrides cannot bypass the service and pre-start failures release their job.

## Explicit Git Finalize

Completing a business Session records lifecycle state only; it never creates a Git commit. A commit requires a separate explicit request. Preview is read-only with respect to Git, while `--commit` enters the serialized Git transaction:

```powershell
.\tools\zircon-session.ps1 finalize preview `
  --message "feat(runtime): converge lifecycle" `
  --path zircon_runtime/src/lifecycle.rs

.\tools\zircon-session.ps1 finalize --commit `
  --message "feat(runtime): converge lifecycle" `
  --path zircon_runtime/src/lifecycle.rs
```

Every requested path must be attributed to the completed Session at its current SHA-256 hash. Every other dirty path attributed to that Session must also appear in the manifest, so untracked files, documentation, tests and scripts cannot be silently omitted. The durable finalize request records four categories (`code`, `docs`, `tests`, `scripts`) and a separate `untracked_paths` inventory.

Before index mutation, the service rejects a degraded/stale baseline, an active Git mutex, foreign leases, queued or `needs_rebase` patches, foreign staged paths, protected/global plan output, output outside the registered numbered child plan, and an unresolved Failure routed to the Session plan. Staged added lines are also scanned for Enterprise WeChat webhook URLs and credential markers. Webhook configuration remains local and must never enter Git.

The service persists the pre-transaction HEAD and exact Git index bytes after taking its database mutex, calculates each approved worktree file's Git-cleaned blob identity, stages only the approved paths, then verifies both the exact staged name set and staged blob identities. This closes the last-write race between attribution checking and staging. After optional validation commands, it repeats scope, blob and secret checks. The final commit is built from the verified index tree and advances `HEAD` with an expected-old-SHA compare-and-swap, so repository hooks or validation cannot silently widen the commit. A pre-commit or validation failure atomically restores the prior index and preserves every worktree file. Successful commits record their SHA and open a new baseline epoch that advances only committed paths; other Sessions' dirty files remain visible as baseline differences.

Service restart restores a persisted pre-commit index when HEAD did not advance, returns an interrupted Session to `completed`, and marks the request failed. The `ref_updated_sha` intent closes the post-commit/pre-baseline window: if the exact expected scoped commit already advanced HEAD before a process interruption, startup reconciles its SHA and commit-derived partial baseline instead of reporting a false failure or capturing the full dirty worktree.

Health probes keep a short three-second timeout. Mutating service commands use a separate five-minute client timeout so a busy shared disk, Git hook or configured validation command is not misreported as an offline daemon.

Workflow/skill maintenance uses the same transaction with explicit `--maintenance`; the daemon authorizes it with a separate local `ZIRCON_COORDINATOR_MAINTENANCE_TOKEN` capability that is never written to the runtime descriptor or Git. The ordinary shared service bearer and a client boolean are insufficient. Authorized maintenance bypasses business attribution/status checks but retains index scope, repository path, semantic-message and secret guards. Business intermediate versions continue to live in coordinator snapshots rather than Git history.

## Stable Validation Copies

Validation copies provide a stable source view without a branch, worktree or repo-local build directory:

```powershell
.\tools\zircon-session.ps1 validation-copy materialize `
  --path Cargo.toml `
  --path zircon_runtime/Cargo.toml `
  --path zircon_runtime/src/lib.rs

.\tools\zircon-session.ps1 validation-copy run <job-id> -- cargo check --workspace
.\tools\zircon-session.ps1 validation-copy cleanup <job-root>
```

The manifest is materialized under `{drive}:\targets\zircon-engine\verify\{job-id}\source`. Planning pins one HEAD SHA; every unowned tracked path is read from that exact commit, so concurrent finalization cannot create a mixed-version copy. Paths owned by the requesting Session are copied from the worktree only while their attribution hash still matches. Unowned untracked paths, `.git`, coordinator state and repository build output are rejected. The resolved `verify` root and job root are revalidated during plan, materialize, run and cleanup; junction/symlink escapes fail closed.

Validation commands acquire the job's `running` state and run with `CARGO_TARGET_DIR` fixed to the adjacent `{job-root}\target`; a second run and cleanup are rejected until execution returns to `materialized`. Exit code and bounded stdout/stderr evidence are stored in SQLite. Cleanup requires the owning Session, atomically reserves `cleanup_pending`, and cannot race a run.

Cleanup accepts only a job root already recorded by the service and only when its resolved path is a direct child of an allowlisted `verify` root. It removes that single job tree, including the adjacent target, then records the removal.

Validation runs persist the real validation child PID. Startup releases only dead `running` reservations, while live processes remain protected. An interrupted `cleanup_pending` reservation returns to `materialized` only during startup; periodic maintenance never releases a deletion still owned by the live daemon. Ordinary deletion failures also roll back the reservation so the owner can retry safely.

## M5 Validation

M5 acceptance uses generated temporary Git repositories for all commit mutations. Coverage proves completion is non-committing, explicit finalization contains exactly the approved categorized files, foreign index state survives rejection, validation failure restores the index, webhook material is blocked, the Git mutex has one owner, validation overlays reject stale hashes, command evidence uses the adjacent target, and cleanup cannot escape its job root.

## Follow-up Boundaries

M6 will migrate and archive legacy Session notes, perform scheduled-task cutover, add retention/GC and complete the real-repository rollout audit. M5 does not import or delete legacy Session artifacts.
