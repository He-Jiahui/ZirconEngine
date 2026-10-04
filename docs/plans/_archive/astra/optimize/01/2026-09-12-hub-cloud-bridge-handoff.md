---
record_kind: dependency_handoff
status: blocked_owner_scope
created_at: 2026-09-12
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: HUB cloud-sync bridge
session: astra-hub-cloud-bridge-handoff-20260912
---

# Hub cloud-sync bridge ownership handoff

## Finding

The local service already exposes project cloud endpoints:
`zircon_hub/src/service/http/mod.rs:93-115` routes head, usage, retention,
maintenance, commit, and blob upload/download. The cloud DTOs and authorization
logic are implemented under `zircon_hub/src/service/cloud/`, including
`snapshot::Snapshot`/`CommitRequest` and the blob store.

The desktop account bridge does not expose those routes. `ServiceRequest` in
`zircon_hub/src/account/service.rs:19-72` has no cloud variants;
`AccountActionRequest` in `zircon_hub/src/tauri_app/account_commands.rs:77-188`
has no cloud action; and `web/src/account/protocol.ts:44-51` has no cloud action
type. `web/src/account/controller.ts` has no cloud transport method, while
`web/src/pages/CloudPage.tsx:199-233` still renders reserved local-delivery
services rather than a cloud head/commit workflow.

## Why this is not an independent one-file slice

The existing account transport only builds GET/JSON-POST requests and parses a
bounded JSON response (`zircon_hub/src/account/service.rs:249-370`). Cloud
commit requires a typed JSON body plus operation/replay semantics; blob upload
and download require PUT/GET binary bodies; and the operation journal has no
cloud commit payload. A real bridge therefore needs coordinated changes to the
request transport, account action DTO/dispatch, response parsing/state, and UI
workflow. Adding only an enum variant would compile neither a usable desktop
path nor a safe journal contract.

Ownership is also split: the prior catalog-route session
`astra-hub-catalog-route-repair-20260911` still owns the account service source
scope while waiting for managed validation, and
`astra-hub-service-followup-20260911` owns `web/src/account/controller.ts` and
`web/tests/account_state.test.mjs`. The current Cloud workflow session
`astra-hub-explicit-workflow-target-20260911` owns `web/src/pages/CloudPage.tsx`
and its browser tests. This handoff deliberately edits none of those paths.

## Dependency-ready implementation order

1. Agree on the smallest first contract: read-only `CloudHead` (organization,
   project, optional snapshot) before commit or binary blob transfer.
2. Add a typed `ServiceRequest::CloudHead`, bounded response DTO validation, an
   `AccountActionRequest::CloudHead`, and the matching TypeScript action/controller
   state. Add source/protocol tests for path, UUID validation, empty head, and
   malformed snapshot rejection.
3. Separately design `CloudCommit` operation identity/replay and conflict
   projection; do not reuse `CatalogLicense` journal payloads.
4. Add blob PUT/GET as a separate binary transport contract with explicit byte
   limits, digest verification, cancellation, and restart behavior.
5. Only after those contracts land, connect `CloudPage` and add browser/native
   workflow evidence.

## Required validation

The eventual bridge requires managed focused Rust tests for account transport,
cloud DTO validation, operation replay/conflict, and HTTP route contracts, plus
the account Web suite and a real service/Tauri probe. No Cargo/native command
was run for this handoff; status remains `blocked_owner_scope` rather than an
implementation or acceptance claim.
