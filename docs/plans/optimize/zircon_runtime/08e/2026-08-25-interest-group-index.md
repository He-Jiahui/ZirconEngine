---
title: Runtime08E Interest Group Index
category: zircon_runtime
report_id: Runtime08E-interest-group-index-2026-08-25
date: 2026-08-25
session_id: root-runtime08e-interest-groups-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime08E Interest Group Index

## Scope

This slice implements the group-condition lookup portion of 08E P1-16. It does not claim the
parent plan's World replication, per-connection baseline/ACK, relevancy graph, dormancy,
transport, or 100K-object product qualification is complete.

## Implementation

`SyncInterestDescriptor` now owns a sorted, duplicate-free group table. Builder insertion and
deserialization both normalize the table, the mutable field is no longer public, and callers can
inspect it through a borrowed slice. JSON remains an array and therefore keeps the existing wire
shape.

`allows_group` now uses binary search over the immutable contiguous table. Both visible-snapshot
collection and scheduled replication already call this method once for every candidate snapshot,
so the change removes a repeated linear membership scan without touching the plugin manager files
owned by other sessions.

## Performance Evidence

| Evidence | Before | After / target | Reduction |
| --- | ---: | ---: | ---: |
| 10K groups x 100K last-group queries | 1,000,000,000 string comparisons | <= 1,400,000 comparisons; <= 500 ms release | 99.86% comparison reduction |
| Duplicate groups from builder or JSON | retained and rechecked on every query | normalized once | redundant query work removed |
| Stable membership lookup allocation | 0 | 0 | contiguous lookup remains allocation-free |

The ignored Windows-native release evidence prints `RUNTIME08E_INTEREST_GROUP_BENCH_V1` with the
group/query counts, legacy and indexed comparison bounds, reduction basis points, and elapsed
nanoseconds. Exact elapsed time is accepted only from coordinator terminal evidence.

## Validation

- Builder ordering, duplicate removal, deserialization normalization, hit/miss behavior, and the
  ignored release performance gate are prepared for a shared Runtime/Editor coordinator batch.
- Exact `rustfmt --check`, scoped `git diff --check`, private-field/source checks, and the absence
  of the former linear membership scan are part of the batch's static preflight.
- No local Cargo lane is launched and no compilation is monitored in real time.
- Final validation ticket, terminal marker values, commit integration, and WeCom delivery remain
  pending.

## Documentation Decision

`docs/engine-architecture/runtime-network-extension.md` describes group-filtering semantics but
does not expose or promise mutable `Vec` storage, so it remains truthful and requires no change.

## Remaining Parent-plan Work

The replication path still materializes and sorts the full candidate set, lacks compiled numeric
group/condition identities, and has no per-connection baseline, ACK, spatial relevancy, dormancy,
or production transport integration. Those remain under 08E M5 and its product-scale gates.
