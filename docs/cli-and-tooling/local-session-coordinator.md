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
  - tools/session_coordinator/cargo_reservations.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cleanup.py
  - tools/session_coordinator/legacy.py
  - tools/session_coordinator/audit.py
  - tools/session_coordinator/processes.py
  - tools/session_coordinator/supervision/
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/git_guard.py
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
  - tools/session_coordinator/cargo_reservations.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cleanup.py
  - tools/session_coordinator/legacy.py
  - tools/session_coordinator/audit.py
  - tools/session_coordinator/processes.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/git_guard.py
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
  - docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-15-milestone-finalize-session-relative-owned-scope.md
  - docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-15-support-slice-exact-finalize-plan-output-conflict.md
  - docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-16-stale-session-pending-cpu-reservation-starvation.md
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
  - tools/session_coordinator/tests/test_cleanup.py
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
  - tools/session_coordinator/tests/test_action_catalog.py
  - tools/session_coordinator/tests/test_action_auth.py
  - tools/session_coordinator/tests/test_action_fingerprint.py
  - tools/session_coordinator/tests/test_action_execution.py
  - tools/session_coordinator/tests/test_action_concurrency.py
  - tools/session_coordinator/tests/test_milestone_cli.py
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - tools/tests/session-coordinator-smoke.Tests.ps1
doc_type: workflow-detail
---

# Local Session Coordinator

## Purpose

The local Session coordinator is the shared-`main` control plane for ZirconEngine development. It gives each Session a typed lifecycle, records a hash-based workspace baseline, stores intermediate file contents outside Git, serializes concrete file writes, governs plan/failure records, and owns isolated Cargo validation lanes.

Business Session work remains service-managed between accepted milestones. Every accepted milestone is an explicit service-owned Git commit; arbitrary checkpoints and hidden intermediate commits remain forbidden. Direct `git commit`, generic completion of a numbered-plan Session, and legacy `finalize --milestone` are rejected so a business change cannot bypass its workflow attempt or WeCom result. The service protects unrelated active Sessions and their dirty files without creating branches or worktrees.

On each writable daemon start, the coordinator installs local `.git/hooks/pre-commit` and `prepare-commit-msg` gates. They block direct Git commits, including `--no-verify`; one pre-existing user hook of either name is preserved with the `.zircon-user` suffix. The Codex pre-tool gate also rejects direct commit forms that override `core.hooksPath` and direct shared-index mutations (`git add`, `rm`, `mv`, `reset`, or `restore --staged`). Coordinator commits use the scoped `commit-tree` path after their gates pass. This is a workflow guardrail for the shared checkout, not a substitute for the service's attribution and milestone checks.

## Runtime and State

Run the Windows entrypoint from the repository root:

```powershell
.\tools\zircon-session.ps1 start -Json
.\tools\zircon-session.ps1 status -Json
```

The wrapper starts Python in a hidden window only when the health endpoint is unavailable. A repository-scoped named mutex serializes automatic startup, and callers probe the fixed health endpoint while a successor is publishing `runtime.json`; this prevents a descriptor-publication gap during a controlled restart from spawning competing daemon wrappers. The shared coordinator binds the fixed loopback endpoint `127.0.0.1:6518` and writes the port, PID and instance metadata to `.codex/state/session-coordinator/runtime.json`; local clients still read that descriptor to reject stale instances. Isolated test coordinators explicitly request an OS-assigned port. The control service is deliberately token-free: all requests from this loopback-only listener are local control requests, so a browser tab remains usable after a daemon restart.

Schema version 16 completes the permissioned controlled-action protocol on top of the read-only workflow facade. It closes `action_kind` at the database boundary and installs compatibility triggers for databases that already applied the early v15 action tables. The runtime descriptor also records the daemon `instance_id`, `started_at`, and supported `control_api_versions`, allowing local clients to reject credentials created by a previous daemon instance. Detailed operator guidance lives in [Workflow Control Center](workflow-control-center.md); module contracts live in [Control Plane](../tools/session_coordinator/control-plane.md) and [Workflow Read Model](../tools/session_coordinator/workflows.md).

Open the local control surface or inspect the same coherent snapshot from the terminal:

```powershell
Start-Process "http://127.0.0.1:6518/"
.\tools\zircon-session.ps1 control snapshot -Json
```

The root URL redirects to `/ui/`. The browser does not need a bearer token, bootstrap ticket, cookie, or CSRF value. The only supported exposure boundary is the exact IPv4 loopback listener; do not proxy or publish the control port to another host.

All mutable coordinator data remains under `.codex/state/session-coordinator/`:

- `coordinator.sqlite3`: WAL database for Sessions, events, baseline epochs, object indexes, snapshots, attributions, leases and patches;
- `objects/`: zlib-compressed SHA-256 objects;
- `runtime.json`: local connection descriptor with the fixed loopback endpoint;
- `coordinator.lock`: single-instance ownership.

The service validates the active Git branch. A checkout that is not on `main` is diagnostic/read-only: health, Session list and Session show remain available, while mutations fail with `not_on_main`.

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

## Failure and Recovery Semantics

- Missing or stale runtime descriptors produce a structured `offline` result and exit code `3`.
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

Schema v4-v7 records Cargo jobs, cleanup reservations and persisted cleanup plans; schema v22 adds reusable-cache identity and cleanup state and repairs databases whose historical v21 marker predated those columns. Schema v30 records every process-tree observation, schema v31 records the Cargo root PID's creation identity, schema v32 distinguishes a Cargo root from a wrapper that supervises sequential Cargo commands, and schema v41 persists each new CPU reservation's canonical compatibility payload without fabricating payloads for historical rows. Jobs use `check`, `test`, `workspace`, and `gpu` lanes with `leased`, `running`, `succeeded`, `failed`, `released`, and `orphaned` states. Targets must remain below one of the nine drive-root trees named `cargo-targets`, `targets`, or `ZirconBuilds` on `D:`, `E:`, or `F:`. A case- and separator-normalized identity plus ancestor/descendant overlap checks prevent Windows path aliases or nested pools from becoming simultaneous writers. Repo-local targets, symlink/junction escapes and arbitrary paths fail with `cargo_target_not_managed`.

```powershell
.\tools\zircon-session.ps1 cargo acquire workspace --ephemeral
.\tools\zircon-session.ps1 cargo acquire test --compatibility-json '{"platform":"windows","toolchain":"1.88.0@x86_64-pc-windows-msvc","target_architecture":"x86_64-pc-windows-msvc","workspace":"Cargo.toml","build_config":"profile=test;features=default"}'
.\tools\zircon-session.ps1 cargo acquire check --ephemeral
.\tools\zircon-session.ps1 cargo list
```

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
When a nominated job reaches `released` after its process tree is empty, the same
release transaction moves its bound CPU reservation to `released`; a later owner
handoff is not required. Reserve/acquire also reconcile a legacy `finished` head only
when its nominated job is already `released`, its recorded process tree is empty, and
its owner is non-executable. Live or executable owners remain FIFO heads and are never
reclaimed by that historical repair path.
Recreating the service does not
rewrite `expires_at`, and the next reserve/acquire transaction removes an expired or
non-executable pending head before applying FIFO, so a stale zero-job owner cannot starve
unrelated validation.

Reusable acquisition requires a complete compatibility document containing platform (`windows` or `wsl`), Rust toolchain, target architecture, repository-relative workspace and canonical build configuration. The service adds normalized repository identity and hashes that document. Source and `Cargo.lock` changes deliberately do not split the pool because Cargo performs unit-level invalidation. Check/test lane labels also do not split it. Exactly one primary directory exists per compatibility key across Sessions and exactly one task may own it; concurrent compatible acquisition returns `cargo_reuse_pool_busy` instead of creating a fallback pool. Legacy duplicate retained directories are demoted to prompt deletion while the newest remains authoritative. Missing compatibility metadata fails closed to ephemeral by default, as does an explicit `--ephemeral` request; release commits ownership state and wakes a single worker that drains pending requests, reserves and revalidates each exact directory, then deletes outside the writer transaction. A locked deletion becomes `failed`; release-driven cleanup leaves it alone, and the daemon's default 30-second watch loop retries failed Cargo cleanup.

The web control center separates the real-time Cargo baseline from the historical audit feed. Its four lifecycle counters and Cargo table use only the latest coordinator record for each target directory that still exists on disk. Consequently, a target deleted after an earlier lock failure is not shown as a current failure, and repeated jobs sharing one reusable directory count once. The history payload remains available to the service for audit, but it does not influence the live cards: `可复用池`, `用后即删`, `待清理`, and `清理失败` describe current directories only.

`validate-matrix.ps1` performs the Windows lifecycle automatically: register the caller, derive the compatibility document, acquire the primary pool with the wrapper PID, immediately enter `try/finally`, record the process command line and root creation identity at start, run validation, record the exit code, and owner-checked release. It marks that PowerShell PID as a **supervisor**, so finish/release ignore the still-live wrapper itself after its sequential Cargo calls have returned but still reject any live Cargo/rustc descendant. Direct `cargo start` jobs remain Cargo-root jobs and retain their live root check. Every observation compares the current root creation identity before traversing descendants: a different identity means Windows has reused the PID, so that unrelated process and its descendants cannot retain the old Cargo target. A matching root—or a known descendant after the matching root exits—continues to protect the target. Pre-identity `orphaned` rows retain their historical terminal state rather than treating a later reused PID as Cargo. WSL Cargo is permitted only through a coordinator-aware Windows host wrapper that acquires with `platform=wsl`, remains alive and heartbeats while its `wsl.exe` child runs, and translates only the granted path to its mounted equivalent; direct unleased WSL Cargo is forbidden. Explicit `-TargetDir` and inherited `CARGO_TARGET_DIR` are normalized through the same policy and cannot create an alternate primary directory. Dry-run jobs are audited but their directories are not created. The daemon converts dead running jobs and dead/timed-out pre-start leases to `orphaned` and immediately retries pending ephemeral cleanup.

## Cleanup and Service-Owned Maintenance

Cleanup is deliberately two-phase:

```powershell
.\tools\cleanup-stale-targets.ps1
.\tools\cleanup-stale-targets.ps1 -Apply -WhatIf
.\tools\cleanup-stale-targets.ps1 -Apply
```

Reusable caches use the reviewed two-phase retention path. Planning persists an immutable, expiring `plan_id` with its candidate snapshot, retention and status. Apply accepts only that server-stored plan, can run once, and may shrink it after revalidating job history, managed-root realpath, overlapping live PID, active lease and positive retention. Under disk pressure, the daemon evicts idle reusable pools oldest-first until the free-space reserve is restored; active pools remain protected. Ephemeral lanes bypass only the age delay, never the path/identity/process/lease checks. A short SQLite transaction writes a cleanup reservation; deletion runs outside the global writer lock; a final short transaction records success/failure and clears the reservation. New Cargo acquisition observes the reservation, and daemon restart recovers abandoned reservations.

The coordinator also governs every physical directory beneath the nine approved D/E/F roots. A path is protected only when it belongs to a recorded Cargo job, validation-copy job, or workflow artifact. `artifact audit` reports every other directory; `artifact cleanup` removes one revalidated candidate per invocation and records `delete_started`, `deleted`, or `delete_failed` events. The daemon runs the same sweep on its 30-second loop. Before Cargo acquisition or validation-copy work starts, the service fails closed with `unmanaged_artifacts_detected` while any unknown directory remains. Therefore Sessions must register a Cargo lane or validation copy before it creates an output directory; raw Cargo targets, RenderDoc captures, or ad-hoc `ZirconBuilds` folders are not supported execution paths.

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

An accepted milestone must use the workflow-aware local action path, which keeps the Session `active` while recording the exact `M<n>` attempt:

```powershell
.\tools\zircon-session.ps1 milestone prepare --session-id <session-id> --milestone M2
.\tools\zircon-session.ps1 milestone validate --session-id <session-id> --run-id <run-id> --milestone M2 --template coordinator-actions
# A distinct reviewer Session submits its accepted review after validation completes.
.\tools\zircon-session.ps1 milestone review --session-id <reviewer-session> --executor-session-id <session-id> --run-id <run-id> --milestone M2 --critical-count 0 --important-count 0 --summary "accepted"
.\tools\zircon-session.ps1 milestone commit --session-id <session-id> --run-id <run-id> --milestone M2 --summary "add variable shaping visibility diagnostics"
```

The local CLI treats `previewed` and `executing` controlled actions as non-terminal. After one preview and one confirmation, it polls `GET /control/v1/actions/{action_id}` until the same durable action reaches a terminal state, then returns that action's result to the milestone command. Materialization-heavy actions such as `validation.start` therefore keep one action and one validation copy: an initial `executing` response with `result: null` is not reported as `invalid_response` and never triggers a duplicate preview or validation job. Polling uses the command deadline; exceeding it reports `command_timeout` with the action identity so callers can inspect the existing action instead of retrying it blindly.

`milestone commit` requires a concrete `--summary`. `MilestoneWorkflowService` combines it with the registered plan module and actual manifest class to build a specific plain Conventional Commit subject, such as `feat(frameworks): add render dependency diagnostics`; generic `workflow`, `milestone`, and `complete M2 milestone` summaries are rejected. It rechecks live gate state under the Git mutex, commits the exact service-bound manifest, records the accepted node, and sends WeCom exactly once after the commit SHA exists. For the example plan under `docs/plans/zircon_runtime/frameworks/`, the service uses `frameworks` on the WeCom first line as `核心内容摘要：【frameworks】M2 · <title>：<summary>`; the committed subject and the notification's fourth line remain unprefixed. A notification failure is recorded but never rolls back the commit or auto-retries delivery.

`milestone close-goal` treats commit intent states by recovery semantics: `prepared` and `committed` remain blocking because their ref outcome may still need reconciliation; `reconciled` is accepted evidence; terminal `failed` with no commit SHA remains immutable audit history and does not block closeout. The service never rewrites or deletes failed attempts merely to complete a Goal.

`finalize --milestone` is retained only as a compatibility command and is not valid for business Session closeout: it cannot identify a workflow milestone or invoke the workflow-managed WeCom notification.

Milestone commit paths must have live leases owned by the Session and current-hash attribution. The service re-imports canonical Failure Markdown, rejects validator diagnostics or open Failure nodes where the Session plan is either origin or fixer, takes `git_mutex`, rechecks the exact index and staged blob identities, and advances `HEAD` with compare-and-swap. The Session remains active after success. The workflow-aware `milestone commit` command is the only business-Session milestone commit path; a plain `git commit` is outside the workflow.

Owned-scope eligibility is Session-relative, not global-baseline-relative. Attribution proves that the requesting Session owns the exact current file bytes; the coordinator separately compares those bytes with the current `HEAD` checkout to prove that the manifest contains a real commit delta. A later global baseline capture may absorb a dirty hash for shared health tracking, but it cannot erase an already attributed tracked change that still differs from `HEAD`. The unchanged-path gate remains active for content that truly matches `HEAD`, and omitted-owned-path, live-lease, staged-blob, Failure and secret gates are unchanged.

Every requested path must be attributed to the completed Session at its current SHA-256 hash. Every other dirty path attributed to that Session must also appear in the manifest, so untracked files, documentation, tests and scripts cannot be silently omitted. The durable finalize request records four categories (`code`, `docs`, `tests`, `scripts`) and a separate `untracked_paths` inventory.

Before index mutation, the service rejects a degraded/stale baseline, an active Git mutex, foreign leases, queued or `needs_rebase` patches, foreign staged paths, protected/global plan output, output outside the registered numbered child plan, and an unresolved Failure routed to the Session plan. Staged added lines are also scanned for Enterprise WeChat webhook URLs and credential markers. Webhook configuration remains local and must never enter Git.

The service persists the pre-transaction HEAD and exact Git index bytes after taking its database mutex, calculates each approved worktree file's Git-cleaned blob identity, stages only the approved paths, then verifies both the exact staged name set and staged blob identities. This closes the last-write race between attribution checking and staging. After optional validation commands, it repeats scope, blob and secret checks. The final commit is built from the verified index tree and advances `HEAD` with an expected-old-SHA compare-and-swap, so repository hooks or validation cannot silently widen the commit. A pre-commit or validation failure atomically restores the prior index and preserves every worktree file. Successful commits record their SHA and open a new baseline epoch that advances only committed paths; other Sessions' dirty files remain visible as baseline differences.

Service restart restores a persisted pre-commit index when HEAD did not advance, returns an interrupted Session to `completed`, and marks the request failed. The `ref_updated_sha` intent closes the post-commit/pre-baseline window: if the exact expected scoped commit already advanced HEAD before a process interruption, startup reconciles its SHA and commit-derived partial baseline instead of reporting a false failure or capturing the full dirty worktree.

Health probes keep a short three-second timeout. Mutating service commands use a separate five-minute client timeout. If that deadline is genuinely reached, the client reports typed `command_timeout` with the command and deadline rather than falsely reporting the daemon as offline; callers inspect the typed job/session status before retrying. `session heartbeat`, lease claim/release, and the complete Cargo acquire/start/heartbeat/finish/release lifecycle each use their own short SQLite transaction and never wait behind baseline observation or validation-copy work. Baseline observation and direct validation-copy materialize/run/cleanup keep their own durable status transitions and do not own that foreground lifecycle lane; shared-worktree patch, finalization and Failure mutations remain serialized.

Workflow/skill maintenance uses the same transaction with explicit `--maintenance`; the daemon authorizes it with a separate local `ZIRCON_COORDINATOR_MAINTENANCE_TOKEN` capability that is never written to the runtime descriptor or Git. The ordinary shared service bearer and a client boolean are insufficient. Authorized maintenance bypasses business attribution/status checks but retains index scope, repository path, semantic-message and secret guards. Business intermediate versions continue to live in coordinator snapshots rather than Git history.

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

Validation commands acquire the job's `running` state and run with `CARGO_TARGET_DIR` fixed to the adjacent `{job-root}\target`; a second run and cleanup are rejected until execution returns to `materialized`. Exit code and bounded stdout/stderr evidence are stored in SQLite. After terminal evidence is imported, the coordinator automatically reserves `cleanup_pending` and removes the single job tree. Artifact-producing raw Cargo commands are rejected by the repository `PreToolUse` Hook; the Hook writes only a sanitized local denial record and is a workflow guardrail rather than a credential boundary.

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
- A validation copy records the real child PID. Startup and periodic maintenance release `running` only after that PID dies. `cleanup_pending` keeps its reservation and is retried against the exact recorded root every 30 seconds; no live process is eligible for deletion.
- If the daemon is unavailable, stop writes that require leases/finalize, preserve worktree files, and run `status -Json` for structured diagnostics. Session notes remain a compatibility view, but they do not grant file ownership.
- For emergency read-only evidence, use ordinary Git read commands and the Failure/plan validators. Do not run direct target deletion, invent a free-form status, write global plan indexes, or create a checkpoint commit.

## M6 Validation

M6 temporary-repository tests cover deterministic legacy reports, unknown-status preservation, live PID/reference classification, idempotent import, hash-preserving archive, retention with live patch references, quarantine-backed GC rollback boundaries, archived restore preview, pinned plan-root audit, daemon-owned maintenance ticks, and both startup cutover dry-runs. The real rollout imported 131 root notes, archived 121 with identical hashes, retained 10 active/recent notes, recorded four successful ticks, and left one repo-local legacy Cargo root diagnostic-only. Baseline reconciliation remained fail-closed on 64 changes owned by concurrent business Sessions.
