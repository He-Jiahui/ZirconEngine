---
related_code:
  - tools/session_coordinator/control_plane/auth.py
  - tools/session_coordinator/control_plane/contracts.py
  - tools/session_coordinator/control_plane/events.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/control_plane/http_security.py
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/control_plane/snapshot.py
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
doc_type: operator-guide
---

# Workflow Control Center

## Current M1 Surface

M1 adds a read-only, loopback-only control facade to the existing Session coordinator. It exposes one coherent snapshot of service health, workflow projections, Sessions, Failures, collaboration state, validation activity, Git baseline state and audit history. It does not add browser-side mutations, a tray process or the final visual shell; those belong to later milestones.

The facade is versioned under `/control/v1`. Existing coordinator commands and authenticated legacy routes remain available and unchanged.

## Opening the Local Control Surface

Start or verify the coordinator first, then request a short-lived Observer bootstrap:

```powershell
.\tools\zircon-session.ps1 start -Json
.\tools\zircon-session.ps1 ui ticket --role observer -Json
.\tools\zircon-session.ps1 ui open
```

`ui open` asks the daemon for a one-time ticket and opens the loopback URL. The ticket expires after 30 seconds, is stored only as a digest, can be consumed once and is bound to the current daemon instance. Successful consumption creates an eight-hour `HttpOnly`, `SameSite=Strict` cookie scoped to `/control`; ordinary output does not print the ticket.

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
- `GET /control/v1/events/stream` streams ordered Server-Sent Events from `Last-Event-ID` or the `cursor` query parameter and rejects capacity overflow instead of silently adding unbounded clients.

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

## Validation State

M1 unit and integration coverage is defined in the files listed in this document header. Full M1 acceptance evidence is recorded under the owning numbered plan after the M1-T gate; this document does not claim that pending gate has passed.
