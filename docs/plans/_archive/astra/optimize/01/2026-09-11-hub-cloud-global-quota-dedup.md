---
record_kind: milestone
status: implemented_pending_validation
created_at: 2026-09-11
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: HUB cloud global CAS quota deduplication
session: astra-hub-cloud-global-quota-20260911
---

# Hub cloud global CAS quota deduplication

## Finding and ownership

`P1-61/G28` was open at the lowest Hub cloud-store layer. Before this slice,
`zircon_hub/src/service/cloud/store.rs:257-270` admitted a new tenant blob
against `SUM(bytes), COUNT(*)` over every `cloud_blobs` row. The physical CAS
is digest-addressed and shared across tenants, so one live digest retained by
two tenants could consume the global byte/object ceiling twice before the
physical writer was reached. The contract requires a global unique-file
ceiling (`docs/plans/astra/features/hub/04-retention-recovery.md:132-135`),
and Hub03 assigns quota/retention/GC to P1-61 (`docs/plans/optimize/zircon_hub/03-marketplace-account-auth-organization-cloud-repository-provider-review.md:447-449`)
with G28 covering shared-blob correctness (`docs/plans/optimize/zircon_hub/03-marketplace-account-auth-organization-cloud-repository-provider-review.md:643`).

The lowest owner is the local cloud CAS admission helper. This slice does not
touch HTTP/CloudPage transport, account/catalog routes, team identity, or the
retention/GC owner.

## Implementation and regression

- `zircon_hub/src/service/cloud/store.rs:258-280` now distinguishes a digest
  already present in any tenant from a physically new digest. New digests use
  `global_unique_usage` (`:306-315`), which groups by digest and sums one
  `MAX(bytes)` row per digest. `MAX` keeps malformed cross-tenant metadata
  fail-closed until normal recovery can reject inconsistent physical data;
  project logical bytes/object limits remain unchanged.
- `zircon_hub/src/service/cloud/tests/blobs.rs:150-249` adds
  `global_quota_counts_shared_digest_once_across_tenants` and
  `shared_digest_reuse_is_allowed_when_global_quota_is_full`. The first
  creates two tenant rows for one digest near the global byte ceiling and
  proves a third tenant can upload a new digest; the second fills the unique
  global budget and proves another tenant can reuse the existing CAS digest.
  Fixtures intentionally supply quota-edge metadata without allocating
  multi-gigabyte files; the existing physical writer remains the final
  encrypted-byte/object guard.

## Validation boundary

- `rustfmt --edition 2021 --check zircon_hub/src/service/cloud/store.rs zircon_hub/src/service/cloud/tests/blobs.rs` passed.
- Scoped `git diff --check -- zircon_hub/src/service/cloud/store.rs zircon_hub/src/service/cloud/tests/blobs.rs` passed (only the repository's existing LF/CRLF warning was emitted).
- Post-edit SHA-256: `store.rs` `a74f3f2751ab88e28b3270ae7597e46c39a35c74e2e7d5f10df96626bc1e2826`; `blobs.rs` `804ba50640f8a31e856941a195d047c65596f0e7b290d17f0f7c69f9d8f278b5`.
- No Cargo, native, Tauri, or external `E:\Git\zr_vm` command was run. Managed Rust regression and product acceptance remain pending; this record is not an acceptance claim.

No commit was created. Source/test/record leases are released in the session
handoff after this static review.
