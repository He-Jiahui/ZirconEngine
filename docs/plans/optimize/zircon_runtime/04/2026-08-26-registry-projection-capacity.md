---
title: Runtime04 Registry Projection Capacity
category: zircon_runtime
report_id: Runtime04-registry-projection-capacity-2026-08-26
date: 2026-08-26
session_id: optimize-runtime04-direct-reference-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Registry Projection Capacity

## Scope

Registry rebuild paths repeatedly materialize metadata and registry entries. The previous
`Vec::new()` allocations grew while scanning known metadata, building dependency projections, and
publishing each document's registry entries.

## Implementation

The scan, dependency projection, and registry-entry builders now reserve their known lower bounds.
Root-entry detection is evaluated once, preserving the existing entry order, tags, and cardinality.
No registry identity or dependency semantics changed.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Known-capacity vector reservations | 0 | 4 |
| Registry entry order/cardinality | preserved | preserved |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME04_REGISTRY_CAPACITY_PROJECTION_BENCH_V1` with both p95
durations, sample/iteration/entry counts, and reserved-slot reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, and projection equivalence tests are prepared. The
release benchmark is batched with targeted source-entry and dependency-path capacity checks in one
Runtime crate command; commit integration, terminal p95 values, and WeCom delivery remain
coordinator-owned.
