---
title: Runtime04 Pack Reader and Writer Sort Capacity
category: zircon_runtime
report_id: Runtime04-pack-reader-writer-sort-capacity-2026-08-26
date: 2026-08-26
session_id: optimize-runtime04-direct-reference-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Pack Reader and Writer Sort Capacity

## Scope

Pack validation and emission sort deterministic chunk offsets, asset paths, and chunk hashes.
Those keys are unique at their respective validation boundaries, so stable sorting adds work with
no observable insertion-order contract. Writer output vectors also know the input asset count.

## Implementation

Reader chunk-offset ordering and writer asset-path/chunk-hash ordering now use unstable sorting.
Writer chunk, asset, and deduplicated-asset output vectors reserve the collected asset count.
Canonical pack bytes, duplicate rejection, binary-search ordering, and validation errors remain
unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Stable sort calls | 3 | 0 |
| Writer known-capacity output vectors | 0 | 3 |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME04_PACK_READER_UNSTABLE_OFFSET_SORT_BENCH_V1` and
`RUNTIME04_PACK_WRITER_UNSTABLE_PATH_SORT_BENCH_V1`, each with legacy/optimized p95,
sample/iteration/count data, and sort/reservation reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, canonical-order equivalence tests, and pack extent
checks are prepared. Both ignored benchmarks run in one Runtime crate release command; commit
integration, terminal p95 values, and WeCom delivery remain coordinator-owned.
