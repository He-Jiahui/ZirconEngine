---
title: Runtime04 Unstable Asset Registry Entry Sort
category: zircon_runtime
report_id: Runtime04-unstable-asset-registry-entry-sort-2026-08-26
date: 2026-08-26
session_id: optimize-runtime04-direct-reference-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Unstable Asset Registry Entry Sort

## Scope

`AssetRegistryIndex::entries` materializes the deterministic path-ordered registry projection.
Asset paths are unique at admission, so equal comparator keys cannot carry insertion-order
semantics and the stable sorter adds unnecessary work.

## Implementation

The projection now uses `sort_unstable_by` with the existing path comparator. The returned entry
references, ordering, and cardinality are unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Stable sort calls | 1 | 0 |
| Canonical path order | preserved | preserved |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME04_ASSET_REGISTRY_ENTRY_SORT_BENCH_V1` with both p95
durations, sample/iteration/entry counts, and stable-sort reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, and canonical-order equivalence tests are
prepared. The release benchmark is submitted together with shader record sorting in one Runtime
crate command; commit integration, terminal p95 values, and WeCom delivery remain
coordinator-owned.
