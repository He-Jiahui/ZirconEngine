---
title: Runtime129 VM State Hash Indexes
category: zircon_runtime
report_id: Runtime129-vm-state-hash-indexes-2026-08-25
date: 2026-08-25
session_id: root-runtime129-vm-state-hash-index-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime129 VM State Hash Indexes

## Scope

This batch optimizes the reflected VM state validation and schema-migration foundations retained
by Runtime129/Runtime40. It does not turn hot-reload state into a SaveGame participant and does not
close the parent plan's service, identity, capture, envelope, durability, cloud, checkpoint, or
product qualification work.

## Implementation

`VmStateBlob` validation now builds pre-sized `HashSet<&str>` indexes for the authoritative type
table and each object's field names. Type identities, objects, and fields are still visited in
input order, so duplicate/missing diagnostics keep the same first-error precedence.

VM state migration now uses pre-sized hash indexes for target types, source fields, serializable
target fields, rename sources, and rename targets. No hash container is iterated to determine a
serialized or diagnostic result: migrated fields remain in target schema order, type identities
remain in target schema order, and rename/default selection is unchanged.

## Performance Evidence

| Evidence | Before | After / target | Result |
| --- | ---: | ---: | ---: |
| 16,384 source type identities + 16,384 object probes | ordered-tree insert/lookup, `O(log N)` each | pre-sized hash insert/lookup, expected `O(1)` each | ordered-tree lookup depth removed |
| 16,384 fields in one reflected object | ordered-tree duplicate admission, `O(log N)` each | pre-sized hash admission, expected `O(1)` each | ordered-tree lookup depth removed |
| 8,192 target schemas + 8,192 probes | ordered-tree insert/lookup, `O(log N)` each | pre-sized hash insert/lookup, expected `O(1)` each | ordered-tree lookup depth removed |
| 4,096 migrated fields, including 2,048 renames | ordered maps/sets for four field/rename indexes | four pre-sized hash indexes | ordered-tree lookup depth removed |
| Validation release P95 | unbounded | <= 80% of legacy and <= 20 ms | pending terminal evidence |
| Target-type index release P95 | unbounded | <= 80% of legacy and <= 20 ms | pending terminal evidence |
| Field migration release P95 | unbounded | <= 90% of legacy and <= 50 ms | pending terminal evidence |

The ignored Windows release tests alternate 15 legacy/optimized sample pairs and print raw sample
vectors plus nearest-rank P50/P95 under
`RUNTIME129_VM_STATE_VALIDATION_HASH_INDEX_BENCH_V1` and
`RUNTIME129_VM_STATE_MIGRATION_HASH_INDEX_BENCH_V1`. Exact latency and reduction percentages are
accepted only from the coordinator's terminal output.

## Validation

- RED source contracts required hash indexes while production still used `BTreeMap`/`BTreeSet`.
- A behavior test fixes input-order error precedence when an earlier duplicate field and a later
  missing type are both present.
- The migration benchmark compares the complete migrated field sequence against the legacy path
  before timing.
- Static GREEN confirms all six temporary production indexes are pre-sized hash containers.
- Scoped `rustfmt --check` and `git diff --check` pass locally.
- Focused release correctness and performance tests are prepared for one managed Runtime batch.
- Terminal marker data, integration commit, and WeCom delivery remain pending.

## Documentation Decision

The public VM state DTO, JSON representation, migration API, diagnostic variants, and ordering
contracts are unchanged. This numbered optimization record is sufficient for the internal index
change.

## Remaining Parent-plan Work

Runtime129 still requires a real SaveGame/Checkpoint service, explicit participants, stable
identity, consistent bounded capture, versioned envelopes, deterministic migration planning,
durable platform storage, cloud/server policy, product callers, historical fixtures, fault tests,
and full-scale qualification.
