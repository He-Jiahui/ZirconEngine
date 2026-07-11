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
  - tools/session_coordinator/server.py
implementation_files:
  - tools/session_coordinator/control_plane/__init__.py
  - tools/session_coordinator/control_plane/auth.py
  - tools/session_coordinator/control_plane/contracts.py
  - tools/session_coordinator/control_plane/events.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/control_plane/http_security.py
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/server.py
plan_sources:
  - docs/superpowers/specs/2026-07-11-workflow-control-center-and-tray-design.md
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
tests:
  - tools/session_coordinator/tests/test_control_auth.py
  - tools/session_coordinator/tests/test_control_events.py
  - tools/session_coordinator/tests/test_control_http.py
  - tools/session_coordinator/tests/test_control_security.py
  - tools/session_coordinator/tests/test_control_snapshot.py
doc_type: module-detail
---

# Session Coordinator Control Plane

## Responsibility

`control_plane` is the versioned browser and automation read facade over the existing coordinator application. It owns API envelopes, Observer bootstrap authentication, loopback request checks, coherent snapshots, ordered event replay and the small HTTP adapter. Domain mutations remain in coordinator services; the control plane must not write SQLite tables directly except for its own web tickets and sessions.

## Module Boundaries

- `contracts.py` defines the v1 envelope, correlation identifiers and sanitized error shape.
- `http_security.py` validates exact loopback Host and Origin values before browser processing.
- `auth.py` persists only SHA-256 ticket and cookie digests and binds both to one daemon instance.
- `snapshot.py` assembles all dashboard panels inside one deferred SQLite transaction.
- `events.py` owns monotonically ordered replay, bounded retention and explicit client-capacity accounting.
- `router.py` maps bounded v1 routes to application services without knowing socket details.
- `http.py` translates `BaseHTTPRequestHandler` input/output, enforces the one MiB body limit and performs SSE streaming.
- `server.py` composes the module and delegates `/control/v1/*` and `/ui/*` before the legacy bearer route handling.

## Invariants

1. The listener remains loopback-only and browser Host validation fails closed.
2. A bootstrap ticket has one role, one daemon instance, one expiry and at most one successful consumption.
3. Browser cookies are opaque, digest-backed, `HttpOnly`, `SameSite=Strict` and scoped to `/control`.
4. Snapshot cursor and panel data come from the same SQLite read transaction.
5. Event replay is ordered and bounded to a 4,096-position logical window with 256-event batches; stale and future cursors both result in refresh-required semantics.
6. Internal exception detail, bearer tokens, maintenance tokens, ticket values and cookie values never enter response bodies.
7. M1 is read-only. No generic command endpoint or direct database mutation endpoint exists; every HTTP verb under `/control/v1` receives the same sanitized v1 envelope, with HEAD suppressing the body.

## Data Flow

The authenticated CLI asks the legacy coordinator route to issue a bootstrap ticket. A browser consumes that ticket through the loopback facade and receives an Observer cookie. Subsequent snapshot, workflow-detail and SSE requests resolve the cookie to an identity, call projection services and serialize only versioned contracts. A daemon restart changes `instance_id`, invalidating previously issued browser credentials.

## Edge Cases

- Expired, consumed, unknown or previous-instance tickets return an authentication error without revealing which digest matched.
- Missing or spoofed Host values are rejected before ticket or cookie lookup.
- Oversized request bodies are rejected before JSON parsing.
- Unknown routes return a bounded v1 not-found envelope.
- SSE capacity exhaustion returns an explicit service-unavailable response; disconnect cleanup releases the slot, and a five-second socket-write deadline prevents a non-reading client from retaining it indefinitely.
- Cursor eviction never produces a partial reconstruction. The consumer must obtain a new coherent snapshot.

## Verification

The focused tests cover ticket lifetime and reuse, cookie attributes and instance binding, Host/Origin validation, snapshot transaction shape, replay ordering/capacity, HTTP bootstrap and response contracts. Existing server and Session regression suites verify that control-plane composition does not break legacy coordinator behavior.
