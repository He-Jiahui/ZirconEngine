---
title: Runtime09F2 Immutable Lightmap Slot Index
category: zircon_runtime
report_id: Runtime09F2-lightmap-slot-index-2026-08-24
date: 2026-08-24
session_id: root-runtime09f2-lightmap-slot-index-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09F2 Immutable Lightmap Slot Index

## Scope

This slice implements the indexed lookup and stable-frame allocation part of `P1-1`. It does not
claim the parent plan's baked-lighting generation authority, atlas readiness, probe-volume,
streaming, feature ownership, or product benchmark milestones are complete.

## Implementation

`LightmapConsumeContract` now owns a private slot table normalized into ascending instance-id order
at both constructor and deserialization boundaries. Stable sorting preserves the previous
first-match result for malformed duplicate input, while validation still rejects duplicates. The
validated generation can no longer be mutated through the public DTO field.

`slot_for_instance` now uses a partition-point lookup over that immutable table instead of scanning
every slot. Duplicate validation uses adjacent entries and no longer allocates a temporary
`HashSet`. GPU Scene synchronization directly queries the generation-owned table and removes the
per-frame `HashMap` allocation and full-table copy.

## Performance Evidence

| Evidence | Before | After / target | Reduction |
| --- | ---: | ---: | ---: |
| 100K slots x 10K last-slot queries | 1,000,000,000 slot comparisons | <= 170,000 comparisons; <= 500 ms release | 99.983% comparison reduction |
| Stable GPU Scene frame with 100K slots | 1 `HashMap` allocation + 100K inserts/copies | 0 index allocations + 0 index inserts/copies | 100% frame-index work removed |
| Contract duplicate validation | 1 capacity-sized `HashSet` allocation | adjacent scan, 0 temporary allocations | 100% temporary allocation removal |

The ignored Windows-native release evidence prints `RUNTIME09F2_LIGHTMAP_SLOT_BENCH_V1` with slot
and query counts, legacy and indexed comparison bounds, reduction basis points, and elapsed
nanoseconds. Exact elapsed time is accepted only from coordinator terminal evidence.

## Validation

- Constructor ordering, deserialization ordering, duplicate first-match compatibility, generation
  replacement, serde roundtrip, GPU consumer behavior, and the ignored release performance gate are
  prepared for a shared coordinator batch.
- Exact `rustfmt --check`, scoped `git diff --check`, private-field source checks, and removal of the
  renderer-local index are part of the batch's static preflight.
- No local Cargo lane is launched and no compilation is monitored in real time.
- Final validation ticket, terminal marker values, commit integration, and WeCom delivery remain
  pending.

## Remaining Parent-plan Work

The baked-lighting set still lacks an atomic generation that binds atlas revision, descriptor,
probe grid, instance table, GPU handles, and last-good publication. Bake completeness, atlas/device
validation, resource hot reload, probe-volume ownership, feature gates, and product-scale visual
qualification remain open.
