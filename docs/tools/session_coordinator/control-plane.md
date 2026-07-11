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
  - tools/session_coordinator/control_plane/assets.py
  - tools/session_coordinator/control_plane/artifact_downloads.py
  - tools/session_coordinator/control_plane/actions/catalog.py
  - tools/session_coordinator/control_plane/actions/models.py
  - tools/session_coordinator/control_plane/actions/permissions.py
  - tools/session_coordinator/control_plane/actions/fingerprint.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/actions/service.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/patches.py
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/web/src/pages/ActionsPage.tsx
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
  - tools/session_coordinator/control_plane/assets.py
  - tools/session_coordinator/control_plane/artifact_downloads.py
  - tools/session_coordinator/control_plane/actions/catalog.py
  - tools/session_coordinator/control_plane/actions/models.py
  - tools/session_coordinator/control_plane/actions/permissions.py
  - tools/session_coordinator/control_plane/actions/fingerprint.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/actions/service.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/patches.py
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/web/src/pages/ActionsPage.tsx
  - tools/session_coordinator/web/src/components/actions/ActionDialog.tsx
  - tools/session_coordinator/web/src/components/actions/RiskSummary.tsx
plan_sources:
  - docs/superpowers/specs/2026-07-11-workflow-control-center-and-tray-design.md
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
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
  - tools/session_coordinator/web/src/__tests__/actions.test.tsx
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
- `server.py` composes the module and delegates `/control/v1/*` and `/ui/*` before the legacy bearer route handling.

## Invariants

1. The listener remains loopback-only and browser Host validation fails closed.
2. A bootstrap ticket has one role, one daemon instance, one expiry and at most one successful consumption.
3. Browser cookies are opaque, digest-backed, `HttpOnly`, `SameSite=Strict` and scoped to `/control`.
4. Snapshot cursor and panel data come from the same SQLite read transaction.
5. Event replay is ordered and bounded to a 4,096-position logical window with 256-event batches; stale and future cursors both result in refresh-required semantics.
6. Internal exception detail, bearer tokens, maintenance tokens, ticket values and cookie values never enter response bodies.
7. M1-M2 remain read-only for Observer sessions. M3 mutations require a short-lived elevated role, a Session binding when applicable, CSRF validation and a catalog entry. No generic command, SQL or direct database mutation endpoint exists.
8. API paths never fall back to HTML. Only extensionless navigation below `/ui/` may receive the SPA shell.
9. Artifact paths are database-selected, canonically confined and never disclosed to the browser.

## Data Flow

The authenticated CLI asks the legacy coordinator route to issue a bootstrap ticket. A browser consumes that ticket through the loopback facade and receives an Observer cookie. A local runtime caller may separately issue a one-use elevation grant bound to the same actor, daemon instance and optional Session. Consuming it rotates the CSRF token and grants a short-lived Operator, Committer or Maintainer role; Maintainer issuance additionally requires the local maintenance capability. A daemon restart changes `instance_id`, invalidating all previous browser credentials and grants.

For a controlled action, the browser submits only a catalog kind and its exact typed parameters. The service checks permission and Session scope, captures an impact summary, the exact Patch/Validation/Failure resource set, and a state fingerprint, then returns a two-minute preview with an explicit confirmation phrase. Confirm records an immutable approval reason and executes only if the identity, daemon instance, scope, phrase and current fingerprint still match. Fingerprint revalidation and the typed side effect share the same daemon mutation gate, closing the preview/execute race against CLI commands, maintenance and other actions. Any intervening state change yields `action_state_changed`; the UI obtains a new preview and displays its impact diff without automatically retrying the mutation.

The production React console validates and installs one coherent snapshot before opening SSE at its cursor. Duplicate event IDs are ignored. A gap, malformed event or `resync_required` signal causes a fresh snapshot rather than client-side inference. Coordinator values render as text nodes, never as HTML.

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
- Red operations remain present but disabled through M3. Service drain is preview-only; attempts to confirm it fail closed.

## Verification

The focused tests cover ticket lifetime and reuse, cookie attributes and instance binding, Host/Origin validation, snapshot transaction shape, replay ordering/capacity, HTTP bootstrap and response contracts, static cache/fallback behavior, traversal rejection and artifact range confinement. M3 action tests cover closed-catalog/database-enum parsing, role/CSRF/elevation abuse, complete resource fingerprint invalidation, immutable approvals, runtime confirmation, the shared mutation gate, preview-pinned resources, asynchronous cancellable validation and repeated stale-confirm races with no side effect. The PowerShell `-ControlledActions` smoke gate performs real browser elevation plus preview/confirm against a temporary coordinator, while the independent web `check` command verifies strict types, fresh impact-diff rendering, production build output and forbidden distribution material.
