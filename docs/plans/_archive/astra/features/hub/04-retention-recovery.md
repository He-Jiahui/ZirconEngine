---
status: in_progress
source_recheck_required: true
plan_family: docs/plans/astra/features/hub
parent_plan: docs/plans/astra/optimize/01-review-and-repair.md
owner_session: astra-full-domain-20260905
plan_sources:
  - docs/plans/astra/features/hub/02-local-service-authority.md
  - docs/plans/optimize/zircon_hub/03-marketplace-account-auth-organization-cloud-repository-provider-review.md
---

# Hub Retention And CAS Recovery

## Contract

This backend slice implements the S5 retention prerequisite of
[the local service plan](02-local-service-authority.md), following Hub03 P1-61
and G28. It does not close desktop sync, restore UI, key rotation or G28 as a
whole. Source and regression tests require managed validation before acceptance.

Retention is project scoped and persisted in SQLite. An absent policy has
revision `0`, `keepLatest: null`, `trashSeconds: 86400`, and `legalHold: false`.
This preserves the existing retain-all behavior: the 129th retained snapshot
is rejected, with no head or reference changes. No service TOML field changes.

An owner or admin may explicitly set `keepLatest` to 1..128, `trashSeconds`
to 0..2592000, and `legalHold`. A policy mutation carries one canonical
`operationId` and `expectedRevision`; success increments the project retention
revision once. Both `retentionUpdated` and `retentionConflict` are durable,
replayable operation results. The same operation ID with a changed payload
is rejected. Authorization uses current organization membership inside the
immediate write transaction, including replay. Receipt lookup rechecks current
project membership. This revision is distinct from organization policy revision
and snapshot head revision; its audit uses the current organization revision.

The latest N snapshots and current head keep all manifest/blob references.
Older snapshots first enter trash, with a fixed expiration set when first
trashed. Increasing N, reverting to retain-all, or enabling legal hold rescues
the affected existing trash entries. Legal hold protects every snapshot still
present when its transaction commits, and suspends orphan retirement for the
project. Releasing hold starts a fresh grace period. Already purged revisions
and previously authorized tombstones are not restored by a later policy change.
Trash snapshots remain downloadable through their existing blob references.

Each successful upload, including duplicate bytes, creates or renews a 24-hour
lease. The lease is persisted across restart; v6 migration grants existing v5
blob rows the same initial lease without enabling a policy. After lease expiry,
an unreferenced upload in a rolling-policy project may be retired. A client
resuming later must re-upload before committing if the row was collected.
An in-progress commit owns the same SQLite writer lock as maintenance, and
publishes its references before retention decisions in that transaction.

## Persistence And Execution

Schema v6 adds retention policies, snapshot trash, upload leases, prepared
maintenance operations and GC tombstones, plus digest indexes. Existing v5 store UUID/key-fingerprint binding,
root lock, database owner lock, external key and regular-file checks remain.
Only explicit incremental migration admits v6; unknown schema versions fail.

Head publication, expired revision/reference pruning, tenant blob retirement,
GC tombstone insertion, audit and commit receipt share one immediate transaction.
A tombstone is created only after the final tenant blob row disappears. Files
remain accounted until collection; other tenants' retained references and live
uploads continue to own their rows and prevent physical deletion.

Physical GC runs only on tombstones from an already committed transaction.
Its own immediate transaction rechecks all tenants' blob rows, snapshot
references and upload leases while excluding writers. It removes only a
validated digest-named regular ciphertext, verifies its size/digest, and then
acknowledges the tombstone. Missing files are an idempotent retry. A failure
returns an error and leaves unacknowledged work; no successful collection is
reported for a failed batch. In-memory physical counters follow actual deletes
even if database acknowledgement rolls back, and restart rebuilds them.

Each pass admits at most 64 candidate objects and 64 MiB of ciphertext.
Startup retries one pending batch after validating the store binding and all
remaining authoritative blobs. Upload collects previously authorized project tombstones
in a separate writer transaction before its own writes. Snapshot commit changes logical
references without allocating physical CAS bytes, so it does not run prior collection.
After fresh authorization, durable committed/conflict receipts remain replayable even
when an unrelated pending GC object is corrupt or cannot be removed.
Recovery acquires the SQLite writer before the physical accounting mutex, as
upload/collection do; a blocked recovery cannot hold the lock needed by a writer.
Explicit maintenance first commits pruning and retirement with a prepared
operation row, then collects a bounded batch in a new transaction.
There is no timer or new background thread. Idle projects require maintenance
requests; `pendingObjects` includes eligible rows not yet staged as well as
durable tombstones, so callers can repeat while work remains.

Maintenance requires a canonical `operationId`, scoped to the principal and
project fingerprint. Its prepared row reserves that operation ID across all
service mutation kinds; replay returns `operation_id_conflict` for another
payload and a schema trigger also prevents a conflicting terminal receipt.
The first phase records the original pruned-revision count exactly once.
The second phase atomically publishes `maintenanceCompleted`, audit and the
terminal operation receipt, and removes the prepared row. Concurrent retries
return the same terminal result. Further batches require new operation IDs.

After prepare, loss of authority, delete failure, audit failure or transaction
acknowledgement failure returns `operation_outcome_unknown`. The original
operation may already have pruned references or removed files; this response
does not claim rollback. Until completion, `GET /v1/operations/{operationId}`
returns the existing `unknown` state. Resending the same maintenance request
resumes its prepared row after current admin authority is restored. A fresh
unauthorized request is forbidden; a completed replay also requires current
admin authority. Startup may complete GC tombstones but never impersonates the
principal or creates their operation receipt. A subsequent authorized retry
finishes that receipt. `collectedObjects` counts acknowledgements made by the
finishing pass, including missing-file retries; objects acknowledged by startup
or another operation are not counted again. Physical counters track actual
deletes even when the terminal receipt/audit transaction rolls back.

The 128 retained-snapshot ceiling still counts unexpired trash and held
snapshots. Long grace or hold can therefore cause capacity rejection. Explicit
zero-grace rolling retention supports continued editing beyond revision 128;
physical bytes still wait for upload lease expiry and remain subject to the
existing global byte/object ceilings.

## HTTP And Usage

All routes are below
`/v1/organizations/{organization}/projects/{project}/cloud` and retain verified
OIDC, current tenant authority, browser-Origin rejection and bounded workers.

| Route | Contract |
|---|---|
| `GET /retention` | Current policy/revision; project readers; no-store |
| `POST /retention` | Owner/admin policy CAS and terminal operation receipt; 64 KiB JSON limit; conflict HTTP 409 |
| `POST /maintenance` | Owner/admin bounded maintenance with operationId JSON, prepared recovery and terminal receipt; pruned, collected and pending counts; no-store |
| `GET /usage` | Existing limits plus policy, trash count, pending GC count and upload lease duration |

`physicalBytes` includes encrypted tenant allocations and that project's pending
tombstones until acknowledgement; physical deduplication can attribute the same
live digest to multiple tenants. The store's separate physical counter enforces
the global unique-file ceiling. This is a fixed local-service allocation policy,
not a billing claim.

## Recovery Boundaries

A consistent backup must preserve the SQLite snapshot, its referenced CAS files,
the CAS `.store-id`, and the matching external key. Restore may move all paths
while preserving identity. Copying only an old database onto a later pruned CAS
can lack referenced data and is rejected; retention cannot recreate bytes absent
from the backup. Tombstones copied in a consistent backup can replay safely.

Unpublished ciphertext with no database row or tombstone remains counted and
preserved during recovery, as required by the existing backup/adoption contract.
This slice does not infer deletion authority for unknown orphan files. The
Windows database PathGuard rejects hardlinks before migration; other platforms'
canonical-path owner locks do not collapse hardlink aliases. Database
audit/operation-receipt growth, tenant billing plans, offline journal and trash
restore UX need their existing owners; none is accepted by this slice.

## Required Validation

Managed Windows validation must compile and run service cloud, storage and HTTP
tests with `--no-default-features --features local-service --lib`; existing
snapshot, blob, binding and OIDC contracts remain in the test set. New contracts
cover policy replay/authority/conflict, schema v5 adoption, 140 sequential commits,
trash/hold, shared digests, upload expiry/restart, same-base writers racing
maintenance, failed audit rollback, failed file deletion and database-ack recovery.
Maintenance regression sources additionally cover concurrent duplicate operation
IDs, cross-kind reservations, demotion after prepare, completion audit/receipt
faults, restart/query/retry and writer/recovery lock ordering.

Scoped rustfmt/parser and whitespace checks provide source preparation only.
The external dirty `E:/Git/zr_vm` input is now pinned through an isolated commit,
and the existing managed copy route has materialized the matching workspace
topology. The current run receipt is recorded in `02-local-service-authority.md`;
no executed Rust result has been accepted yet. G28 additionally requires release storage scale,
real disk/power-loss fault injection, backup/restore and long offline workflows;
no acceptance or test-pass record is added here until those gates have evidence.
