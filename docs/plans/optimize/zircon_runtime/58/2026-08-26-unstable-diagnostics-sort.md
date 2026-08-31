---
title: Runtime58 Unstable Diagnostics Sort
category: zircon_runtime
report_id: Runtime58-unstable-diagnostics-sort-2026-08-26
date: 2026-08-26
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime58 Unstable Diagnostics Sort

## Scope

Native live-host diagnostics are required to be deterministic and unique, but duplicate removal
follows immediately after ordering. The shared diagnostics normalizer now uses `sort_unstable`,
preserving the ordered unique result while removing stable-sort bookkeeping for every report that
uses the normalizer.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | --- |
| 4,096 diagnostics, 2,048 unique | stable sort `1` | unstable sort `1` | same deterministic unique output |
| Windows-native release p95 | dynamic evidence pending | <= 95% of legacy p95 | stable sort bookkeeping removed |

The ignored release benchmark alternates 17 samples over 512 iterations and prints
`RUNTIME58_UNSTABLE_DIAGNOSTICS_SORT_BENCH_V1` with both p95 timings, sample/iteration/entry and
unique counts, and stable-sort counts. Exact elapsed-time evidence is accepted only from the
coordinator terminal receipt.

## Validation

- Functional coverage compares legacy and optimized ordered-unique outputs.
- Source contracts prevent stable sorting from returning to the shared normalizer.
- Exact Rustfmt and scoped diff checks are required before submission.
- One Windows-native release Cargo invocation batches this task with the preallocated merge task;
  no per-task Cargo lane is launched.
- Commit integration, benchmark values, record finalization, and automatic WeCom delivery remain
  coordinator-owned and pending.
