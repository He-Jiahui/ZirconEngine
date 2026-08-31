---
title: Runtime58 Unstable Component Identity Dedup
category: zircon_runtime
report_id: Runtime58-unstable-component-identity-dedup-2026-08-26
date: 2026-08-26
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime58 Unstable Component Identity Dedup

## Scope

Runtime registration replay gathers component type IDs from the descriptor and runtime entry
manifests, then sorts and deduplicates them before building the access authority. The required
deterministic unique result now uses `sort_unstable`, avoiding stable-sort bookkeeping without
changing component admission or ordering.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | --- |
| 4,096 component IDs, 2,048 unique | stable sort `1` | unstable sort `1` | same sorted unique identity set |
| Windows-native release p95 | dynamic evidence pending | <= 95% of legacy p95 | stable sort bookkeeping removed |

The ignored release benchmark alternates 17 samples over 512 iterations and prints
`RUNTIME58_UNSTABLE_COMPONENT_ID_DEDUP_BENCH_V1` with both p95 timings, sample/iteration/entry
counts, unique counts, and stable-sort counts. Exact elapsed-time evidence is accepted only from
the coordinator terminal receipt.

## Validation

- Functional coverage retains the exact unique component count and compares stable/unstable
  results.
- Source contracts prevent the old stable sort from returning to component identity preparation.
- Exact Rustfmt and scoped diff checks are required before submission.
- One Windows-native release Cargo invocation batches this task with the diagnostic dedup task;
  no per-task Cargo lane is launched.
- Commit integration, benchmark values, record finalization, and automatic WeCom delivery remain
  coordinator-owned and pending.
