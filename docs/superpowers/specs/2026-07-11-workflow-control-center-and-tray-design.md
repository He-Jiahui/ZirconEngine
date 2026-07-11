# Zircon Workflow Control Center and Windows Tray Design

**Date:** 2026-07-11
**Status:** Review requested
**Scope:** Local Jenkins-style workflow visualization, controlled web operations, and Windows tray supervision for the Zircon Session Coordinator.

## 1. Goals

Build a local workflow control plane that makes concurrent ZirconEngine development observable and safely operable without introducing another scheduling authority.

The delivered system must:

- visualize Goal, Milestone, Slice, Validation, Review, Failure, Commit, and notification progress;
- expose Session, lease, delayed patch, Cargo lane, validation-copy, Git mutex, baseline, Failure graph, and audit state;
- provide controlled operations through typed actions rather than arbitrary shell, Git, Cargo, SQL, or file commands;
- keep the Session Coordinator alive when the browser, Zircon Hub, or tray UI is closed;
- add a Windows tray supervisor for health, startup, draining, stop, restart, recovery, and notifications;
- preserve the shared-`main`, no-worktree, service-owned commit and current-hash attribution rules;
- remain local-only in the first release.

## 2. Non-goals

- No LAN or internet access in the first release.
- No arbitrary pipeline language or user-supplied shell executor.
- No second workflow database or Node backend.
- No direct browser access to SQLite or `.codex/state`.
- No LocalSystem Windows Service in the first release.
- No coupling to the `zircon_app -> zircon_runtime -> zircon_editor` engine runtime chain.
- No replacement for the existing plan, Failure, lease, Cargo, Git finalize, and cleanup domain services.

## 3. Selected Architecture

The Session Coordinator remains the only scheduling and mutation authority. It gains a loopback web control plane, workflow projections, controlled actions, SSE events, and static UI hosting. A separate lightweight Tauri tray supervises the process but does not own workflow state.

```text
Windows login
   |
   +-- Session Coordinator daemon
   |    +-- Session / lease / Failure / Git / Cargo domains
   |    +-- SQLite authority
   |    +-- 127.0.0.1 control API
   |    +-- SSE event stream
   |    +-- React static console
   |
   +-- Zircon Workflow Tray
        +-- process identity verification
        +-- start / drain / stop / restart
        +-- open browser console or Zircon Hub
        +-- native notifications
```

Ownership is fixed:

- `tools/session_coordinator`: authoritative daemon and domain services;
- `tools/session_coordinator/control_plane`: HTTP transport, authentication, projections, and action protocol;
- `tools/session_coordinator/workflows`: workflow topology, state machine, attempts, and artifacts;
- `tools/session_coordinator/supervision`: drain, shutdown, and recovery contracts;
- `tools/session_coordinator/web`: React/Vite web console;
- `tools/session_tray`: independent Tauri 2 tray supervisor;
- `zircon_hub`: optional entry point that opens or embeds the same console, never a second authority.

The tray and daemon register independently at user login. Exiting the tray never stops the daemon. Closing the browser or Hub never stops either process.

## 4. Alternatives Considered

### Hub-centric control plane

Putting the dashboard and process lifecycle entirely in Zircon Hub maximizes immediate UI reuse, but makes control availability depend on the Hub process. It also encourages Hub to become a second workflow owner. Rejected as the primary architecture.

### Separate Node workflow server

A Node service would simplify some web tooling, but duplicates authentication, lifecycle, persistence, and scheduling around the existing Python coordinator. Rejected because it creates two control planes and new race boundaries.

### Selected: daemon-native control plane plus thin tray

This keeps one authority, leaves the browser optional, supports process supervision, and permits Hub integration without transferring ownership.

## 5. Workflow Model

### 5.1 Hierarchy

```text
WorkflowRun (one Session Goal execution)
+-- MilestoneRun M1
|   +-- SliceRun M1.1
|   +-- SliceRun M1.2
|   +-- ValidationGate M1-T
|   +-- ReviewGate
|   +-- CommitAttempt
|   +-- NotificationAttempt
+-- MilestoneRun M2
+-- GoalCloseout
```

Definitions:

- `WorkflowRun`: one registered Goal execution;
- `MilestoneRun`: an accepted milestone boundary;
- `SliceRun`: one implementation slice assigned to a Session;
- `ValidationGate`: build, test, static-analysis, or visual acceptance;
- `ReviewGate`: independent Critical/Important review result;
- `CommitAttempt`: service-owned atomic commit evidence;
- `NotificationAttempt`: one post-commit delivery attempt;
- `FailureDependency`: an applicable origin/fixer Failure edge;
- `ArtifactReference`: immutable reference to a plan record, log, report, commit, or validation copy.

Markdown plans define intended topology. SQLite owns live execution state. The UI never infers current status by scanning every historical Markdown artifact.

### 5.2 Enumerated states

Workflow states:

```text
registered, active, waiting_dependency, waiting_lease,
resolving_failure, waiting_validation, waiting_review,
finalizing, succeeded, failed, cancelled, stale, archived
```

Node states:

```text
pending, ready, running, waiting_external,
succeeded, failed, cancelled, skipped
```

Supervision states:

```text
starting, healthy, degraded, draining, stopping,
offline, recovering, read_only, identity_mismatch,
fatal_integrity_error
```

Free-form state values are forbidden. Human explanations belong in `status_reason`, `diagnostic_code`, `diagnostic_details`, and `last_transition_at`.

### 5.3 Current state versus history

Every retry creates an immutable Attempt. The latest accepted Attempt supplies current state; previous failed or pending attempts stay visible in history but cannot poison a later success.

```text
M2-T
+-- Attempt 1: failed
+-- Attempt 2: failed
+-- Attempt 3: succeeded  <- current
```

Foreign Failure diagnostics are reported but do not block the current plan. Only Failures related through the current plan, numbered child directory, or origin/fixer record are applicable.

### 5.4 Standard execution path

```text
Preflight
  -> acquire leases
  -> implementation slices
  -> validation
  -> independent review
  -> Failure graph acceptance
  -> atomic Git commit
  -> one notification attempt
  -> milestone succeeded
```

All implementation slices must succeed before validation. Validation or review failure returns execution to the lowest broken shared layer. Milestone success keeps the Session active. Goal completion requires all milestones to succeed.

## 6. Persistence

Add these tables through monotonic migrations:

- `workflow_runs`
- `workflow_nodes`
- `workflow_edges`
- `workflow_attempts`
- `workflow_artifacts`
- `workflow_diagnostics`
- `action_requests`
- `action_approvals`
- `notification_attempts`
- `service_supervision_events`
- `web_control_sessions`

Reuse existing authoritative tables for Sessions, leases, patches, Failure nodes, Cargo jobs, finalize requests, events, and baseline epochs. Workflow tables reference domain IDs and do not copy their authority.

Session plan registration imports a versioned topology after validating duplicate IDs, cycles, missing dependencies, numbered-plan ownership, and child directory placement. The plan content hash pins the topology version. Later changes create a reviewed topology diff and cannot silently rewrite a running workflow.

Large logs and reports live under:

```text
.codex/state/session-coordinator/artifacts/
+-- logs/
+-- validation/
+-- actions/
+-- diagnostics/
```

SQLite stores only metadata, summaries, hashes, state, and artifact references.

## 7. Web Security

### 7.1 Network boundary

The first release binds only to `127.0.0.1`. Host and Origin must resolve to loopback. LAN access is not supported. API responses use `Cache-Control: no-store`; the UI uses a strict CSP with no external scripts or resources.

### 7.2 Browser bootstrap

The browser opens through `zircon-session ui open` or the tray:

1. CLI or tray authenticates with the local runtime token.
2. The daemon creates a single-use ticket valid for 30 seconds.
3. The browser visits `/ui/bootstrap/<ticket>`.
4. The service consumes the ticket and issues an `HttpOnly`, `SameSite=Strict` cookie.
5. The browser is redirected to a URL containing no credential.

The browser never receives or stores the runtime bearer token, maintenance capability, or webhook. Control sessions bind to the current daemon instance and expire on restart or inactivity. Mutations require CSRF protection.

### 7.3 Permission levels

| Level | Capabilities |
|---|---|
| Observer | Read state, logs, graphs, and history |
| Operator | Validation, own leases, safe patches, drain preview |
| Committer | One scoped, accepted milestone commit |
| Maintainer | Cleanup, retention, archive, recovery, startup changes |

Browser sessions default to Observer. Operator access is short-lived and issued by CLI or tray. Committer access binds to a Session and manifest. Maintainer elevation requires the separate local maintenance capability and cannot be self-issued by the browser.

## 8. Controlled Actions

The web UI can invoke only a fixed Action Catalog. There is no arbitrary command endpoint.

Read-only actions include service snapshots, workflow/session queries, Failure graphs, leases, patches, Cargo jobs, events, and audit queries.

Controlled mutations include Session heartbeat/activation, own-lease claim/release, safe patch processing, validation start/cancel, Failure refresh, plan topology refresh, milestone commit, Session completion, service drain/restart, reviewed cleanup/retention, archive, and startup registration.

Explicitly forbidden surfaces:

- arbitrary shell;
- arbitrary Cargo arguments;
- arbitrary Git commands;
- arbitrary SQL;
- arbitrary file writes or deletion;
- browser-supplied filesystem paths;
- browser-supplied webhook content.

### 8.1 Two-phase action protocol

Every mutation uses `Preview -> Confirm -> Execute`.

Preview persists an expiring action ID, impact set, permission, warnings, state hash, and confirmation phrase. Confirm supplies the action ID, CSRF token, confirmation phrase, and reason. The daemon recomputes current state. Changes to HEAD, index, lease, Failure, plan, Cargo, Session, or target files invalidate the preview.

Risk classes:

- green: read-only;
- yellow: lease, validation, safe patch, Session transition, drain;
- red: commit, stop/restart, cleanup, retention, archive, startup mutation.

Red actions require impact display, typed confirmation, reason, immutable audit event, and no automatic retry.

## 9. Control API

All endpoints live under `/control/v1`.

Read endpoints cover snapshot, workflows, graphs, attempts, Sessions, Failures, leases, patches, Cargo jobs, finalize requests, events, audit, and service state. Lists use bounded cursor pagination and allowlisted filters; SQL-like expressions are forbidden.

Action endpoints:

```text
POST /control/v1/actions/preview
POST /control/v1/actions/{action_id}/confirm
POST /control/v1/actions/{action_id}/cancel
GET  /control/v1/actions/{action_id}
```

Errors have stable codes, correlation IDs, retryability metadata, and sanitized details. Python tracebacks, secrets, environment variables, and internal capabilities never reach the client.

### 9.1 Snapshot and events

`GET /control/v1/snapshot` runs in one read-only transaction and returns a projection version plus event cursor. The browser applies the snapshot, then connects to `/control/v1/events/stream` from that cursor.

SSE supports `Last-Event-ID`, 15-second heartbeats, bounded client queues, at most eight local clients, and `resync_required` for slow or expired consumers. Large log content is never sent through SSE.

## 10. Web Console

The React/Vite console reuses Zircon Hub MUI tokens and components but is independently built and served by the daemon.

Primary pages:

- Overview
- Workflows
- Sessions
- Failure Graph
- File Collaboration
- Build and Validation
- Git Milestones
- Event Audit
- Service Management
- Settings

The top bar always shows coordinator health, branch/read-only state, baseline, active Session count, running tasks, alerts, and SSE state.

### 10.1 Jenkins-style pipeline

The Workflow page renders dependency-aware stages for Preflight, Implementation, Validation, Review, Commit, and Notification. Color is supplemented by icon and text. Nodes show status, duration, Session, attempt, evidence, block reason, and allowed actions.

The node detail drawer shows dependencies, plan links, leases, commands, exit codes, review, applicable Failures, commit scope, notification result, artifacts, and the entire attempt timeline. Failed nodes lead with the lowest shared-layer diagnosis.

### 10.2 Logs and data scale

Logs support stream selection, pause/follow, search, timestamp and level filters, virtual scrolling, range download, truncation status, and structured diagnostics. Log text is never rendered as HTML.

Large graphs use hierarchy, collapsing, filtering, and viewport rendering rather than creating one DOM node per workflow node.

### 10.3 Domain views

- Failure Graph distinguishes applicable, foreign, invalid, and fixed records.
- File Collaboration displays lease owner, expiry, attribution, baseline/index/worktree state, delayed patches, and conflicts.
- Build and Validation displays managed Cargo lanes, pinned validation copies, PID, duration, exit code, resource summary, and logs.
- Git Milestones displays classified scope, staged/attributed blob comparison, plan evidence, Failure and review gates, commit message, shortstat, final SHA, baseline epoch, and notification result.
- Event Audit makes every Action ID, actor, parameter summary, result, and timestamp traceable.

The default language is Chinese while persisted enum values remain English. Keyboard navigation, focus control, screen-reader labels, and non-color status indicators are required.

## 11. Static UI Deployment

Production build output resides in `tools/session_coordinator/web/dist` with a hashed asset manifest. CI/type gates run TypeScript checking, component tests, production build, secret/source-map scan, and manifest verification.

`index.html` is no-cache; hashed assets are immutable. API routes never fall back to SPA HTML. Directory browsing is disabled. Artifact downloads validate ownership by ID, prevent path traversal and MIME sniffing, and respect retention references.

## 12. Windows Tray Supervisor

`tools/session_tray` is a Tauri 2 application with one named mutex per repository-path hash. It validates daemon identity using runtime repository, PID, command line, authenticated health, instance ID, and process creation time. A stale runtime file never grants permission to terminate a PID.

Tray states are healthy, busy, degraded, draining, read-only, offline, recovering, identity mismatch, and fatal integrity error. The icon, tooltip, menu, and notifications reflect those states.

Menu operations include opening the workflow console or Hub, starting, draining and stopping, restarting, pausing/resuming new work, maintenance preview, logs, diagnostic copy, startup registration, and tray exit. Invalid operations are disabled by state.

Exiting the tray does not stop the daemon. Stop/restart first drains new writes, waits for Git/Cargo/patch critical sections, writes final supervision evidence, requests graceful shutdown, verifies process exit, and then removes owned runtime state. Force termination is a Maintainer-only advanced recovery action and only applies to a fully verified process identity.

### 12.1 Recovery

Unexpected exit backoff is 1, 2, 5, 15, and 30 seconds. Five failed restarts within ten minutes stop automatic recovery. Stable operation for ten minutes clears the count.

Automatic restart is forbidden for explicit user stop, migration failure, identity mismatch, valid competing instance, fatal integrity error, maintenance-held offline state, or exhausted recovery attempts.

### 12.2 Startup model

The coordinator remains a current-user scheduled task. The tray uses current-user login startup. Both use absolute repository paths and separate path-hash identities. No token, webhook, or maintenance capability is stored in startup definitions.

The first release does not use LocalSystem because Git credentials, user directories, Cargo environment, browser sessions, and interactive notification ownership are user-scoped. A future unattended worker would be a separate product boundary.

## 13. Service Drain and Recovery

Normal stop transitions `healthy -> draining -> stopping -> offline`.

Draining rejects new write work while allowing reads and existing critical sections to complete. The UI displays remaining Git mutex, Cargo, patch, and maintenance operations. Timeout allows cancel or an advanced force-stop preview; it never silently kills the process.

Schema migration failure starts diagnostic read-only mode. The web UI retains health, diagnostics, and log download but disables mutations. Migrations are monotonic and transactional; the browser cannot execute repair SQL.

## 14. Performance Targets

The first release supports 200 historical/concurrent Sessions, 100 active workflows, 5,000 nodes, 100,000 retained events, 10,000 artifact records, eight SSE clients, and 500 MB range-readable log files.

Targets:

- health P95 under 100 ms;
- consistent snapshot P95 under 800 ms;
- ordinary list query P95 under 300 ms;
- event display latency P95 under 500 ms;
- non-external action preview under one second;
- production UI interactive under two seconds locally.

The control plane must not materially delay watcher, lease, Git mutex, Cargo, or maintenance operations.

## 15. Validation Strategy

Backend tests cover state machines, topology import, attempts, action hashes, permissions, Failure scoping, draining, artifacts, API schemas, authentication, CSRF, version negotiation, SSE replay, concurrency, and recovery.

Security tests cover ticket replay, cookie/CSRF abuse, malicious Host/Origin, non-loopback connections, path traversal, artifact enumeration, XSS logs, SSE session isolation, forged action IDs, state changes after preview, stale PID substitution, and secret leakage.

Frontend tests cover workflow graphs, history, confirmation, SSE resync, large-list virtualization, accessibility, read-only/degraded behavior, and API errors.

Tray tests cover named mutex, identity validation, stale runtime, lifecycle operations, backoff, explicit-stop behavior, startup registration, exit semantics, and multi-repository isolation.

End-to-end tests use temporary Git repositories, databases, ports, target roots, and process fixtures. Real shared-main state is never used for mutation tests.

## 16. Milestones

### M1: Control-plane foundation

Add workflow/event schema, projections, loopback `/control/v1`, snapshot, SSE replay, and contract tests. Exit gate: existing CLI behavior remains unchanged and the control plane is read-only.

### M2: Read-only web console

Add React/Vite, overview, workflow graph, Session, Failure, lease, Cargo, logs, and audit pages. Exit gate: all coordinator state is observable with no mutation surface.

### M3: Controlled actions

Add bootstrap tickets, Cookie/CSRF, permissions, Action Catalog, Preview/Confirm, audit, and yellow operations. Exit gate: no catalog-external or repository-external operation is possible.

### M4: Milestone and Goal management

Add plan topology, Workflow/Node/Attempt projections, validation/review/Failure gates, milestone commit, and notification result. Exit gate: a complete milestone can be supervised and safely advanced from the UI.

### M5: Windows tray supervision

Add Tauri tray, identity verification, lifecycle operations, drain, recovery, notifications, and startup management. Exit gate: tray exit leaves the daemon alive and no stale PID can be killed.

### M6: Stabilization and release

Add security, load, 24-hour soak, crash recovery, accessibility, packaging, Hub entry, operational guide, and failure guide. Exit gate: production web and real Windows desktop acceptance pass.

Every accepted milestone produces a normal Conventional Commit. Business work commits through the coordinator atomic path. Workflow infrastructure and documentation commit normally. Every successful commit triggers one four-line WeCom notification without automatic retry. No branch, worktree, checkpoint commit, Session tag, or repo-local Cargo target is allowed.

## 17. Acceptance Criteria

The objective is complete only when evidence proves all of the following:

- the web UI provides real-time Workflow, Session, Failure, lease, Cargo, Git, artifact, and audit visualization;
- every mutation is cataloged, permissioned, previewed, confirmed, state-revalidated, and audited;
- browser sessions never receive long-lived service secrets;
- dangerous operations cannot automatically retry;
- the tray safely supervises the daemon and cannot kill unrelated processes;
- the daemon survives browser, Hub, and tray closure;
- current state is independent from historical failed attempts;
- foreign Failure diagnostics do not block unrelated workflows;
- atomic commit never absorbs another Session's files;
- startup, drain, stop, restart, recovery, migration failure, and upgrade paths have verified evidence;
- production web, Windows tray, and 24-hour service operation pass acceptance.

## 18. Deliberate Architectural Boundary

This control center is repository developer tooling. It does not become a fourth engine root package, does not participate in runtime ECS/editor authority, and does not bypass the fixed `zircon_app`/`zircon_runtime`/`zircon_editor` architecture. Zircon Hub may consume the control facade, but it cannot own coordinator domain objects or write coordinator persistence directly.
