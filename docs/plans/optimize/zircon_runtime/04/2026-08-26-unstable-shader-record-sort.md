---
title: Runtime04 Unstable Shader Resource Record Sort
category: zircon_runtime
report_id: Runtime04-unstable-shader-record-sort-2026-08-26
date: 2026-08-26
session_id: optimize-runtime04-direct-reference-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Unstable Shader Resource Record Sort

## Scope

Shader resource record export deduplicates IDs and locators before publishing a canonical
locator/id order. The comparator is total over the unique output, so stable ordering is not
observable after deduplication.

## Implementation

The final record projection now uses `sort_unstable_by` with the existing locator-then-ID
comparator. Duplicate rejection, canonical order, and exported records remain unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Stable sort calls | 1 | 0 |
| Duplicate identity checks | unchanged | unchanged |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME04_SHADER_RESOURCE_RECORD_SORT_BENCH_V1` with both p95
durations, sample/iteration/record counts, and stable-sort reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, and record equivalence tests are prepared. The
release benchmark is batched with asset registry entry sorting in one Runtime crate command;
commit integration, terminal p95 values, and WeCom delivery remain coordinator-owned.
