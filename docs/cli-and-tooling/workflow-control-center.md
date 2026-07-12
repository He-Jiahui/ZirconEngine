---
related_code:
  - tools/session_coordinator/codex_sync/worker.py
  - .codex/hooks/zircon_session_sync.py
  - .codex/hooks.json
  - tools/install-codex-session-hook.ps1
  - tools/session_coordinator/codex_sync/hook.py
  - tools/session_coordinator/codex_sync/spool.py
  - tools/session_coordinator/codex_sync/discovery.py
  - tools/session_coordinator/codex_sync/models.py
  - tools/session_coordinator/codex_sync/store.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/control_plane/auth.py
  - tools/session_coordinator/control_plane/contracts.py
  - tools/session_coordinator/control_plane/events.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/control_plane/http_security.py
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/run-control-validation.ps1
  - tools/session_coordinator/control_plane/assets.py
  - tools/session_coordinator/control_plane/artifact_downloads.py
  - tools/session_coordinator/control_plane/actions/catalog.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/actions/fingerprint.py
  - tools/session_coordinator/control_plane/actions/models.py
  - tools/session_coordinator/control_plane/actions/permissions.py
  - tools/session_coordinator/control_plane/actions/service.py
  - tools/session_coordinator/soak.py
  - tools/session_coordinator/supervision/lifecycle.py
  - tools/session_coordinator/supervision/service.py
  - tools/session_tray/src/app.rs
  - tools/session_tray/src/coordinator_client.rs
  - tools/session_tray/src/lifecycle.rs
  - tools/session_tray/src/menu.rs
  - tools/session_tray/src/recovery.rs
  - tools/session_tray/src/startup.rs
  - tools/session_tray/src/tray_state.rs
  - tools/session_coordinator/web/src/App.tsx
  - tools/session_coordinator/web/src/api/contracts.ts
  - tools/session_coordinator/web/src/api/validation.ts
  - tools/session_coordinator/web/src/pages/SessionsPage.tsx
  - tools/session_coordinator/web/src/pages/ActionsPage.tsx
  - tools/session_coordinator/web/src/components/actions/ActionActivityList.tsx
  - tools/session_coordinator/workflows/projections.py
  - tools/session_coordinator/client.py
  - tools/session_coordinator/cli.py
implementation_files:
  - tools/session_coordinator/codex_sync/worker.py
  - .codex/hooks/zircon_session_sync.py
  - .codex/hooks.json
  - tools/install-codex-session-hook.ps1
  - tools/session_coordinator/codex_sync/hook.py
  - tools/session_coordinator/codex_sync/spool.py
  - tools/session_coordinator/codex_sync/discovery.py
  - tools/session_coordinator/codex_sync/models.py
  - tools/session_coordinator/codex_sync/store.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/control_plane/auth.py
  - tools/session_coordinator/control_plane/contracts.py
  - tools/session_coordinator/control_plane/events.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/control_plane/http_security.py
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/run-control-validation.ps1
  - tools/session_coordinator/control_plane/assets.py
  - tools/session_coordinator/control_plane/artifact_downloads.py
  - tools/session_coordinator/control_plane/actions/catalog.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/actions/fingerprint.py
  - tools/session_coordinator/control_plane/actions/models.py
  - tools/session_coordinator/control_plane/actions/permissions.py
  - tools/session_coordinator/control_plane/actions/service.py
  - tools/session_coordinator/soak.py
  - tools/session_coordinator/supervision/lifecycle.py
  - tools/session_coordinator/supervision/service.py
  - tools/session_tray/src/app.rs
  - tools/session_tray/src/coordinator_client.rs
  - tools/session_tray/src/lifecycle.rs
  - tools/session_tray/src/menu.rs
  - tools/session_tray/src/recovery.rs
  - tools/session_tray/src/startup.rs
  - tools/session_tray/src/tray_state.rs
  - tools/session_coordinator/web/src/App.tsx
  - tools/session_coordinator/web/src/api/contracts.ts
  - tools/session_coordinator/web/src/api/validation.ts
  - tools/session_coordinator/web/src/pages/SessionsPage.tsx
  - tools/session_coordinator/web/src/pages/ActionsPage.tsx
  - tools/session_coordinator/web/src/components/actions/ActionActivityList.tsx
  - tools/session_coordinator/workflows/projections.py
  - tools/session_coordinator/client.py
  - tools/session_coordinator/cli.py
plan_sources:
  - docs/superpowers/specs/2026-07-13-codex-session-hook-sync-design.md
  - docs/plans/zircon_tooling/session_coordinator/02-codex-session-hook-sync.md
  - docs/superpowers/specs/2026-07-11-workflow-control-center-and-tray-design.md
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
tests:
  - tools/session_coordinator/tests/test_codex_worker.py
  - tools/session_coordinator/tests/test_codex_hook.py
  - tools/session_coordinator/tests/test_codex_spool.py
  - tools/tests/codex-session-hook.Tests.ps1
  - tools/session_coordinator/tests/test_codex_discovery.py
  - tools/session_coordinator/tests/test_codex_store.py
  - tools/session_coordinator/tests/test_database.py
  - tools/session_coordinator/tests/test_supervision_schema.py
  - tools/session_coordinator/tests/test_control_auth.py
  - tools/session_coordinator/tests/test_control_events.py
  - tools/session_coordinator/tests/test_control_http.py
  - tools/session_coordinator/tests/test_control_security.py
  - tools/session_coordinator/tests/test_control_snapshot.py
  - tools/session_coordinator/tests/test_control_assets.py
  - tools/session_coordinator/tests/test_artifact_downloads.py
  - tools/session_coordinator/tests/test_action_catalog.py
  - tools/session_coordinator/tests/test_action_auth.py
  - tools/session_coordinator/tests/test_action_fingerprint.py
  - tools/session_coordinator/tests/test_action_execution.py
  - tools/session_coordinator/tests/test_action_concurrency.py
  - tools/session_coordinator/tests/test_supervision_actions.py
  - tools/session_coordinator/tests/test_supervision_service.py
  - tools/session_coordinator/tests/test_server.py
  - tools/session_coordinator/tests/test_control_load.py
  - tools/session_coordinator/tests/test_control_recovery.py
  - tools/session_coordinator/tests/test_control_security_matrix.py
  - tools/session_coordinator/tests/test_soak.py
  - tools/tests/workflow-control-center-smoke.Tests.ps1
  - tools/tests/workflow-control-center-soak.ps1
  - tools/session_coordinator/web/src/__tests__/components.test.tsx
  - tools/session_coordinator/web/src/__tests__/contracts.test.ts
  - tools/session_coordinator/web/src/__tests__/events.test.ts
  - tools/session_coordinator/web/src/__tests__/failureGraph.test.ts
  - tools/session_coordinator/web/src/__tests__/graphLayout.test.ts
  - tools/session_coordinator/web/src/__tests__/navigation.test.ts
  - tools/session_coordinator/web/src/__tests__/reducer.test.ts
  - tools/session_coordinator/web/src/__tests__/actions.test.tsx
doc_type: operator-guide
---

# Workflow Control Center and Windows Tray

## Current Surface

M1 adds the loopback-only control facade, M2 adds the production read console, M3 adds the closed controlled-action protocol, M4 adds workflow topology and milestone management, and M5 adds Windows tray supervision plus controlled service lifecycle. The coordinator remains the only mutation authority; the browser and tray cannot supply arbitrary shell commands, Git/Cargo arguments, SQL, repository paths, webhook content, or generic command kinds.

The console is the Jenkins-like observation surface for Workflow/Node/Attempt state, Session ownership, Failure graph, file leases, delayed patches, Cargo validation, Git finalize evidence, artifacts, logs, and audit history. The tray is only a verified local supervisor: it renders the same supervision state, opens the console through a one-time Observer ticket, and invokes lifecycle operations through the same preview/confirm action protocol.

The facade is versioned under `/control/v1`. Existing coordinator commands and authenticated legacy routes remain available and unchanged.

## Codex Source Session Boundary

Schema v27 introduces a read-only Codex source projection. The discovery layer scans only rollout files below the configured Codex active and archived roots, accepts only canonical working directories inside this repository, and parses only the first `session_meta` record plus a bounded 64 KiB lifecycle tail. Prompts, assistant output, goals, instructions, tool payloads, attachments, environment values, credentials, webhook material, and raw hook JSON are never copied into SQLite, Git, logs, or events.

`codex_sessions` records source presence and the closed `active`, `idle`, `archived`, or `unavailable` state. It is deliberately separate from the existing `sessions` table: Codex presence does not create a business Session, claim a file lease, queue a patch, start Cargo, advance a workflow, or authorize a commit. The only automatic relationship is an exact `codex_sessions.thread_id == sessions.session_id` binding; titles, plan paths, goals, and message text are never used for fuzzy association.

An absent rollout is marked `unavailable` only after two complete directory-membership scans. A truncated or incomplete scan cannot remove source presence. Reconciliation is transactional and emits only thread IDs, enums, counts, timestamps, and sanitized diagnostic codes.

### Lifecycle Hook installation and trust

The repository declares command handlers for `SessionStart`, `UserPromptSubmit`, `Stop`, `SubagentStart`, and `SubagentStop` in `.codex/hooks.json`. They run alongside matching user, managed, and plugin Hooks; the existing global `notify` command is a separate mechanism and is neither forwarded nor modified. Project Hooks run only after Codex trusts the project layer and the exact definition. Installation never reads, writes, or bypasses that trust decision.

```powershell
.\tools\install-codex-session-hook.ps1 -Action Query
.\tools\install-codex-session-hook.ps1 -Action Install
.\tools\install-codex-session-hook.ps1 -Action Update
.\tools\install-codex-session-hook.ps1 -Action Remove -DryRun
.\tools\install-codex-session-hook.ps1 -Action Remove
```

After `Install`, a changed `Update`, or a Git update to `.codex/hooks.json`, open Codex `/hooks` and review the project definition. `Query` conservatively reports that configured project Hooks require manual trust review because the installer intentionally has no access to the trust store. Repeating an unchanged Install/Update is byte-stable and does not claim to refresh trust.

Every invocation reduces stdin to IDs, closed event/source/permission enums, canonical repository cwd, safe model/subagent metadata, and a timestamp. The trigger is atomically persisted below `%LOCALAPPDATA%/Zircon Session Coordinator/codex-hook/<repository-key>/pending` before a best-effort authenticated 250 ms wake request. The Hook never reads transcript files and never persists prompt, assistant message, tool payload, attachment, environment, token, webhook, or raw stdin content. Offline, stale, slow, identity-mismatched, and pre-H3 daemons leave the sanitized trigger queued; they never cause the Hook to start a process or wait for reconciliation.

`Stop` always returns valid continuation JSON, including malformed-input and internal-import fallback paths. Other configured events are silent on success. The pending queue is capped at 1,024 entries; corrupt external items are moved to the repository-scoped quarantine, and valid items can be acknowledged only with a committed reconcile run ID. `Remove` deletes only the exact managed project definition, its owned `features.hooks` line, and the verified repository spool. A modified project `hooks.json` fails removal closed, while unrelated TOML keys/comments and every global/user/plugin Hook source remain intact.

### Reconciliation service and controlled recovery

After schema and repository identity validation, the daemon starts exactly one `zircon-codex-session-sync` worker. Startup performs a full pass; Hook and authenticated HTTP wakeups coalesce through one event; a 30-second membership tick reparses only path/size/mtime/location changes; and a 15-minute full pass rereads every bounded source to repair rare timestamp-preserving changes. A wake received during a run produces at most one immediate follow-up. Shutdown first stops and joins this worker, allowing the in-flight transaction to commit and acknowledge its captured spool batch before HTTP/database teardown.

The worker is suppressed on non-main, draining, read-only, identity-mismatch, fatal-integrity, and other supervision states that reject mutations. Failure records expose only `codex_sync_failed`; exception text is not copied into health, events, SQLite, or the browser. A later wake retries normally. Health includes bounded run counts, last run ID, sanitized error code, running state, and pending-wake state.

`POST /control/v1/codex-sync/wake` is runtime-token only. It requires exact loopback transport, a body no larger than 4 KiB, the current repository key, and trigger schema 1, then returns `202` after setting the wake event without scanning on the request thread. Hook signaling additionally requires runtime descriptor PID/creation time and `coordinator.lock` PID agreement; a stale descriptor or competing daemon chain therefore remains queued offline.

Maintainers may use the closed `codex.sessions.reconcile` action through the normal Preview/Confirm/audit protocol. Its parameter object must be exactly empty; paths, Codex homes, thread IDs, prompts, or arbitrary payloads are rejected. Confirm only enqueues the same worker and cannot invoke a second reconciliation path. Schema v28 atomically extends the action audit enum while preserving action approvals, supervision events, lifecycle intents, their foreign keys, uniqueness rules, and immutable-history triggers.

### Codex Session browser projection

The Sessions route renders two intentionally separate panels. The business Session panel remains the authority for plan state, leases, validation and commits. The Codex source panel is read-only presence: it shows a shortened thread ID, closed state and source-location labels, last lifecycle event/activity/sync timestamps, safe origin/CLI metadata, exact business binding and sanitized diagnostic code. It never receives a rollout path, raw revision, prompt, message, goal, tool input or environment value.

The server returns at most 1,000 Codex rows, active-first and then newest-activity-first, plus total/truncation, state/source counts, pending queue depth and last reconciliation status. Browser contracts validate every nested field and enum before updating state. A legacy daemon snapshot without `codexSessions` is normalized to an empty panel during rolling upgrade; unexpected fields and oversized diagnostics fail closed.

Action Activity is identity-scoped to the current daemon, actor, browser session and bound Session. The page restores the newest bounded records after refresh, resumes polling `executing` actions from `sessionStorage`, and renders actor, reason, result and sanitized error evidence. Read-only, identity-mismatch and fatal-integrity states disable mutation controls while preserving this audit view.

## Opening the Local Control Surface

Start or verify the coordinator first, then request a short-lived Observer bootstrap:

```powershell
.\tools\zircon-session.ps1 start -Json
.\tools\zircon-session.ps1 ui ticket --role observer -Json
.\tools\zircon-session.ps1 ui open
```

`ui open` asks the daemon for a one-time ticket and opens the loopback URL. The ticket expires after 30 seconds, is stored only as a digest, can be consumed once and is bound to the current daemon instance. Successful consumption creates an eight-hour `HttpOnly`, `SameSite=Strict` cookie scoped to `/control`; ordinary output does not print the ticket.

After authentication the daemon serves the production console at `/ui/`. Deep links below `/ui/` use the console shell, while `/control/v1/*` never falls back to HTML. The page starts as Observer even when the daemon runs on `main`; mutation controls remain disabled until a short-lived, Session-bound elevation is consumed.

For terminal inspection without a browser:

```powershell
.\tools\zircon-session.ps1 control snapshot -Json
```

## Read-Only Endpoints

- `POST /control/v1/bootstrap-tickets` issues an Observer ticket through the existing bearer-authenticated local client.
- `GET /ui/bootstrap/{ticket}` consumes a ticket, installs the control cookie and redirects to a credential-free URL.
- `GET /control/v1/meta` reports API and daemon-instance metadata.
- `GET /control/v1/snapshot` returns the bounded coherent dashboard snapshot.
- `GET /control/v1/workflows/{run-id}` returns a workflow projection with its current accepted attempt and immutable attempt history.
- `GET /control/v1/logs?limit={count}&before={event-id}` returns a bounded audit range for virtualized log paging.
- `GET /control/v1/events/stream` streams ordered Server-Sent Events from `Last-Event-ID` or the `cursor` query parameter and rejects capacity overflow instead of silently adding unbounded clients.
- `GET /control/v1/artifacts/{opaque-id}` downloads coordinator-owned evidence. The database mapping, not a browser path, selects the file; resolved files must remain below the workflow artifact root. Single byte ranges are supported and bounded.
- `POST /control/v1/elevation-grants` is runtime-authenticated and issues one-use elevated grants; Maintainer also requires the separate maintenance capability.
- `POST /control/v1/auth/elevate` consumes a grant with the existing Observer cookie and returns the one in-memory CSRF token.
- `GET /control/v1/actions/catalog` and `GET /control/v1/actions/{action-id}` expose the closed catalog and sanitized status.
- `POST /control/v1/actions/preview`, `POST /control/v1/actions/{action-id}/confirm`, and `POST /control/v1/actions/{action-id}/cancel` implement the two-phase mutation protocol.

Every JSON response uses the v1 envelope and carries a correlation identifier. Unexpected internal exceptions are logged server-side and returned as sanitized error contracts.

## Browser Trust Boundary

The server listens only on `127.0.0.1`. Browser-facing requests must use an exact loopback `Host`; requests with a non-loopback Host fail closed. Every M3 mutation requires an elevated role, the `HttpOnly` control cookie, an exact loopback Origin and the current `X-CSRF-Token`. Elevation rotates the CSRF token and expires after 15 minutes; daemon restart invalidates the cookie/grant instance binding.

No bearer token, maintenance capability, ticket value, cookie value or Enterprise WeChat endpoint belongs in Git, API payloads, dashboard logs or screenshots.

## Permission Issuance

Observer is automatic. Operator and Committer grants are issued only through the bearer-authenticated local CLI/tray path; Maintainer additionally requires the separate process-local maintenance capability. A grant is stored as a digest, expires after 60 seconds and can be consumed once.

```powershell
.\tools\zircon-session.ps1 control elevate `
  --role operator `
  --session-id workflow-control-center-20260711-1915 `
  --actor local-cli `
  -Json
```

Paste only the returned one-time grant into the console's **受控操作** page. The browser cannot issue a grant. Committer must bind to a Session; Operator should normally bind as well. A grant whose actor, daemon instance or Session binding differs from the cookie is rejected. Do not persist grants or CSRF tokens in files, screenshots, logs or Git.

## Action Lifecycle and Audit

All mutations use `Preview -> Confirm -> Execute`:

1. Preview validates the closed typed parameters, role and Session binding, then stores Action ID, risk, impact, warnings, confirmation-phrase hash, expiry and a state fingerprint.
2. Confirm requires the exact phrase and a non-empty reason. Under the daemon's shared mutation gate it recomputes HEAD, index, baseline, target hashes, leases, Failure Markdown/graph, delayed patches, validation copies, plan hash, Cargo jobs, Session status and daemon identity.
3. A mismatch records `state_changed` and performs no side effect. The UI creates a new preview, shows the added/removed impact and fingerprint change, and never retries the mutation automatically.
4. A match writes an immutable approval row and keeps the shared mutation gate through the typed side effect. Patch, Failure and validation actions execute against the exact resource set pinned by preview; success or sanitized failure is recorded in both `action_requests` and the event audit.

Yellow actions cover Session heartbeat/activation, Session-write-scope lease claim, own-lease release, own delayed-patch processing, allowlisted validation templates/cancel, Failure refresh, topology refresh and drain preview. Drain preview intentionally has no lifecycle executor before M5. Validation source and command are server-derived; browser paths and argv are not accepted. Validation start registers the child process while holding the mutation gate and completes asynchronously, so cancellation and other control operations remain available while the command runs.

Useful stable denial codes include `action_kind_unknown`, `action_parameters_invalid`, `action_disabled`, `action_permission_denied`, `action_session_scope_mismatch`, `csrf_invalid`, `action_confirmation_mismatch`, `action_expired`, `action_state_changed` and `action_lease_conflict`. Use the Action ID to find the corresponding `action.*` event without copying credentials.

## Snapshot and Event Consistency

Snapshot assembly uses one deferred SQLite read transaction. The snapshot cursor and all panels therefore describe one database view. Consumers apply only events after that cursor. The logical replay window retains the latest 4,096 event positions independently of longer audit retention. If a requested cursor is stale or ahead of the database, the server instructs the client to refresh its snapshot instead of guessing at missing state. Each connection reads at most 256 events per batch, has a five-second socket-write deadline, and occupies one of eight explicit client slots.

Workflow lists are projections over coordinator-owned data. M1 creates one stable control-center workflow per Session and a Goal node whose fallback state follows the typed Session lifecycle. Session changes and their workflow projection commit in the same SQLite writer transaction, including maintenance-driven stale/archive transitions. Attempts are database-enforced immutable; a newer accepted attempt becomes current while earlier attempts remain inspectable, and later heartbeats cannot overwrite that accepted state.

## Recovery

- If a ticket expires or is already used, request a new one with `ui open`.
- If the daemon restarts, old tickets and cookies are rejected by daemon-instance binding; reopen the UI.
- If an event cursor is too old, discard the partial client view and fetch `/control/v1/snapshot` again.
- If the coordinator reports degraded baseline or read-only branch state, treat the control view as diagnostic and resolve the underlying shared-workspace condition through the coordinator workflow.
- If the top bar reports a disconnected event stream or a cursor gap, leave the page open. The client discards partial state and loads a fresh coherent snapshot before reconnecting.
- A migration or SQLite-integrity failure writes only a sanitized `startup-failure.json`; no bearer token, SQL text, database path, or exception detail is published. The tray treats this as `fatal_integrity_error`, disables restart/termination guesses, and leaves repair to an offline operator.
- Closing the browser, Zircon Hub, or tray never stops the daemon. Exiting the tray releases only its repository mutex.

## Windows Tray Supervision

The independent Tauri tray under `tools/session_tray` verifies the repository key, runtime descriptor version, PID creation time, executable, command line, daemon instance, schema/API versions, and authenticated health before enabling operations. Stale descriptors and PID reuse never authorize termination.

Tray states are strict enums: starting, healthy, busy, degraded, draining, stopping, offline, recovering, read-only, identity mismatch, and fatal integrity error. The icon, tooltip and menu are derived from the same enum and last verified identity. Drain, resume, stop, restart, and force-stop are cataloged actions. The first click stores the server preview; only a second click confirms it. Schema v26 and the service preflight jointly enforce at most one accepted/draining stop, restart or force-stop intent per repository. Before installing that unique index, v26 atomically fails every intent/action in a historical multi-active conflict and records `schema.lifecycle_conflict_repaired`; it never guesses which old command should continue. A confirmed lifecycle that is still draining exposes a separate cancel command in both the tray and web Action Activity. Cancellation uses the same serialization gate as Confirm and atomically marks the lifecycle intent and action cancelled, writes audit evidence, and restores supervision to healthy. Ordinary Resume uses that same durable cancellation path when a reversible drain exists; if the associated action is already terminal, Resume atomically fails the orphan instead. Activation failures compensate the new intent/action and release `draining`, while daemon startup fails accepted/draining reversible intents owned by an older daemon instance before accepting new lifecycle work. After the worker enters stopping, cancellation and Resume fail closed as no longer cancellable.

Force-stop is Maintainer-only. It waits for the server-side action and durable intent to reach `succeeded`; failed, cancelled or timed-out actions never call `TerminateProcess`. The daemon atomically commits `stopping`, terminal intent/action proof and `offline`, then keeps its authenticated HTTP transport open. The tray reads both proofs, sends a bearer-authenticated acknowledgement bound to the Action ID, and only then permits transport shutdown. The acknowledgement handoff is single-flight and repeated acknowledgements are idempotent. Both its callback and the unacknowledged 30-second fallback enter the same bounded-backoff shutdown retry loop; scheduling failure preserves the earlier fallback, and post-commit callback or audit failure never rewrites terminal proof or stops retries. The tray waits two seconds for normal process exit; a still-live process must pass the descriptor/process identity check again immediately before termination.

Unexpected exits use bounded 1/2/5/15/30-second recovery. Five failures within ten minutes open the circuit; ten uninterrupted `healthy` minutes clear it. The tray persists its guard, failure window, unconsumed retry deadline, circuit and explicit request flags in a two-generation journal under `.codex/state/session-coordinator`; a tray restart waits the remaining deadline and does not count the same outage twice. Each generation must also satisfy semantic invariants for ordered/bounded failures, retry/count coupling, circuit/opened-at coupling, verified prior guard and mutually exclusive explicit requests. A parseable but incoherent active generation falls back to the previous generation; two invalid generations fail tray startup closed. On launch and after every local recovery-state change, the full projection is written through the authenticated coordinator command into `service_recovery_state` with supervision audit events; non-writable service states retain the pending synchronization until safe. Tray restart therefore cannot silently clear the circuit or leave retry/healthy deadlines stale. Explicit stop, migration/integrity failure, identity mismatch, maintenance hold, read-only state, a valid competing instance, or an unverified first offline observation suppresses recovery and never advances the healthy-reset clock. A server-confirmed explicit Restart may cross only an otherwise unprotected `stopping` guard; every later fatal, mismatch, read-only, maintenance, competing-instance or explicit-stop observation invalidates that request before any retry.

Startup-item query/install/update/remove are explicit tray commands. Coordinator and tray commands produce separate bounded structured results with attempted/success/exit/stdout/stderr fields; Query always captures both halves, so an installed tray cannot hide a missing coordinator task. The combined result is written to the repository-local state directory and surfaced in the native notification and diagnostics JSON, never to Git and never with credentials. For mutating actions, a coordinator failure prevents the tray half from being applied and records it as skipped. Operation failures remain visible until a later successful command clears them.

### Production Packaging and Current-User Install

The production tray is bundled as an unsigned local NSIS installer from `tools/session_tray/tauri.conf.json`. Build it through a coordinator Cargo job with a complete release compatibility identity and set `CARGO_TARGET_DIR` to the returned managed target. Use a matching Tauri 2.11 CLI transiently; do not install `cargo-tauri` globally. The npm CLI cache belongs inside the managed target and is deleted after packaging, while the compatible Rust release pool remains reusable.

The installer writes only to `%LOCALAPPDATA%\Zircon Session Coordinator` and registers uninstall metadata only below `HKCU`. Launch the installed executable with an explicit canonical repository:

```powershell
& "$env:LOCALAPPDATA\Zircon Session Coordinator\zircon-session-tray.exe" `
  --repo-root E:\Git\ZirconEngine
```

The repository argument is required when the current directory is not inside the checkout. Each repository derives a different mutex and runtime identity from its canonical path hash, so two repositories cannot share a tray authority. A same-product NSIS upgrade replaces the current-user installation in place and preserves the explicit repository boundary. Silent uninstall uses the registered `uninstall.exe /S`; it must remove the HKCU uninstall key, install directory, shortcuts and tray process without stopping the coordinator daemon. Installer, executable and web-distribution audits compare actual runtime/environment secrets by value; matching generic protocol words such as `token` is not itself a leak.

## Cargo Build Artifact Lifetime

Coordinator-managed Cargo jobs reuse a target pool only when repository, platform, Rust toolchain, target architecture, workspace, build configuration, feature/flag set, and profile produce the same compatibility key. One compatibility pool permits one active writer. Source and `Cargo.lock` changes remain inside the same pool because Cargo owns incremental invalidation.

Missing compatibility evidence fails closed to an ephemeral target. Releasing an ephemeral job schedules immediate cleanup and revalidates its cleanup reservation, process state, file leases, and managed drive-root path before deletion. A failed deletion is persisted and retried by daemon maintenance. Cargo output is never allowed below the repository or outside the configured `D:`, `E:`, or `F:` drive-root `cargo-targets`, `targets`, or `ZirconBuilds` roots.

The daemon keeps only one authoritative idle directory per compatibility key. Schema v23 demotes historical `retained` rows that lack any part of the compatibility identity to ephemeral cleanup. If a new compatible pool has already adopted the same directory, cleanup closes only the superseded historical record and preserves the physical directory. Historical duplicate pools are otherwise demoted to `delete_on_release`; missing directories are marked deleted before a deterministic replacement is created. When a managed drive falls below the configured free-space reserve, the daemon evicts idle reusable pools from least recently used to newest until the reserve is restored. Active writers, live recorded processes, overlapping leases, cleanup reservations, Windows/WSL-incompatible pools and paths outside the allowlist are never eviction candidates. Cleanup runs outside the SQLite writer transaction so deleting a large target does not freeze Session coordination.

The **验证** page projects this lifecycle without gaining cleanup authority. Its summary counts unique reusable compatibility keys plus ephemeral, pending-cleanup and failed-cleanup job records from the current bounded snapshot. The Cargo table exposes Session ownership, retention policy, compatibility identity, reuse source, cleanup state and sanitized cleanup error. These values are not a disk scan and the browser cannot delete a target or change its policy; every physical cleanup remains a coordinator-owned, path-validated operation.

Control-plane payloads are bounded independently of the SQLite file size. A baseline emits `baseline.degraded` only when it transitions from healthy to degraded; the event stores the total path count plus a bounded sample instead of every changed path. Every snapshot, log-range and SSE boundary replaces any legacy event payload above 16 KiB with a size/reason marker, and schema v24 compacts those historical oversized payloads during a controlled service upgrade. Schema v25 then checkpoints WAL and performs the one-time physical SQLite compaction before the service opens HTTP; it writes the v25 marker only after VACUUM succeeds, so an exception, crash or power loss leaves the upgrade retryable. Snapshot collaboration/validation projections expose byte counts and status summaries for baseline manifests, delayed-Patch object maps and validation-copy manifests rather than sending their internal content to the browser. This keeps the console responsive and prevents a large shared-main worktree from turning audit history into an unbounded response.

## M6 Load and Soak Acceptance

Routine validation uses the deterministic `Quick` profile in a temporary database and artifact root: 40 Sessions, 20 workflows, 500 nodes/attempts, 5,000 events, 500 artifacts, eight SSE clients, and a sparse 16 MiB log. It preserves the same coherence, capacity, bounded-range and P95 assertions while keeping the developer feedback loop short. The original release-scale profile (200 Sessions, 100 workflows, 5,000 nodes/attempts, 100,000 events, 10,000 artifacts and a sparse 500 MiB log) remains explicit opt-in rather than a routine gate.

Use the wrapper so every run retains a complete transcript outside Git for later diagnosis:

```powershell
.\tools\session_coordinator\run-control-validation.ps1 -Profile Quick -Suite H4
.\tools\session_coordinator\run-control-validation.ps1 -Profile Release -Suite H4
```

Logs are written below `%LOCALAPPDATA%\ZirconEngine\SessionCoordinator\validation`. The wrapper prints the exact file after success or failure. `Quick` is the default for milestone development; `Release` is reserved for an explicit release/load investigation and does not replace the source-frozen soak.

The soak entry point always writes raw samples outside Git:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass `
  -File tools/tests/workflow-control-center-soak.ps1 `
  -Hours 24 `
  -IntervalSeconds 60
```

It creates an isolated `main` repository and state root below `%LOCALAPPDATA%/Zircon Session Coordinator/soak-runs/<stamp>`, outside both Git and volatile system Temp maintenance. The harness records health/snapshot/resource samples, injects SSE disconnects and maintenance ticks, performs one controlled restart with successor recovery, verifies event replay continuity, and enforces bounded RSS/handle growth. A failure always atomically writes the external JSON report and retains the workspace for diagnosis; a successful run waits for HTTP/SSE shutdown, removes Git read-only objects safely, deletes the workspace, and records `workspaceRetained=false`. Only a reviewed sanitized summary may later be copied into the numbered plan output directory.

## Production Asset Policy

The web package lives at `tools/session_coordinator/web` and is independent from Zircon Hub runtime behavior. It imports only Hub visual tokens, the MUI theme and the generic `HubPanel`/`HubButton` components. It does not import Hub API calls, Tauri bindings, project DTOs or Hub persistence.

`npm run check` performs strict type checking, Node component/model tests, a production Vite build and a recursive distribution-graph audit. Route pages load on demand; React, MUI/Emotion and remaining vendor code are bounded content-hashed chunks. Production source maps, absolute development URLs, credential/capability names, webhook material, unhashed assets, missing transitive imports and unreachable output files fail that audit. `index.html` is served with `no-store`; content-hashed JavaScript and CSS use one-year immutable caching.

## Validation State

M1-M3 unit and integration coverage is defined in the files listed in this document header. Accepted milestone evidence is recorded under the owning numbered plan directory.
