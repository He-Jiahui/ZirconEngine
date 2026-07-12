# Codex Session Hook Synchronization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task in the existing shared `main` checkout. Do not create a branch or worktree.

**Goal:** Automatically synchronize every local Codex Session rooted in ZirconEngine into the Session Coordinator and expose a bounded, privacy-safe projection in the web control center.

**Architecture:** A project-level Codex `notify` hook writes only a sanitized trigger and forwards the user's existing global notifier. A single coordinator worker performs startup, periodic, and hook-woken incremental reconciliation of read-only Codex rollout metadata into schema-v27 projection tables. Business Sessions remain authoritative for plans, leases, patches, workflows, validation, and commits; Codex source Sessions are displayed separately and bind only by exact thread/session ID.

**Tech Stack:** Python 3.14 standard library (`json`, `tomllib`, `sqlite3`, `pathlib`, `http.client`), PowerShell 7 installer tests, SQLite migrations, existing coordinator HTTP/SSE/action framework, React 19, TypeScript 6, Material UI 9, Node test runner.

**Design source:** `docs/superpowers/specs/2026-07-13-codex-session-hook-sync-design.md`

---

## Repository and execution rules

- Work only in `E:\Git\ZirconEngine` on `main`; no branch, worktree, stash, or repo-local Cargo target.
- Claim concrete files before editing. If another live Session owns a file, queue a delayed patch or advance an independent slice.
- Keep Codex rollout files read-only. Never copy prompts, assistant messages, goals, instructions, tool arguments, attachments, environment values, tokens, webhook URLs, or raw hook JSON into Git, SQLite, logs, SSE, or tests.
- Do not make hook execution depend on daemon availability. The hook must terminate promptly after an atomic external-spool write.
- Do not create business Session state, leases, patches, workflows, validation jobs, or commits from Codex source records.
- Complete H1 then H2 serially. H3 and H4 may proceed independently only after H2 has established the stable domain and hook contracts. H5 is the single integration and release gate.
- Commit each completed milestone with a normal Conventional Commit subject. After every commit, send one four-line WeCom message using the existing `【session_coordinator】` prefix only in the message.

## File map

### New production files

- `tools/codex_session_hook.py` — fast notify argument reduction, repository filter, atomic trigger write, authenticated wake signal, and global-notifier forwarding.
- `tools/install-codex-session-hook.ps1` — idempotent Query/Install/Update/Remove/DryRun management of the exact project notify entry.
- `tools/session_coordinator/codex_sync/__init__.py` — narrow public exports.
- `tools/session_coordinator/codex_sync/models.py` — source-location/state/event enums and immutable discovery/reconcile records.
- `tools/session_coordinator/codex_sync/discovery.py` — bounded rollout membership scan and first-line/tail parser.
- `tools/session_coordinator/codex_sync/spool.py` — repository-scoped trigger queue validation, cap, quarantine, and acknowledgement.
- `tools/session_coordinator/codex_sync/store.py` — v27 projection upsert, exact business binding, missing confirmation, run audit, and event emission.
- `tools/session_coordinator/codex_sync/worker.py` — single-flight wake/coalescing worker and periodic/full-scan schedule.

### New tests and fixtures

- `tools/session_coordinator/tests/codex_rollout_fixture.py` — privacy-safe rollout fixture builder with active, idle, archived, malformed, append-racing, and restored variants.
- `tools/session_coordinator/tests/test_codex_discovery.py` — containment, bounded parsing, state derivation, and privacy tests.
- `tools/session_coordinator/tests/test_codex_store.py` — migration, idempotency, exact binding, missing confirmation, and event tests.
- `tools/session_coordinator/tests/test_codex_spool.py` — trigger schema, queue cap, quarantine, and acknowledgement tests.
- `tools/session_coordinator/tests/test_codex_worker.py` — single-flight, wake coalescing, startup/periodic recovery, and shutdown tests.
- `tools/session_coordinator/tests/test_codex_hook.py` — notify reduction, forwarding, latency, recursion, and online/offline daemon tests.
- `tools/tests/codex-session-hook.Tests.ps1` — installer Query/Install/Update/Remove/DryRun and TOML-preservation acceptance.
- `tools/session_coordinator/web/src/__tests__/codexSessions.test.tsx` — contract rejection, bounded rendering, text-only diagnostics, and business/source separation.

### Existing files to modify

- `.codex/config.toml` — exact project notify command only; preserve sandbox and approval settings.
- `tools/session_coordinator/migrations.py` — schema v27 tables, checks, indexes, and migration registration.
- `tools/session_coordinator/config.py` — Codex home/spool/scan limits and intervals.
- `tools/session_coordinator/app.py` — construct discovery, spool, store, and worker dependencies.
- `tools/session_coordinator/server.py` — worker lifecycle, authenticated wake route, controlled reconcile action wiring, and shutdown join.
- `tools/session_coordinator/control_plane/actions/models.py` — `codex.sessions.reconcile` action enum/catalog contract.
- `tools/session_coordinator/control_plane/actions/executor.py` — enqueue-only reconcile execution.
- `tools/session_coordinator/control_plane/actions/fingerprint.py` — bounded Codex sync state fingerprint.
- `tools/session_coordinator/control_plane/router.py` — authenticated hook wake endpoint and action route mapping.
- `tools/session_coordinator/control_plane/snapshot.py` — bounded `codexSessions` and `codexSync` projection.
- `tools/session_coordinator/web/src/api/contracts.ts` — strict Codex projection types and enums.
- `tools/session_coordinator/web/src/api/validation.ts` — runtime validation for every nested Codex field.
- `tools/session_coordinator/web/src/pages/SessionsPage.tsx` — separate business and Codex Session panels.
- `tools/session_coordinator/web/src/App.tsx` — pass the bounded Codex projection to the Sessions route.
- `tools/session_coordinator/web/src/__tests__/contracts.test.ts` — producer-shape and malformed-input coverage.
- `docs/cli-and-tooling/workflow-control-center.md` — hook installation, privacy, recovery, status, and removal operations.
- `tests/acceptance/workflow-control-center-and-tray.md` — requirement-to-evidence acceptance matrix and final commands.
- `tools/session_coordinator/soak.py` — include Codex worker/queue/session projection continuity in the source-frozen 24-hour gate.
- `tools/session_coordinator/tests/test_soak.py` — two-generation Codex projection continuity and queue-drain assertions.

## H1 — Schema v27 and read-only Codex discovery

### Goal

Establish the lowest shared domain: typed source states, bounded rollout parsing, deterministic state derivation, and transactional projection storage without daemon or UI integration.

### Dependencies

- Existing schema v26 migration chain and `Database.transaction()` semantics.
- Existing `events` and `sessions` tables.
- Read-only access to `$CODEX_HOME/sessions` and `$CODEX_HOME/archived_sessions`.

### Implementation slices

- [ ] **H1.1 Add failing domain/discovery tests and fixtures.** Create fixtures whose first line contains oversized `base_instructions`, messages, goals, fake tokens, and webhook-like strings; assert the discovered record contains only thread ID, cwd, origin/CLI/thread source, safe event enum, turn ID, timestamps, size/mtime, and source location. Assert canonical cwd containment rejects sibling-prefix, symlink escape, alternate-drive, and malformed paths.
- [ ] **H1.2 Add typed discovery models.** Define `CodexSourceLocation(active, archived, missing)`, `CodexSessionState(active, idle, archived, unavailable)`, and a closed `CodexLifecycleEvent` allowlist. Use frozen dataclasses for `CodexDiscoveredSession`, `CodexSourceRevision`, and `CodexDiscoveryDiagnostic`.
- [ ] **H1.3 Implement bounded discovery.** Read at most the first JSONL line plus a 64 KiB tail, tolerate a concurrently partial final line, cap membership at 10,000 rollout files, normalize Windows paths case-insensitively, and sort output by thread ID for deterministic reconciliation.
- [ ] **H1.4 Add schema-v27 failure-first tests.** Prove migration atomicity, idempotency, enum checks, foreign-key behavior, indexes, and rollback to a valid v26 database after injected failure.
- [ ] **H1.5 Implement v27 and `CodexSessionStore`.** Add `codex_sessions` and `codex_sync_runs`; upsert changed revisions, bind only exact `sessions.session_id == thread_id`, require two complete membership scans before `unavailable`, and emit sanitized events only on discovery/state/location/diagnostic changes.
- [ ] **H1.6 Document the module boundary.** Add machine-readable related-code/test headers to `docs/cli-and-tooling/workflow-control-center.md` and explain why Codex source presence is separate from business Session authority.

### Testing stage H1-T

Run from the repository root:

```powershell
python -m unittest -v `
  tools.session_coordinator.tests.test_codex_discovery `
  tools.session_coordinator.tests.test_codex_store `
  tools.session_coordinator.tests.test_database
python -m compileall -q tools/session_coordinator
```

Expected: all discovery/store/database tests pass; no fixture secret value appears in SQLite, event payloads, diagnostics, or command output. If an upper storage assertion fails, repair discovery/model invariants first and rerun upward.

### Exit evidence

- v26→v27 and fresh→v27 both pass.
- Repeated full and incremental reconciliation produce identical rows and no duplicate events.
- Exact binding is proven; fuzzy goal/title/plan matching is absent.
- Commit: `feat(workflow): add Codex session projection`

## H2 — Non-blocking notify hook, spool, and installer

### Goal

Install a project-level Codex notify multiplexer that preserves the existing global notifier, produces only sanitized bounded triggers, and remains useful while the daemon is offline.

### Dependencies

- H1 enums and repository identity rules.
- Existing runtime descriptor and process-identity verification utilities.

### Implementation slices

- [ ] **H2.1 Add hook RED tests.** Exercise no JSON, malformed JSON, final JSON argument, non-Zircon cwd, sibling-prefix cwd, symlink escape, oversized values, online daemon, stale descriptor, slow daemon, recursion, absent global notify, and exact once-only forwarding of the original argument vector.
- [ ] **H2.2 Implement `CodexTriggerSpool`.** Store schema-versioned triggers below `%LOCALAPPDATA%/Zircon Session Coordinator/codex-hook/<repository-key>/pending`; use create-temp + fsync + atomic replace, validate every field, cap files at 1,024, quarantine one corrupt item, and acknowledge only after a committed reconciliation.
- [ ] **H2.3 Implement the hook entry point.** Bound input to 64 KiB, retain at most safe IDs/cwd/event/timestamp, complete the local write before signaling, use a 250 ms authenticated localhost timeout, reject stale repository/process identity, and never start a daemon.
- [ ] **H2.4 Preserve the existing notifier.** Read only the top-level global `notify` array via `tomllib`, detect self-recursion by normalized argv, forward the original Codex arguments once, and avoid persisting the global command anywhere.
- [ ] **H2.5 Implement installer lifecycle.** `Query`, `Install`, `Update`, `Remove`, and `DryRun` must preserve unrelated `.codex/config.toml` keys/comments, write exactly the managed notify entry, report sanitized state, and remove only the repository-scoped spool after identity/path verification.
- [ ] **H2.6 Add the project hook configuration.** Set the project notify command to the committed hook entry point while retaining `approval_policy` and `sandbox_mode` unchanged.
- [ ] **H2.7 Document installation and privacy.** Add exact commands and explain that global notify remains the forwarding source and is never committed.

### Testing stage H2-T

```powershell
python -m unittest -v `
  tools.session_coordinator.tests.test_codex_hook `
  tools.session_coordinator.tests.test_codex_spool `
  tools.session_coordinator.tests.test_runtime_descriptor
pwsh -NoProfile -File tools/tests/codex-session-hook.Tests.ps1
```

Expected: every hook invocation finishes within 500 ms with offline/slow daemon fixtures; the spool contains no fixture secret; the forwarding fixture receives the original argv exactly once; repeated installer operations are byte-stable outside the managed entry.

### Exit evidence

- Hook works with daemon online and offline and never launches a process.
- Existing global notifier forwarding is preserved without Git state.
- Install/update/remove are idempotent and path-safe.
- Commit: `feat(workflow): add Codex notify synchronization hook`

## H3 — Single-flight daemon worker, wake API, and controlled reconciliation

### Goal

Consume hook triggers and recover missed notifications through one repository-scoped worker without blocking HTTP, duplicating scans, or weakening daemon identity rules.

### Dependencies

- H1 store/discovery contracts.
- H2 validated spool format.
- Existing supervision state, action protocol, server lifecycle, and authenticated router.

### Implementation slices

- [ ] **H3.1 Add worker concurrency tests.** Prove startup scan, 30-second membership tick, 15-minute full pass, hook wake, duplicate coalescing, one pending follow-up, stop/join, read-only suppression, and failure isolation.
- [ ] **H3.2 Implement `CodexSyncWorker`.** Use one daemon thread, one wake event, a single-flight lock, monotonic deadlines, bounded batch sizes, and a final drain-safe stop. A second wake while running sets one follow-up flag rather than spawning a thread.
- [ ] **H3.3 Wire startup and maintenance.** Construct the worker only after schema/identity validation, perform initial reconciliation after the daemon is healthy, expose its health to supervision blockers, and join it before closing database/HTTP resources.
- [ ] **H3.4 Add authenticated wake endpoint.** Accept only the runtime token, matching repository key, supported trigger schema, loopback origin, and a body no larger than 4 KiB; return `202` after setting the wake event without scanning in the request thread.
- [ ] **H3.5 Add controlled reconcile action.** Define `codex.sessions.reconcile` as maintainer-only and enqueue-only through preview/confirm/state fingerprint/audit. Parameters are an empty object; arbitrary paths, Codex homes, thread IDs, or raw payloads are rejected.
- [ ] **H3.6 Add recovery and action tests.** Prove stale runtime descriptor, multiple daemon candidates, read-only/fatal supervision, action replay, state change, cancellation-before-execution, and worker failure produce stable sanitized codes.

### Testing stage H3-T

```powershell
python -m unittest -v `
  tools.session_coordinator.tests.test_codex_worker `
  tools.session_coordinator.tests.test_action_catalog `
  tools.session_coordinator.tests.test_action_execution `
  tools.session_coordinator.tests.test_control_auth `
  tools.session_coordinator.tests.test_control_http `
  tools.session_coordinator.tests.test_server `
  tools.session_coordinator.tests.test_supervision_service
```

Expected: one worker executes at a time, HTTP wake latency remains bounded, trigger acknowledgement follows committed reconciliation, and shutdown leaves no worker thread or unacknowledged committed trigger.

### Exit evidence

- Startup, periodic, notify, and controlled-action paths share one reconciler.
- Multiple/stale daemon identity always fails closed.
- Commit: `feat(workflow): run Codex session reconciliation service`

## H4 — Bounded web projection and Session visualization

### Goal

Display all reconciled Codex source Sessions in the existing Sessions route without confusing them with business Session authority or allowing unbounded/private fields into the browser.

### Dependencies

- H1 v27 projection.
- H3 worker/run health.

### Implementation slices

- [ ] **H4.1 Add strict contract RED tests.** Reject unknown state/location/event enums, missing IDs/timestamps, raw JSON fields, oversized diagnostic strings, malformed bindings, and collections above the snapshot limit.
- [ ] **H4.2 Add bounded snapshot projection.** Return at most 1,000 Codex rows ordered by active-first then last activity, plus aggregate total/truncated/state counts, last run, queue depth, last success, and sanitized diagnostic code.
- [ ] **H4.3 Add TypeScript runtime contracts.** Define exact string unions and validate every nested field before state update; legacy daemon snapshots without `codexSessions` remain readable during rolling upgrade.
- [ ] **H4.4 Build separate Codex Session panel.** Show state text/icon, short thread ID with full title, source location, last activity/sync, origin/CLI, exact binding, and diagnostic code. Use `BoundedTable`, text rendering, and non-color status labels.
- [ ] **H4.5 Add sync health summary.** Display queue depth, last successful reconciliation, last terminal code, source counts, and truncation; expose the controlled reconcile action only through the existing Actions page and role gates.
- [ ] **H4.6 Add producer/consumer tests and docs.** Keep snapshots/SSE below existing payload limits and document how an operator distinguishes Codex presence from active business ownership.

### Testing stage H4-T

```powershell
python -m unittest -v `
  tools.session_coordinator.tests.test_control_snapshot `
  tools.session_coordinator.tests.test_control_events `
  tools.session_coordinator.tests.test_control_load
npm --prefix tools/session_coordinator/web run check
```

Expected: Python projection tests pass; Web typecheck, all component/contract tests, production build, and dist verifier pass; no raw rollout text or fixture secret reaches generated assets.

### Exit evidence

- Business and Codex panels are visually and semantically separate.
- Projection remains bounded at 1,000 rows and reports truncation.
- Commit: `feat(workflow): visualize synchronized Codex sessions`

## H5 — Security, recovery, operations, and release acceptance

### Goal

Prove the complete hook-to-daemon-to-web flow under realistic local Codex volume, daemon failures, restart, installation lifecycle, and a new source-frozen 24-hour run.

### Dependencies

- H1 through H4 complete and committed.
- Independent review has no unresolved Critical or Important finding.

### Implementation slices

- [ ] **H5.1 Add deterministic load fixture.** Generate 5,000 privacy-safe rollout files, 1,000 active Zircon rows, 4,000 archived rows, 1,024 queued triggers, partial tails, malformed files, sibling-repository cwd values, and 500 business Session exact/non-exact IDs without writing raw samples to Git.
- [ ] **H5.2 Add security and latency matrix.** Measure initial scan, unchanged incremental scan, hook p95 latency, snapshot p95, wake endpoint p95, queue drain, and Web render bounds. Assert no path traversal, prompt leakage, token leakage, notifier recursion, duplicate worker, or daemon launch.
- [ ] **H5.3 Add restart continuity.** Restart the coordinator once while hook triggers accumulate; prove v27 projection/event cursor continuity, queue acknowledgement after successor health, and exact business bindings unchanged.
- [ ] **H5.4 Complete operator docs and acceptance matrix.** Include install/query/update/remove, offline recovery, stale descriptor, duplicate daemon diagnosis, queue cleanup, privacy guarantees, and final evidence commands.
- [ ] **H5.5 Run full pre-soak gates.** Execute all coordinator Python tests in bounded groups, Web `npm run check`, Windows Tray tests/build in a coordinator-managed D/E/F target pool, `workflow-control-center-smoke.Tests.ps1 -Full`, plan-output audit, Failure audit, `git diff --check`, and an independent Critical/Important review.
- [ ] **H5.6 Run a new source-frozen 24-hour soak.** Extend the source set to `.codex/config.toml`, hook/installer, `codex_sync`, tests, Web source/dist, tray, and soak/smoke entry points. Require duration ≥86,400 seconds, exactly one daemon restart/two instances, all source samples matching, hook queue drained, at least two Codex Sessions retained across restart, no raw data leakage, errors empty, bounded per-instance RSS/handles, event continuity, and successful external-workspace cleanup.
- [ ] **H5.7 Final closeout.** Write only sanitized metrics to `docs/plans/zircon_tooling/session_coordinator/02/`, commit the milestone, send the four-line WeCom record once, remove external soak/hook fixtures, release leases, archive the Session note, and close the Goal only after every requirement is evidenced.

### Testing stage H5-T

```powershell
python -m unittest discover -s tools/session_coordinator/tests -p "test_*.py" -v
npm --prefix tools/session_coordinator/web run check
pwsh -NoProfile -File tools/tests/codex-session-hook.Tests.ps1
pwsh -NoProfile -File tools/tests/workflow-control-center-smoke.Tests.ps1 -Full
python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:\Git\ZirconEngine
python .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/audit_failure_handoffs.py --repo-root E:\Git\ZirconEngine
git diff --check
```

Tray Rust validation must run on Windows with a coordinator-managed `CARGO_TARGET_DIR` below one approved `D:`, `E:`, or `F:` `cargo-targets`, `targets`, or `ZirconBuilds` root. The job must heartbeat, finish, and release in `finally`; compatible output is reused and incomplete compatibility output is deleted immediately.

Expected: every gate passes, independent review reports `0 Critical / 0 Important`, and the final soak JSON reports `passed` with every hook/session/restart/resource/cleanup invariant satisfied.

### Exit evidence

- The complete objective is verified requirement by requirement, not inferred from tests alone.
- Final commit: `feat(workflow): complete Codex session synchronization`
- WeCom receives exactly one four-line message for the commit and no webhook material enters Git.

## 状态与产出记录

执行时逐切片填写；完成一个切片更新一行，不许批量补记。具体记录超过 10 条时全部迁移到 `docs/plans/zircon_tooling/session_coordinator/02/YYYY-MM-DD-<summary>.md`。

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
