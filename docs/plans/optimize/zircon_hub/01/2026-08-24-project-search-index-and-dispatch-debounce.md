---
title: Hub01 Project Search Index and Dispatch Debounce
category: zircon_hub
report_id: Hub01-project-search-index-dispatch-debounce-2026-08-24
date: 2026-08-24
session_id: optimize-hub01-project-search-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_passed
---

# Hub01 Project Search Index and Dispatch Debounce

## Scope

This batch removes repeated normalization and backend dispatch from the Hub project-search hot
path. It preserves the two project pages' filtering semantics, result order, backend query payload,
200 ms quiet window, cancellation on teardown, and current UI behavior. It does not claim the
parent plan's process supervision, durable operation, project identity, or recovery work complete.

## Implementation

Both project pages now build one normalized search index per project snapshot and filter that index
for each query instead of reconstructing and lowercasing every searchable field on every input.
Backend project search uses one shared debounced dispatcher, so a burst replaces pending work and
component teardown cancels it before a stale dispatch can escape.

## Performance Evidence

Managed Windows-native validation ticket `349c9b7d58ac4e5f9af64ce69fef78d2` ran Node 22.13.1 with
21 alternating sample pairs, 10,000 projects, 32 queries per sample, and nearest-rank percentiles.

| Evidence | Before | After | Change |
| --- | ---: | ---: | ---: |
| Search index P50 | 137.851 ms | 42.362 ms | 69.269% lower |
| Search index P95 | 241.261 ms | 82.919 ms | 65.631% lower |
| Normalizations per sample | 320,000 | 10,000 | 96.875% lower |
| Backend dispatches for 21 x 100-input bursts | 2,100 | 21 | 99.000% lower |

The accepted gate requires optimized P95 to be at most 50% of legacy P95. The observed ratio is
34.369%. The debounce gate requires exactly one optimized dispatch per 100-input burst and also
proved cancellation on teardown.

## Validation

- Isolated dependency restore from the repository lock file: passed.
- Full Hub TypeScript typecheck: passed.
- Focused Node behavior and performance tests: 3/3 passed, 0 failed.
- Managed validation ticket: passed with exit code 0.
- No local Cargo lane or Cargo dry-run was launched, polled, or terminated.

## Remaining Parent-plan Work

Hub01 still requires typed build/launch operations, process creation identity, child ownership,
bounded output, durable receipts, activation recovery, and full project lifecycle qualification.
