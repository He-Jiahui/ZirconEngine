---
related_code:
  - tools/jenkins/install-coordinator-worker.ps1
  - tools/zircon-session.ps1
  - tools/build-editor.ps1
  - tools/maintenance/cleanup-stale-targets.ps1
  - tools/install-session-coordinator-task.ps1
  - tools/install-session-tray-startup.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
implementation_files:
  - tools/jenkins/install-coordinator-worker.ps1
  - tools/zircon-session.ps1
  - tools/build-editor.ps1
  - tools/maintenance/cleanup-stale-targets.ps1
  - tools/install-session-coordinator-task.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
plan_sources:
  - user: 2026-10-01 authorize repairing shared coordinator storage support and restoring service for the isolated Jenkins pilot
  - user: 2026-09-29 update the Coordinator operator guide for storage policy, Failure claims, validation scheduling and single-port remote workers
  - user: 2026-07-17 remove global coordinator blocking while retaining scoped finalization safety
  - user: 2026-07-17 optimize coordinator storage after unmanaged Cargo-artifact scan revealed terminal index snapshot growth
  - docs/superpowers/plans/2026-07-17-coordinator-terminal-index-snapshot-retention.md
  - user: 2026-07-16 reduce coordinator friction using two days of session evidence and improve the visual work board
  - docs/superpowers/specs/2026-07-16-coordinator-flow-efficiency-design.md
  - docs/superpowers/plans/2026-07-16-coordinator-flow-efficiency-m1.md
  - user: 2026-07-16 keep coordinator admission nonblocking and replay safe local requests after startup
  - docs/superpowers/plans/2026-07-16-coordinator-offline-replay-nonblocking.md
  - user: 2026-07-11 implement local multi-Session coordination on shared main
  - docs/superpowers/specs/2026-07-11-local-session-coordinator-design.md
  - docs/superpowers/plans/2026-07-11-local-session-coordinator.md
  - docs/superpowers/specs/2026-07-11-session-goal-milestone-closeout-design.md
  - docs/superpowers/plans/2026-07-11-session-goal-milestone-closeout-skill.md
  - docs/superpowers/specs/2026-07-11-workflow-control-center-and-tray-design.md
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
  - docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-15-milestone-finalize-session-relative-owned-scope.md
  - docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-15-support-slice-exact-finalize-plan-output-conflict.md
  - docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-16-stale-session-pending-cpu-reservation-starvation.md
  - docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-16-legacy-open-failure-pins-stale-sessions.md
  - docs/plans/zircon_tooling/session_coordinator/01/failure-2026-08-16-build-editor-product-staging-unregistered.md
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
  - tools/session_coordinator/tests/test_cargo_reservations.py
  - tools/session_coordinator/tests/test_validation_external_worktree_snapshot.py
  - tools/session_coordinator/tests/test_cleanup.py
  - tools/session_coordinator/tests/test_artifact_governance.py
  - tools/session_coordinator/tests/test_artifact_product_staging.py
  - tools/session_coordinator/tests/test_legacy_migration.py
  - tools/session_coordinator/tests/test_retention.py
  - tools/session_coordinator/tests/test_rollout_audit.py
  - tools/session_coordinator/tests/test_git_finalize.py
  - tools/session_coordinator/tests/test_git_guard.py
  - tools/session_coordinator/tests/test_workflow_schema.py
  - tools/session_coordinator/tests/test_workflow_store.py
  - tools/session_coordinator/tests/test_workflow_projections.py
  - tools/session_coordinator/tests/test_control_auth.py
  - tools/session_coordinator/tests/test_control_events.py
  - tools/session_coordinator/tests/test_control_http.py
  - tools/session_coordinator/tests/test_control_security.py
  - tools/session_coordinator/tests/test_control_snapshot.py
  - tools/session_coordinator/tests/test_codex_store.py
  - tools/session_coordinator/web/src/__tests__/contracts.test.ts
  - tools/session_coordinator/web/src/__tests__/components.test.tsx
  - tools/session_coordinator/tests/test_action_catalog.py
  - tools/session_coordinator/tests/test_action_auth.py
  - tools/session_coordinator/tests/test_action_fingerprint.py
  - tools/session_coordinator/tests/test_action_execution.py
  - tools/session_coordinator/tests/test_action_concurrency.py
  - tools/session_coordinator/tests/test_milestone_cli.py
  - tools/session_coordinator/tests/test_offline_command_spool.py
  - tools/session_coordinator/tests/test_failure_claims.py
  - tools/session_coordinator/tests/test_failure_patch_claims.py
  - tools/session_coordinator/tests/test_storage_ledger.py
  - tools/session_coordinator/tests/test_storage_path_index.py
  - tools/session_coordinator/tests/test_low_disk_gc.py
  - tools/session_coordinator/tests/test_cache_budget.py
  - tools/session_coordinator/tests/test_validation_timings.py
  - tools/session_coordinator/tests/test_storage_admission.py
  - tools/session_coordinator/tests/test_cargo_storage_admission.py
  - tools/session_coordinator/tests/test_validation_priority.py
  - tools/session_coordinator/tests/test_validation_groups.py
  - tools/session_coordinator/tests/test_durable_tasks.py
  - tools/session_coordinator/tests/test_worker_credentials.py
  - tools/session_coordinator/tests/test_worker_protocol.py
  - tools/session_coordinator/tests/test_worker_client.py
  - tools/session_coordinator/tests/test_worker_execution.py
  - tools/session_coordinator/tests/test_remote_paths.py
  - tools/session_coordinator/tests/test_remote_objects.py
  - tools/session_coordinator/tests/test_remote_transport.py
  - tools/session_coordinator/tests/test_artifact_relay.py
  - tools/session_coordinator/tests/test_coordinator_remote_e2e.py
  - tools/session_coordinator/tests/test_upgrade_composition_lifecycle.py
  - tools/session_coordinator/tests/test_upgrade_server_routes.py
  - tools/session_coordinator/tests/test_failure_closeout.py
  - tools/session_coordinator/tests/test_provider_registry.py
  - tools/session_coordinator/tests/test_optimization_cli.py
  - tools/session_coordinator/tests/test_optimization_actions.py
  - tools/session_coordinator/tests/test_tls_operator_client.py
  - tools/session_coordinator/tests/test_control_storage.py
  - tools/session_coordinator/tests/test_control_tasks.py
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - tools/tests/session-coordinator-smoke.Tests.ps1
  - tools/tests/build-editor.Tests.ps1
doc_type: workflow-detail
---

# Local Session Coordinator

> Retired on 2026-10-02. This document retains historical implementation and migration detail. Follow [coordinator retirement](coordinator-retirement.md); do not execute the service, startup, registration, lease, validator or integration recipes below. Historical storage exceptions do not authorize outputs outside physical drive-root D/E/F `cargo-targets`.

## Purpose

The local Session coordinator is the shared-`main` control plane for ZirconEngine development. It gives each Session a typed lifecycle, records a hash-based workspace baseline, stores intermediate file contents outside Git, serializes concrete file writes, governs plan/failure records, and owns isolated Cargo validation lanes.

Business Session work remains service-managed between accepted milestones. Every accepted milestone is an explicit service-owned Git commit; arbitrary checkpoints and hidden intermediate commits remain forbidden. Direct `git commit`, generic completion of a numbered-plan Session, and legacy `finalize --milestone` are rejected so a business change cannot bypass its workflow attempt or WeCom result. The service protects unrelated active Sessions and their dirty files without creating branches or worktrees.

The coordinator never installs Git hooks or blocks manual Git commands. On writable startup it removes only legacy coordinator-managed `pre-commit` and `prepare-commit-msg` hooks, restoring a preserved `.zircon-user` hook when present. Manual `git add`, `git commit`, and index operations remain available. Coordinator commits continue to use the scoped `commit-tree` path and internal lease, attribution, manifest, and compare-and-swap checks; those checks govern coordinator automation without taking control of the user's Git workflow.

## Runtime and State

Run the Windows entrypoint from the repository root:

```powershell
.\tools\zircon-session.ps1 start -Json
.\tools\zircon-session.ps1 status -Json
```

The wrapper starts Python in a hidden window only when the health endpoint is unavailable. A repository-scoped named mutex serializes automatic startup, and callers probe the fixed health endpoint while a successor is publishing `runtime.json`; this prevents a descriptor-publication gap during a controlled restart from spawning competing daemon wrappers. The shared coordinator binds the fixed loopback endpoint `127.0.0.1:6518` and writes the port, PID, instance metadata and a fresh per-instance bearer capability to `.codex/state/session-coordinator/runtime.json`. Local CLI, tray and hook clients read that descriptor, authenticate every legacy command and runtime-only control request, and use the bounded `/identity` projection to reject a stale or foreign endpoint. A controlled rollover rotates the capability; a client tracking the already-confirmed rollover reloads the successor descriptor and continues querying the same durable action ID without replaying preview or confirmation. Isolated test coordinators explicitly request an OS-assigned port.

Schema version 16 completes the permissioned controlled-action protocol on top of the read-only workflow facade. It closes `action_kind` at the database boundary and installs compatibility triggers for databases that already applied the early v15 action tables. The runtime descriptor also records the daemon `instance_id`, `started_at`, and supported `control_api_versions`, allowing local clients to reject credentials created by a previous daemon instance. Detailed operator guidance lives in [Workflow Control Center](workflow-control-center.md); module contracts live in [Control Plane](../tools/session_coordinator/control-plane.md) and [Workflow Read Model](../tools/session_coordinator/workflows.md).

Open the local control surface or inspect the same coherent snapshot from the terminal:

```powershell
.\tools\zircon-session.ps1 ui open
.\tools\zircon-session.ps1 control snapshot -Json
```

The browser never receives the runtime bearer. `ui open` uses the authenticated local client to issue a 30-second, single-use Observer ticket and opens `/ui/bootstrap/{ticket}` without printing the ticket. Successful consumption creates an `HttpOnly`, `SameSite=Strict` cookie scoped to `/control`; mutations require the session CSRF value. Local mode enforces the loopback Host/Origin boundary. Explicit LAN/VPN mode admits only its configured HTTPS trusted origin and allowed authorities; do not proxy or expose the default loopback listener. See the [worker protocol guide](coordinator-worker-protocol.md) before starting the explicit remote listener.

All mutable coordinator data remains under `.codex/state/session-coordinator/`:

- `coordinator.sqlite3`: WAL database for Sessions, events, baseline epochs, object indexes, snapshots, attributions, leases and patches;
- `objects/`: zlib-compressed SHA-256 objects;
- `runtime.json`: local connection descriptor with the fixed loopback endpoint and per-instance bearer capability; diagnostics and browser payloads omit the capability;
- `coordinator.lock`: single-instance ownership.

The service validates the active Git branch. A checkout that is not on `main` is diagnostic/read-only: health, Session list and Session show remain available, while mutations fail with `not_on_main`.

### Validation Efficiency

Use a package check to diagnose a broken build baseline before scheduling tests:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -CheckOnly
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter <filter>
python -m tools.session_coordinator.validation_diagnostics --since 2026-09-01
```

`-CheckOnly` runs `cargo check` with the managed target, profile, features and
lockfile policy. It is diagnostic evidence, not a substitute for tests or a product
build. Focused `-LibTests` and `-TestTarget` invocations compile through `cargo test`
without an additional product build. Use `-BuildBeforeTest` when both are required;
artifact publishing, explicit binary builds and general workspace validation keep
their build gate. A failed build stops the dependent test stage and remains a
failed validation. Batch related test filters by package and feature profile.

The Cargo input planner includes dependency production code but only traverses
`cfg(test)` modules for targets compiled as tests. A named integration or binary
test does not require the library dependency's unit-test resources. A workspace
test still checks every selected test target; shared source ownership cannot
suppress required resources. Missing production resources remain errors.

Automatic maintenance retains reusable Cargo pools for seven days after last use
(`CoordinatorConfig.cargo_pool_retention_hours`). Disk-pressure eviction and live
process protection remain active. Explicit cleanup plans retain their requested
age threshold. Failure import keeps one parsed plan snapshot in memory and reuses
it only when every captured Markdown content hash is unchanged; controlled-action
expectations and artifact drift checks still run on cache hits.

The read-only diagnostics command reports ticket latency separately from Cargo
run duration, preparation errors separately from command failures, repeated costly
commands, runner launch/finish failures, and allocated/free SQLite pages. A blocked
runner finish remains a failure even if its process exited zero. The report never
rewrites historical results.
Free pages can be reclaimed through the existing `governance retention-compact`
and maintenance workflow; avoid deleting active build pools to reclaim database
space.

Immutable Cargo tickets now check their selected workspace lockfile, manifests,
package/target selectors, and sealed external archives before entering the queue.
Admission errors preserve their error code and add `details.phase=admission` and
`details.blockers`, each with a `repairCondition`. Missing ownership or changed
source bytes are still rejected even when an identical failure is already known.

The coordinator reuses a deterministic compiler failure only for identical sealed
inputs, baseline, command/features, coverage, actual managed toolchain, inherited
environment digest, and validator implementation. It creates a failed receipt owned
by the submitting Session with `originalFailureTicketId` and `reuseReason`, leaving
the original ticket and diagnostic intact. This receipt starts no Cargo process.
The reusable lookup is limited to the newest 1024 diagnoses within seven days;
the durable original records survive eviction and service restart. Legacy tickets
without a complete identity are not imported into this failure cache.

Network/resource/process failures, linker errors, compiler crashes and test assertion
failures are not reusable diagnoses. To deliberately recheck unchanged failed inputs,
submit a new request ID with `--force-rerun-reason <reason>` (API
`force_rerun_reason`). A forced pass supersedes the original failure; a forced
transient failure preserves it. Concurrent identical pending submissions coalesce,
including forced submissions, and force reasons are retained in submission events.

The fixed-snapshot metadata cache stores at most 16 entries / 16 MiB per daemon.
Concurrent equal topologies share one Cargo metadata execution. Keys cover manifest,
lock/config content, target discovery, command/features, resolved Rust toolchain and
external pin identities. Metadata paths and package IDs rebase into each fresh view;
full source/resource checks continue on every sealed ticket. Restart begins with a
cold metadata cache. `validation.status.metadataCache` reports process-wide counters;
those totals are not per-ticket hit counts. Each ticket receipt and status payload
also exposes the last per-copy observation under `ticket.diagnostic.metadataCache`,
with `metadataCacheAttempts` retaining observations for retries. These observations
remain queryable after terminal cleanup and a daemon restart; a missing observation
means metadata did not start or the copy was not owned by the active worker.

Open Failure dependencies pause affected origin-plan validation in the queue and
are checked again before Cargo starts. Fixing-plan validation and unrelated plans
continue. Explicit workflow identities distinguish independent work inside a plan.
Cargo overlay paths alone cannot prove that a lower dependency is unrelated; when
workflow scope is absent, the origin-plan gate is conservative. A pause preserves
the immutable copy and releases the active-worker slot until the Failure is fixed.
Global baseline `degraded` health does not pause the whole validation queue.

Ticket `timings` expose `admissionMs`, `queuedMs`, `preparationMs`, `cargoMs`,
`cleanupMs` and `totalMs`, with `executionKind` distinguishing execution, pending
work and failure reuse. Unknown historical phase durations remain null. Diagnostic
execution pass rates and phase percentiles exclude reused failures as new executions.

`python -m tools.session_coordinator.benchmark_validation_efficiency` submits one
repeatable managed library check; its `--sample` mode records durable phase evidence,
metadata counters, pool bytes and isolated Cargo-home bytes without starting another
run. Use distinct request IDs with unchanged package/source inputs for first and
repeated samples. It does not clear existing pools or relax admission rules. These
measurements describe that workload; full Runtime test gates remain required.

### Dirty sibling Git worktree capture

An immutable Cargo ticket normally rejects a dirty discovered sibling Git checkout
with `validation_ticket_external_worktree_dirty`; a clean sibling uses its pinned
commit archive. To validate current dirty source bytes, opt in through the ticket's
`coverage` object:

```json
{
  "externalWorktreeCaptures": [
    {
      "repoRoot": "E:\\Git\\zr_vm",
      "commit": "<full-40-hex-discovered-HEAD>",
      "ignoredPaths": ["generated/schema.rs"]
    }
  ]
}
```

`repoRoot` must exactly match the coordinator-discovered canonical sibling root,
and `commit` must match its full 40-hex Git HEAD. Only `repoRoot`, `commit`, and optional
`ignoredPaths` are accepted. The opt-in captures live tracked files, including
staged and unstaged content and deletions, plus nonignored untracked files. It
recurses through initialized submodules. Ignored files stay excluded unless each
is named as an exact, existing, regular, portable relative file in its owning
repository or submodule; directories, globs, Git metadata, traversal, and path
collisions are rejected. At most 64 ignored paths of 4096 UTF-8 bytes each may
be requested.

Tracked `Cargo.toml`, `Cargo.lock`, Rust toolchain files, and `.cargo/config*`
must still match the discovered commit. Added nonignored versions are rejected;
ignored manifests and configuration at Cargo lookup locations also fail closed.
Ignored lockfiles without a tracked counterpart stay outside the capture.
Capture is bounded to 16 sibling repositories, 100,000 entries and 512 MiB of
live tree data per repository, a 256 MiB archive per repository, and 512 MiB of
archives in total. The opt-in uses a submission-wide deadline of at most five
minutes. The coordinator inventories before and after archiving, hashes file
content, and rejects source or Git-state drift.

The coordinator retains the canonical request in `coverage.externalWorktreeCaptures`
and generates `coverage.externalSources` with the sealed archive hash and mount
descriptor. Coordinator-owned `coverage.externalWorktreeProvenance` records the
manifest hash, file and byte counts, submodule commits, and each explicitly
included ignored file's path, hash, byte count, and mode. Callers cannot supply
that provenance or an archive hash. The queued worker materializes the sealed
ticket archive rather than rereading the live sibling. Archive extraction applies
only 0644/0755 file modes. On Windows, tracked files use Git's index executable
bit; on POSIX, tracked files use the live executable bit. Nonignored untracked and
explicitly included ignored files use the live bit on either platform. Windows ACL
and executable behavior require separate platform evidence.

This opt-in retains managed ticket ownership, source-manifest checks, Cargo
preflight, target-root and lease rules. Eleven focused Windows-native Python
regressions passed. Controlled rollover action `30864af5e5de4bf296d6f9e34ec09550`
and intent `1d4fccdc8fe940459645d80094cb19a5` succeeded with successor daemon
`368317a5b35a45fe925cfbc84b2fe564`; the 39 queued tickets survived that
handoff. Two subsequent ABI-A3 Host ticket requests were rejected during archive
admission with `validation_ticket_external_worktree_changed` as live, untracked
`zr_vm` coverage TSVs changed. No opt-in ticket was accepted. A quiet external
worktree interval, real managed Windows Cargo validation, and product acceptance
remain pending; the source tests and daemon rollover do not close those gates.

### Local offline intent queue

The coordinator never introduces a global drain barrier: `service.drain` is an audit-only blocker observation, and production rejects global stop, restart, and force-stop before they can close admission. When the local runtime descriptor is absent or the fixed loopback endpoint explicitly refuses the connection, the CLI may atomically persist only these state-convergent requests under `.codex/state/session-coordinator/offline-command-queue/`: `session.register`, `session.heartbeat`, and `lease.heartbeat`. An offline registration must already carry `--session-id` or `CODEX_THREAD_ID`; the CLI never serializes a fresh random manual identity for later replay. A preflight timeout is typed `command_preflight_timeout`; timeout or uncertain transport loss after POST is typed `command_post_timeout` or `command_post_transport_unknown` with its request ID and is never queued. The client queries that durable request instead of guessing whether the daemon applied it. Every JSON envelope is repository-key-bound, size-limited, exact-schema validated, written through a flushed temporary file, and placed in FIFO order. The same allowlist is enforced while reading queue files, so a locally planted queue file cannot elevate into a Cargo, lifecycle, finalization, or controlled-action request.

Cargo operations, reservations, process starts, lifecycle requests, controlled actions, commits, cleanup, retention, and finalization are never queued. A post-dispatch failure retains its request-bound query path because replaying it could create work, alter a safety boundary, or duplicate an irreversible side effect. A queued command is not local execution: it has no effect until a healthy daemon acknowledges it.

`tools/zircon-session.ps1 start` ends with the normal `status` request. A healthy `status` automatically replays pending local intents in FIFO order through a non-waiting local single-consumer lock, deletes only acknowledged items, stops at a new transport loss without reordering, and moves a terminal server rejection to the visible `failed/` queue directory while retaining its later suffix. Operators can inspect the queue or directly replay pending items:

```powershell
python -m tools.session_coordinator --repo-root E:\Git\ZirconEngine --json offline-queue status
python -m tools.session_coordinator --repo-root E:\Git\ZirconEngine --json offline-queue replay
```

## Session Lifecycle

Register the current Codex thread and activate it:

```powershell
.\tools\zircon-session.ps1 session register `
  --display-name "runtime plan 02" `
  --plan-path "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md" `
  --write-scope "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md" `
  --write-scope "docs/plans/zircon_runtime/frameworks/02" `
  --write-scope "tools/session_coordinator"
.\tools\zircon-session.ps1 session set-status active
```

`CODEX_THREAD_ID` is used when `--session-id` is omitted. Manual shells receive a generated UUID if neither is available.

The parent plan file and its numbered child-plan directory are separate write scopes. Register both before editing a plan table, a milestone output record, or a failure link; then claim an exact live lease and attribute the current hash. Attribution without the live lease is rejected. `session register --write-scope` replaces the stored scope rather than appending it, so correction commands must repeat all existing business paths plus the newly required plan paths. The milestone service rejects an immutable manifest that has lost current attribution rather than absorbing it into another Session's commit.

The only persisted status values are `registered`, `active`, `waiting_lease`, `resolving_failure`, `waiting_validation`, `finalizing`, `completed`, `stale`, `archived`, and `cancelled`. The transition table lives in `models.py`; invalid transitions fail without changing the database. Explanatory text belongs in `status_reason` rather than inventing another status string.

## Baseline Epochs

Initialize and inspect the workspace baseline:

```powershell
.\tools\zircon-session.ps1 baseline init
.\tools\zircon-session.ps1 baseline diff
.\tools\zircon-session.ps1 baseline scan
```

An epoch records HEAD, the Git index tree, and SHA-256 hashes for tracked and non-ignored files. Coordinator state is excluded. `baseline scan` compares current content to the epoch. A change does not get reverted; the baseline becomes `degraded` and the path remains on disk.

Claim before attributing a known change, then reconcile the existing epoch without absorbing any dirty file:

```powershell
.\tools\zircon-session.ps1 lease claim README.md
.\tools\zircon-session.ps1 baseline attribute README.md
.\tools\zircon-session.ps1 baseline reconcile
```

`baseline reconcile` recalculates every difference, requires exact current-hash attribution, clears only the degraded marker, and keeps the epoch manifest unchanged. It fails with the remaining paths if even one change is unattributed. `baseline accept --reason ...` is a separate operator override that captures a new full-worktree epoch; do not use it to clear degradation in a shared dirty workspace. Neither action creates a Git commit.

The thirty-second observer does not repeatedly hash a large workspace while that same epoch is already `degraded` and HEAD is unchanged: it preserves the degraded state until an explicit reconcile or acceptance. A HEAD change still receives a fresh observation, but derives its pinned committed manifest through one streaming `git archive` instead of spawning `git cat-file` once per tracked file; the archive output retains Git's checked-out content filters. Background observation, manual read-only `watch scan`, explicitly requested `baseline scan`, Cargo orphan reconciliation, validation-copy cleanup, and periodic retention work do not hold the foreground mutation mutex; their own SQLite transactions and epoch checks retain correctness. This avoids workspace scanning or background work starving Session registration, `cargo finish`, lease, and heartbeat writes without weakening the baseline gate.

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

An operator can cancel an unapplied patch with `task cancel patch:<patch_id> --reason "obsolete handoff"`, or with the controlled `task.cancel` action using `taskId` and `reason`. Cancellation accepts `queued` and `needs_rebase` patches, including patches whose owning Session is archived or cancelled. Repeated cancellation is idempotent; `applying` and `applied` patches are rejected.

Cancelled patches leave the active task list and remain available in cancelled/history queries. The service preserves source bytes, patch/base/current objects and earlier error evidence, and records the actor, reason and action request ID in a `patch.cancelled` event. Cancellation does not release unrelated live leases.


## Failure and Recovery Semantics

- Missing or stale runtime descriptors produce a structured `offline` result and exit code `3`, except for the explicit safe local intent allowlist, which returns `queued` after durable local persistence.
- Invalid requests and state transitions produce exit code `2`.
- SQLite transactions roll back as a unit on error.
- Object writes use an atomic temporary-file replacement and verify SHA-256 on read.
- Patch application failures retain snapshots and return `failed`; leases are released in `finally`.
- External workspace edits are preserved and mark the baseline degraded.
- `stop` asks the local loopback service to shut down, then removes only runtime/lock files owned by its PID.

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

Graph diagnostics cover schema errors, duplicate lifecycles, self-edges, cycles and excessive dependency depth. The filename prefix supplies the coordinator's canonical `open`/`fixed` state; a conflicting frontmatter status remains a validator diagnostic but cannot abort the graph transaction or unrelated Cargo/Session commands. Only `failure-*` records participate in the live dependency graph: moved `fixed-*` artifacts stay indexed for audit but cannot manufacture a current cycle or depth block. Open failures sort before fixed records and then by creation date/slug. Registering a Session with a fixing plan imports current Markdown; applicable failures are returned in `open_failures` and the Session enters `resolving_failure` instead of an untyped blocked state.

After architectural repair and upward validation, `failure return` requires the lifecycle key, accepted-fix date, root cause, architecture repair, validation and return summary. The service rewrites the artifact as `fixed-*`, moves it into the origin child directory, replaces ordinary handoff bullets with concise `fixed 已修复` relative summaries, reruns the validator, and rolls back all file changes if any write or import fails. For a Markdown table row it rewrites only each matching link token, preserving all cells, non-link evidence, separators, and unrelated links; a plan table is never collapsed into a bullet.

## M3 Validation

M3 tests use generated temporary plan trees. They never move or rewrite live business handoff artifacts. The real-repository M3 audit is read-only and reports concurrent invalid/in-progress artifacts as diagnostics rather than treating them as coordinator-owned fixes.

The coordination context script now queries service health, indexed Session count and Failure graph first. Its offline fallback recursively scans both `docs/plans` and `.codex/plans`, correcting the previous formal-root omission.

## Managed Cargo Jobs

Schema v4-v7 records Cargo jobs, cleanup reservations and persisted cleanup plans; schema v22 adds reusable-cache identity and cleanup state and repairs databases whose historical v21 marker predated those columns. Schema v30 records every process-tree observation, schema v31 records the Cargo root PID's creation identity, schema v32 distinguishes a Cargo root from a wrapper that supervises sequential Cargo commands, schema v41 persists each new CPU reservation's canonical compatibility payload, schema v42 extends the same durable contract to the single GPU lane with an immutable approved target directory, and schema v43 adds one durable FIFO successor behind an already bound lane reservation. Jobs use `check`, `test`, `workspace`, and `gpu` lanes with `leased`, `running`, `succeeded`, `failed`, `released`, and `orphaned` states. Targets must remain below one of the three drive-root `cargo-targets` trees on `D:`, `E:`, or `F:`. A case- and separator-normalized identity plus ancestor/descendant overlap checks prevent Windows path aliases or nested pools from becoming simultaneous writers. Repo-local targets, symlink/junction escapes and arbitrary paths fail with `cargo_target_not_managed`.

```powershell
.\tools\zircon-session.ps1 cargo acquire workspace --ephemeral
.\tools\zircon-session.ps1 cargo acquire test --compatibility-json '{"platform":"windows","toolchain":"1.88.0@x86_64-pc-windows-msvc","target_architecture":"x86_64-pc-windows-msvc","workspace":"Cargo.toml","build_config":"profile=test;features=default"}'
.\tools\zircon-session.ps1 cargo acquire check --ephemeral
.\tools\zircon-session.ps1 cargo list
.\tools\zircon-session.ps1 cargo list --history --limit 100
```

The default `cargo list` query returns at most the 100 newest active leased or
running jobs. Use `--history` only for bounded diagnosis; `--limit` is capped at
500. The response includes `activeOnly`, `limit`, and `truncated`, so health
pollers can page or narrow their query instead of loading the complete job
history.

CPU FIFO reservations persist one exact compatibility payload and command fingerprint. A
Session may create, renew, or acquire new managed Cargo work only while it is in a
non-terminal executable state; `completed`, `stale`, `archived`, and `cancelled` owners
fail with `cargo_session_not_executable`. Only an unconsumed `pending` reservation with
`job_id=NULL` is governed by its absolute TTL. Both explicit
`SessionService.set_status(..., STALE)` and maintenance-driven `mark_stale` terminalize
that unconsumed claim in the same database transaction as the Session transition, while
a reservation already bound to a leased or running job follows the nominated job
lifecycle and is never expired by the pending TTL; abnormal orphan reconciliation
expires the bound reservation in the same transaction so a dead job cannot retain FIFO.
Each CPU or GPU lane may retain one bound `leased`/`running` FIFO head and one later
`pending` successor. The successor owns only its canonical command and compatibility
payload: it creates neither a target directory nor a process, cannot be consumed until
the head reaches `released`, and prevents generic acquire from entering between the
terminal head and its owner-authorized consume. A second pending successor is rejected.
When a nominated job reaches `released` after its process tree is empty, the same
release transaction moves its bound CPU reservation to `released`; a later owner
handoff is not required. Reserve/acquire also reconcile a legacy `finished` head only
when its nominated job is already `released`, its recorded process tree is empty, and
its owner is non-executable. Live or executable owners remain FIFO heads and are never
reclaimed by that historical repair path.
While a coordinator is under a persistent maintenance hold, a configured maintenance
Session may use the narrow `consume-cpu-reservation` command to bind its already-pending
FIFO reservation without opening generic Cargo admission:

```powershell
.\tools\zircon-session.ps1 cargo consume-cpu-reservation <reservation-id> `
  --session-id <configured-maintenance-session> --lane-kind test
```

The command accepts exactly those three values. It reads the target pool, canonical
compatibility document, and command fingerprint from the durable reservation; client
target, compatibility, and command overrides are rejected. The one SQLite transaction
rechecks the executable owner, pending expiry, FIFO head, and exact canonical payload,
then creates one `leased` job with no PID. Retrying the same request returns that same
unstarted job. Generic `cargo acquire`, new reservations, `cargo start`, and every
unconfigured Session remain denied while the hold is active.

The same typed contract applies to the global GPU lane. A scoped maintenance Session
first stores an exact command, canonical compatibility and approved target; consuming it
creates the sole unstarted GPU job without a generic acquire. The target is durable
reservation data, not a consume/run argument, so a held RenderDoc or DX12 job cannot
fall back to a different pool:

```powershell
.\tools\zircon-session.ps1 cargo reserve-gpu `
  --session-id <configured-maintenance-session> `
  --target-dir E:\cargo-targets\zircon-engine\render18-af-m3-plugin `
  --compatibility-json '<canonical-compatible-json>' -- <exact-supervised-command>
.\tools\zircon-session.ps1 cargo consume-gpu-reservation <reservation-id> `
  --session-id <configured-maintenance-session>
```

Only the configured maintenance Session may create or consume this GPU reservation while
the hold is active. Repeating identical owner, target, compatibility and command inputs
is idempotent; a foreign Session, a second pending GPU reservation, a client target override, or
generic `cargo acquire gpu` is rejected. `cargo run-reserved` below starts the leased GPU
job only after rechecking the same durable command fingerprint.

After the coordinator has restored that Session to `active` through its controlled
action path, its exact reservation-bound command uses `cargo run-reserved`, not generic
`cargo run`. This path accepts the reservation ID, job ID, Session ID, and command only;
it atomically records a durable `start_pending` acknowledgement and its dedicated
launch deadline for that exact reservation/job/command binding before returning. The
daemon then rechecks the source manifest, derives allowed `RUSTFLAGS`/
`CARGO_INCREMENTAL` values from the canonical reservation payload, and spawns the
supervised process asynchronously. A valid pending launch is protected from the generic
300-second leased-job watchdog. A stale Session, a different command, or a client
environment/target override is rejected without starting Cargo; a pre-spawn or launch
failure becomes an explicit `launch_failed` disposition without fabricating a Cargo run
or exit result. Repeating the same command request ID returns the same acknowledgement
and cannot launch a second process. If a process was registered but cleanup cannot prove
it stopped, the request is terminal `launch_failed` while the job/reservation remain
owned and non-reusable until process-tree reconciliation proves death. On successor
startup, a predecessor `start_pending` with an already registered PID/run is restored as
`started`; one with no registered process is immediately terminalized as
`cargo_launch_interrupted_before_spawn`. It is never silently left until the 900-second
deadline and never rescheduled across daemons.

```powershell
.\tools\zircon-session.ps1 cargo run-reserved --session-id <session-id> `
  <reservation-id> <job-id> -- cargo test -p zircon_runtime <exact-filter> --locked
```

When a held daemon restarts, startup restores the scope from the latest successful
`service.drain` action rather than the most recent supervision-event row. A same-state
drain may be intentionally coalesced without emitting another event, so using event
order could silently drop a newer union scope. Each replacement drain must carry the
whole required Session union; the daemon additionally unions the local bootstrap scope
when present.

### Bounded drains and persistent maintenance

`service.drain` is an auditable blocker observation, not an admission barrier: it
records the active jobs and returns while the coordinator remains `healthy`; new tasks
continue to be admitted. Task health, timeout, orphan reconciliation and cleanup are
job-level concerns, so one slow task cannot freeze unrelated Sessions. Startup still
closes any legacy active drain at its durable deadline, preventing historical records
from recreating an indefinite `draining` state.

The watcher checks every live managed job independently. After five minutes without a
job heartbeat it emits one `cargo.health_timeout` audit event with the observed process
tree. A live process is never silently killed or reused; its own lane remains protected,
but all unrelated lanes and Sessions remain admissible. Once the owner heartbeats again,
the next stale period is reported independently.

Production disables `service.stop`, `service.restart`, and `service.force_stop`: each is
rejected before an intent or supervision-state transition is created. This installation
therefore has no global maintenance hold or `draining` state to recover from; maintenance
must be performed through task-scoped operations while normal task admission remains open.
The older explicit-release rule remains only for historical records so an expired executor
Session can never make a legacy hold unreleasable.

Recreating the service does not
rewrite `expires_at`, and the next reserve/acquire transaction removes an expired or
non-executable pending head before applying FIFO, so a stale zero-job owner cannot starve
unrelated validation.

Reusable acquisition requires a complete compatibility document containing platform (`windows` or `wsl`), Rust toolchain, target architecture, repository-relative workspace and canonical build configuration. The service adds normalized repository identity and hashes that document. Source and `Cargo.lock` changes deliberately do not split the pool because Cargo performs unit-level invalidation. Check/test lane labels also do not split it. Exactly one primary directory exists per compatibility key across Sessions and exactly one task may own it; concurrent compatible acquisition returns `cargo_reuse_pool_busy` instead of creating a fallback pool. Legacy duplicate retained directories are demoted to prompt deletion while the newest remains authoritative. Missing compatibility metadata fails closed to ephemeral by default, as does an explicit `--ephemeral` request; release commits ownership state and wakes a single worker that drains pending requests, reserves and revalidates each exact directory, then deletes outside the writer transaction. A locked deletion becomes `failed`; release-driven cleanup leaves it alone, and the daemon's default 30-second watch loop retries failed Cargo cleanup.

Storage inventory uses an exact ancestor-path index for known and protected directories
and accounting roots.
It preserves the existing lexical `Path` and Windows case semantics through normalized
anchors and complete separator-delimited components, without allocating ancestor `Path`
objects during queries. The index uses Python 3.13+ [`PurePath.parser`](https://docs.python.org/3/library/pathlib.html#pathlib.PurePath.parser);
the Windows repair was validated with Python 3.14.4. A
completed inventory remains required for capacity admission; a live health endpoint or
a running scan does not establish available storage.

Cleanup reservations protect the entire storage root when their scope is unknown.
Cargo's declared write scope includes the physical target and the full canonical and
target-derived `zircon-engine` cache, pool and scratch namespaces. Physical-path
admission rejects reparse points and aliases before comparing scopes. The union of
these paths is bound to the exact live reservation; capacity revalidation and final
allocation recheck overlap across the entire scope in the same writer transaction.
Checking only the final target is insufficient. Legacy declarations remain protected
and grant no deletion or adoption permission. Inventory freshness, capacity thresholds
and FIFO admission remain unchanged.

The cleanup-scope repair is loaded in the restored schema-80 service after a
confirmed rollover and read/write identity check. Actual Cargo acceptance remains
pending while the current capacity snapshot is stale; deployment alone does not
close that gate. The [isolated Jenkins pilot plan](../plans/jenkins-coordinator-pilot.md)
owns the concrete restoration and validation evidence.

The start invocation's retained native handle confirms exit 0 while its caller is
still collecting two output streams. This output-collection tail does not imply a
failed start: the authenticated successor remains healthy. Preserve the caller
until its external Job ownership is identified; native exit proof alone does not
mean stdout has been collected. Current Cargo failure reflects stale capacity at
acquisition, not a demonstrated cleanup-scope failure.

Executor source changes require a fresh immutable driver manifest before pilot
submission, even when they belong to another Session. Resealing preserves the
foreign source and prior receipt identities; it does not require restarting the
formal service. The pilot submission preflight repeats executor and agent checks
before its final authenticated capacity-freshness check. An incomplete or stale
snapshot remains inadmissible; the formal 420-second threshold is unchanged.

Future Jenkins dispatch replacement must gate new claims per authorized task
family while existing validations, Cargo runs and fixture protection continue.
No family dispatch router or unified legacy/Jenkins history adapter is currently
implemented. The pilot's [conditional cutover design](jenkins-coordinator-pilot.md)
specifies its proposed epochs, drain proofs and read-only history boundaries;
it grants no authority to stop this service or retire its modules.

The pilot now has an actual managed Cargo check with released/exit-0 formal job
evidence, but its nested process grace timed out before a pilot receipt or Jenkins
archive was written. Formal job success alone does not prove the whole execution
tree is terminal or the pilot accepted. Live cache observations cover one job's
writer and retained physical identity; they do not establish cross-job reuse.

`cargo.compiler_cache_prepare` prepares a shared compiler-cache daemon only for
an already registered running supervisor identified by Session, job, PID and
native birth. `compiler_cache_preparation.prepare_compiler_cache` verifies active
target ownership, generation, filesystem identity and the exact live storage
claim before and after preparation. `CargoJobRunner.prepare_compiler_cache_server`
derives canonical cache/temp paths and port with the existing 12G policy; it runs
in the formal service outside the short-lived validation Job. This command grants
no Cargo acquisition, Home or scratch authority. `cargo.start` keeps its existing
nonblocking supervisor registration.

The typed schema-1 binding has exactly 20 fields: schemaVersion, sessionId, jobId,
supervisorPid, supervisorCreationTime, targetDir, targetKey, targetGeneration,
targetFilesystemIdentity, daemonPid, daemonCreationTime, serverPort, cacheDirectory,
temporaryDirectory, executable, bindingMarkerPath, serviceInstanceId, servicePid,
serviceCreationTime and status. The validator order is Start, Workspace, formal
prepare, then Push. Push calls `verify_prepared_compiler_cache_binding` using a
read-only current-claim check, exact native birth/generation/filesystem/runtime
and marker verification; it neither starts the shared daemon nor constructs an
application or requests a WAL writer. Amount, FIFO, the 420-second freshness
threshold and generic eight-second native grace remain unchanged. Source tests
and an empty-argument rejection establish guard behavior, not actual daemon
preparation success; current proof and pending gates belong to the pilot plan.

Filesystem scanning uses a fresh nofollow stat when classifying symlinks and
computes category totals only when that category has no cached total. The small
repair preserves cancellation, protection and admission contracts; its deployed
regressions do not guarantee a scan finishes within 420 seconds. A completed scan
may already be stale when read, so submission still requires the current final
freshness check. A dry baseline launcher proves neither Cargo execution nor
performance; current deployment evidence remains in the single pilot gate record.

Longer fixture/native observer lifetimes extend observation only. They do not
extend formal snapshot freshness, the trial submission freshness check or the
generic native process grace; a partial scan remains inadmissible. A footprint
budget-exhaustion diagnostic alone does not establish the cause of aggregation
latency.

Read-only publication and stack samples establish observation boundaries, not
the cause of aggregation latency or a successful capacity gate. Preserve the
original snapshot timestamp and reject partial inventory; observer completion
does not permit relaxing either formal or trial freshness. A short successful
metric decode also does not exclude contention in the formal connection.

Protection matching during the filesystem walk uses the same exact ancestor semantics.
Preparing the resolved protection set checks cancellation and the time budget; an
interrupted preparation returns an empty, incomplete footprint rather than scanning with
partial protection. Directory-loop cancellation identifies the directory still on the
scan stack, including cancellation before the first directory is visited.

Terminal validation tickets release their source pins. Their historical manifest hashes
remain queryable, but do not require retired source objects to exist forever. Active
tickets still require indexed source objects; independently retained snapshots and
non-candidate objects from the existing Retention reference collector remain protected,
including active durable-task source and external objects. Unknown live references still
make the inventory incomplete.

The background `storage-gc` worker survives SQLite `TransactionAdmissionBusy` at its
worker boundary, logs the contention, and waits 250 ms with an interruptible stop event
before another iteration. A request that could not be claimed stays queued. If a claimed
request cannot persist its terminal result, its durable record stays `running`; the worker
does not replay it or report success. That ambiguous request still requires reconciliation
before its result can be accepted.

The web control center separates the real-time Cargo baseline from the historical audit feed. Its four lifecycle counters and Cargo table use only the latest coordinator record for each target directory that still exists on disk. Consequently, a target deleted after an earlier lock failure is not shown as a current failure, and repeated jobs sharing one reusable directory count once. The history payload remains available to the service for audit, but it does not influence the live cards: `可复用池`, `用后即删`, `待清理`, and `清理失败` describe current directories only.

The default browser snapshot is deliberately current-first: it includes every non-terminal
Session, workflow, Cargo job, validation copy, finalization request, and open Failure, plus
only the 50 most recent terminal records for each of those domains. The page therefore does
not deserialize years of archived sessions or stale build attempts before displaying current
work. The last 200 sanitized audit events remain available for immediate diagnosis; complete
history remains in the coordinator SQLite ledger and the dedicated log/audit interfaces, not
in the startup payload.

### Quiet sync and the operator work board

Codex rollout discovery separates source metadata refresh from a visible lifecycle change.
A periodic scan may update a file revision, size, or observation timestamp without emitting a
`codex.session.updated` event or adding a `codex.sync.completed` audit record. A new rollout,
lifecycle/identity change, diagnostic, unavailable source, and every operator-triggered sync
remain visible events. This keeps the timeline focused on work that changed rather than the
thirty-second observer's bookkeeping while retaining the complete current source revision in
the database.

The browser's Overview page adds a bounded `experience` projection for the last 24 hours:
`静默同步` shows quiet runs over total sync runs, and `资源阻塞` lists at most 20 current
Cargo reservations or jobs with only their owning Session, lane kind, state, and creation time.
It does not expose command lines, historical retry noise, or a global admission state. A
blocker is scoped to its resource owner: it tells the next developer which validation lane is
occupied, never that unrelated Session registration or file work must wait. During a rolling
daemon/UI upgrade, a missing projection renders as `0/0` and no blockers instead of breaking
the control surface.

The Overview page also renders a compact `validation.artifactLifecycle` maintenance-debt
summary: reusable pools, ephemeral targets, pending cleanup, and cleanup failures. A pending
or failed cleanup is actionable only from Validation details and is never an admission gate:
the panel explicitly preserves open Session admission and never requests a global drain. Its
counts retain the current-directory semantics above, so historical jobs and already-deleted
targets do not inflate the operator's cleanup work.

The Windows tray follows the same always-admitting policy. It exposes operationally valid
actions only: open the local console, refresh the tray state, diagnostics, startup-item
management, and exit. It intentionally omits global drain/stop/restart/force-stop commands,
because those operations are disabled by the coordinator and must not appear as clickable
controls that later fail.

`validate-matrix.ps1` performs the Windows lifecycle automatically: register the caller, derive the compatibility document, acquire the primary pool with the wrapper PID, immediately enter `try/finally`, record the process command line and root creation identity at start, run validation, record the exit code, and owner-checked release. It marks that PowerShell PID as a **supervisor**, so finish/release ignore the still-live wrapper itself after its sequential Cargo calls have returned but still reject any live Cargo/rustc descendant. Direct `cargo start` jobs remain Cargo-root jobs and retain their live root check. Every observation compares the current root creation identity before traversing descendants: a different identity means Windows has reused the PID, so that unrelated process and its descendants cannot retain the old Cargo target. A matching root—or a known descendant after the matching root exits—continues to protect the target. Pre-identity `orphaned` rows retain their historical terminal state rather than treating a later reused PID as Cargo. WSL Cargo is permitted only through a coordinator-aware Windows host wrapper that acquires with `platform=wsl`, remains alive and heartbeats while its `wsl.exe` child runs, and translates only the granted path to its mounted equivalent; direct unleased WSL Cargo is forbidden. Explicit `-TargetDir` and inherited `CARGO_TARGET_DIR` are normalized through the same policy and cannot create an alternate primary directory. Dry-run jobs are audited but their directories are not created. The daemon converts dead running jobs and dead/timed-out pre-start leases to `orphaned` and immediately retries pending ephemeral cleanup.

Managed `cargo run` authorizes the current Session, exact reservation command, target owner,
and CPU FIFO in one durable transaction before it creates a child process. The transaction
records a short-lived `running` launch intent with no PID. On Windows, the runner then creates
the child suspended inside a non-inheritable kill-on-close Job Object, atomically binds its PID,
creation identity, and run record to that exact intent, starts both log readers, and resumes the
child only after each reader reports that its output file is open. A spawn or pre-resume setup
failure closes the Job and rolls the intent and reservation back to `leased`. If the coordinator
stops between authorization and spawn, startup reconciliation expires the PID-less intent
without replaying the command; an interrupted suspended run projects as `launch_failed` rather
than pretending it executed. Runtime log writes continue draining after a local file failure.
A runtime pipe read failure is only signalled by the reader; the collector remains the sole Job
handle owner and terminates the complete tree before waiting for the root or publishing terminal
evidence. If a child exists but cleanup cannot prove termination, the coordinator atomically
records the job, reservation, run, PID, creation identity, and original rejection code so restart
continues to block overlapping target use. An active local collector also holds terminal run
projection until the complete Job tree, job finish, and reservation release have completed;
restart-only reconciliation handles runs with no surviving collector.

### Shared Development Build Policy

`tools/dev/dev-fast-build.ps1`, `validate-matrix.ps1`, and automatic validation use the
same compiler policy and retained compatibility pools. A pool has a fixed source
workspace and private dependency extraction tree; its exclusive lease covers
content synchronization, compilation, and final integrity verification. Unchanged
files keep their timestamps. Task IDs, test filters, snapshot paths, source changes,
and feature selection do not multiply pools; Cargo fingerprints invalidate affected
units. Toolchain, effective compiler/profile configuration, accepted target, and
linker identity do. Ambient compiler overrides are scrubbed consistently before
both the PowerShell and coordinator launch.

Both PowerShell entry points accept `-LinkMode auto|static|dev-dynamic` and
`-Linker auto|lld|system`. On Windows, `auto` selects development DLLs for ordinary
development and validation. Release, profiling, performance features, and Runtime
product DLLs keep static product linking. Fast MSVC mode uses the active Rust
toolchain's `rust-lld` with the stable `lld-link` flavor; `system` is available for
linker diagnosis. Automatic linker selection keeps the system linker for release,
profiling features, and benchmark commands, including benchmarks using a dev/test
profile. An explicit `-Linker lld` still selects LLD for diagnosis.

Add `-CargoTimings` to either PowerShell entry point to write Cargo's per-crate
HTML timing report under the managed pool's `cargo-timings` directory. It does not
change compiler compatibility or invalidate a successful snapshot check. Test
execution preserves the preceding compile report instead of replacing it with
an empty fresh-build report.

`reuse` disables dev/test debug information and global incremental compilation,
then enables package incremental compilation for Runtime, Editor, and App only.
`compact` disables every package's incremental compilation; `diagnostic` supplies
full debug information. Development DLL mode uses optimization level 1 for its
two carrier packages and third-party optimization level 3 in dev/test. App/Editor
and workspace builds also set Runtime's package optimization level to 3 because
its rlib symbols enter the Runtime DLL and otherwise exceed Windows' export limit.
Runtime-only tests keep the normal Runtime optimization level and use just the
dependency DLL. App/Editor remain at their normal optimization level; all three
retain package incrementality. Debug assertions remain enabled. Rust development DLLs require
matching toolchain and dependency configurations; staged development artifacts
include their DLL dependencies and Rust standard library.

Development DLL staging writes each DLL and its receipt to a private temporary
file, flushes and verifies its bytes, then creates the final name without
overwriting existing content. The output filesystem must support hard links.
On Windows, the temporary name is deleted when its handle closes, including
process termination; a retry verifies and reuses any completely published files.
This uses Python's [temporary-file delete-on-close behavior](https://docs.python.org/3/library/tempfile.html#tempfile.NamedTemporaryFile).
Conflicting existing files or receipts remain unchanged and fail staging.

Artifact publication uses the target selected by the managed policy. Implicit
host builds read `<pool>/<profile>`; an explicit Cargo target reads
`<pool>/<target>/<profile>`, including when that target equals the host. Custom
JSON target specifications use the file stem for the artifact directory. The
same target is passed to `rustc --print target-libdir` when staging development
DLLs. Explicit-target compilation has a separate compatibility key because
Cargo applies Rust flags differently to host build scripts and procedural macros.

Runtime's default library is `rlib`. Request the existing product C ABI explicitly:

```powershell
.\tools\dev\dev-fast-build.ps1 -Profile client3d -Package zircon_runtime -Action build -RuntimeProductDll
.\tools\dev\dev-fast-build.ps1 -Profile server -Package zircon_runtime -Action test -LibTests -TestFilter task_timer_rejects_zero_interval
```

The product option runs `cargo rustc -p zircon_runtime --lib --crate-type cdylib`
with the selected business features, preserves `zircon_runtime.dll` and artifact
receipts, requires the Runtime feature closure to include `dynamic-api`,
and forbids development-link features. The managed entries validate the requested
closure; `cargo-zircon` also checks the actual Cargo compiler artifact's features
before requesting a product DLL. Existing validator publication
requests for `zircon_runtime.dll` map to this option. Focused tests perform the
corresponding test-profile check before code generation; later filters reuse a
successful check and Cargo's current test executable. Metrics separate queue,
source synchronization, checking, compile/link, and test execution, with cache
counter deltas and disk observations. Source metrics also separate sealing,
dependency integrity verification, and retirement of untrusted compiled artifacts.
The final `[managed-validation-metrics]` log entry embeds the complete receipt,
so phase timings, source identity, cache counters and disk observations survive
ephemeral target deletion. Benchmark sampling validates its job, exit status and
source digest before accepting it; older log entries can still read `receiptPath`.
A failed stage is not successful performance
evidence; explicit workspace validation remains available.

For repeatable measurements, both PowerShell entries accept `-SourceSnapshot`
and `-SourceSnapshotDigest` together. The source directory must use the sealed
`inputs/source` layout; the SHA-256 digest covers its complete sibling input
inventory. The coordinator still allocates the compiler pool and holds its lease.
`python -m tools.session_coordinator.benchmark_build_reuse capture --pool <pool>
--output <approved-root>/zircon-engine/cache/build-benchmarks/<name>` captures an
existing verified input set. Its `sample` command uses the managed developer entry;
`--test-target <name>` selects an integration test instead of library unit tests.
`summarize` requires exactly three successful samples with distinct validation job
IDs, identical command selectors, compiler-pool compatibility, and source hashes.
An unmatched test filter, failed stage, or changed snapshot rejects the sample.
The optional baseline variant disables package incrementality and is paired with
static/system linking; report that baseline configuration with timing comparisons.

Dependency source repair invalidates compiled artifacts and successful-check
receipts. A persistent marker keeps the pool unavailable when an occupied DLL or
interrupted cleanup prevents retirement. Runtime-only development staging requires
the dependency carrier and standard library; App/Editor staging also requires the
Runtime carrier. Non-test Cargo profiles use explicit test-target checking so
test-only code is checked before compilation.

Set `ZR_TEST_DEVELOPMENT_PROFILE` to an existing managed pool's `debug` directory
and run `python -m unittest tools.tests.test_development_artifacts` to validate
real Windows DLL staging. The opt-in test loads from a private temporary directory,
checks missing required libraries in isolated processes, and verifies reuse while
the Runtime DLL is loaded. It does not alter the pool's artifacts.

## Cleanup and Service-Owned Maintenance

Cleanup is deliberately two-phase:

```powershell
.\tools\maintenance\cleanup-stale-targets.ps1
.\tools\maintenance\cleanup-stale-targets.ps1 -Apply -WhatIf
.\tools\maintenance\cleanup-stale-targets.ps1 -Apply
```

The repository's managed pools, incremental data, dependency trees, and shared
caches across all approved roots share a 60 GiB idle budget. Incremental data has a
16 GiB sub-budget within that total. Pressure recovery targets 48 GiB, preferring
terminal scratch, old incremental configurations, then least-recently-used pools;
live processes and leases retain their files. Build admission still requires 35 GiB
free on the target drive. Runtime disk peaks are reported separately from idle use.

`-HistoricalRepositoryTarget` explicitly includes the old
`E:\Git\ZirconEngine\target` migration. Preview with `-WhatIf`, then use `-Apply`:
the entry checks the exact approved path, Cargo artifact ownership, links, and live
compiler/artifact processes again before deleting. New builds cannot write there.
Client timeouts on accepted cleanup/audit requests are recovered by request ID;
the original operation is not submitted again. The validator and cleanup wrapper
wait for that durable request to finish by default, including temporary capacity
rejections from the status endpoint. Requests rejected before admission retry at
most three times; a stale Cargo acquisition retries only when the transaction
fence rejected it before creating a job. Explicit recovery diagnostics
may set a positive timeout; zero means no observation deadline and does not change
the coordinator's execution or cancellation policy.

Cleanup matches a historical Cargo PID with its recorded process creation time.
A proven reused PID does not protect an old pool; matching or unreadable identities,
active leases, and recorded live descendants still retain the files. The deletion
receipt records the root creation time and the corresponding liveness decision.
When Windows denies a process handle, cleanup checks the system process list and
then compares the current PID creation time through CIM. An absent or reused PID
can release its old pool; an unavailable or inconclusive secondary query retains
the target. A live matching process still protects its output.

Reusable caches use the reviewed two-phase retention path. Planning persists an immutable, expiring `plan_id` with its candidate snapshot, retention and status. Apply accepts only that server-stored plan, can run once, and may shrink it after revalidating job history, managed-root realpath, overlapping live PID, active lease and positive retention. Under disk pressure, the daemon evicts idle reusable pools oldest-first until the free-space reserve is restored; active pools remain protected. Ephemeral lanes bypass only the age delay, never the path/identity/process/lease checks. A short SQLite transaction writes a cleanup reservation and `cleanup.target_deletion_started`; deletion runs outside the global writer lock; a final short transaction writes `cleanup.target_deletion_completed`, records success/failure and clears the reservation. Both events share a random `deletion_id` and record the trigger (`prompt_cleanup`, `pressure_eviction`, or `explicit_plan`), canonical target identity, owner job/Session, pre-delete job/process state, executing process/thread, result and error. New Cargo acquisition observes every overlapping parent/child reservation. Daemon restart settles an interrupted deletion from the durable start event and current disk fact as `deleted_before_restart` or `retained_after_restart` before releasing the abandoned reservation, so a missing target is never explained only by `cleanup_status=deleted`.

The coordinator also governs every physical directory beneath the three approved D/E/F drive-root `cargo-targets` directories. A path is protected only when it belongs to a recorded Cargo job, validation-copy job, or workflow artifact. `artifact audit` reports every other directory; `artifact cleanup` removes one revalidated candidate per invocation and records `delete_started`, `deleted`, or `delete_failed` events. The daemon runs the same sweep on its 30-second loop. Before Cargo acquisition or validation-copy work starts, the service fails closed with `unmanaged_artifacts_detected` while any unknown directory remains. Therefore Sessions must register a Cargo lane or validation copy before it creates an output directory; raw Cargo targets, RenderDoc captures, or ad-hoc build folders are not supported execution paths.

The Codex `PreToolUse` Hook also rejects direct `mkdir`, `md`, `New-Item`, and `ni` creation below those roots when the target is explicit. Its audit line contains only the Session id, relative working directory, operation category, and denial reason—never the output path or full command. This catches ordinary Session mistakes before the filesystem changes; the daemon sweep and fail-closed acquisition remain the authoritative fallback for scripts or tools whose output path cannot be inferred safely by the Hook.

```powershell
.\tools\zircon-session.ps1 artifact audit -Json
.\tools\zircon-session.ps1 artifact cleanup -Json
```

The PowerShell cleanup wrapper delegates unmanaged deletion to these coordinator commands; it no longer calls `Remove-Item` for unregistered artifacts itself.

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

M4 adds unit coverage for direct-child and junction/symlink escapes, unavailable roots, case aliases, nested legacy overlap, explicit reuse, foreign-session mutation, running/pre-start orphan reconciliation, positive retention, reviewed-plan non-expansion, reservation/acquire/start concurrency, transaction-free deletion, deletion evidence and interrupted-deletion settlement. The PowerShell smoke also validates the scheduled-task plan, while validator tests assert that command-line/environment overrides cannot bypass the service and pre-start failures release their job.

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

An accepted milestone must use the workflow-aware local action path, which keeps the Session `active` while recording the exact `M<n>` attempt:

```powershell
.\tools\zircon-session.ps1 milestone prepare --session-id <session-id> --milestone M2
.\tools\zircon-session.ps1 milestone validate --session-id <session-id> --run-id <run-id> --milestone M2 --template coordinator-actions
# A distinct reviewer Session submits its accepted review after validation completes.
.\tools\zircon-session.ps1 milestone review --session-id <reviewer-session> --executor-session-id <session-id> --run-id <run-id> --milestone M2 --critical-count 0 --important-count 0 --summary "accepted"
.\tools\zircon-session.ps1 milestone commit --session-id <session-id> --run-id <run-id> --milestone M2 --summary "add variable shaping visibility diagnostics"
```

Native-plugin performance validation uses a separate one-time authorization before the
validation action. The target Session names the benchmark and profile, while the
Coordinator selects the only eligible `materialized` copy owned by the same numbered
source plan; callers cannot provide a job ID, grant ID, Cargo filter, or environment:

```powershell
.\tools\zircon-session.ps1 milestone grant-benchmark `
  --session-id <target-session-id> `
  --source-session-id <source-session-id> `
  --run-id <run-id> `
  --milestone M1 `
  --benchmark-name native_runtime_broadcast_8_plugin_benchmark `
  --cargo-profile release

.\tools\zircon-session.ps1 milestone validate `
  --session-id <target-session-id> `
  --run-id <run-id> `
  --milestone M1 `
  --template native-plugin-benchmark `
  --benchmark-name native_runtime_broadcast_8_plugin_benchmark `
  --cargo-profile release
```

The durable grant binds the source and target Sessions, FIFO reservation, named case,
profile, server-generated command, milestone-scoped manifest and the complete immutable
copy input manifest. Validation rechecks both manifest domains independently, consumes
only the target Session's FIFO head, and records the root PID with the grant and workflow
binding before terminal collection. `ZR_BENCHMARK_SOURCE_MANIFEST` and
`ZR_BENCHMARK_CARGO_PROFILE` are derived from that binding and injected only into the
benchmark child. Ordinary synchronous and asynchronous validation children remove any
inherited values for those keys.

On Windows, the benchmark root is created suspended and atomically assigned to a
non-inheritable kill-on-close Job Object before its first instruction. The Coordinator
persists the root PID and creation time before resume and keeps the Job handle until the
root and all descendants are terminal. Root exit alone is not terminal evidence: the
collector terminates and waits for the complete Job before draining EOF, importing the
workflow result, or releasing the preserved copy. This prevents an exited intermediate
process from orphaning a grandchild that can still mutate the copy.

Startup reconciliation denies an unregistered `launching` grant so it cannot wedge the
FIFO. A `consumed` grant without terminal evidence is process-identity checked, its
workflow validation is rejected, and its copy is preserved. A collector or evidence
failure may leave the copy explicitly `failed`; recovery accepts that durable failed/no-run
state without rewriting or deleting its contents. Cancellation authority follows the
active grant's target Session rather than the source Session that owns the preserved copy.

The local CLI treats `previewed` and `executing` controlled actions as non-terminal. After one preview and one confirmation, it polls `GET /control/v1/actions/{action_id}` until the same durable action reaches a terminal state, then returns that action's result to the milestone command. Materialization-heavy actions such as `validation.start` therefore keep one action and one validation copy: an initial `executing` response with `result: null` is not reported as `invalid_response` and never triggers a duplicate preview or validation job. Polling uses the command deadline; exceeding it reports `command_timeout` with the action identity so callers can inspect the existing action instead of retrying it blindly.

`milestone commit` requires a concrete `--summary`. `MilestoneWorkflowService` combines it with the registered plan module and actual manifest class to build a specific plain Conventional Commit subject, such as `feat(frameworks): add render dependency diagnostics`; generic `workflow`, `milestone`, and `complete M2 milestone` summaries are rejected. It rechecks live gate state under the Git mutex, commits the exact service-bound manifest, records the accepted node, and sends WeCom exactly once after the commit SHA exists. For the example plan under `docs/plans/zircon_runtime/frameworks/`, the service uses `frameworks` on the WeCom first line as `核心内容摘要：【frameworks】M2 · <title>：<summary>`; the committed subject and the notification's fourth line remain unprefixed. A notification failure is recorded but never rolls back the commit or auto-retries delivery.

`milestone close-goal` treats commit intent states by recovery semantics: `prepared` and `committed` remain blocking because their ref outcome may still need reconciliation; `reconciled` is accepted evidence; terminal `failed` with no commit SHA remains immutable audit history and does not block closeout. The service never rewrites or deletes failed attempts merely to complete a Goal.

`finalize --milestone` is retained only as a compatibility command and is not valid for business Session closeout: it cannot identify a workflow milestone or invoke the workflow-managed WeCom notification.

Milestone commit paths must have live leases owned by the Session and current-hash attribution. The service re-imports canonical Failure Markdown, rejects validator diagnostics or open Failure nodes where the Session plan is either origin or fixer, takes `git_mutex`, rechecks the exact index and staged blob identities, and advances `HEAD` with compare-and-swap. The Session remains active after success. The workflow-aware `milestone commit` command is the only business-Session milestone commit path; a plain `git commit` is outside the workflow.

Owned-scope eligibility is Session-relative, not global-baseline-relative. Attribution proves that the requesting Session owns the exact current file bytes; the coordinator separately compares those bytes with the current `HEAD` checkout to prove that the manifest contains a real commit delta. A later global baseline capture may absorb a dirty hash for shared health tracking, but it cannot erase an already attributed tracked change that still differs from `HEAD`. The unchanged-path gate remains active for content that truly matches `HEAD`, and omitted-owned-path, live-lease, staged-blob, Failure and secret gates are unchanged.

Every material path requested by an ordinary finalize must be attributed to the completed Session at its current SHA-256 hash. Every other dirty path attributed to that Session must also appear in the manifest, so untracked files, documentation, tests and scripts cannot be silently omitted. The durable finalize request records four categories (`code`, `docs`, `tests`, `scripts`) and a separate `untracked_paths` inventory.

A Failure closeout may additionally bind immutable proof under `.codex/state/` into its exact snapshot manifest. Coordinator state remains unleaseable and is never staged or committed: it is revalidated by the closeout acceptance gate and again by the under-mutex precommit snapshot guard. Attribution and live leases still apply to every material path that enters the commit, while a changed state proof, an ordinary unattributed dirty path, or a closeout without that guard fails closed.

Validation-copy tickets whose focused test needs current files outside the eventual
Failure commit may declare `coverage.validationSupportPaths`. Each declared path
must also be present as a non-tombstone entry in the immutable ticket source
manifest. Closeout validation excludes only that explicit, immutable support set
when comparing the ticket overlay with the exact snapshot; a malformed,
undeclared, or snapshot-overlapping support path fails closed. Tickets without
the field retain strict source-manifest equality, so validation dependencies are
never silently promoted to commit scope.

Before index mutation, the service requires the baseline `HEAD` to remain current, rejects an active Git mutex, foreign leases, queued or `needs_rebase` patches, foreign staged paths, protected/global plan output, output outside the registered numbered child plan, and an unresolved Failure routed to the Session plan. A degraded baseline is retained as a workspace-health observation rather than a global finalization gate: an exactly attributed, scope-complete Session may commit without waiting for unrelated worktree changes to be reconciled. Staged added lines are scanned for maintenance capabilities and generic credentials. An intentional Enterprise WeChat webhook URL or `WECOM_WEBHOOK_KEY` configuration may enter a service-managed Git commit, but its value remains absent from coordinator persistence and error output.

The service persists the pre-transaction HEAD and exact Git index bytes after taking its database mutex, calculates each approved worktree file's Git-cleaned blob identity, stages only the approved paths, then verifies both the exact staged name set and staged blob identities. This closes the last-write race between attribution checking and staging. After optional validation commands, it repeats scope, blob and secret checks. The final commit is built from the verified index tree and advances `HEAD` with an expected-old-SHA compare-and-swap, so repository hooks or validation cannot silently widen the commit. A pre-commit or validation failure atomically restores the prior index and preserves every worktree file. Successful commits record their SHA and open a new baseline epoch that advances only committed paths; other Sessions' dirty files remain visible as baseline differences.

Service restart restores a persisted pre-commit index when HEAD did not advance, returns an interrupted Session to `completed`, and marks the request failed. The `ref_updated_sha` intent closes the post-commit/pre-baseline window: if the exact expected scoped commit already advanced HEAD before a process interruption, startup reconciles its SHA and commit-derived partial baseline instead of reporting a false failure or capturing the full dirty worktree.

The persisted Git index bytes are a recovery-only BLOB. They exist only while a
request is `finalizing`; every `committed` or `failed` transition clears them in
the same transaction that records the terminal result. Schema 47 performs the
same terminal-only cleanup for historical records before daemon admission and
then compacts an existing SQLite database. It never clears a live `finalizing`
record, does not alter baseline manifests, and does not introduce a global drain
or registration gate. If physical compaction fails, the schema marker remains
absent so a later safe startup retries rather than reporting reclaimed storage.

Health probes keep a short three-second timeout. A health timeout is reported as
`command_preflight_timeout` with `submission=not_submitted`, which is the only timeout
state that permits a fresh submission. Every command receives a request ID before that
probe, and `/command` durably records `accepted` before dispatch. A five-minute POST
response timeout is reported as `command_post_timeout` with `submission=accepted` or
`unknown`; callers query `GET /command/requests/<request-id>` and must not blindly replay
the command. Completed and failed requests return their durable result. Durable mutation
request IDs are exactly-once for the same command payload; replay-safe commands retain
that guarantee within their documented key window. `session heartbeat`, lease
claim/release, and the complete Cargo acquire/start/heartbeat/finish/release lifecycle
each use their own short SQLite transaction without acquiring the foreground
mutex used by baseline observation or validation-copy work. SQLite writer
contention still uses the bounded busy wait described below. Baseline observation and direct validation-copy
materialize/run/cleanup keep their own durable status transitions and do not own that
foreground lifecycle lane; shared-worktree patch, finalization and Failure mutations
remain serialized.

The wrapper exposes the same repository-verified recovery query:

```powershell
.\tools\zircon-session.ps1 request-status <request-id>
```

This recovery command directly reads the request endpoint and validates the
`repositoryKey` carried by that response. It intentionally does not depend on `/health`,
so a slow health projection cannot block the only durable request lookup.

For an accepted `session.register`, the registration mutation and terminal receipt
commit together. Native SQLite `BUSY` or `LOCKED` while acquiring that mutation's
initial transaction returns `command_admission_busy`; its details identify the
request, `mutation_admission` phase, SQLite result, configured wait and measured
wait, with `callbackStarted=false`. The normal ten-second SQLite busy wait remains
bounded. A writer released within that wait can allow the registration to proceed.
The registration callback is never retried by this error path.

Validation-ticket timing projections select only the six event types used to reconstruct
submission, status, copy/run links, failure reuse, and cleanup. Event order and phase
calculations are unchanged. Dependency, attention, and other audit events remain in the
ledger, but their payloads are not decoded by the timing projection while a ticket claim
holds its writer transaction.

If the same writer also prevents recording the failure, the existing request can
still read `accepted` until maintenance writes its deferred terminal failure.
Query that request ID; do not create a replacement registration. Startup recovery
records `command_execution_interrupted` if the process ended before a terminal
receipt was durable. Preparation or callback failures retain their own handling;
this diagnostic does not identify the competing writer. Historical failed
receipts retain their original error and are not rewritten by the upgrade.

Journal payload storage is bounded. The first successful caller still receives its full
live response, but a response larger than 256 KiB is persisted as a digest tombstone;
later duplicate or recovery queries report `responseOmitted` and never re-execute the
command. For mutating commands, full terminal payloads are compacted after seven days or
outside the newest 10,000 terminal requests. Their minimal request ID, command/payload
fingerprint, terminal status, and result digest remain durable without expiry so
compaction cannot make an old mutation request executable again. Non-persisting,
replay-safe read-only commands and the state-convergent Session/lease/Cargo heartbeat
commands use a separate replay-safe key window: terminal rows expire after one day and
are capped at the newest 10,000. `cleanup.plan` persists an apply-capable plan and
therefore remains durable rather than entering that bounded key window.
Those operations cannot create a second Cargo start or irreversible mutation if an old
ID is later reused. `cargo.run_reserved` is always durable, and its attached start audit
and original start acknowledgement are never removed by request-payload compaction or
replay-safe-key cleanup. Maintenance shares a fixed 256-row budget across expiry,
count-window cleanup, and durable payload compaction; large backlogs converge over
multiple ticks instead of producing an unbounded scan or transaction.

Workflow/skill maintenance uses the same transaction with explicit `--maintenance`; the daemon authorizes it with a separate local `ZIRCON_COORDINATOR_MAINTENANCE_TOKEN` capability that is never written to the runtime descriptor or Git. The ordinary shared service bearer and a client boolean are insufficient. Authorized maintenance bypasses business attribution/status checks but retains index scope, repository path, semantic-message and secret guards. Business intermediate versions continue to live in coordinator snapshots rather than Git history.

When persistent maintenance hold is active, `finalize.preview` and `finalize.commit` are available only as `operation@session-id` calls for a Session named in the daemon's maintenance scope. They remain constrained to that Session's live leases and attributed manifest paths; generic finalization, generic Git staging, and normal Cargo admission stay denied. This permits an audited dependency-lock closure without reopening the shared mutation window.

## Managed Product Staging

Production wrappers must acquire a product staging lease before creating a directory below a governed drive-root `cargo-targets` directory. The caller supplies a closed purpose, intended final path, and its PID; the Coordinator verifies the live process identity and generates the only accepted staging path. There is no caller-controlled staging path or prefix exemption.

```powershell
.\tools\zircon-session.ps1 artifact staging-acquire `
  --purpose build-editor --final-path D:\cargo-targets\editor-current --owner-pid $PID
.\tools\zircon-session.ps1 artifact staging-begin-publish `
  --lease-id <lease-id> --owner-pid $PID
.\tools\zircon-session.ps1 artifact staging-complete-publish `
  --lease-id <lease-id> --owner-pid $PID
# Failure only, after both staging and final paths are absent:
.\tools\zircon-session.ps1 artifact staging-release `
  --lease-id <lease-id> --owner-pid $PID
```

`staging-begin-publish` seals the existing staging directory's filesystem identity before the wrapper performs its root-bound atomic rename. `staging-complete-publish` accepts only the same identity at the exact final path, then replaces the temporary exemption with a durable published-artifact identity. A copied directory, caller-created final path, foreign PID, missing path, or illegal state transition fails closed. Filesystem and process probes occur before the short SQLite write transaction; the write segment revalidates the immutable owner/path/status snapshot and performs only a CAS-style lifecycle update.

Startup recovery preserves a live owner's `active` or `publishing` lease. If the owner died after the atomic move, recovery completes publication only when the final directory has the sealed staging identity; every other interrupted state becomes `recovered` and loses its governance exemption. A published path remains managed only while its filesystem identity matches. Deleting and recreating the same pathname therefore produces an ordinary unmanaged artifact rather than inheriting historical authority.

## Stable Validation Copies

Validation copies provide a stable source view without a branch, worktree or repo-local build directory:

```powershell
.\tools\zircon-session.ps1 validation-copy materialize `
  --path Cargo.toml `
  --path zircon_runtime/Cargo.toml `
  --path zircon_runtime/src/lib.rs

# `materialize` returns its job immediately. Poll until `status` is materialized.
.\tools\zircon-session.ps1 validation-copy status <job-id>
.\tools\zircon-session.ps1 validation-copy run <job-id> -- cargo check --workspace
.\tools\zircon-session.ps1 validation-copy cleanup <job-root>
```

`materialize` creates its durable job and returns before copying files. The detached worker performs filesystem I/O outside the foreground mutation mutex, so Session heartbeats, leases and Cargo `finish`/`release` requests cannot wait behind a large manifest. Its status is `materializing` until terminal `materialized` or `failed`; a materializing job cannot be run, cancelled or cleaned up. Planning pins one HEAD SHA. The worker extracts all unowned tracked files from that exact commit through one Git archive stream, then overlays only the requesting Session's current-hash-attributed paths from the worktree. This avoids per-file Git processes and prevents concurrent finalization from creating a mixed-version copy. Unowned untracked paths, `.git`, coordinator state and repository build output are rejected. The resolved `verify` root and job root are revalidated during plan, materialize, run and cleanup; junction/symlink escapes fail closed.

Validation commands acquire the job's `running` state and run with `CARGO_TARGET_DIR` fixed to the adjacent `{job-root}\target`; a second run and cleanup are rejected until execution returns to `materialized`. Exit code and bounded stdout/stderr evidence are stored in SQLite. Ordinary validation reserves `cleanup_pending` and removes the single job tree after terminal evidence is imported. An authorized native-plugin benchmark instead returns the pre-existing copy to `materialized` so the Coordinator-selected source tree remains intact; denied, stale, foreign, replayed, or out-of-FIFO launches do not mutate it. Artifact-producing raw Cargo commands are rejected by the repository `PreToolUse` Hook; the Hook writes only a sanitized local denial record and is a workflow guardrail rather than a credential boundary.

Cleanup accepts only a job root already recorded by the service and only when its resolved path is a direct child of an allowlisted `verify` root. It removes that single job tree, including the adjacent target, then records the removal.

Validation runs persist the real validation child PID. Startup releases only dead `running` reservations and removes an interrupted `materializing` tree before a retry; live processes remain protected. `cleanup_pending` is durable: both startup and the daemon's 30-second loop retry deletion of that exact recorded job root. A failed deletion stays visible as pending rather than being rewritten to `materialized`; its validation run evidence stays available for diagnosis.

## M5 Validation

M5 acceptance uses generated temporary Git repositories for all commit mutations. Coverage proves completion is non-committing, explicit finalization contains exactly the approved categorized files, foreign index state survives rejection, validation failure restores the index, webhook material is blocked, the Git mutex has one owner, validation overlays reject stale hashes, command evidence uses the adjacent target, and cleanup cannot escape its job root.

## Legacy Session Migration

Migration is report-first. The report parser reads only root-level `.codex/sessions/*.md`, accepts current YAML frontmatter and older loose `key: value` notes, computes a SHA-256 for every source, and never treats `.codex/sessions/archive/` as active input.

```powershell
.\tools\zircon-session.ps1 legacy report --report E:\temp\zircon-legacy-report.json -Json
.\tools\zircon-session.ps1 legacy import --dry-run --report E:\temp\zircon-import-preview.json -Json
.\tools\zircon-session.ps1 legacy import --apply --report E:\temp\zircon-import-applied.json -Json
```

Known status aliases map to the fixed enum. `working`, `in_progress`, and `implementing` become `active`; `done` and `complete` become `completed`; exact service statuses remain exact. Unknown or retired values such as `blocked` are preserved verbatim in `status_reason` and classified from evidence rather than persisted as a new status. Only a live PID, a note updated inside ten minutes, a fresh service heartbeat, an active lease, or a pending/rebase patch overrides even a terminal source label and keeps the note active. An open Failure remains a durable priority record in the Failure graph, but never becomes a synthetic Session heartbeat: an otherwise inactive root note still becomes `stale` and may be archived. Import is hash-keyed and idempotent, preserves newer service state, imports a numbered plan link where available, uses the source mtime for legacy timestamps, clears obsolete terminal timestamps on reactivation, and never moves or deletes the source note.

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

One maintenance tick performs enum-only stale classification, Cargo orphan reconciliation, dead validation-child recovery, a WAL checkpoint, and retention/Cargo cleanup planning. The daemon-owned periodic tick also imports and journal-archives inactive root notes, archives service-native stale Sessions after 24 hours with no live liveness signal, and applies revalidated retention/Cargo plans. A queued patch, live lease, or running Cargo job retains its owner; an open Failure does not. Failure priority stays queryable independently of both legacy-note and native-Session archival:

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
- A validation copy records the real child PID. Startup and periodic maintenance release `running` only after that PID dies. `cleanup_pending` keeps its reservation and is retried against the exact recorded root every 30 seconds; no live process is eligible for deletion.
- If the daemon is unavailable, stop writes that require leases/finalize, preserve worktree files, and run `status -Json` for structured diagnostics. Session notes remain a compatibility view, but they do not grant file ownership. The Windows tray keeps bounded recovery failures across restarts, but immediately clears an old circuit only after a replacement daemon passes descriptor, process-identity, and authenticated-health verification as a new instance.
- For emergency read-only evidence, use ordinary Git read commands and the Failure/plan validators. Do not run direct target deletion, invent a free-form status, write global plan indexes, or create a checkpoint commit.

## M6 Validation

M6 temporary-repository tests cover deterministic legacy reports, unknown-status preservation, live PID/reference classification, idempotent import, hash-preserving archive, retention with live patch references, quarantine-backed GC rollback boundaries, archived restore preview, pinned plan-root audit, daemon-owned maintenance ticks, and both startup cutover dry-runs. The real rollout imported 131 root notes, archived 121 with identical hashes, retained 10 active/recent notes, recorded four successful ticks, and left one repo-local legacy Cargo root diagnostic-only. Baseline reconciliation remained fail-closed on 64 changes owned by concurrent business Sessions.

## 实时任务看板与执行观察

`/ui/` 默认显示当前任务，统计报表位于 `/ui/statistics`。看板按需处理、运行、等待排序，区分资源排队、依赖阻塞和归档暂停；归档 Session 的未完成任务仍保留原任务及回执，执行器领取时重新检查归属与依赖。Session、类型、状态筛选与分页共享完整分类计数，单页最多 200 项，网页默认 50 项。选中项、焦点及列表滚动位置在刷新时保留；窄屏详情使用抽屉。

任务身份采用 `session:`、`thread:`、`validation:`、`copy:`、`cargo:`、`patch:`、`finalize:`、`cleanup:` 加持久对象 ID。页面展示真实阶段、耗时、最近工具活动、执行器健康、恢复次数和关联任务；仅在明确总工作量时显示百分比。完整写入范围、补丁目标与历史报表仍通过原接口按需加载。

| 只读接口 | 内容与边界 |
|---|---|
| `GET /control/v1/tasks?state=active&limit=50` | 轻量投影、完整筛选计数、分页 `nextCursor`、实例身份、`eventCursor` 和投影耗时。可选 `kind`、`session`、`cursor`。 |
| `GET /control/v1/tasks/{taskId}` | 当前任务、阶段记录及验证事件各最多 100 项、关联对象、独立审计摘要与既有受控操作入口。 |
| `GET /control/v1/tasks/{taskId}/output?channel=stdout&cursor=0` | stdout/stderr 字节游标；每次最多 64 KiB，返回 `startCursor`、下一游标及文件代次。`tail=true` 首次读取末尾；截断或轮转返回 `reset`。 |

输出使用增量 UTF-8 解码和及时 flush，stdout/stderr 分别读取，不依赖换行或 8 KiB 累积。输出存储失败时继续排空管道，避免执行器阻塞；终态仍由原任务及验证证据决定。网页支持跟随、暂停、搜索和有界历史窗口；浏览历史期间继续接收新输出，回到最新时使用保留的实时游标。审计标签展示事件摘要，不替代进程输出。

网页保持单条 SSE 连接，并以可见页面一秒任务轮询、半秒输出轮询补充同步；事件去重与合并刷新不重建 SSE。游标失效、实例变化和重连触发重新同步。失败时保留已有内容，并显示断线或数据过期。任务投影的文件与进程检查在数据库读事务结束后执行，短期缓存合并并发读取；无状态变化的 worker tick 不触发看板刷新。 SSE 游标上下界在同一 SQL 语句中通过两次索引定位读取，避免遍历全部历史事件。

### 物化恢复和聊天关联

新物化任务记录 PID、进程创建时间、执行阶段、两秒心跳及固定输入摘要。恢复先证明原执行器失效，再以原子领取绑定原输入；同一输入最多恢复两次。活执行器、缺少创建时间或旧记录缺少可靠身份时保留原归属并显示诊断；输入漂移或恢复超限进入需处理状态。恢复不得清理其他任务目录或改写验收回执。

活动日志按文件身份与字节位置增量读取，跨读取窗口保留生命周期状态及最近工具活动。注册支持 `session register --thread-id <thread-id>` 显式关联聊天与业务 Session；每个身份只允许一个关联，未可靠关联的聊天独立显示。日志观察仅保存公开工具名称和状态，不保存工具参数或聊天文本。

### 前端产物与本机客户端

前端构建、测试编译和浏览器缓存均使用协调器的 artifact lease，物理目录只能位于盘根 `D:/cargo-targets`、`E:/cargo-targets` 或 `F:/cargo-targets`。构建入口使用 Vite `--configLoader runner`，避免配置编译进入仓库内 `.vite-temp`。资源图校验后通过 staging lease 原子发布，并写入运行态 `control-web-release.json`；服务重载后读取该发布目录。发布完成与运行实例已加载是两个独立状态。

Python 客户端的回环 HTTP 使用禁用代理的专用 opener，避免 Windows 系统代理截获本机命令；不修改系统代理配置。命令超时仍按原 request ID 对账，不重放未知或已接受的操作。

## Current Coordinator command surface

The `zircon-session` parser and dispatch code now include these command groups:

- `failure claim-next`, `failure claim`, `failure renew`, `failure release`, `failure submit-fix`, `failure claims`, `failure worker`, and `failure closeout-validate`.
- `task list`, `task get`, `task result`, `task files`, `task diff`, `task output`, `task events-read`, `task events-ack`, `task priority`, `task cancel`, and `task retry`.
- `worker list`, `worker pairing-code`, `worker drain`, and `worker revoke`.
- `storage summary`, `storage reclamation`, and `storage reclaim`; `validation slots` reads the validation-slot projection.

Examples through the PowerShell wrapper:

```powershell
.\tools\zircon-session.ps1 failure claim-next --session-id <session-id> --worker-id <worker-id>
.\tools\zircon-session.ps1 failure renew <claim-id> --generation <generation> --session-id <session-id> --worker-id <worker-id>
.\tools\zircon-session.ps1 failure closeout-validate <closeout-id> --session-id <session-id> --job-id <job-id> --cargo-run-id <cargo-run-id> --validation-bindings-json '[{"lifecycleKey":"...","claimId":"...","generation":1,"workerId":"...","validationTicketId":"...","taskId":"...","attemptId":"..."}]'
.\tools\zircon-session.ps1 worker pairing-code --allowed-label windows --ttl-seconds 600
.\tools\zircon-session.ps1 worker drain <node-id> --draining
.\tools\zircon-session.ps1 worker drain <node-id> --undrain
.\tools\zircon-session.ps1 worker revoke <node-id>
.\tools\zircon-session.ps1 task output <task-id> --channel stdout --cursor 0
.\tools\zircon-session.ps1 storage summary
```

Task, worker, and storage mutations use the authenticated controlled-action preview and confirmation flow. Failure claims and `failure closeout-validate` use the authenticated command path with their own claim and evidence checks. `failure closeout-validate` accepts either one complete validation-ticket/Task/Attempt tuple or `--validation-bindings-json` containing exact per-claim tuples (`lifecycleKey`, `claimId`, positive `generation`, `workerId`, `validationTicketId`, `taskId`, and `attemptId`); the forms are mutually exclusive, and bindings reject repeated lifecycle, claim, ticket, Task, or Attempt identities. The `worker pairing-code` command issues a one-use code; it does not enroll the node. The source exposes `POST /remote/v1/workers/pair` for code exchange, and there is no `zircon-session worker pair` or worker CLI pairing subcommand. The worker `serve` entry point requires its already issued credential. `CoordinatorUpgradeServices` now constructs and injects the durable task, provider, worker protocol, storage, and validation services before the transport starts. The running instance still must report positive transport status fields before remote work is treated as active.

## Remote worker operator checkpoint

The single-port remote transport uses the coordinator's `aiohttp` listener (default `http://127.0.0.1:6518`) for control, browser/SSE traffic, worker WebSocket, and bounded object transfer. LAN/VPN access uses HTTPS and WSS on that same port; workers connect outbound and require no inbound listener. The source-level operator routes, start/install/connect sequence, and worker enrollment flow are documented in the [worker protocol guide](coordinator-worker-protocol.md). The [storage and task policy](coordinator-storage-and-task-policy.md) defines the local disk boundary, remote worker storage_root contract, scheduler limits, and attempt evidence.

Start first local access with `.\tools\zircon-session.ps1 start`, then open `http://127.0.0.1:6518`. For LAN/VPN TLS startup and the worker package, pairing, install, and connect commands, follow the [worker protocol guide](coordinator-worker-protocol.md); its parser-only help checks do not start a host.

Failure claims and remote task/node leases are independent lifecycles. Failure claims renew every 30 seconds and expire after 300 seconds. Remote workers heartbeat every 10 seconds by default; their task/node lease TTL is 60 seconds by default and can be configured only within 5-300 seconds. A remote lease expiry or stale result is attempt evidence and does not create a validation receipt or PASS.

The coordinator CLI exposes `worker list`, `worker pairing-code`, `worker drain`, and `worker revoke`; the corresponding controlled actions are `worker.pairing_code`, `worker.drain`, and `worker.revoke`. The coordinator `serve` command accepts `--host`, `--remote-enabled`, `--tls-certificate`, `--tls-private-key`, `--trusted-ca`, repeated `--trusted-origin`, and repeated `--allowed-host`. `--host` accepts only `127.0.0.1` or `0.0.0.0`; remote mode binds `0.0.0.0`. The worker module separately exposes `serve`, `status`, and `capabilities`; a worker must be given an explicit dedicated absolute `storage_root`. That remote root is not a local Cargo target exception: local Coordinator validation and build wrappers still accept only drive-root D:\cargo-targets, E:\cargo-targets, and F:\cargo-targets. `serve --help` and `remote_worker serve --help` are parser-only checks; use the start/install/connect sequence in the worker guide for the actual one-port listener and outbound worker connection.

Before treating remote work as enabled, check `/transport/v1/status` for `workerProtocolAvailable` and `objectTransferAvailable`. The final same-source loopback TLS/WSS E2E passed 1/1 in 77.081 test seconds (receipt envelope 91.654 seconds); it exercises input transfer, PASS reconciliation, output/SSE/artifacts, reuse indexing, reconnect, stale-result fencing, and exact stop proof. The refreshed ZIP is `aa878ecdacb8da27bbaff58b7ff6e2fbdc74f393278f5f33da74c41a6174247a` (131,291 bytes, 28 entries); see the [worker guide](coordinator-worker-protocol.md) for the manifest, receipts, and install commands. Neither test activates the primary instance or proves cross-machine acceptance. The user will connect the two Windows workers later. Actual Tiny/heavy-light Cargo remains pending: isolated fixture databases cannot reserve the shared host CPU through a Main pending reservation. Use one authority after runtime activation or a dedicated worker. Earlier TaskService-only board benchmark evidence is P95 187.37 ms and 56,179 bytes over 30 uncached 50-task calls plus 10,000 historical rows; it is not production HTTP P95.

## Artifact cleanup and validation status

Historical GC events 283762/283763 acted on an artifact owned by live PID 47564 that MAIN had not registered. Cleanup now deletes only MAIN-registered artifacts; legacy paths without durable ownership remain diagnostic, and missing paths only cancel reservations and release zero bytes. The separate Cargo fixture uses MAIN Artifact Governance acquire/release and does not migrate the main instance. The 18:29:59 read-only gate was CLEAR; three foreign jobs were released and their PIDs were dead with empty process trees, but no Cargo run record exists, so this is not validation acceptance.

Reused earlier browser evidence: `browser-evidence-27124/browser-acceptance.json` records 35/35 functional checks for Task files, pagination, lazy diff, logs, stage, narrow layout, auth, reconnect with one SSE stream, and unknown storage. `browser-evidence-51296/browser-claim-acceptance.json` records 19/19 controlled Failure claim-next/renew/release preview-and-execute cases, referencing the 35-case acceptance; these covered cases remain valid. Keep the failed aggregate `browser-evidence-11216/browser-failure.json` (3.758 seconds, manual selection of a different Task) separate from those passing counts. Focused run 41736 passed timing checks for SSE at 234 ms and same-Task rendering at 748 ms, but exposed selected-row and scroll failures. The repaired UI passed targeted follow-up `browser-evidence-23156/browser-stage-acceptance.json` 7/7: same-Task commit-to-visible 825.7 ms, stdout 126 ms, selected-row attention and ID preserved, focused search output preserved, scroll position 320-to-320, and one SSE stream. The TypeScript typecheck passed. The 41736 failure is retained as prior diagnostic evidence; the 23156 run is the current focused result.

Keep source validation and runtime activation as separate evidence. The authorized maintenance window on 2026-10-01 activated schema 80 on primary instance `201e5da3fce642a8b08c5ee5b9b41d86`, PID 31580, at `127.0.0.1:6518`. The original route `ecb63515762247eb9661ce6d955163dd` failed after its materializer was lost; its recorded copy was removed. This recovery closes interrupted execution without claiming validation PASS. LAN TLS, actual Cargo acceptance, and worker connections retain their separate acceptance requirements.
## Final source evidence and runtime boundary

The [delivery evidence index](../../.codex/state/coordinator-low-disk-20260930/final-delivery-source-evidence.json) binds the final source, isolated web build, worker archive, and saved receipts. Relevant final checks are:

| Area | Final evidence |
|---|---|
| Durable file scope | 19/19 affected tests; public content manifests include captured absent paths without changing their sealed input hash or durable input rows. A/M/D, drift, keyset paging, exact-path reads, and unknown-baseline guards are covered. |
| Storage API | 7/7 tests; typed protection references and reasons remain visible, omissions are marked, and the full HTTP JSON envelope is bounded to 200 KiB. No scan is started by this read adapter. |
| Browser | 42 checks on the rebuilt UI; 260 public sealed paths, lazy A/M/D diffs, drift, one SSE stream, retained selection/focus/scroll, controlled Failure claim actions, real scoped storage and GC, and narrow metric rows sized to content. |
| Cleanup | Final checks cover stale-generation cleanup, current-owner ledger references, idle high/low watermarks, cold-first pressure, active/protected exclusions, native replacement fences, restart retry, and explicit cleanup callers. |
| Worker | Current package imports and CLI checks plus the final same-source TLS/WSS receipt in the [worker guide](coordinator-worker-protocol.md). The worker in the TLS test runs from the checkout. |

The final browser receipt is `root-browser-storage-files-final-attempt9-receipt.json` (SHA-256 `6a4034bda1ec5f995885cc82cb65006973bd1fb6180375a6e6d6bcde6d97fd03`) under the protected D fixture. Measured requests were 483.3 ms for three file pages, 395.1 ms for the diff set, and 82.6 ms for the 3,397-byte storage response. The selected Task update appeared in 1,445 ms. These are actual browser/HTTP samples, not a production HTTP P95 claim. The build typecheck and 29-asset verification passed; the previous web dist and receipt remain preserved.

Scoped GC deleted only its exact owned cold target, retained its active target and hard link, reported zero failed bytes, and reduced managed bytes. Its logical removal was 1,048,576 bytes; the native D-volume free-space delta was 688,128 bytes, so the receipt accurately retains `partial` with `actual_freed_below_logical_estimate`. Volume-wide deltas can include unrelated concurrent allocations or frees; they are separate from logical file size and do not prove per-object physical savings. Failed earlier browser and TLS receipts remain diagnostic evidence.

The historical [read-only delivery checkpoint](../../.codex/state/coordinator-low-disk-20260930/delivery-runtime-checkpoint.json) at 2026-10-01 04:25:18 UTC records Main schema 73, the original route still `materializing`, its copy still `planned` without worker/manifest/run proof, `rootReleased:false`, and all 13 protected paths unchanged. The later authorized maintenance window activated schema 80 and closed that interrupted copy; its live records supersede the checkpoint's runtime status. LAN activation, actual Tiny/heavy-plus-light Cargo, and cross-machine acceptance retain separate gates. The user will provide worker connections and request physical acceptance later. A momentarily empty active-Cargo list does not provide shared-host authority to an isolated fixture database.

#### Legacy task cancellation and interrupted receipts

`task.cancel` also retires queued legacy validation tickets whose owning Session
is archived, completed, or cancelled. It rejects a linked active copy or executor.
The coordinator atomically records a cancelled scheduling task even when old
validator metadata cannot be attached under the current source-seal contract.
This record has no executable command or acceptance authority. Original ticket
status, source objects, hashes, and validation evidence remain available in history.
Cancelled tasks are absent from active and paused task lists.

Exclusive daemon startup fails legacy in-process materializers that lost their
executor before handoff. Copies with a PID, worker identity, or current local
materializer remain protected. The linked materializing ticket receives a failed
receipt; normal terminal-ticket cleanup then handles its physical copy. Startup
also reconciles old accepted resume intents whose controlled action already failed,
and closes interrupted command requests without replaying their mutations.

Deleted orphan Cargo receipts remain cancelled history and do not consume the
bounded active-task health budget. Retained orphan resources stay visible.

The authorized live cleanup cancelled 21 obsolete patches and 40 validation
scheduling tasks, plus one superseded maintenance patch. Each cancellation has
a controlled action receipt. Original ticket rows, source manifests, patch
objects, and failed validation evidence remain in history. See the
[live cleanup verification](../../.codex/state/session-coordinator/record-cleanup-live-verified-20261001-01a0f599.json)
for record status and active/paused queue reconciliation.

The schema-80 listener requires `aiohttp==3.14.3`, pinned in
`tools/session_coordinator/requirements.in`. Verify that the interpreter used by
the actual daemon can import `aiohttp` and `aiohttp.web` before reloading it.
Start through `python -m tools.session_coordinator` or the documented PowerShell
launcher; executing its `__main__.py` as a standalone script cannot resolve
package-relative imports. The maintenance installation uses binary wheels
under `E:\cargo-targets\coordinator-runtime`, with no repository-local compiler
output or compiler cache.
