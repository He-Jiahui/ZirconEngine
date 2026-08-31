---
title: Runtime58 Unstable Diagnostic Dedup
category: zircon_runtime
report_id: Runtime58-unstable-diagnostic-dedup-2026-08-26
date: 2026-08-26
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime58 Unstable Diagnostic Dedup

## Scope

The native live-host replay and bridge-reload reports still need deterministic diagnostics and
skipped-plugin ordering, but duplicates are removed immediately afterward. Their stable sorts were
replaced with `sort_unstable`, preserving the ordered unique report contract while reducing sort
bookkeeping on the hot aggregation path.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | --- |
| 8,192 report entries, 4,096 unique | stable sorts `2` | unstable sorts `2` | equal deterministic ordering after dedup |
| Windows-native release p95 | dynamic evidence pending | <= 95% of legacy p95 | stable sort bookkeeping removed |

The ignored release benchmark alternates 17 samples over 512 iterations and prints
`RUNTIME58_UNSTABLE_DIAGNOSTIC_DEDUP_BENCH_V1` with both p95 timings, sample/iteration/entry
counts, unique counts, and stable-sort counts. Exact elapsed-time evidence is accepted only from
the coordinator terminal receipt.

## Validation

- Functional coverage compares the legacy and optimized ordered-unique results.
- Source contracts prevent a stable sort from returning to either report finalizer.
- Exact Rustfmt and scoped diff checks are required before submission.
- One Windows-native release Cargo invocation batches this task with the component identity task;
  no per-task Cargo lane is launched.
- Commit integration, benchmark values, record finalization, and automatic WeCom delivery remain
  coordinator-owned and pending.
