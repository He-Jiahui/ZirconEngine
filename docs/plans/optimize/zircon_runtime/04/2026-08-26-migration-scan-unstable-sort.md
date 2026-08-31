---
title: Runtime04 Migration Scan Unstable Sort
category: zircon_runtime
report_id: Runtime04-migration-scan-unstable-sort-2026-08-26
date: 2026-08-26
session_id: optimize-runtime04-direct-reference-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Migration Scan Unstable Sort

## Scope

Migration inventory scan used stable ordering for directory entries, root-relative identities, and
path sort/dedup collections. Each changed path has a complete comparison key and deduplicates on
that same key, so stable tie preservation is not observable for these bounded projections.

## Implementation

Directory entry ordering, root-relative identity ordering, and the shared path sort/dedup helper
now use unstable sorting. Existing comparison keys and dedup predicates are unchanged. The test
fixture compares reverse-ordered duplicate-heavy results with the legacy stable implementation.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Targeted stable sort sites | 3 | 0 |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME04_MIGRATION_SCAN_UNSTABLE_SORT_BENCH_V1` with identity/path
legacy and optimized p95, sample/iteration/cardinality counts, and `stable_sorts=3->0`.

## Validation

Scoped rustfmt, diff checks, source contracts, and ordering/dedup equivalence tests are prepared.
The ignored benchmark runs in one Runtime crate release command; commit integration, terminal p95
values, and WeCom delivery remain coordinator-owned.
