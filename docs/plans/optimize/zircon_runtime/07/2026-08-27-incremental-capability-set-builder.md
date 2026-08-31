---
title: Runtime07 Incremental Capability Set Builder
category: zircon_runtime
report_id: Runtime07-incremental-capability-set-builder-2026-08-27
date: 2026-08-27
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Incremental Capability Set Builder

## Scope

This slice removes repeated full-vector sorting and deduplication from
`CapabilitySet::with`. It preserves ascending unique builder output, exact case-sensitive string
identity, and the compatibility behavior that repairs unsorted or duplicate vectors populated
through the public field or deserialization. It does not change `contains`, manifest ordering,
capability authorization, or plugin publication.

## Change

- Route `CapabilitySet::with` through one sorted-unique insertion helper.
- On normal sorted/unique storage, find duplicates or insertion positions with binary search and
  insert only a new value.
- Detect externally populated malformed storage and retain the previous push/sort/dedup repair.
- Cover ordered insertion, duplicate suppression, malformed storage repair, and unsorted manifest
  membership behavior with Rust tests and a source contract.

## Deterministic Performance Evidence

The standalone optimized Rust model uses 16,384 admissions over 1,024 unique capability strings
and alternates legacy/optimized order for 15 samples. Both implementations publish identical
output, and the malformed-storage repair is checked separately.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Full-vector sort calls | 16,384 | 0 | 100% |
| Full-vector dedup calls | 16,384 | 0 | 100% |
| Builder P50 | 721.216 ms | 270.246 ms | 62.529% |
| Builder P95 | 778.526 ms | 406.394 ms | 47.800% |

Evidence marker: `RUNTIME07_INCREMENTAL_CAPABILITY_SET_BUILDER_MODEL_V1`.

## Validation

- `python -m unittest tools.tests.test_runtime07_incremental_capability_set_builder_performance_contract -v`: 3 passed.
- Exact-file Rust formatting and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in the asynchronous coordinator batch.

## Remaining Parent-plan Work

Runtime07 still owns the larger catalog generation, resolver, execution-budget, lifecycle lease,
debugger, isolation, and product-scale validation gaps recorded in the canonical review.
