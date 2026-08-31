---
title: Runtime04 Single-lookup Trim Asset Collection
category: zircon_runtime
report_id: Runtime04-single-lookup-trim-asset-collection-2026-08-26
date: 2026-08-26
session_id: optimize-runtime04-direct-reference-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Single-lookup Trim Asset Collection

## Scope

`ZrPackTrimPlanner::collect_assets` rejects duplicate asset paths while retaining the first input
asset. The previous `contains_key` followed by `insert` performed two B-tree lookups for every
unique and duplicate candidate.

## Implementation

The collector now uses `BTreeMap::entry`: an occupied entry emits the existing duplicate
diagnostic and duplicate path, while a vacant entry inserts the asset exactly once. The first
asset remains authoritative, and duplicate diagnostics and ordering are unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| B-tree lookups per candidate | 2 | 1 |
| First-asset retention | preserved | preserved |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME04_PACK_TRIM_ENTRY_LOOKUP_BENCH_V1` with both p95 durations,
sample/iteration/asset/duplicate counts, and lookup reduction `2 -> 1`.

## Validation

Scoped rustfmt, diff checks, source contracts, collector equivalence, and deterministic planner
report regressions are prepared. The release benchmark is batched with report sorting in one
Cargo invocation; commit integration, terminal p95 values, and WeCom delivery remain
coordinator-owned.
