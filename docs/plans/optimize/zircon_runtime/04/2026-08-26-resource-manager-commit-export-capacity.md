---
title: Runtime04 Resource Manager Commit and Export Capacity
category: zircon_runtime
report_id: Runtime04-resource-manager-commit-export-capacity-2026-08-26
date: 2026-08-26
session_id: optimize-runtime04-direct-reference-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Resource Manager Commit and Export Capacity

## Scope

Resource mutation preflight and commit assembled operation-sized maps/vectors from zero, sorted a
unique staged order with stable sorting, and built receipt maps without a size bound. Ready-record
export also used stable sorting despite a locator/id comparison key.

## Implementation

Preflight reserves both operation-indexed maps. Commit materializes staged values with exact capacity,
uses unstable ordering for unique insertion order, and reserves receipt maps. Ready-record export
uses the same complete comparator with `sort_unstable_by`; no state transition or output order
changes.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Commit targeted stable sorts | 1 | 0 |
| Ready-record export stable sorts | 1 | 0 |
| Operation-indexed map/vector starts | 0 | bounded by operation/staged count |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME04_RESOURCE_MANAGER_COMMIT_UNSTABLE_ORDER_BENCH_V1` and
`RUNTIME04_RESOURCE_MANAGER_REGISTRY_EXPORT_SORT_BENCH_V1` with legacy/optimized p95 and cardinality
counts.

## Validation

Scoped rustfmt, diff checks, source contracts, ordering equivalence, and public export behavior tests
are prepared. Both ignored benchmarks run in one Runtime crate release command; commit integration,
terminal p95 values, and WeCom delivery remain coordinator-owned.
