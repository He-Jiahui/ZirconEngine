---
title: Runtime63 Schema Generation Hash Index
category: zircon_runtime
report_id: Runtime63-schema-generation-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime63-two-task-hash-index-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime63 Schema Generation Hash Index

## Scope

This slice replaces only the component schema-generation metadata table with a `HashMap`.
Compiled dynamic-field validation now resolves a component type revision through expected
constant-time lookup. The descriptor owner remains a `BTreeMap`, so descriptor iteration,
registration order independence, duplicate admission, plugin-prefix validation, and persistent
registry equality retain their existing semantics.

Catalog generation advancement, unchanged-descriptor short-circuiting, descriptor removal, and
missing-generation fallback to zero are unchanged.

## Performance Workload

The release workload fills 512 component type IDs with long shared prefixes and performs 4,096
stable generation hits for the final entry.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered generation-index lookups | 4,096 | 0 |
| Hash generation-index lookups | 0 | 4,096 |
| Descriptor ordered-table policy changes | 0 | 0 |
| Allocations on generation hits | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME63_SCHEMA_GENERATION_HASH_INDEX_BENCH_V1`. Acceptance requires hash lookup P95 to be at
least 30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `runtime63_batch_schema_generation_hash_index_preserves_generation_and_order`
  covers per-type generation changes, missing lookups, and unchanged ordered descriptor output.
- `runtime63_batch_schema_generation_hash_index_keeps_descriptor_ordered` locks the
  split ownership between the ordered descriptor table and hash generation metadata.
- `runtime63_batch_schema_generation_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.
- The managed `runtime63_batch_` release gate covers this task and subscription-key hash routing in
  one Cargo invocation: 2 source contracts, 6 Rust tests, and 2 performance rows. Dynamic marker
  values, integration commit, and WeCom delivery remain coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime63 still owns reflection catalog transactions, stable type schemas, compiled property
plans, mutation transactions, inspection publication, and subscription cursors. This slice only
converges per-type schema-generation lookup.
