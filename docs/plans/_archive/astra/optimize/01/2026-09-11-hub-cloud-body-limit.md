---
record_kind: dependency_handoff
status: blocked_owner_scope
created_at: 2026-09-12
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: HUB local account/team/catalog/cloud slice audit
session: astra-hub-cloud-body-limit-20260911
---

# Hub local cloud slice audit: no independent body-limit repair

## Finding

The suspected cloud upload body-limit gap is not real. The service-wide
`DefaultBodyLimit::max(65536)` in `zircon_hub/src/service/http/mod.rs:116`
only inserts Axum's default-limit extension. Axum-core's implementation and
documentation state that the limit is applied only by `FromRequest`
implementations that call `into_limited_body`; a raw `Request` extractor is not
limited automatically. The cloud handlers consume raw requests and apply
explicit limits themselves:

- `zircon_hub/src/service/http/cloud_routes.rs:68-86` uses
  `to_bytes(request.into_body(), cloud::MAX_MANIFEST_BYTES + 65536)`.
- `zircon_hub/src/service/http/cloud_routes.rs:190-200` uses
  `to_bytes(request.into_body(), cloud::MAX_BLOB_BYTES)`.

Adding route-local `DefaultBodyLimit` layers would therefore be redundant and
would not change the observable behavior. The existing HTTP regression at
`zircon_hub/src/service/http/cloud_routes/tests.rs:380-404` already exercises an
oversized blob and expects the explicit cloud limit to return
`capacity_exceeded`/HTTP 429. The temporary >64 KiB probe was removed without
leaving a source or test change.

## Why no second independent P0/P1 slice landed

The remaining user-visible cloud gap is the typed desktop bridge, not a single
backend route. The local service routes already exist at
`zircon_hub/src/service/http/mod.rs:93-115`, while the account transport,
Tauri action DTO/dispatch, TypeScript protocol/controller, and CloudPage are
separate owners. The exact dependency handoff is recorded in
`2026-09-12-hub-cloud-bridge-handoff.md`; editing one enum or route would not
produce a usable or safe sync workflow.

No other dependency-independent P0/P1 defect was confirmed in the unowned
local cloud/team/account paths during this read-only pass. In particular,
cloud CAS, tenant authorization, manifest/blob bounds, retention leases, and
GC recovery have existing focused source regressions; managed Rust/native
acceptance remains pending under the local-service plan.

## Evidence and validation boundary

- Axum-core source inspected locally:
  `C:/Users/HeJiahui/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/axum-core-0.5.6/src/extract/default_body_limit.rs:17-24,210-224`.
- Cloud handler source inspection confirms explicit `to_bytes` limits above.
- No Cargo, native, Tauri, or external `zr_vm` command was run.
- No production source or test file was changed by this audit.

The next dependency-ready slice is a read-only `CloudHead` bridge contract,
but it must be scheduled with the account transport, Tauri command, protocol,
and CloudPage owners together. This record remains `blocked_owner_scope`; it
does not claim service, desktop, or native acceptance.
