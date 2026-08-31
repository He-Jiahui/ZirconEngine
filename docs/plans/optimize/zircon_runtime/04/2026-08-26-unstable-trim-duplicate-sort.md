---
title: Runtime04 Unstable Trim Report Sorting
category: zircon_runtime
report_id: Runtime04-unstable-trim-duplicate-sort-2026-08-26
date: 2026-08-26
session_id: optimize-runtime04-direct-reference-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Unstable Trim Report Sorting

## Scope

Trim report collections are sorted for deterministic output. Missing dependency records and
diagnostic/duplicate strings have no equal-key insertion-order semantics: equal values are
identical in the serialized report, and duplicate paths are deduplicated immediately after sort.

## Implementation

The report now uses `sort_unstable_by` for missing dependencies and `sort_unstable` for duplicate
paths and diagnostics. The existing comparator keys, deduplication boundary, output order for
distinct values, and report fields remain unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Stable report sorts | 3 | 0 |
| Report contents | exact | exact |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME04_PACK_TRIM_REPORT_SORTING_BENCH_V1` with both p95
durations, sample/iteration/count data, and stable-sort reduction `3 -> 0`.

## Validation

Functional report-order equivalence, source contracts, scoped rustfmt, and diff checks are
prepared. This task shares one Windows-native release Cargo lane with the entry lookup task;
commit integration, terminal p95 values, and WeCom delivery remain coordinator-owned.
