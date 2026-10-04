---
record_kind: milestone
status: implemented_pending_validation
created_at: 2026-09-12
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: HUB catalog license revision canonicalization
session: astra-hub-catalog-revision-canonical-20260912
---

# Hub catalog license revision canonicalization

## Finding and ownership

The catalog release wire contract serializes revisions as positive decimal
strings (`zircon_hub/src/service/catalog/mod.rs:129-140`). The account client
already rejects aliases such as `"01"`, but `accept_license` parsed the request
revision to `i64` and queried revision `1`, allowing a non-canonical alias to
create an entitlement and return `Publication.revision = "01"`. This violated
the shared catalog contract at the server boundary and was independent of the
account transport route repair.

The exact owner is the local catalog service and its focused tests:

- `zircon_hub/src/service/catalog/mod.rs`
- `zircon_hub/src/service/catalog/tests.rs`

## TDD implementation

The regression `license_acceptance_rejects_noncanonical_revision_alias` first
asserted that accepting revision `"01"` returns `ServiceError::InvalidRequest`
and leaves `entitlements` empty. A source RED probe confirmed the test was
present while the old implementation only parsed `i64` and had no canonical
guard. The smallest fix now rejects non-positive revisions and any value where
`revision.to_string() != request.revision` immediately after parsing, before
fingerprinting or opening the write transaction.

## Validation boundary

- The focused source RED probe passed before implementation: test anchor found,
  parse-only implementation found, canonical guard absent.
- `git diff --check -- zircon_hub/src/service/catalog/mod.rs
  zircon_hub/src/service/catalog/tests.rs` passed after implementation.
- A post-edit structural GREEN probe confirms the canonical guard and focused
  regression coexist; the Rust test remains pending managed execution.
- Related current Hub Web evidence supplied by the coordinator is
  `node --test web/tests/*.test.mjs`: 205 total, 120 passed, 85
  environment-skipped, 0 failed; the run emitted only a nonfatal WebSocket
  port warning. `npm run typecheck` exited 0 from `zircon_hub`. These results
  do not promote skipped browser coverage or real Tauri/native acceptance.
- No Cargo, native, Tauri, or external `E:\Git\zr_vm` command was run.
- Post-edit file hashes are recorded in the session handoff message; this
  record does not claim service or native acceptance.

No commit was created. Source/test/record leases are released after this static
review.
