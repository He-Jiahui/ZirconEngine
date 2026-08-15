---
related_code:
  - tools/session_coordinator/control_plane/__init__.py
  - tools/session_coordinator/control_plane/auth.py
  - tools/session_coordinator/control_plane/contracts.py
  - tools/session_coordinator/control_plane/events.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/control_plane/http_security.py
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/work_continuations.py
  - tools/session_coordinator/control_plane/assets.py
  - tools/session_coordinator/control_plane/artifact_downloads.py
  - tools/session_coordinator/control_plane/actions/catalog.py
  - tools/session_coordinator/control_plane/actions/models.py
  - tools/session_coordinator/control_plane/actions/permissions.py
  - tools/session_coordinator/control_plane/actions/fingerprint.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/actions/service.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/sessions.py
  - tools/session_coordinator/workflows/plan_import.py
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/supervision/lifecycle.py
  - tools/session_coordinator/patches.py
  - tools/session_coordinator/isolated_patch_contract.py
  - tools/session_coordinator/isolated_patch_finalize.py
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/web/src/pages/ActionsPage.tsx
  - tools/session_coordinator/web/src/pages/OverviewPage.tsx
  - tools/session_coordinator/web/src/components/actions/ActionDialog.tsx
  - tools/session_coordinator/web/src/components/actions/RiskSummary.tsx
implementation_files:
  - tools/session_coordinator/control_plane/__init__.py
  - tools/session_coordinator/control_plane/auth.py
  - tools/session_coordinator/control_plane/contracts.py
  - tools/session_coordinator/control_plane/events.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/control_plane/http_security.py
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/work_continuations.py
  - tools/session_coordinator/control_plane/assets.py
  - tools/session_coordinator/control_plane/artifact_downloads.py
  - tools/session_coordinator/control_plane/actions/catalog.py
  - tools/session_coordinator/control_plane/actions/models.py
  - tools/session_coordinator/control_plane/actions/permissions.py
  - tools/session_coordinator/control_plane/actions/fingerprint.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/actions/service.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/sessions.py
  - tools/session_coordinator/workflows/plan_import.py
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/supervision/lifecycle.py
  - tools/session_coordinator/patches.py
  - tools/session_coordinator/isolated_patch_contract.py
  - tools/session_coordinator/isolated_patch_finalize.py
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/web/src/pages/ActionsPage.tsx
  - tools/session_coordinator/web/src/pages/OverviewPage.tsx
  - tools/session_coordinator/web/src/components/actions/ActionDialog.tsx
  - tools/session_coordinator/web/src/components/actions/RiskSummary.tsx
plan_sources:
  - docs/superpowers/specs/2026-07-11-workflow-control-center-and-tray-design.md
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
  - docs/superpowers/plans/2026-07-17-coordinator-adaptive-cpu-burst-lanes.md
tests:
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
  - tools/session_coordinator/tests/test_server.py
  - tools/session_coordinator/tests/test_isolated_patch_finalize.py
  - tools/session_coordinator/tests/test_supervision_actions.py
  - tools/session_coordinator/tests/test_sessions.py
  - tools/session_coordinator/web/src/__tests__/contracts.test.ts
  - tools/session_coordinator/web/src/__tests__/actions.test.tsx
  - tools/session_coordinator/web/src/__tests__/components.test.tsx
  - tools/session_coordinator/web/src/__tests__/navigation.test.ts
  - tools/tests/workflow_control_center_smoke.py
doc_type: module-detail
---

# Session Coordinator Control Plane

## Responsibility

`control_plane` is the versioned browser and automation facade over the existing coordinator application. It owns API envelopes, browser authentication, loopback request checks, coherent snapshots, ordered event replay, typed controlled-action orchestration and the small HTTP adapter. Domain mutations remain in coordinator services; the control plane writes only its own browser credentials and append-only action request/approval audit records.

## Module Boundaries

- `contracts.py` defines the v1 envelope, correlation identifiers and sanitized error shape.
- `http_security.py` validates exact loopback Host and Origin values before browser processing.
- `auth.py` persists only SHA-256 ticket and cookie digests and binds both to one daemon instance.
- `snapshot.py` assembles all dashboard panels inside one deferred SQLite transaction.
- `events.py` owns monotonically ordered replay, bounded retention and explicit client-capacity accounting.
- `router.py` maps bounded v1 routes to application services without knowing socket details.
- `http.py` translates `BaseHTTPRequestHandler` input/output, enforces the one MiB body limit and performs SSE streaming.
- `assets.py` resolves only built `/ui/` assets, enforces cache policy and confines SPA fallback to navigation routes.
- `artifact_downloads.py` resolves opaque evidence IDs below the configured artifact root and implements bounded byte ranges.
- `actions/catalog.py` is the closed operation allowlist. It exposes typed parameter schemas, risk, required role and whether an action is enabled or preview-only.
- `actions/fingerprint.py` captures the Git, Session, lease, target and coordinator state used by preview/confirm optimistic concurrency control.
- `actions/service.py` persists preview, denial, confirmation and completion state, while `actions/executor.py` invokes existing domain services without accepting shell commands, SQL or browser-supplied paths.
- `supervision/lifecycle.py` owns the durable handoff between a controlled service action and a successor daemon. The catalog's `service.rollover` path is deliberately separate from stop/restart: it reloads only after the process monitor reports no live managed Cargo descendants.
- `server.py` composes the module and delegates `/control/v1/*` and `/ui/*` before the legacy bearer route handling.
- `isolated_patch_contract.py` defines the immutable request/result identity and the
  allowlisted validation environment. `isolated_patch_finalize.py` owns the
  maintainer-only, single-target path from an immutable HEAD blob plus explicit
  patch to a CAS-published commit. It never consumes a compile ticket or reads the
  live target as commit input.

## Invariants

1. The listener remains loopback-only and browser Host validation fails closed.
2. A bootstrap ticket has one role, one daemon instance, one expiry and at most one successful consumption.
3. Browser cookies are opaque, digest-backed, `HttpOnly`, `SameSite=Strict` and scoped to `/control`.
4. Snapshot cursor and panel data come from the same SQLite read transaction.
5. Event replay is ordered and bounded to a 4,096-position logical window with 256-event batches; stale and future cursors both result in refresh-required semantics.
6. Internal exception detail, bearer tokens, maintenance tokens, ticket values and cookie values never enter response bodies.
7. M1-M2 remain read-only for Observer sessions. M3 mutations require a short-lived elevated role, a Session binding when applicable, CSRF validation and a catalog entry. No generic command, SQL or direct database mutation endpoint exists.
8. API paths never fall back to HTML. Only extensionless navigation below `/ui/` may receive the SPA shell.
9. Artifact paths are database-selected and canonically confined. The browser receives only safe validation-lane state; commands, absolute targets, compatibility/reuse payloads, PID state, exit codes, and cleanup errors remain coordinator-internal.
10. A maintenance-hold bootstrap may create one previously unknown Session only through `session.activate@maintenanceSessionId`, with Maintainer authority and complete display name, plan path, and write scope. Its action fingerprint and pre-creation audit events bind to the existing maintenance Session; ordinary Session activation remains target-session-bound.
11. `service.rollover` keeps supervision `healthy` and admission open. It never creates a global drain or maintenance hold; it rejects a real managed Cargo PID tree and leaves leased-but-unstarted jobs, reservations, compatibility payloads and FIFO state durable for the successor.
12. A successful direct or controlled `session.heartbeat` renews only the caller Session's live file leases and returns the renewed count. If the caller was marked `stale`, that same heartbeat atomically resumes it as `active` with an auditable status event; it never revives an archived/cancelled Session, an expired CPU reservation, or a prior validation wait. An active Session cannot silently outlive its write lease while another Session is allowed to claim the same path.
13. A lease scope is hierarchy-exclusive: a live directory lease conflicts with every foreign descendant file or directory lease, and a live child-file lease conflicts with a later foreign ancestor claim. A directory owner may validate its own descendants without manufacturing duplicate child leases. This makes shared-main writes exclusive at the module boundary rather than only at byte-identical paths.
14. Automatic Session stale marking skips only a Session with a managed Cargo job in `running` state. It does not renew file leases or reservations, shield an unstarted `leased` job, or change FIFO admission; once the job is terminal, the ordinary liveness window applies again.
15. A healthy rollover successor coalesces a second `service.rollover` requested during its 60-second stabilization window. The duplicate action succeeds with an auditable `coalesced` result and does not schedule another shutdown, close Session admission, or alter leases and FIFO reservations.
16. `maintenance finalize-patch` is an isolated maintenance finalizer, not an
    integration candidate. It requires symbolic `main`, a live target lease,
    ancestor base HEAD, exact unchanged target blob, one explicit patch and
    non-empty validation commands. Validation runs from the derived temporary-index
    checkout with an explicit environment allowlist. Publication uses the raw shared
    index bytes as a CAS identity, preserves the mixed worktree bytes and foreign
    staged projection, and aligns only the target entry after main advances.

## Data Flow

The authenticated CLI asks the legacy coordinator route to issue a bootstrap ticket. A browser consumes that ticket through the loopback facade and receives an Observer cookie. A local runtime caller may separately issue a one-use elevation grant bound to the same actor, daemon instance and optional Session. Consuming it rotates the CSRF token and grants a short-lived Operator, Committer or Maintainer role; Maintainer issuance additionally requires the local maintenance capability. A daemon restart changes `instance_id`, invalidating all previous browser credentials and grants.

For a controlled action, the browser submits only a catalog kind and its exact typed parameters. The service checks permission and Session scope, captures an impact summary, the exact Patch/Validation/Failure resource set, and a state fingerprint, then returns a two-minute preview with an explicit confirmation phrase. Confirm records an immutable approval reason and executes only if the identity, daemon instance, scope, phrase and current fingerprint still match. Fingerprint revalidation and the typed side effect share the same daemon mutation gate, closing the preview/execute race against CLI commands, maintenance and other actions. Any intervening state change yields `action_state_changed`; the UI obtains a new preview and displays its impact diff without automatically retrying the mutation.

`milestone prepare --milestone M<n>` now forwards that exact normalized node key
to the typed `topology.refresh` prepare variant. It refreshes only semantic
topology identity: ordinary plan prose, Failure links, and status text reuse the
active immutable topology version rather than replacing it. The returned
preparation names the requested node plus its current-version manifest ID and
hash, so later validation/review/commit actions cannot silently aggregate an
older same-numbered slice.

The maintainer-only rollover action writes an `awaiting_restart` lifecycle intent before asking the current loopback listener to exit. It does not rewrite Cargo ledger rows or state admission as draining. The successor persists the new descriptor on the fixed loopback endpoint, recognizes that exact intent and marks the same controlled action succeeded. A normal interrupted action is still failed on successor startup; only the explicit rollover handoff is preserved.

If a second local monitor requests the same rollover while that successor is still
stabilizing, the successor returns a successful `coalesced` action instead of
replacing itself again. This is an idempotency window, not a global cooldown:
Sessions remain executable, and a later explicit reload follows the normal
live-Cargo safety check.

`session.heartbeat` is a lightweight non-blocking mutation. The legacy command route and the typed control action both renew the Session record and its own active leases in the same request flow; the response includes `leases.renewed`. This preserves source ownership during long validation waits without extending any foreign lease or changing Cargo lane admission.

When a target file contains unrelated live edits, an operator may pipe one unified
diff to `maintenance finalize-patch --patch-stdin`. The command binds the request to
`--expected-head` and `--expected-blob`, builds the patch in a temporary Git index,
and checks that exactly `--target` changed with mode `100644`. It then checks out that
temporary tree to an isolated directory and runs every `--validation-command` there.
The durable prepared, validated, index-locked, and finalized events record base
HEAD/blob, patch hash, derived blob, actual parent HEAD, validation commands and
status, commit SHA, and staged projection fingerprints. If current HEAD advanced only
on other paths, publication uses it as the new parent. A target blob change, branch
switch, lease loss, validation failure, shared-index drift, worktree drift, or
`update-ref` CAS failure rejects the request. A crash after main publication remains
in the ordinary finalize ledger; startup recovery restores the captured index and
emits the specialized finalized evidence. No compile ticket is accepted because a
ticket for the live mixed overlay does not validate this derived blob.

The default business-Session liveness window is 86400 seconds (24 hours). It is deliberately
longer than the 300-second lease plus 120-second grace and the independent Cargo
reservation TTL: a missed coordinator heartbeat no longer turns ordinary focused
work into a false global-looking interruption, while abandoned file ownership and
pending validation still release through their own bounded rules.

A `running` managed Cargo job is direct coordinator-owned proof that its owner is
still validating. The stale sweep therefore leaves that Session executable until
the job becomes terminal, then uses the unchanged business liveness rule. This
guard never renews a lease, reservation, or heartbeat timestamp.

Validation history and live target rows share a narrow browser projection: job
owner, lane kind, state, lifecycle timestamps, cleanup policy/status, and one
enumerated process-observation conclusion. A running job can therefore say
`observed`, `awaiting_observation`, or `reconciling` without exposing a PID,
raw process-tree timestamp, command, or target path. The server still checks
target existence and aggregates artifact lifecycle locally; the browser cannot
infer process identity or a compatibility payload from the work board.

The production React console validates and installs one coherent snapshot before opening SSE at its cursor. Its Overview page projects Session admission separately from exclusive validation ownership: it prefers the current Cargo-target projection over historical experience summaries, then displays each busy lane as a local wait with bounded elapsed time. It also turns open Failure rows into a plan-level WIP recommendation: operators pull one fixing plan at a time, beginning with the highest-priority oldest open Failure, rather than treating the total as a single bulk intervention. Ordinary Session admission remains open, so operators do not mistake one validation lane for a global drain. Duplicate event IDs are ignored. A gap, malformed event or `resync_required` signal causes a fresh snapshot rather than client-side inference. Coordinator values render as text nodes, never as HTML.

For a `waiting_validation` or `waiting_lease` Session, the same snapshot may
also expose one bounded continuation from its trusted numbered plan: the first
unchecked implementation/documentation slice, never a test-stage checkbox. It
is code-first advice only. The worker claims its concrete scope, completes one
slice, and then selects the next primary-milestone code slice while the queued
validation retains its FIFO position. The coordinator never changes Session
status, consumes a reservation, or claims a foreign scope merely by projecting
this advice; validation returns to the front only after code candidates are
exhausted or it becomes terminal.

The Overview also leads its sync indicator with `codexSessions.lastRun`: an
unchanged successful scan is `安静`, a real lifecycle update is shown as `+N`,
and a failed, partial, diagnostic, or unavailable scan is `需关注`. The 24-hour
experience totals remain visible only as trend context. Historical activity
therefore cannot make a currently quiet synchronizer look like a Session gate.

## Edge Cases

- Expired, consumed, unknown or previous-instance tickets return an authentication error without revealing which digest matched.
- Missing or spoofed Host values are rejected before ticket or cookie lookup.
- Oversized request bodies are rejected before JSON parsing.
- Unknown routes return a bounded v1 not-found envelope.
- SSE capacity exhaustion returns an explicit service-unavailable response; disconnect cleanup releases the slot, and a five-second socket-write deadline prevents a non-reading client from retaining it indefinitely.
- Cursor eviction never produces a partial reconstruction. The consumer must obtain a new coherent snapshot.
- Static traversal, directory requests and missing file extensions cannot enumerate the distribution tree. Hashed assets are immutable; `index.html` remains `no-store`.
- Artifact downloads larger than the direct-response bound require a bounded byte range; multi-range and out-of-root paths fail closed.
- Elevation grants are single-use and actor-bound. Browser roles fall back to Observer when elevation expires.
- Runtime-authenticated preview and confirmation reuse the Session binding recorded on the request, so the protocol does not rely on a browser cookie while still rejecting cross-request actor or daemon changes.
- A rollover with a non-empty managed process-tree PID list fails with `lifecycle_rollover_live_cargo`; it neither terminates nor releases the job. Empty PID lists on unstarted leases are intentionally not treated as a drain blocker.
- An isolated patch that touches another path, changes target mode, carries a credential marker, observes an already-staged target, or sees target blob drift is rejected before `HEAD` publication. A post-publication interruption stays in the ordinary recoverable finalize ledger with its original index snapshot.
- Red operations remain present but disabled through M3. Service drain is preview-only; attempts to confirm it fail closed.

## Verification

The focused tests cover ticket lifetime and reuse, cookie attributes and instance binding, Host/Origin validation, snapshot transaction shape, replay ordering/capacity, HTTP bootstrap and response contracts, static cache/fallback behavior, traversal rejection and artifact range confinement. Snapshot regressions also prove that process observation becomes a bounded enum without emitting a PID. M3 action tests cover closed-catalog/database-enum parsing, role/CSRF/elevation abuse, complete resource fingerprint invalidation, immutable approvals, runtime confirmation, the shared mutation gate, preview-pinned resources, asynchronous cancellable validation and repeated stale-confirm races with no side effect. `test_supervision_actions` additionally proves a rollover keeps admission open while preserving a leased job and that a live Cargo PID tree is rejected without termination. The PowerShell `-ControlledActions` smoke gate performs real browser elevation plus preview/confirm against a temporary coordinator, while the independent web `check` command verifies strict types, fresh impact-diff rendering, production build output and forbidden distribution material.
