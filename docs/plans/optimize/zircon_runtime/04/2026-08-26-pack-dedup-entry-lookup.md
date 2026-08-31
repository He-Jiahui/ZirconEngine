---
title: Runtime04 Pack Dedup Entry Lookup
category: zircon_runtime
report_id: Runtime04-pack-dedup-entry-lookup-2026-08-26
date: 2026-08-26
session_id: optimize-runtime04-direct-reference-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Pack Dedup Entry Lookup

## Scope

`ZrPackDedupTable::insert_or_get` admits repeated content hashes while preserving the first chunk
index. The old path performed a `BTreeMap::get` and then a second `insert` lookup for new hashes.

## Implementation

Admission now uses one `BTreeMap::entry` lookup and handles occupied/vacant states directly. First
index assignment, duplicate identity, table length, and content hash output remain unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| B-tree lookups per admission | 2 | 1 |
| First-index semantics | preserved | preserved |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME04_PACK_DEDUP_ENTRY_LOOKUP_BENCH_V1` with legacy/optimized
p95, sample/iteration/chunk/unique counts, and lookup reduction `2 -> 1`.

## Validation

Scoped rustfmt, diff checks, source contracts, and first-index equivalence tests are prepared. The
ignored benchmark runs in one Runtime crate release command; commit integration, terminal p95
values, and WeCom delivery remain coordinator-owned.
