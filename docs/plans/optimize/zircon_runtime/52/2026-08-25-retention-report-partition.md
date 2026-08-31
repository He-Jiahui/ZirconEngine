---
title: Runtime52 Retention, Manifest Query, and Metadata Index Hotpaths
category: zircon_runtime
report_id: Runtime52-retention-report-partition-2026-08-25
date: 2026-08-25
session_id: root-runtime52-retention-partition-20260825
implementation_status: implementation_complete
validation_status: local_validation_passed_managed_validation_pending
---

# Runtime52 Retention, Manifest Query, and Metadata Index Hotpaths

## Scope

This slice reduces data movement in Runtime52 retention preview report construction, removes the
linear canonical-manifest slot lookup, and prevents metadata-only mutations from cloning complete
scene slots and tag vectors. It aligns with DSA-P1-043/044, DSA-P2-008, the DSA-P1-049 query
direction, and the 100K-slot evidence target. It does not claim the parent plan's service,
durability, CAS, restore, product-consumer, persistent indexing, or broader retention-policy work
is complete.

## Implementation

Both committed-archive and staged-upsert retention previews now use one shared ordered partition
helper. The helper advances cursors through the already sorted scope and kept sets while canonical
slot IDs are moved directly into retained and removed vectors. This preserves canonical ordering
without repeating logarithmic set lookups for every slot.

The committed-archive path no longer clones the full canonical ID list a second time or clones all
removed IDs into a temporary `BTreeSet`. The staged-upsert path likewise moves IDs from its existing
canonical set directly into the two report vectors instead of cloning removed IDs twice.

`RuntimeSessionArchiveManifest::slot` now uses binary search over canonical sorted slot summaries.
The isolated helper falls back to the legacy linear search only when a caller deserializes an
out-of-order manifest, preserving the previous compatibility behavior instead of assuming every
public DTO was constructed through the canonical artifact path.

Metadata replacement previously cloned the full `RuntimeSessionSlot` merely to remove its old
secondary-index rows. That copied the complete `DynamicScene`, slot ID, and metadata before the
actual metadata assignment. It then cloned the replacement tag vector before rebuilding tag
indices, and copied strings again when removing empty tag buckets. The mutation now moves the old
metadata out with `mem::replace`; one isolated secondary-index helper borrows both old and new tag
slices, and empty buckets are removed through borrowed tag names. Scene and complete tag-vector
clones on the unique-archive metadata path are both eliminated while updated/tag ordering remains
unchanged.

## Performance Evidence

| Evidence | Before | After / target | Reduction |
| --- | ---: | ---: | ---: |
| 100K slots, retain 50K | 250K report-path string clones | 100K canonical string clones | 60.000% clone reduction |
| Same workload classification | 300K ordered-set membership queries | 150K sorted cursor rows, zero membership lookup | logarithmic hot-loop lookup removed |
| Report classification passes | two canonical scans plus removed-set construction | one ordered partition | redundant pass and set removed |
| Retention release P95, 21 alternating pairs | 135.540 ms | 29.273 ms | 78.403% reduction |
| Manifest lookup P95, 100 queries over 100K slots, 21 alternating pairs | 110.032 ms | 0.260 ms | 99.764% reduction |
| Manifest lookup comparisons | 5,000,005 | at most 1,700 | 99.966% ceiling reduction |
| Metadata index, 32 updates with 1 MiB scene | 32 scene payload clones | 0 | 100.000% clone reduction |
| Metadata index, same workload with 64 tags | 64 complete tag-vector clones | 0 | 100.000% clone reduction |
| Metadata index release P95, 21 alternating pairs | 13.958 ms | 1.637 ms | 88.274% reduction |

The ignored Windows-native release evidence prints `RUNTIME52_RETENTION_PARTITION_BENCH_V1`,
`RUNTIME52_SORTED_LOOKUP_BENCH_V1`, and `RUNTIME52_METADATA_INDEX_BENCH_V1` with scale, structural
counts, nearest-rank P95 values, and the reduction percentage. The retention gate requires
optimized P95 at most 80% of legacy; manifest lookup requires at most 20%; metadata indexing
requires at most 50%. The values above are local diagnostics; the managed terminal rows remain
authoritative.

## Validation

- Static RED proved the isolated production helper and module wiring were absent.
- The first real-module release run exposed a stale unpadded-ID fixture; after correcting the
  fixture, canonical ordering and retained/removed behavior pass against the included production
  helper.
- The first optimized implementation measured only 17.585% P95 improvement and failed the 20%
  gate. Replacing per-row `BTreeSet` lookup with sorted cursors raised the local reduction to
  76.074% in the latest combined run.
- Canonical manifest lookup, missing ID, and deliberately unsorted compatibility behavior pass
  against the included production helper; the 100K-slot lookup gate reports 99.582% P95 reduction.
- Nine source contracts, four isolated behavior tests, all three ignored release gates, scoped
  `rustfmt --check`, and `git diff --check` pass locally. The metadata helper measured
  `legacy_p95_ns=13,958,400 -> optimized_p95_ns=1,636,800`, an `88.274%` reduction.
- The managed three-task ticket will also run the existing Runtime metadata-mutation behavior batch
  so the production `RuntimeSessionArchivePayload` wiring is compile- and behavior-checked rather
  than accepted from source shape alone.
- No local Cargo lane is launched and no coordinator compile is monitored in real time.
- Final validation ticket, terminal marker values, integration commit, and WeCom delivery remain
  pending.

## Documentation Decision

The public session-archive documentation does not promise the internal report-partition algorithm.
Retention selection and canonical ordering are unchanged, so this scoped optimization record is the
only documentation change.

## Remaining Parent-plan Work

The product service owner, platform store, persistent revision/digest CAS, crash durability,
bounded parsing, restore coordination, full policy engine, and product-scale qualification remain
open under Runtime52.
