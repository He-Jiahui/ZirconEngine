---
record_kind: milestone
status: implemented_pending_validation
created_at: 2026-09-11
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: HUB catalog route contract repair
session: astra-hub-catalog-route-repair-20260911
---

# Hub catalog route contract repair

The account client now targets the service's canonical license route for both
catalog entitlement reads and catalog-license mutations. The service router is
the authority: `zircon_hub/src/service/http/mod.rs:84-91` exposes
`/v1/organizations/{organization}/licenses` for `GET` and `POST`; the local
deployment probe exercises the same path at
`zircon_hub/deploy/local/probes/service-probe.mjs:213-218`.

Changed files:

- `zircon_hub/src/account/service.rs` — `CatalogEntitlements` request path.
- `zircon_hub/src/account/operations.rs` — `CatalogLicense` operation path.
- `zircon_hub/src/account/package/tests.rs` — expected license route.

The focused observable regression in `src/account/service.rs` specifies the
canonical `/licenses` route. A non-Cargo red/green source check observed the
old `/entitlements` implementation before the fix and `/licenses` afterward;
the Rust test itself remains pending managed execution. The same structural
check confirmed both client paths and the updated test expectation use
`/licenses`.

Managed Rust validation remains pending. The required focused/account tests and
the local-service lib suite must run through the coordinator after the external
`E:\\Git\\zr_vm` dirty-worktree blocker is resolved; this record does not claim
native, Tauri, or full service acceptance.
