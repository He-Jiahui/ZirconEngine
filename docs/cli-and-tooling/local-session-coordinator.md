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
  - tools/session_coordinator/legacy.py
  - tools/session_coordinator/audit.py
  - tools/session_coordinator/processes.py
  - tools/session_coordinator/supervision/
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/control_plane/auth.py
  - tools/session_coordinator/control_plane/contracts.py
  - tools/session_coordinator/control_plane/events.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/control_plane/http_security.py
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/control_plane/actions/catalog.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/actions/fingerprint.py
  - tools/session_coordinator/control_plane/actions/service.py
  - tools/session_coordinator/workflows/models.py
  - tools/session_coordinator/workflows/store.py
  - tools/session_coordinator/workflows/projections.py
  - tools/zircon-session.ps1
  - tools/cleanup-stale-targets.ps1
  - tools/install-session-coordinator-task.ps1
  - tools/install-session-tray-startup.ps1
  - tools/session_tray/
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
  - tools/session_coordinator/legacy.py
  - tools/session_coordinator/audit.py
  - tools/session_coordinator/processes.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/control_plane/auth.py
  - tools/session_coordinator/control_plane/contracts.py
  - tools/session_coordinator/control_plane/events.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/control_plane/http_security.py
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/control_plane/actions/catalog.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/actions/fingerprint.py
  - tools/session_coordinator/control_plane/actions/service.py
  - tools/session_coordinator/workflows/models.py
  - tools/session_coordinator/workflows/store.py
  - tools/session_coordinator/workflows/projections.py
  - tools/zircon-session.ps1
  - tools/cleanup-stale-targets.ps1
  - tools/install-session-coordinator-task.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
plan_sources:
  - user: 2026-07-11 implement local multi-Session coordination on shared main
  - docs/superpowers/specs/2026-07-11-local-session-coordinator-design.md
  - docs/superpowers/plans/2026-07-11-local-session-coordinator.md
  - docs/superpowers/specs/2026-07-11-session-goal-milestone-closeout-design.md
  - docs/superpowers/plans/2026-07-11-session-goal-milestone-closeout-skill.md
  - docs/superpowers/specs/2026-07-11-workflow-control-center-and-tray-design.md
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
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
  - tools/session_coordinator/tests/test_legacy_migration.py
  - tools/session_coordinator/tests/test_retention.py
  - tools/session_coordinator/tests/test_rollout_audit.py
  - tools/session_coordinator/tests/test_git_finalize.py
  - tools/session_coordinator/tests/test_workflow_schema.py
  - tools/session_coordinator/tests/test_workflow_store.py
  - tools/session_coordinator/tests/test_workflow_projections.py
  - tools/session_coordinator/tests/test_control_auth.py
  - tools/session_coordinator/tests/test_control_events.py
  - tools/session_coordinator/tests/test_control_http.py
  - tools/session_coordinator/tests/test_control_security.py
  - tools/session_coordinator/tests/test_control_snapshot.py
  - tools/session_coordinator/tests/test_action_catalog.py
  - tools/session_coordinator/tests/test_action_auth.py
  - tools/session_coordinator/tests/test_action_fingerprint.py
  - tools/session_coordinator/tests/test_action_execution.py
  - tools/session_coordinator/tests/test_action_concurrency.py
  - tools/tests/session-coordinator-smoke.Tests.ps1
doc_type: workflow-detail
---

# Local Session Coordinator

## Purpose

The local Session coordinator is the shared-`main` control plane for ZirconEngine development. It gives each Session a typed lifecycle, records a hash-based workspace baseline, stores intermediate file contents outside Git, serializes concrete file writes, governs plan/failure records, and owns isolated Cargo validation lanes.

Business Session work remains service-managed between accepted milestones. Every accepted milestone is an explicit service-owned Git commit; arbitrary checkpoints and hidden intermediate commits remain forbidden. The service protects unrelated active Sessions and their dirty files without creating branches or worktrees.

## Runtime and State

Run the Windows entrypoint from the repository root:

```powershell
.\tools\zircon-session.ps1 start -Json
.\tools\zircon-session.ps1 status -Json
```

The wrapper starts Python in a hidden window only when the health endpoint is unavailable. The daemon binds an operating-system-assigned port on `127.0.0.1` and writes its port, PID and random bearer token to `.codex/state/session-coordinator/runtime.json`.

Schema version 16 completes the permissioned controlled-action protocol on top of the read-only workflow facade. It closes `action_kind` at the database boundary and installs compatibility triggers for databases that already applied the early v15 action tables. The runtime descriptor also records the daemon `instance_id`, `started_at`, and supported `control_api_versions`, allowing local clients to reject credentials created by a previous daemon instance. Detailed operator guidance lives in [Workflow Control Center](workflow-control-center.md); module contracts live in [Control Plane](../tools/session_coordinator/control-plane.md) and [Workflow Read Model](../tools/session_coordinator/workflows.md).

Open the current Observer surface or inspect the same coherent snapshot from the terminal:

```powershell
.\tools\zircon-session.ps1 ui ticket --role observer -Json
.\tools\zircon-session.ps1 ui open
.\tools\zircon-session.ps1 control snapshot -Json
```

Observer tickets are opaque, single-use and valid for 30 seconds. Consumption creates an eight-hour `HttpOnly`, `SameSite=Strict` cookie bound to the current daemon instance and scoped to `/control`. M3 mutations require a CLI/tray-issued one-use elevation grant, short-lived role, Session binding and rotated CSRF token; the browser cannot self-elevate or submit arbitrary commands or paths.

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

Attribute a known change, then reconcile the existing epoch without absorbing any dirty file:

```powershell
.\tools\zircon-session.ps1 baseline attribute README.md
.\tools\zircon-session.ps1 baseline reconcile
```

`baseline reconcile` recalculates every difference, requires exact current-hash attribution, clears only the degraded marker, and keeps the epoch manifest unchanged. It fails with the remaining paths if even one change is unattributed. `baseline accept --reason ...` is a separate operator override that captures a new full-worktree epoch; do not use it to clear degradation in a shared dirty workspace. Neither action creates a Git commit.

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

Graph diagnostics cover schema errors, duplicate lifecycles, self-edges, cycles and excessive dependency depth. The filename prefix supplies the coordinator's canonical `open`/`fixed` state; a conflicting frontmatter status remains a validator diagnostic but cannot abort the graph transaction or unrelated Cargo/Session commands. Open failures sort before fixed records and then by creation date/slug. Registering a Session with a fixing plan imports current Markdown; applicable failures are returned in `open_failures` and the Session enters `resolving_failure` instead of an untyped blocked state.

After architectural repair and upward validation, `failure return` requires the lifecycle key, accepted-fix date, root cause, architecture repair, validation and return summary. The service rewrites the artifact as `fixed-*`, moves it into the origin child directory, replaces both plan links with concise `fixed 已修复` relative summaries, reruns the validator, and rolls back all file changes if any write or import fails.

## M3 Validation

M3 tests use generated temporary plan trees. They never move or rewrite live business handoff artifacts. The real-repository M3 audit is read-only and reports concurrent invalid/in-progress artifacts as diagnostics rather than treating them as coordinator-owned fixes.

The coordination context script now queries service health, indexed Session count and Failure graph first. Its offline fallback recursively scans both `docs/plans` and `.codex/plans`, correcting the previous formal-root omission.

## Managed Cargo Jobs

Schema v4-v7 records Cargo jobs, cleanup reservations and persisted cleanup plans; schema v22 adds reusable-cache identity and cleanup state and repairs databases whose historical v21 marker predated those columns. Jobs use `check`, `test`, `workspace`, and `gpu` lanes with `leased`, `running`, `succeeded`, `failed`, `released`, and `orphaned` states. Targets must remain below one of the nine drive-root trees named `cargo-targets`, `targets`, or `ZirconBuilds` on `D:`, `E:`, or `F:`. A case- and separator-normalized identity plus ancestor/descendant overlap checks prevent Windows path aliases or nested pools from becoming simultaneous writers. Repo-local targets, symlink/junction escapes and arbitrary paths fail with `cargo_target_not_managed`.

```powershell
.\tools\zircon-session.ps1 cargo acquire workspace --ephemeral
.\tools\zircon-session.ps1 cargo acquire test --compatibility-json '{"platform":"windows","toolchain":"1.88.0@x86_64-pc-windows-msvc","target_architecture":"x86_64-pc-windows-msvc","workspace":"Cargo.toml","build_config":"profile=test;features=default"}'
.\tools\zircon-session.ps1 cargo acquire check --ephemeral
.\tools\zircon-session.ps1 cargo list
```

Reusable acquisition requires a complete compatibility document containing platform (`windows` or `wsl`), Rust toolchain, target architecture, repository-relative workspace and canonical build configuration. The service adds normalized repository identity and hashes that document. Source and `Cargo.lock` changes deliberately do not split the pool because Cargo performs unit-level invalidation. Check/test lane labels also do not split it. Exactly one primary directory exists per compatibility key across Sessions and exactly one task may own it; concurrent compatible acquisition returns `cargo_reuse_pool_busy` instead of creating a fallback pool. Legacy duplicate retained directories are demoted to prompt deletion while the newest remains authoritative. Missing compatibility metadata fails closed to ephemeral by default, as does an explicit `--ephemeral` request; release commits ownership state and wakes a single worker that drains pending requests, reserves and revalidates each exact directory, then deletes outside the writer transaction. A locked deletion becomes `failed`; release-driven cleanup leaves it alone, and the daemon's default 30-second watch loop retries failed Cargo cleanup.

`validate-matrix.ps1` performs the Windows lifecycle automatically: register the caller, derive the compatibility document, acquire the primary pool with the wrapper PID, immediately enter `try/finally`, record the process command line at start, run validation, record the exit code, and owner-checked release. WSL Cargo is permitted only through a coordinator-aware Windows host wrapper that acquires with `platform=wsl`, remains alive and heartbeats while its `wsl.exe` child runs, and translates only the granted path to its mounted equivalent; direct unleased WSL Cargo is forbidden. Explicit `-TargetDir` and inherited `CARGO_TARGET_DIR` are normalized through the same policy and cannot create an alternate primary directory. Dry-run jobs are audited but their directories are not created. The daemon converts dead running jobs and dead/timed-out pre-start leases to `orphaned` and immediately retries pending ephemeral cleanup.

## Cleanup and Service-Owned Maintenance

Cleanup is deliberately two-phase:

```powershell
.\tools\cleanup-stale-targets.ps1
.\tools\cleanup-stale-targets.ps1 -Apply -WhatIf
.\tools\cleanup-stale-targets.ps1 -Apply
```

Reusable caches use the reviewed two-phase retention path. Planning persists an immutable, expiring `plan_id` with its candidate snapshot, retention and status. Apply accepts only that server-stored plan, can run once, and may shrink it after revalidating job history, managed-root realpath, overlapping live PID, active lease and positive retention. Under disk pressure, the daemon evicts idle reusable pools oldest-first until the free-space reserve is restored; active pools remain protected. Ephemeral lanes bypass only the age delay, never the path/identity/process/lease checks. A short SQLite transaction writes a cleanup reservation; deletion runs outside the global writer lock; a final short transaction records success/failure and clears the reservation. New Cargo acquisition observes the reservation, and daemon restart recovers abandoned reservations.

The wrapper scans only direct child directories of the exact nine approved roots. It excludes every coordinator-known target and any ancestor containing a nested managed pool, skips reparse points, checks the stale cutoff, refreshes coordinator state immediately before deletion, and directly removes only stale unmanaged children. This is the intentional unmanaged exception: they have no service job history to retain, while managed paths remain service-owned.

The user-level startup definition is idempotent and repository-specific. The preferred backend uses Task Scheduler; systems that deny task creation can use the current-user Run key while the daemon owns the 15-minute maintenance cadence internally:

```powershell
.\tools\install-session-coordinator-task.ps1 -Action Install -DryRun
.\tools\install-session-coordinator-task.ps1 -Action Update -DryRun
.\tools\install-session-coordinator-task.ps1 -Action Query -DryRun
.\tools\install-session-coordinator-task.ps1 -Action Remove -DryRun
.\tools\install-session-coordinator-task.ps1 -Action Cutover -Backend UserStartup -DryRun
```

The scheduled-task backend creates one hidden at-logon daemon task with limited user privileges. The `UserStartup` backend writes only a repo-hash-scoped `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` value. In both cases the daemon itself runs legacy-note import/archive, stale Session archive, snapshot/object retention, and heavy managed Cargo maintenance every 15 minutes under a non-blocking maintenance mutex; failed ephemeral Cargo cleanup is lighter and follows the default 30-second watch loop. Any older repo-scoped external maintenance task is disabled only after the daemon passes the health/tick gates.

Cutover writes a durable `preparing` record before the first startup mutation, journals each legacy-task disable, and preserves the original enablement set across idempotent reruns. An interrupted `preparing` record must be rolled back explicitly before another cutover. Both backends verify the exact startup command, and rollback restores only tasks that were enabled before cutover. Task-not-found is distinguished from registry/task access failures, and automatic legacy discovery accepts only the canonical cleanup script or an exact `cd /d <repo-root>` action, so a similarly prefixed repository cannot be retired accidentally. Dry-run prints exact commands without changing either backend. No webhook address, credential or machine secret is part of this service configuration.

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

An accepted milestone uses the same service-owned mutex while keeping the Session `active`:

```powershell
.\tools\zircon-session.ps1 finalize --commit --milestone `
  --session-id <session-id> `
  --message "feat(runtime): complete M2 milestone" `
  --path zircon_runtime/src/lifecycle.rs `
  --path docs/plans/zircon_runtime/frameworks/02/2026-07-11-m2.md
```

Git subjects must be plain Conventional Commits and must not begin with a full-width module prefix. For the example plan under `docs/plans/zircon_runtime/frameworks/`, the service uses `frameworks` only when it formats the WeCom first line as `核心内容摘要：【frameworks】...`; the committed subject and the notification's fourth line remain unprefixed.

Milestone commit paths must have live leases owned by the Session and current-hash attribution. The service re-imports canonical Failure Markdown, rejects validator diagnostics or open Failure nodes where the Session plan is either origin or fixer, takes `git_mutex`, rechecks the exact index and staged blob identities, and advances `HEAD` with compare-and-swap. The Session remains active after success. This command is the only business-Session commit path; a plain `git commit` is outside the workflow.

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

## Legacy Session Migration

Migration is report-first. The report parser reads only root-level `.codex/sessions/*.md`, accepts current YAML frontmatter and older loose `key: value` notes, computes a SHA-256 for every source, and never treats `.codex/sessions/archive/` as active input.

```powershell
.\tools\zircon-session.ps1 legacy report --report E:\temp\zircon-legacy-report.json -Json
.\tools\zircon-session.ps1 legacy import --dry-run --report E:\temp\zircon-import-preview.json -Json
.\tools\zircon-session.ps1 legacy import --apply --report E:\temp\zircon-import-applied.json -Json
```

Known status aliases map to the fixed enum. `working`, `in_progress`, and `implementing` become `active`; `done` and `complete` become `completed`; exact service statuses remain exact. Unknown or retired values such as `blocked` are preserved verbatim in `status_reason` and classified from evidence rather than persisted as a new status. A live PID, a note updated inside ten minutes, an active service heartbeat/lease, a pending/rebase patch, or an open Failure overrides even a terminal source label and keeps the note active. Without activity evidence, the note becomes `stale`. Import is hash-keyed and idempotent, preserves newer service state, imports a numbered plan link where available, uses the source mtime for legacy timestamps, clears obsolete terminal timestamps on reactivation, and never moves or deletes the source note.

Archive is a separate explicit operation:

```powershell
.\tools\zircon-session.ps1 legacy archive --dry-run --report E:\temp\zircon-archive-preview.json -Json
.\tools\zircon-session.ps1 legacy archive --apply --report E:\temp\zircon-archive-applied.json -Json
```

Only a `stale`, `completed`, or `cancelled` note older than 24 hours with no live reference is eligible. Apply first persists a full `planned` intent, rechecks activity while holding the same SQLite writer reservation used by heartbeat/lease changes, moves it to a new collision-safe path under `.codex/sessions/archive/`, verifies SHA-256, and then changes the service Session to `archived`. Startup restores every moved file from any intent that never committed. The daemon performs this journaled operation periodically; live/recent notes are excluded from both stale and archive transitions.

## Snapshot Retention and Object GC

Object collection is also two-phase:

```powershell
.\tools\zircon-session.ps1 retention plan --report E:\temp\zircon-retention-plan.json -Json
.\tools\zircon-session.ps1 retention apply --plan-id <plan-id> --dry-run -Json
.\tools\zircon-session.ps1 retention apply --plan-id <plan-id> -Json
```

Active Session snapshots are retained. Completed/cancelled snapshots remain for 14 days, archived snapshots for 30 days, and a snapshot created after an old terminal timestamp receives its own full retention window. Every object referenced by a retained snapshot or delayed-patch record remains live. Object producers write content plus the referencing row in one SQLite writer transaction. GC holds that same writer reservation from final candidate revalidation through quarantine moves and database deletion, so no concurrent producer can create an unrestorable reference. Startup restores pre-commit quarantine and discards only residue whose plan already committed; failed deterministic plans can be safely replanned and retried.

## Maintenance and Rollout Audit

One maintenance tick performs enum-only stale classification, Cargo orphan reconciliation, dead validation-child recovery, a WAL checkpoint, and retention/Cargo cleanup planning. The daemon-owned periodic tick also imports and journal-archives inactive root notes, archives service-native stale Sessions after 24 hours with no lease/patch/Failure, and applies revalidated retention/Cargo plans:

```powershell
.\tools\zircon-session.ps1 maintenance tick -Json
.\tools\zircon-session.ps1 maintenance tick --apply-cleanup --apply-retention -Json
.\tools\zircon-session.ps1 audit all --report E:\temp\zircon-rollout-audit.json -Json
```

All apply-style migration, retention, and cleanup commands require the separate local `ZIRCON_COORDINATOR_MAINTENANCE_TOKEN` in both daemon and operator-client environments. The shared runtime bearer can report, plan, and audit, but cannot import/archive Sessions or delete snapshots, objects, or Cargo lanes. The daemon's internal periodic path does not expose this capability through `runtime.json`.

`audit all` is read-only and deterministic for unchanged inputs. It reports branch, baseline health, enum violations, Session count, recursive formal and legacy plan counts, Failure validator diagnostics, configured target roots, unsafe recorded Cargo targets, legacy Session/archive counts, legacy repo-local Cargo artifacts, and successful maintenance-tick count. Repo-local `target/codex-shared-*` paths are diagnostics only; rollout never imports or deletes them.

## Startup Cutover and Rollback

Review the exact task commands first:

```powershell
.\tools\install-session-coordinator-task.ps1 -Action Cutover -DryRun
.\tools\install-session-coordinator-task.ps1 -Action Cutover
.\tools\install-session-coordinator-task.ps1 -Action Cutover -Backend UserStartup -DryRun
.\tools\install-session-coordinator-task.ps1 -Action Cutover -Backend UserStartup
```

Cutover creates/updates either the repo-hash-scoped at-logon task or the current-user startup value. It persists an atomic `preparing` rollback record, starts the daemon, requires health → plan-only maintenance → health → plan-only maintenance → health, verifies the exact repo-scoped legacy task is not running, and only then disables it. The daemon owns later destructive ticks, so old and new cleanup actors never overlap. Every disable is journaled immediately; any error removes/disables the new startup, stops the daemon, and re-enables only tasks changed by that run. It never deletes the legacy task. On the 2026-07-11 workstation, task creation was denied by local Windows policy, so the reviewed `UserStartup` backend completed the gate.

```powershell
.\tools\install-session-coordinator-task.ps1 -Action Rollback -DryRun
.\tools\install-session-coordinator-task.ps1 -Action Rollback
```

Rollback disables/removes the new startup registration, verifies the coordinator is offline, and only then re-enables the exact recorded legacy tasks. Registry deletion errors fail closed. Webhook URLs, maintenance capabilities, runtime bearer tokens, and machine-specific task state never enter Git.

## Recovery and Emergency Offline Mode

- Queued patches, object manifests, cleanup plans, finalize intents, archive manifests, and maintenance ticks are durable SQLite records. Restart the daemon with `zircon-session.ps1 start`; startup reconciles stale locks before accepting mutations.
- A finalize interrupted before ref update restores the persisted index. A ref-updated/baseline-pending finalize rebuilds the baseline from the exact commit before marking the request committed.
- A validation copy records the real child PID. Startup and periodic maintenance release `running` only after that PID dies. `cleanup_pending` is recovered only at daemon startup, never while the live daemon may still be deleting.
- If the daemon is unavailable, stop writes that require leases/finalize, preserve worktree files, and run `status -Json` for structured diagnostics. Session notes remain a compatibility view, but they do not grant file ownership.
- For emergency read-only evidence, use ordinary Git read commands and the Failure/plan validators. Do not run direct target deletion, invent a free-form status, write global plan indexes, or create a checkpoint commit.

## M6 Validation

M6 temporary-repository tests cover deterministic legacy reports, unknown-status preservation, live PID/reference classification, idempotent import, hash-preserving archive, retention with live patch references, quarantine-backed GC rollback boundaries, archived restore preview, pinned plan-root audit, daemon-owned maintenance ticks, and both startup cutover dry-runs. The real rollout imported 131 root notes, archived 121 with identical hashes, retained 10 active/recent notes, recorded four successful ticks, and left one repo-local legacy Cargo root diagnostic-only. Baseline reconciliation remained fail-closed on 64 changes owned by concurrent business Sessions.
