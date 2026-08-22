---
related_code:
  - tools/session_coordinator/__main__.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/client.py
  - tools/session_coordinator/config.py
  - tools/session_coordinator/offline_queue.py
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
  - tools/session_coordinator/cargo_run_registration.py
  - tools/session_coordinator/cargo_runner.py
  - tools/session_coordinator/artifact_product_staging.py
  - tools/session_coordinator/artifact_governance.py
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
  - tools/session_coordinator/codex_sync/worker.py
  - tools/session_coordinator/codex_sync/store.py
  - tools/session_coordinator/web/src/api/contracts.ts
  - tools/session_coordinator/web/src/api/validation.ts
  - tools/session_coordinator/web/src/pages/OverviewPage.tsx
  - tools/session_coordinator/control_plane/actions/catalog.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/actions/fingerprint.py
  - tools/session_coordinator/control_plane/actions/service.py
  - tools/session_coordinator/workflows/models.py
  - tools/session_coordinator/workflows/store.py
  - tools/session_coordinator/workflows/projections.py
  - tools/zircon-session.ps1
  - tools/build-editor.ps1
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
  - tools/session_coordinator/offline_queue.py
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
  - tools/session_coordinator/cargo_run_registration.py
  - tools/session_coordinator/cargo_runner.py
  - tools/session_coordinator/artifact_product_staging.py
  - tools/session_coordinator/artifact_governance.py
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
  - tools/session_coordinator/codex_sync/worker.py
  - tools/session_coordinator/codex_sync/store.py
  - tools/session_coordinator/web/src/api/contracts.ts
  - tools/session_coordinator/web/src/api/validation.ts
  - tools/session_coordinator/web/src/pages/OverviewPage.tsx
  - tools/session_coordinator/control_plane/actions/catalog.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/actions/fingerprint.py
  - tools/session_coordinator/control_plane/actions/service.py
  - tools/session_coordinator/workflows/models.py
  - tools/session_coordinator/workflows/store.py
  - tools/session_coordinator/workflows/projections.py
  - tools/zircon-session.ps1
  - tools/build-editor.ps1
  - tools/cleanup-stale-targets.ps1
  - tools/install-session-coordinator-task.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
plan_sources:
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
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - tools/tests/session-coordinator-smoke.Tests.ps1
  - tools/tests/build-editor.Tests.ps1
doc_type: workflow-detail
---

# Local Session Coordinator

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

The browser never receives the runtime bearer. `ui open` uses the authenticated local client to issue a 30-second, single-use Observer ticket and opens `/ui/bootstrap/{ticket}` without printing the ticket. Successful consumption creates an `HttpOnly`, `SameSite=Strict` cookie scoped to `/control`; browser requests also require the loopback Host/Origin boundary and mutations require the session CSRF value. A daemon restart invalidates the ticket, cookie and elevation grants. Do not open the root URL directly, proxy the listener, or publish the control port to another host.

All mutable coordinator data remains under `.codex/state/session-coordinator/`:

- `coordinator.sqlite3`: WAL database for Sessions, events, baseline epochs, object indexes, snapshots, attributions, leases and patches;
- `objects/`: zlib-compressed SHA-256 objects;
- `runtime.json`: local connection descriptor with the fixed loopback endpoint and per-instance bearer capability; diagnostics and browser payloads omit the capability;
- `coordinator.lock`: single-instance ownership.

The service validates the active Git branch. A checkout that is not on `main` is diagnostic/read-only: health, Session list and Session show remain available, while mutations fail with `not_on_main`.

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

Schema v4-v7 records Cargo jobs, cleanup reservations and persisted cleanup plans; schema v22 adds reusable-cache identity and cleanup state and repairs databases whose historical v21 marker predated those columns. Schema v30 records every process-tree observation, schema v31 records the Cargo root PID's creation identity, schema v32 distinguishes a Cargo root from a wrapper that supervises sequential Cargo commands, schema v41 persists each new CPU reservation's canonical compatibility payload, schema v42 extends the same durable contract to the single GPU lane with an immutable approved target directory, and schema v43 adds one durable FIFO successor behind an already bound lane reservation. Jobs use `check`, `test`, `workspace`, and `gpu` lanes with `leased`, `running`, `succeeded`, `failed`, `released`, and `orphaned` states. Targets must remain below one of the nine drive-root trees named `cargo-targets`, `targets`, or `ZirconBuilds` on `D:`, `E:`, or `F:`. A case- and separator-normalized identity plus ancestor/descendant overlap checks prevent Windows path aliases or nested pools from becoming simultaneous writers. Repo-local targets, symlink/junction escapes and arbitrary paths fail with `cargo_target_not_managed`.

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

## Cleanup and Service-Owned Maintenance

Cleanup is deliberately two-phase:

```powershell
.\tools\cleanup-stale-targets.ps1
.\tools\cleanup-stale-targets.ps1 -Apply -WhatIf
.\tools\cleanup-stale-targets.ps1 -Apply
```

Reusable caches use the reviewed two-phase retention path. Planning persists an immutable, expiring `plan_id` with its candidate snapshot, retention and status. Apply accepts only that server-stored plan, can run once, and may shrink it after revalidating job history, managed-root realpath, overlapping live PID, active lease and positive retention. Under disk pressure, the daemon evicts idle reusable pools oldest-first until the free-space reserve is restored; active pools remain protected. Ephemeral lanes bypass only the age delay, never the path/identity/process/lease checks. A short SQLite transaction writes a cleanup reservation and `cleanup.target_deletion_started`; deletion runs outside the global writer lock; a final short transaction writes `cleanup.target_deletion_completed`, records success/failure and clears the reservation. Both events share a random `deletion_id` and record the trigger (`prompt_cleanup`, `pressure_eviction`, or `explicit_plan`), canonical target identity, owner job/Session, pre-delete job/process state, executing process/thread, result and error. New Cargo acquisition observes every overlapping parent/child reservation. Daemon restart settles an interrupted deletion from the durable start event and current disk fact as `deleted_before_restart` or `retained_after_restart` before releasing the abandoned reservation, so a missing target is never explained only by `cleanup_status=deleted`.

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
each use their own short SQLite transaction and never wait behind baseline observation
or validation-copy work. Baseline observation and direct validation-copy
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

Production wrappers must acquire a product staging lease before creating a directory below a governed `ZirconBuilds` root. The caller supplies a closed purpose, intended final path, and its PID; the Coordinator verifies the live process identity and generates the only accepted staging path. There is no caller-controlled staging path or prefix exemption.

```powershell
.\tools\zircon-session.ps1 artifact staging-acquire `
  --purpose build-editor --final-path D:\ZirconBuilds\editor-current --owner-pid $PID
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
