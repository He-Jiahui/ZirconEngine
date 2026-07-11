---
related_code:
  - tools/session_coordinator/control_plane/auth.py
  - tools/session_coordinator/control_plane/contracts.py
  - tools/session_coordinator/control_plane/events.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/control_plane/http_security.py
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/control_plane/assets.py
  - tools/session_coordinator/control_plane/artifact_downloads.py
  - tools/session_coordinator/web/src/App.tsx
  - tools/session_coordinator/workflows/projections.py
  - tools/session_coordinator/client.py
  - tools/session_coordinator/cli.py
implementation_files:
  - tools/session_coordinator/control_plane/auth.py
  - tools/session_coordinator/control_plane/contracts.py
  - tools/session_coordinator/control_plane/events.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/control_plane/http_security.py
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/control_plane/assets.py
  - tools/session_coordinator/control_plane/artifact_downloads.py
  - tools/session_coordinator/web/src/App.tsx
  - tools/session_coordinator/workflows/projections.py
  - tools/session_coordinator/client.py
  - tools/session_coordinator/cli.py
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
  - tools/session_coordinator/web/src/__tests__/components.test.tsx
  - tools/session_coordinator/web/src/__tests__/contracts.test.ts
  - tools/session_coordinator/web/src/__tests__/events.test.ts
  - tools/session_coordinator/web/src/__tests__/failureGraph.test.ts
  - tools/session_coordinator/web/src/__tests__/graphLayout.test.ts
  - tools/session_coordinator/web/src/__tests__/navigation.test.ts
  - tools/session_coordinator/web/src/__tests__/reducer.test.ts
doc_type: operator-guide
---

# Workflow Control Center

## Current M2 Surface

M1 adds the read-only, loopback-only control facade. M2 adds the production browser console that renders the coherent snapshot and ordered event stream. The console covers overview, workflow pipelines, Sessions, Failure graph, collaboration leases and delayed patches, Cargo/validation-copy activity, milestone Git evidence, audit, logs and service metadata. It contains no browser-side mutations; controlled actions and the tray process belong to later milestones.

The facade is versioned under `/control/v1`. Existing coordinator commands and authenticated legacy routes remain available and unchanged.

## Opening the Local Control Surface

Start or verify the coordinator first, then request a short-lived Observer bootstrap:

```powershell
.\tools\zircon-session.ps1 start -Json
.\tools\zircon-session.ps1 ui ticket --role observer -Json
.\tools\zircon-session.ps1 ui open
```

`ui open` asks the daemon for a one-time ticket and opens the loopback URL. The ticket expires after 30 seconds, is stored only as a digest, can be consumed once and is bound to the current daemon instance. Successful consumption creates an eight-hour `HttpOnly`, `SameSite=Strict` cookie scoped to `/control`; ordinary output does not print the ticket.

After authentication the daemon serves the production console at `/ui/`. Deep links below `/ui/` use the console shell, while `/control/v1/*` never falls back to HTML. The page remains explicitly read-only even when the daemon itself runs on `main` in read-write mode.

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

Every JSON response uses the v1 envelope and carries a correlation identifier. Unexpected internal exceptions are logged server-side and returned as sanitized error contracts.

## Browser Trust Boundary

The server listens only on `127.0.0.1`. Browser-facing requests must use an exact loopback `Host`; requests with a non-loopback Host fail closed. State-changing browser requests are not part of M1. Later milestones must additionally require an elevated role, CSRF token and serialized service command path before enabling actions.

No bearer token, maintenance capability, ticket value, cookie value or Enterprise WeChat endpoint belongs in Git, API payloads, dashboard logs or screenshots.

## Snapshot and Event Consistency

Snapshot assembly uses one deferred SQLite read transaction. The snapshot cursor and all panels therefore describe one database view. Consumers apply only events after that cursor. The logical replay window retains the latest 4,096 event positions independently of longer audit retention. If a requested cursor is stale or ahead of the database, the server instructs the client to refresh its snapshot instead of guessing at missing state. Each connection reads at most 256 events per batch, has a five-second socket-write deadline, and occupies one of eight explicit client slots.

Workflow lists are projections over coordinator-owned data. M1 creates one stable control-center workflow per Session and a Goal node whose fallback state follows the typed Session lifecycle. Session changes and their workflow projection commit in the same SQLite writer transaction, including maintenance-driven stale/archive transitions. Attempts are database-enforced immutable; a newer accepted attempt becomes current while earlier attempts remain inspectable, and later heartbeats cannot overwrite that accepted state.

## Recovery

- If a ticket expires or is already used, request a new one with `ui open`.
- If the daemon restarts, old tickets and cookies are rejected by daemon-instance binding; reopen the UI.
- If an event cursor is too old, discard the partial client view and fetch `/control/v1/snapshot` again.
- If the coordinator reports degraded baseline or read-only branch state, treat the control view as diagnostic and resolve the underlying shared-workspace condition through the coordinator workflow.
- If the top bar reports a disconnected event stream or a cursor gap, leave the page open. The client discards partial state and loads a fresh coherent snapshot before reconnecting.

## Production Asset Policy

The web package lives at `tools/session_coordinator/web` and is independent from Zircon Hub runtime behavior. It imports only Hub visual tokens, the MUI theme and the generic `HubPanel`/`HubButton` components. It does not import Hub API calls, Tauri bindings, project DTOs or Hub persistence.

`npm run check` performs strict type checking, Node component/model tests, a production Vite build and a distribution audit. Production source maps, absolute development URLs, credential/capability names, webhook material, unhashed assets and unreferenced output files fail that audit. `index.html` is served with `no-store`; content-hashed JavaScript and CSS use one-year immutable caching.

## Validation State

M1 and M2 unit and integration coverage is defined in the files listed in this document header. Accepted milestone evidence is recorded under the owning numbered plan directory.
