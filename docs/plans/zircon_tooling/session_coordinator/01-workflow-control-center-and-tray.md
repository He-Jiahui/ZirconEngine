# Zircon Workflow Control Center and Windows Tray Implementation Plan

## Cross-Plan Failure Status

- Open: [Cargo PID reuse identity guard](01/failure-2026-07-14-cargo-pid-reuse-identity-guard.md) is the current failure-priority repair for Editor Layout 15 managed screenshot validation. It must preserve live Cargo descendant protection while rejecting a reused root PID with a different creation identity.
- fixed 已修复：[failure-return-plan-table-row-corruption](../../zircon_editor/editor/07/fixed-2026-07-15-failure-return-plan-table-row-corruption.md)
- fixed 已修复：[plan-output-audit-counts-lifecycle-links](../../zircon_editor/editor/12/fixed-2026-07-15-plan-output-audit-counts-lifecycle-links.md)
- Open: [Goal closeout counts terminal failed commit intents](01/failure-2026-07-15-goal-closeout-counts-terminal-failed-intents.md) must keep terminal failed attempts as immutable history without blocking an otherwise accepted Goal closeout.
- fixed 已修复：[stale-session-pending-cpu-reservation-starvation](../../zircon_editor/editor/07/fixed-2026-07-16-stale-session-pending-cpu-reservation-starvation.md)
- fixed 已修复：[milestone-finalize-session-relative-owned-scope](../../zircon_runtime/text/01/fixed-2026-07-15-milestone-finalize-session-relative-owned-scope.md)
- fixed 已修复：[milestone-finalize-per-path-blob-verification-stall](../../zircon_runtime/frameworks/05/fixed-2026-07-15-milestone-finalize-per-path-blob-verification-stall.md)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a local Jenkins-style workflow console and a Windows tray supervisor that visualize the authoritative Session Coordinator and expose only typed, permissioned, previewed, confirmed, and audited operations.

**Architecture:** The existing Python Session Coordinator remains the sole workflow and mutation authority. It gains a modular `/control/v1` control plane, versioned workflow projections, an independently built React console, and explicit supervision contracts; a separate Tauri tray authenticates to that facade and never writes SQLite or repository state directly.

**Tech Stack:** Python 3 standard library (`http.server`, `sqlite3`, `secrets`, `hashlib`, `unittest`), SQLite WAL, React 19.2.7, TypeScript 6.0.3, Vite 8.0.16, MUI 9.0.1, Tauri 2.11.2, Rust 2021, PowerShell, Windows user startup facilities, Git CLI, and service-managed Cargo/validation-copy lanes.

**Approved design:** `docs/superpowers/specs/2026-07-11-workflow-control-center-and-tray-design.md`

**Execution boundary:** Work directly on shared `main`; do not create a branch, worktree, stash checkpoint, hidden intermediate Git commit, or repository-local Cargo target. M1 and M2 execute serially. After M2, concurrent Sessions may work only on the disjoint scopes named in §3 and must claim concrete files through `tools/zircon-session.ps1` before editing.

---

## 1. Dependency Order

```text
existing Session Coordinator + file leases
  -> M1 control-plane contracts and read-only browser authentication
  -> M2 read-only Jenkins-style console and static hosting
  -> M3 controlled actions and elevated permissions
  -> M4 workflow topology, gates, commit, Goal closeout, notification
  -> M5 Windows tray supervision and recovery
  -> M6 security, load, soak, packaging, Hub entry, release acceptance
```

M5 identity/start/open-console work may proceed in parallel with M3 after M2. M5 drain/stop/restart integration waits for the M3 action protocol. M4 depends on M3. M6 acceptance begins only after M3, M4, and M5 have individually passed their testing stages.

## 2. Machine-Readable Workflow Topology

The coordinator imports this fenced JSON as schema version 1. Human prose remains authoritative for implementation detail; the JSON fixes node identity and dependency edges.

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-workflow-control-center-and-tray",
  "goal": "Local workflow visualization and Windows tray supervision",
  "milestones": [
    {"id": "M1", "title": "Control-plane foundation", "depends_on": []},
    {"id": "M2", "title": "Read-only web console", "depends_on": ["M1"]},
    {"id": "M3", "title": "Controlled actions", "depends_on": ["M2"]},
    {"id": "M4", "title": "Milestone and Goal management", "depends_on": ["M3"]},
    {"id": "M5", "title": "Windows tray supervision", "depends_on": ["M2", "M3"]},
    {"id": "M6", "title": "Stabilization and release", "depends_on": ["M4", "M5"]}
  ]
}
```

## 3. File Ownership and Module Map

### Coordinator core and control plane

- Modify `tools/session_coordinator/migrations.py`: monotonic schema versions 14-17 only; never rewrite migrations 1-13.
- Modify `tools/session_coordinator/models.py`: shared workflow, node, action-risk, permission, and supervision enums.
- Modify `tools/session_coordinator/server.py`: retain composition, daemon lifetime, runtime descriptor, and legacy `/health`/`/command`; delegate control requests instead of adding route logic to this already-large file.
- Modify `tools/session_coordinator/client.py` and `tools/session_coordinator/cli.py`: UI bootstrap/open and supervised lifecycle commands.
- Create `tools/session_coordinator/workflows/`: topology import, immutable attempts, persistence, projections, and gate evaluation.
- Create `tools/session_coordinator/control_plane/`: contracts, auth, cookies/CSRF, snapshot, events, HTTP routing, action catalog, preview/confirm, and static assets.
- Create `tools/session_coordinator/supervision/`: drain state, critical-section inventory, shutdown/restart intent, and recovery evidence.
- Create focused tests under `tools/session_coordinator/tests/`; no new control-plane test logic goes into the general `test_server.py` file.

### Web console

- Create `tools/session_coordinator/web/`: independent React/Vite source, package lock, tests, and production `dist`.
- Import visual-only tokens and generic primitives from `zircon_hub/web/src/theme` and selected generic components. Never import Hub state, Tauri APIs, actions, project models, or persistence.
- Create typed domain pages for Overview, Workflows, Sessions, Failure Graph, File Collaboration, Build and Validation, Git Milestones, Event Audit, Service Management, and Settings.

### Tray

- Create `tools/session_tray/`: independent Tauri application and Cargo workspace, with modules for runtime descriptor parsing, process identity, coordinator client, lifecycle, recovery, startup, menu, and notifications.
- Keep `tools/session_tray` out of the root Cargo workspace so it does not become another engine root package. Its builds must use a coordinator-managed drive-root target lane.

### Integration, docs, and acceptance

- Modify `tools/install-session-coordinator-task.ps1`: install/query/remove coordinator and tray user-startup entries without embedding capabilities.
- Modify `zircon_hub/src/tauri_app/commands.rs` and the minimum matching Hub UI files only in M6: open the same browser console through a coordinator-issued ticket; do not embed workflow authority.
- Modify `docs/cli-and-tooling/local-session-coordinator.md` and create `docs/cli-and-tooling/workflow-control-center.md`.
- Create `tests/acceptance/workflow-control-center-and-tray.md`, `tools/tests/workflow-control-center-smoke.Tests.ps1`, and `tools/tests/workflow-control-center-soak.ps1`.

### Safe parallel scopes after M2

| Lane | Exclusive scope | May begin | Dependency stop |
|---|---|---|---|
| A | `tools/session_coordinator/control_plane/actions*`, `workflows/`, action tests | M2 accepted | M4 waits for M3 |
| B | `tools/session_tray/`, tray tests/assets | M2 accepted | lifecycle mutation waits for M3 contracts |
| C | load/security fixtures, operator docs, non-mutating acceptance harness | M2 accepted | final evidence waits for M4 and M5 |

No lane may edit `migrations.py`, `models.py`, `server.py`, `cli.py`, the plan definition, or shared generated web artifacts without a dedicated lease and an agreed sequencing point.

## 4. Cross-Cutting Contracts

### 4.1 Enumerated state

The persisted values are exact and free-form status values are rejected:

```python
class WorkflowState(str, Enum):
    REGISTERED = "registered"
    ACTIVE = "active"
    WAITING_DEPENDENCY = "waiting_dependency"
    WAITING_LEASE = "waiting_lease"
    RESOLVING_FAILURE = "resolving_failure"
    WAITING_VALIDATION = "waiting_validation"
    WAITING_REVIEW = "waiting_review"
    FINALIZING = "finalizing"
    SUCCEEDED = "succeeded"
    FAILED = "failed"
    CANCELLED = "cancelled"
    STALE = "stale"
    ARCHIVED = "archived"

class WorkflowNodeState(str, Enum):
    PENDING = "pending"
    READY = "ready"
    RUNNING = "running"
    WAITING_EXTERNAL = "waiting_external"
    SUCCEEDED = "succeeded"
    FAILED = "failed"
    CANCELLED = "cancelled"
    SKIPPED = "skipped"

class SupervisionState(str, Enum):
    STARTING = "starting"
    HEALTHY = "healthy"
    DEGRADED = "degraded"
    DRAINING = "draining"
    STOPPING = "stopping"
    OFFLINE = "offline"
    RECOVERING = "recovering"
    READ_ONLY = "read_only"
    IDENTITY_MISMATCH = "identity_mismatch"
    FATAL_INTEGRITY_ERROR = "fatal_integrity_error"
```

### 4.2 API envelope

Every `/control/v1` JSON response uses one of these shapes:

```json
{"ok": true, "data": {}, "meta": {"apiVersion": 1, "correlationId": "f2dce5b1-7c31-4e8b-b7bb-3f6e29f0a244"}}
{"ok": false, "error": {"code": "stable_code", "message": "sanitized", "retryable": false, "details": {}}, "meta": {"apiVersion": 1, "correlationId": "f2dce5b1-7c31-4e8b-b7bb-3f6e29f0a244"}}
```

The response never contains a Python traceback, environment variable, runtime bearer, maintenance capability, CSRF secret, webhook address, or filesystem path that the authenticated role is not allowed to see.

### 4.3 Mutation protocol

All M3+ mutations use exactly:

```text
POST /control/v1/actions/preview
POST /control/v1/actions/{action_id}/confirm
POST /control/v1/actions/{action_id}/cancel
GET  /control/v1/actions/{action_id}
```

`preview` stores action kind, canonical server-derived parameters, actor, role, risk, impact, warnings, confirmation phrase, expiry, and a fingerprint of HEAD/index/baseline/leases/Failure graph/plan/Cargo/Session state. `confirm` recomputes that fingerprint before execution. Red actions never retry automatically.

### 4.4 Commit notification

After a successful service-owned Git commit, the coordinator constructs exactly four lines from server-side evidence and performs one notification attempt:

```text
核心内容摘要：{milestone_summary_zh}
提交时间：{commit_time_iso8601}
修改情况统计：{git_shortstat}
提交的commit内容：{full_commit_sha} {commit_subject}
```

The service invokes the existing WeCom skill adapter using only its process environment. It never accepts webhook content from the browser, stores a webhook URL/key, writes one into startup configuration, includes it in logs, or retries a failed delivery.

## Milestone M1: Control-Plane Foundation and Read-Only Browser Authentication

**Goal:** Add a versioned, consistent, read-only control facade that a browser can access without receiving the daemon runtime bearer token.

**In-scope behaviors:** Schema v14; workflow/run/node read models; immutable event cursor; one-transaction snapshot; SSE replay and resync; one-time 30-second Observer ticket; HttpOnly/SameSite cookie; strict loopback Host/Origin policy; `/control/v1/meta`; modular HTTP delegation; CLI `ui open`; legacy CLI compatibility.

**Dependencies:** Existing schema v13, Session/lease/Failure/Cargo/finalize domains, shared-main baseline service, and approved design.

### Implementation slices

- [ ] **M1.1 Define enums and schema v14.**

  **Files:** modify `tools/session_coordinator/models.py`, `tools/session_coordinator/migrations.py`; create `tools/session_coordinator/workflows/models.py`; test `tools/session_coordinator/tests/test_workflow_schema.py`.

  Migration 14 creates `workflow_runs`, `workflow_nodes`, `workflow_edges`, `workflow_attempts`, `workflow_artifacts`, `workflow_diagnostics`, `web_control_sessions`, and `web_bootstrap_tickets`. Every state/kind column has a SQL `CHECK` matching the Python enums. Tickets store only a SHA-256 digest, single-use flag, expiry, role=`observer`, actor, and daemon instance ID. Tests assert upgrade 13→14, clean install→14, enum rejection, foreign keys, duplicate-edge rejection, and no downgrade mutation.

- [ ] **M1.2 Build workflow persistence and projections.**

  **Files:** create `tools/session_coordinator/workflows/store.py`, `projections.py`, `__init__.py`; test `test_workflow_store.py`, `test_workflow_projections.py`.

  Stable interfaces are `WorkflowStore.ensure_session_run(session_id: str, plan_path: str | None) -> WorkflowRunRecord`, `append_attempt(node_id: str, state: WorkflowNodeState, evidence: dict[str, object]) -> WorkflowAttemptRecord`, `current_attempts(run_id: str) -> dict[str, WorkflowAttemptRecord]`, `WorkflowProjectionService.workflow_summaries(connection: sqlite3.Connection) -> list[dict[str, object]]`, and `workflow_detail(connection: sqlite3.Connection, run_id: str) -> dict[str, object]`.

  Current state always comes from the latest accepted attempt. Earlier failures remain in `attemptHistory` and cannot override a later success. Foreign Failure nodes appear as diagnostics and never become dependency edges.

- [ ] **M1.3 Build control contracts, consistent snapshot, and event replay.**

  **Files:** create `tools/session_coordinator/control_plane/contracts.py`, `snapshot.py`, `events.py`, `__init__.py`; test `test_control_snapshot.py`, `test_control_events.py`.

  `ControlSnapshotService.build()` opens one deferred read transaction, captures `MAX(events.event_id)`, then queries Sessions, workflows, applicable Failures, leases, patches, Cargo jobs, finalize requests, baseline, recent audit events, and supervision state before commit. `EventStreamService.read_after(cursor, limit=256)` orders by `event_id`; stale cursors return `resync_required`. SSE sends `id`, `event`, and JSON `data`, emits a 15-second heartbeat, bounds each client queue, and rejects the ninth concurrent client.

- [ ] **M1.4 Build Observer bootstrap and loopback security.**

  **Files:** create `tools/session_coordinator/control_plane/auth.py`, `http_security.py`; test `test_control_auth.py`, `test_control_security.py`.

  Stable interfaces are `WebControlAuth.issue_bootstrap_ticket(actor: str, instance_id: str, ttl_seconds: int = 30) -> str`, `consume_bootstrap_ticket(raw_ticket: str, instance_id: str) -> WebSessionRecord`, and `authenticate_cookie(cookie_header: str, instance_id: str) -> WebSessionRecord`.

  Bind only `127.0.0.1`; allow only loopback Host values for the bound port; require an allowlisted loopback Origin for browser API calls; emit `HttpOnly; SameSite=Strict; Path=/control; Max-Age=28800`; consume each ticket once; invalidate every web session when daemon instance ID changes. Tests cover replay, expiry, forged cookie, wrong instance, malicious Host/Origin, non-loopback bind request, and secret-free errors.

- [ ] **M1.5 Extract HTTP routing and compose it into the daemon.**

  **Files:** create `tools/session_coordinator/control_plane/router.py`, `http.py`; modify `tools/session_coordinator/server.py`, `client.py`, `cli.py`, `config.py`; test `test_control_http.py`, extend `test_server.py` only for composition/lifecycle assertions.

  `CoordinatorRequestHandler` delegates `/control/v1/*` and `/ui/*` to `ControlPlaneHttp`; legacy `/health`, `/command`, and `/shutdown` remain compatible. The runtime descriptor adds `instance_id`, `control_api_versions: [1]`, and `started_at` but never adds credentials beyond the existing local runtime token. CLI commands are:

  ```text
  zircon-session ui ticket --role observer --json
  zircon-session ui open
  zircon-session control snapshot --json
  ```

  `ui open` requests a ticket with bearer authentication and launches `/ui/bootstrap/{single_use_ticket}`; it never prints the ticket in normal human output.

- [ ] **M1.6 Record M1 documentation and slice evidence.**

  **Files:** modify `docs/cli-and-tooling/local-session-coordinator.md`; create `docs/cli-and-tooling/workflow-control-center.md`; write one record per completed slice under `docs/plans/zircon_tooling/session_coordinator/01/` after coordinator authorization.

  Document the trust boundary, runtime descriptor additions, Observer bootstrap, snapshot/event cursor, diagnostic commands, and recovery behavior. No concrete evidence row is appended to this protected plan definition.

**Lightweight checks:** after each Python slice run `python -m compileall -q tools/session_coordinator`; run only the named pure-model test module when a slice changes no transport or process lifecycle.

### Testing stage M1-T

Run in a temporary Git repository/state root:

```powershell
python -m unittest tools.session_coordinator.tests.test_workflow_schema tools.session_coordinator.tests.test_workflow_store tools.session_coordinator.tests.test_workflow_projections -v
python -m unittest tools.session_coordinator.tests.test_control_snapshot tools.session_coordinator.tests.test_control_events tools.session_coordinator.tests.test_control_auth tools.session_coordinator.tests.test_control_security tools.session_coordinator.tests.test_control_http -v
python -m unittest tools.session_coordinator.tests.test_server tools.session_coordinator.tests.test_sessions -v
python -m compileall -q tools/session_coordinator
git diff --check -- tools/session_coordinator docs/cli-and-tooling docs/plans/zircon_tooling/session_coordinator/01
```

Expected: all named tests pass; a ticket is single-use; snapshot cursor and SSE replay are consistent; legacy commands remain green; browser-visible bodies contain no token/capability/traceback. Debug in order: migration/enums → auth/session → snapshot transaction → event replay → HTTP composition.

**Exit evidence:** schema version 14; `/control/v1/meta` and snapshot contract fixtures; successful Observer cookie flow; eight-client SSE acceptance and ninth-client rejection; unchanged legacy CLI suite; M1 testing record in the numbered `01/` directory; one workflow-infrastructure commit followed by one four-line WeCom notification.

## Milestone M2: Read-Only Jenkins-Style Web Console

**Goal:** Serve a production React console that makes every existing coordinator domain observable in real time without exposing any mutation surface.

**In-scope behaviors:** Independent Vite build; Chinese-first shell; Overview; workflow pipeline; Sessions; Failure graph; leases/patches; Cargo/validation copies; Git milestones; event audit; service/settings read-only views; logs; SSE resync; virtualized large lists; keyboard/accessibility basics; static asset security.

**Dependencies:** M1 accepted, Observer bootstrap works, API contract version 1 fixed.

### Implementation slices

- [ ] **M2.1 Scaffold the independent web package and shell.**

  **Files:** create `tools/session_coordinator/web/package.json`, `package-lock.json`, `tsconfig.json`, `vite.config.ts`, `index.html`, `src/main.tsx`, `src/App.tsx`, `src/theme.ts`, `src/styles.css`, `src/navigation.ts`; test `src/__tests__/navigation.test.tsx`.

  Package scripts are exactly `dev`, `typecheck`, `test`, `build`, and `check`. `check` runs typecheck, component tests, production build, then the static manifest verifier. Vite binds `127.0.0.1`, emits relative hashed assets to `dist`, disables production source maps, and allowlists only the console root plus explicitly imported visual-only Hub source files. The shell contains ten routes, a persistent health top bar, navigation, global resync/error banner, and an accessible main landmark.

- [ ] **M2.2 Implement strict API types, store, and SSE reconciliation.**

  **Files:** create `src/api/contracts.ts`, `client.ts`, `events.ts`, `validation.ts`, `src/state/controlStore.tsx`, `reducer.ts`; test `contracts.test.ts`, `reducer.test.ts`, `events.test.ts`.

  ```ts
  export interface ControlSnapshot {
    projectionVersion: number;
    eventCursor: number;
    service: ServiceProjection;
    workflows: WorkflowSummary[];
    sessions: SessionProjection[];
    failures: FailureProjection;
    collaboration: CollaborationProjection;
    validation: ValidationProjection;
    git: GitProjection;
    audit: AuditEvent[];
  }

  export type ConnectionState = "connecting" | "live" | "resyncing" | "offline";
  ```

  Runtime validation rejects missing enums/IDs/arrays instead of casting unknown JSON. Startup fetches one snapshot, then connects from `eventCursor`; `resync_required` fetches a new snapshot; duplicate event IDs are ignored; a gap triggers resync; no event payload is rendered as HTML.

- [ ] **M2.3 Implement workflow pipeline, node drawer, and attempt history.**

  **Files:** create `src/pages/WorkflowsPage.tsx`, `src/components/workflow/WorkflowPipeline.tsx`, `StageColumn.tsx`, `WorkflowNodeCard.tsx`, `NodeDetailDrawer.tsx`, `AttemptTimeline.tsx`, `graphLayout.ts`; test each reducer/layout and `WorkflowsPage.test.tsx`.

  Stages are Preflight, Implementation, Validation, Review, Commit, and Notification. Every node displays icon, text state, duration, Session, attempt number, and block reason; color is supplementary. The drawer shows dependencies, plan/artifact links, leases, command/exit evidence, applicable Failure nodes, commit scope, notification result, and immutable attempt history. Tests prove a succeeded latest attempt wins over two historical failures and foreign Failure diagnostics do not block the stage.

- [ ] **M2.4 Implement domain pages and bounded rendering.**

  **Files:** create all remaining page files plus `src/components/failure/FailureGraph.tsx`, `collaboration/LeaseTable.tsx`, `validation/ValidationLaneTable.tsx`, `git/MilestoneCommitEvidence.tsx`, `audit/VirtualAuditList.tsx`, `logs/LogViewer.tsx`; test the page-specific filters and empty/degraded states.

  The Failure graph distinguishes applicable/open/fixed/foreign/invalid nodes. Collaboration shows owner, expiry, base/current hash, delayed patch, conflict, HEAD/index/worktree and baseline. Validation shows lane, target root, validation-copy path alias, PID, duration, exit code, and artifact link. Log viewer uses ranged fetch, virtual rows, pause/follow, search, timestamp/level filters, truncation label, and text-only rendering. Lists cap DOM rows and preserve stable keyboard focus.

- [ ] **M2.5 Serve production assets with strict caching and downloads.**

  **Files:** create `tools/session_coordinator/control_plane/assets.py`, `artifact_downloads.py`; modify `router.py`, `config.py`; create `tools/session_coordinator/tests/test_control_assets.py`, `test_artifact_downloads.py`, `web/scripts/verify-dist.mjs`.

  `index.html` is `no-store`; hashed JS/CSS are `public,max-age=31536000,immutable`; API paths never fall back to HTML; directory listing is impossible; unknown routes fall back only under `/ui/`; artifact download accepts an opaque artifact ID, resolves metadata in SQLite, verifies the resolved path stays under the artifact root, sets `nosniff`, and supports bounded byte ranges. The verifier rejects source maps, absolute development URLs, runtime token names, maintenance capability names, webhook values, missing hashes, and unreferenced assets.

- [ ] **M2.6 Complete accessibility, visual reuse, docs, and evidence.**

  **Files:** use `zircon_hub/web/src/theme/tokens.ts`, `muiTheme.ts`, and generic `HubPanel`/`HubButton` as read-only visual imports; create console-specific status components; update operator docs; record slices under the numbered `01/` directory.

  Do not import `hubApi`, Hub DTOs, Hub Tauri calls, project actions, or Hub persistence. Component tests cover focus return from drawers, keyboard navigation, screen-reader labels, non-color state text, degraded/read-only banners, and SSE disconnect/reconnect.

**Lightweight checks:** `npm --prefix tools/session_coordinator/web run typecheck` after type-only slices; Python compileall after asset-router changes. Component/unit suites run in M2-T rather than after every view file.

### Testing stage M2-T

```powershell
npm --prefix tools/session_coordinator/web ci
npm --prefix tools/session_coordinator/web run check
python -m unittest tools.session_coordinator.tests.test_control_assets tools.session_coordinator.tests.test_artifact_downloads tools.session_coordinator.tests.test_control_http -v
powershell -NoProfile -ExecutionPolicy Bypass -File tools/tests/workflow-control-center-smoke.Tests.ps1 -ReadOnlyConsole
git diff --check -- tools/session_coordinator tools/session_coordinator/web docs/cli-and-tooling docs/plans/zircon_tooling/session_coordinator/01
```

Expected: typecheck, component tests, build, manifest verification, static security tests, and smoke flow pass; the browser opens through a consumed Observer ticket; all pages load from a consistent snapshot and update through SSE; network inspection contains no bearer token. Debug in order: API validator/store → page projection → asset serving → browser bootstrap → visual/accessibility behavior.

**Exit evidence:** production `dist` manifest and hashes; screenshots at 1568×1003 and 1280×800; keyboard/accessibility record; SSE resync record; zero mutation endpoints invoked by the M2 bundle; M2 commit and one successful or recorded-failed non-retried WeCom attempt.

## Milestone M3: Permissioned Controlled Actions

**Goal:** Add a closed Action Catalog with short-lived elevation, CSRF protection, two-phase confirmation, state revalidation, and immutable audit evidence.

**In-scope behaviors:** Schema v15; Observer/Operator/Committer/Maintainer roles; elevation issued only by authenticated CLI/tray; CSRF; action preview/confirm/cancel/status; green/yellow/red risk; state fingerprint; heartbeat, own-lease, safe patch, validation, Failure refresh, topology refresh, drain preview; action UI.

**Dependencies:** M2 read-only console accepted.

### Implementation slices

- [ ] **M3.1 Add schema v15 and typed catalog.** Create `action_requests`, `action_approvals`, and `web_elevation_grants` with enum checks and expiry; create `control_plane/actions/catalog.py`, `models.py`, `permissions.py`; define an immutable `ActionSpec` for every allowed action and no generic command action. Tests enumerate the catalog and prove arbitrary shell/Git/Cargo/SQL/path kinds are unrepresentable.
- [ ] **M3.2 Implement elevation and CSRF.** Extend `auth.py` with one-use elevation grants bound to actor, role, daemon instance, optional Session, and expiry. Rotate CSRF on elevation, require cookie plus header for mutations, and reject browser self-elevation. Tests cover downgrade, expiry, replay, cross-session Committer use, missing/mismatched CSRF, and restart invalidation.
- [ ] **M3.3 Implement preview/fingerprint/confirm.** Create `actions/fingerprint.py`, `preview.py`, `executor.py`, `service.py`. Fingerprint canonical JSON for HEAD, index tree, baseline epoch/health, target file hashes, active leases, applicable Failure graph revision, plan hash, Cargo jobs, Session status, and daemon instance. Confirm recomputes it inside the action transaction; mismatch returns `action_state_changed` and performs no side effect.
- [ ] **M3.4 Wire yellow actions to existing domain services.** Each executor accepts a validated dataclass and calls `SessionService`, `LeaseService`, `PatchService`, `WorkspaceCopyService`, `FailureGraphService`, or `SupervisionService`; it never calls `CoordinatorApplication.command()` with browser text. Validation commands come from allowlisted server templates, not browser argv. Tests assert ownership, path derivation, critical-section locking, cancellation semantics, and immutable audit rows.
- [ ] **M3.5 Add confirmation UI and action history.** Create `src/actions/catalog.ts`, `actionClient.ts`, `ActionDialog.tsx`, `RiskSummary.tsx`, and page-local action menus. Yellow actions show impact/warnings/expiry/reason and require explicit confirm; red UI components exist but remain disabled until M4/M5 server specs register. Stale preview displays a fresh diff instead of retrying.
- [ ] **M3.6 Document permissions and write slice evidence.** Update operator docs with role issuance, action lifecycle, audit lookup, and denial codes; write records only under `01/`.

### Testing stage M3-T

```powershell
python -m unittest tools.session_coordinator.tests.test_action_catalog tools.session_coordinator.tests.test_action_auth tools.session_coordinator.tests.test_action_fingerprint tools.session_coordinator.tests.test_action_execution tools.session_coordinator.tests.test_action_concurrency -v
npm --prefix tools/session_coordinator/web run check
powershell -NoProfile -ExecutionPolicy Bypass -File tools/tests/workflow-control-center-smoke.Tests.ps1 -ControlledActions
git diff --check -- tools/session_coordinator tools/session_coordinator/web docs/cli-and-tooling docs/plans/zircon_tooling/session_coordinator/01
```

Expected: catalog-external operations are rejected; CSRF/elevation tests pass; a concurrent HEAD/lease/file change invalidates confirm; audit rows exist for denial/cancel/success/failure; no red action is active. Correct auth/fingerprint defects before executor or UI defects.

**Exit evidence:** schema 15; catalog inventory; negative security matrix; deterministic concurrency test repeated 20 times; controlled-action UI screenshot/evidence; M3 commit plus one four-line notification attempt.

## Milestone M4: Workflow Topology, Gates, Commit, Goal Closeout, and Notification

**Goal:** Supervise a complete milestone from registered plan topology through validation/review/Failure gates to an atomic service-owned commit and recorded notification result.

**In-scope behaviors:** Schema v16 topology versions, gate evidence, review evidence, and notification attempts; fenced topology parser plus heading fallback; cycle/missing-owner detection; immutable attempts/artifacts/diagnostics; current-versus-history; validation/review/Failure gates; red milestone commit; Goal closeout; service-built four-line WeCom notification; workflow UI controls.

**Dependencies:** M3 action protocol accepted; existing `git_finalize.py`, Failure graph, validation-copy, and closeout policy remain authoritative.

### Implementation slices

- [ ] **M4.1 Add schema v16, then parse and version plan topology.** Migration 16 creates `workflow_topology_versions`, `workflow_gate_evidence`, `workflow_review_evidence`, and `notification_attempts`. Create `workflows/topology.py`, `plan_import.py`; accept exactly one `zircon-workflow` fenced JSON block, validate schema/id/dependencies/numbered owner/content hash, and persist a new version instead of rewriting a running graph. For plans without a block, deterministically import `## Milestone Mx` and bold `Mx.y` checkbox slices. Tests cover upgrade 15→16, duplicate IDs, cycles, missing dependencies, changed hashes, malformed fences, and protected child ownership.
- [ ] **M4.2 Implement attempts, artifacts, diagnostics, and gate evaluator.** Create `attempts.py`, `artifacts.py`, `gates.py`. A gate returns a typed decision with `allowed`, stable code, blocking node IDs, applicable Failure IDs, required evidence, and current attempt IDs. Validation failure sends the workflow to the lowest failed shared layer; foreign Failures remain diagnostics only.
- [ ] **M4.3 Integrate validation and independent review.** Add server-derived validation templates and review evidence records. A milestone cannot enter finalizing until all slices, the named testing stage, Critical/Important review, Failure audit, plan-output audit, and commit manifest are accepted. Tests cover skipped nodes, retried gates, stale evidence, and current-attempt replacement.
- [ ] **M4.4 Implement red milestone commit and Goal closeout actions.** The preview derives file scope from Session attribution and compares staged/attributed blobs. Confirm acquires the Git mutex, revalidates gates/fingerprint, calls `GitFinalizeService`, records SHA/shortstat/baseline epoch, then marks only that milestone succeeded. Goal closeout requires all milestones succeeded and applies the repository closeout invariants. Tests prove another Session's staged/untracked files cannot enter the commit and rollback restores the prior index/ref on failure.
- [ ] **M4.5 Implement one-shot notification adapter.** Create `notifications.py`; format the four server-derived lines, call `$HOME/.codex/skills/wecom-push-message/scripts/send-wecom-message.ps1` once, record timestamp/exit/errcode/sanitized error, and never retry. Do not persist message transport credentials or browser text. Commit success is not rolled back when notification fails.
- [ ] **M4.6 Complete workflow UI controls and records.** Enable validation/review/commit/Goal actions by role and gate state; show commit manifest, SHA, shortstat, baseline, notification attempt, and retry prohibition; add topology diff and attempt timeline views; update docs and write slice/testing evidence under `01/`.

### Testing stage M4-T

```powershell
python -m unittest tools.session_coordinator.tests.test_workflow_topology tools.session_coordinator.tests.test_workflow_attempts tools.session_coordinator.tests.test_workflow_gates tools.session_coordinator.tests.test_workflow_commit tools.session_coordinator.tests.test_notifications -v
python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:\Git\ZirconEngine
python .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py --repo-root E:\Git\ZirconEngine
npm --prefix tools/session_coordinator/web run check
powershell -NoProfile -ExecutionPolicy Bypass -File tools/tests/workflow-control-center-smoke.Tests.ps1 -MilestoneCommit
```

Expected: topology/gate/current-attempt/commit tests pass; a temporary repository milestone commits only attributed files; one notification attempt is recorded for success and separately for failure; real plan/failure audits have no new errors. Debug topology → attempt selection → gates → finalize transaction → notification → UI.

**Exit evidence:** schema 16; imported topology hash; accepted validation/review/Failure/output gates; atomic temporary-repo commit proof; notification success/failure fixtures; M4 commit and immediate four-line notification attempt.

## Milestone M5: Windows Tray Supervision

**Goal:** Provide a lightweight current-user tray that safely identifies, starts, observes, drains, stops, restarts, and recovers the coordinator without becoming a second workflow authority.

**In-scope behaviors:** Schema v17 supervision events and recovery state; independent Tauri workspace; named mutex per repository hash; runtime descriptor validation; PID/process creation/command-line/repo/instance/authenticated-health identity; status icon/menu; open console/Hub; drain/stop/restart; explicit-stop persistence; crash backoff; five-in-ten-minute circuit breaker; user startup; native notifications; tray exit leaves daemon alive.

**Dependencies:** M2 for read/open; M3 for supervised lifecycle actions.

### Implementation slices

- [ ] **M5.1 Add schema v17 and scaffold the tray workspace.** Migration 17 creates `service_supervision_events` and `service_recovery_state` with exact supervision-state checks, daemon instance/process creation identity, reason code, actor, action ID, timestamp, and recovery counters. Create `tools/session_tray/Cargo.toml` with an empty local `[workspace]`, Tauri 2.11.2, serde/serde_json, thiserror, and Windows 0.54 APIs; create `build.rs`, `tauri.conf.json`, `capabilities/default.json`, icons, `src/main.rs`, `src/lib.rs`. Configure no visible main window and only required tray/notification/shell permissions. Build target comes from a coordinator Cargo lane outside the repo.
- [ ] **M5.2 Implement runtime descriptor and identity verification.** Create `runtime_descriptor.rs`, `process_identity.rs`, `repository_identity.rs`. Compute a normalized repository SHA-256 key; acquire a Windows named mutex; verify descriptor repo, PID liveness, process creation time, executable/command line, authenticated `/health`, instance ID, and API version before any lifecycle action. A stale descriptor never authorizes termination.
- [ ] **M5.3 Implement client, state machine, menu, and icon.** Create `coordinator_client.rs`, `tray_state.rs`, `menu.rs`, `notifications.rs`. Poll health with bounded timeouts; map exact supervision enums to icon/tooltip/menu; disable invalid actions; open browser through a one-time ticket; expose diagnostic copy with secrets redacted. Exiting tray releases only the tray mutex and never sends stop.
- [ ] **M5.4 Implement start, drain, stop, restart, and force recovery.** Create `lifecycle.rs`. Offline start invokes hidden `tools/zircon-session.ps1 start -Json`; normal drain/stop/restart use M3 action preview/confirm and display active Git/Cargo/patch sections. Force termination is Maintainer-only, requires a second identity verification immediately before `TerminateProcess`, and is unavailable for mismatch/fatal migration states.
- [ ] **M5.5 Implement recovery and startup.** Create `recovery.rs`, `startup.rs`. Unexpected exits retry after 1/2/5/15/30 seconds; five failures in ten minutes open the circuit; ten healthy minutes clear it. Explicit stop, migration failure, identity mismatch, valid competing instance, fatal integrity error, maintenance-held offline state, or exhausted attempts never auto-restart. Register tray and coordinator separately for the current user with absolute paths and no credentials.
- [ ] **M5.6 Add Rust/unit/Windows smoke tests and docs.** Unit-test descriptor parsing, state transitions, backoff, circuit breaker, explicit stop, and menu enablement. Windows integration fixtures use a harmless child process to prove stale PID substitution and unrelated-process protection. Update operator docs and write numbered output records.

### Testing stage M5-T

Allocate a managed lane, then run from `tools/session_tray`:

```powershell
$lease = (& .\tools\zircon-session.ps1 cargo acquire test --session-id workflow-control-center-20260711-1915 -Json | ConvertFrom-Json)
$env:CARGO_TARGET_DIR = $lease.job.target_dir
try {
    cargo fmt --manifest-path tools/session_tray/Cargo.toml -- --check
    cargo test --manifest-path tools/session_tray/Cargo.toml --locked
    cargo build --manifest-path tools/session_tray/Cargo.toml --locked
    powershell -NoProfile -ExecutionPolicy Bypass -File tools/tests/workflow-control-center-smoke.Tests.ps1 -TrayLifecycle
} finally {
    & .\tools\zircon-session.ps1 cargo release $lease.job.job_id --session-id workflow-control-center-20260711-1915 -Json
}
```

Expected: Rust tests/build pass; real tray shows correct state; tray exit leaves daemon PID unchanged; explicit stop stays stopped; stale runtime cannot kill the fixture; crash circuit opens exactly after the fifth failure. The `finally` block releases the exact service-returned Cargo lane.

**Exit evidence:** packaged tray binary; identity/mutex fixture logs; state/menu screenshots; drain and restart audit events; startup query evidence; tray-exit survival proof; M5 commit plus one four-line notification attempt.

## Milestone M6: Security, Load, Soak, Packaging, Hub Entry, and Release Acceptance

**Goal:** Prove the complete control center and tray satisfy security, performance, recovery, accessibility, packaging, and 24-hour operational requirements on real Windows.

**In-scope behaviors:** Malicious-input matrix; 200 Sessions/100 workflows/5,000 nodes/100,000 events/10,000 artifacts/eight SSE clients/500 MB log range reads; browser and tray packaging; optional Hub open-console entry; v13→v17 upgrade/read-only migration failure; 24-hour soak; operator/failure guides.

**Dependencies:** M4 and M5 accepted.

### Implementation slices

- [ ] **M6.1 Build deterministic load/security fixtures.** Create `tools/session_coordinator/tests/load_fixture.py`, `test_control_load.py`, `test_control_security_matrix.py`; generate the exact design-scale dataset in a temporary database/artifact root and measure P95 health/snapshot/list/event/action preview without mutating the shared repository.
- [ ] **M6.2 Add browser accessibility and end-to-end acceptance.** Add production-browser tests for ticket replay, cookie/CSRF, Host/Origin, traversal, artifact enumeration, XSS logs, SSE isolation/resync, action-state races, keyboard focus, screen-reader labels, non-color status, 1280×800 and 1568×1003 layouts. Store screenshots/reports under `docs/tests/workflow-control-center/` through an authorized record path or acceptance artifact owner.
- [ ] **M6.3 Harden migration, upgrade, crash, and drain recovery.** Test v13→latest with preserved domain rows, injected migration failure to diagnostic read-only mode, daemon crash during SSE/action/commit notification, tray restart circuit, drain timeout/cancel, and old-web/new-daemon API mismatch. No recovery path executes repair SQL from the browser.
- [ ] **M6.4 Add the optional Zircon Hub entry.** Add one typed Hub action that asks the coordinator client for a one-time Observer ticket and opens the browser console. Hub never reads SQLite, stores a runtime token, embeds a workflow store, or stops the daemon. Update only the minimal action DTO/text/command/UI contracts and run the focused Hub contract suites.
- [ ] **M6.5 Package startup and operator workflows.** Update installer scripts for coordinator/tray install/query/remove/dry-run; build production web and tray bundles; verify absolute current-user startup paths, upgrade behavior, uninstall behavior, secret absence, and multi-repository path-hash isolation.
- [ ] **M6.6 Run and accept the 24-hour soak.** `tools/tests/workflow-control-center-soak.ps1` records minute health/SSE/resource samples, scheduled maintenance, injected browser disconnects, one controlled daemon restart, tray recovery, event continuity, memory/handle growth, and final audit. It writes outside Git during the run, then copies only the sanitized summary and metrics artifact into the authorized numbered output directory.
- [ ] **M6.7 Complete docs, independent review, and Goal closeout.** Update the operator guide, failure/recovery guide, module docs, acceptance matrix, and numbered records; run independent Critical/Important review; resolve applicable `failure-*.md`; then execute the service-owned final milestone commit and Goal closeout.
- [x] **M6.8 Harden managed CPU reservation lifecycle.** Require executable owner Sessions for reserve/acquire/renew; atomically expire only pending no-job reservations on stale transitions; preserve absolute pending expiry across restart; terminalize reservations with orphaned jobs; persist canonical compatibility payloads in schema 41; prove payload, stale cleanup, orphan handoff, and FIFO progression against the production daemon.

### Testing stage M6-T

```powershell
python -m unittest discover -s tools/session_coordinator/tests -v
npm --prefix tools/session_coordinator/web run check
cargo test --manifest-path tools/session_tray/Cargo.toml --locked
cargo build --manifest-path tools/session_tray/Cargo.toml --locked
cargo test -p zircon_hub --locked
powershell -NoProfile -ExecutionPolicy Bypass -File tools/tests/workflow-control-center-smoke.Tests.ps1 -Full
powershell -NoProfile -ExecutionPolicy Bypass -File tools/tests/workflow-control-center-soak.ps1 -Hours 24
python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:\Git\ZirconEngine
python .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py --repo-root E:\Git\ZirconEngine
git diff --check -- tools/session_coordinator tools/session_tray zircon_hub docs tests
```

Expected: all suites pass; design-scale P95 targets are met; the soak completes 24 hours with event continuity and no unbounded resource growth; browser/Hub/tray closure does not stop the daemon; tray never kills an unrelated process; real plan/failure audits contain no new violations. Debug the lowest failing domain first and repeat the affected acceptance stage; repeat the full soak if a fix changes daemon lifecycle, SSE, recovery, maintenance, or resource retention.

**Exit evidence:** complete requirement matrix mapped to commands/artifacts; load percentiles; security report; production bundle hashes; Windows startup/drain/stop/restart/recovery evidence; 24-hour soak summary; independent review with zero Critical/Important findings; final M6 commit and four-line notification result.

## 5. Milestone Commit and Notification Discipline

At each accepted milestone:

1. Confirm every completed slice and testing stage has one canonical record under `docs/plans/zircon_tooling/session_coordinator/01/`.
2. Run plan-output and Failure audits when the milestone wrote either record class.
3. Classify files as production, documentation, tests/scripts, generated web assets, and untracked additions; include only this Session's attributed blobs.
4. Create one normal Conventional Commit for the complete milestone. Do not add a Session tag to the Git subject or create checkpoint commits.
5. Immediately perform one WeCom push with the exact four-line server-derived format. Record failure without automatic retry.
6. Advance the baseline epoch and keep the Session active until all six milestones and Goal closeout are accepted.

## 6. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

计划定义保持只读。执行时在 `docs/plans/zircon_tooling/session_coordinator/01/` 中按 `{date}-{summary}.md` 保存每个切片和测试阶段的标准记录；`failure-*`/`fixed-*` 继续遵守专用移动语义。

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M6 | M6.8 managed CPU reservation lifecycle | `accepted / commit_pending` | 2026-07-16 | schema 41 production daemon `5421e008fda84be6b42480cc0c602cec`; fresh Python `70/70 + 15/15 + 37/37`; production payload reservation `0bbc781e...`, orphan-bound reservation `c692e731...`, following FIFO job `2853a1a8...`; handoff audit `161/0`; independent review `P0/P1/P2=0/0/0`; fixed-return immutable-manifest selector regression `8/8` preserves applicable Failure priority while excluding unrelated fixing-plan failures only for completed return slices; exact evidence in `01/2026-07-16-m6-8-cpu-reservation-lifecycle-hardening.md` |

- fixed 已修复：[mutation-queue-finish-lease-stall](../../zircon_editor/editor/02/fixed-2026-07-14-mutation-queue-finish-lease-stall.md)
- fixed 已修复：[mutation-queue-offline-recurrence](../../zircon_editor/editor/02/fixed-2026-07-14-mutation-queue-offline-recurrence.md)
- fixed 已修复：[milestone-validation-copy-template-scope](../../zircon_plugins/08/fixed-2026-07-14-milestone-validation-copy-template-scope.md)
- fixed 已修复：[cargo-release-retains-live-child-process-lock](../../zircon_editor/editor/02/fixed-2026-07-14-cargo-release-retains-live-child-process-lock.md)
- fixed 已修复：[milestone-session-relative-line-ending-drift](../../zircon_runtime/text/01/fixed-2026-07-15-milestone-session-relative-line-ending-drift.md)
- fixed 已修复：[support-slice-exact-finalize-plan-output-conflict](../../zircon_editor/editor/02/fixed-2026-07-16-support-slice-exact-finalize-plan-output-conflict.md)
- open / Plugins12 重复里程碑编号错误选择历史切片 manifest：[repeated-milestone-slice-manifest-selection-conflict](01/failure-2026-07-15-repeated-milestone-slice-manifest-selection-conflict.md)
- open / 活跃受管 ephemeral target 被误判为 unmanaged：[live-ephemeral-target-misclassified-unmanaged](01/failure-2026-07-15-live-ephemeral-target-misclassified-unmanaged.md)
- open / 原生 slice closeout checker 仍依赖共享暂存区：[native-slice-closeout-checker-staged-index-contract-drift](01/failure-2026-07-16-native-slice-closeout-checker-staged-index-contract-drift.md)
- open / lifecycle orphan recovery 被 maintenance hold 完整性约束阻断，服务无法启动：[lifecycle-orphan-recovery-maintenance-hold-integrity-deadlock](01/failure-2026-07-16-lifecycle-orphan-recovery-maintenance-hold-integrity-deadlock.md)
- fixed 已修复：[stale-session-pending-cpu-reservation-starvation](../../zircon_editor/editor/07/fixed-2026-07-16-stale-session-pending-cpu-reservation-starvation.md)

## 7. Completion Audit

The Goal is not complete merely because M1-M5 code exists. Before closeout, map every acceptance criterion in the approved design to current authoritative evidence and classify it as proven, contradicted, incomplete, weak, or missing. Completion requires all criteria proven, all required commands freshly passing, no open applicable Failure, no Critical/Important review item, a clean owned scope, daemon/browser/Hub/tray survival evidence, and the full 24-hour soak.
