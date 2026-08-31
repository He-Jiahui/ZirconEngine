---
title: Runtime04 Pack Delta Capacity
category: zircon_runtime
report_id: Runtime04-pack-delta-capacity-2026-08-26
date: 2026-08-26
session_id: optimize-runtime04-direct-reference-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Pack Delta Capacity

## Scope

Delta generation scans target assets to classify reused, changed, and removed assets. The previous
path grew five result containers from zero even though base/target asset counts bound every result.

## Implementation

Change/removal collection is centralized in helpers that reserve base and target asset counts.
Delta chunk output reserves the unique changed-hash count before reading payloads. Ordering,
duplicate hash handling, manifest validation, and base-apply behavior remain unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Result vectors starting capacity | 0 | bounded by base/target count |
| Delta chunk vector starting capacity | 0 | unique changed hash count |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME04_PACK_DELTA_CAPACITY_BENCH_V1` with legacy/optimized p95,
sample/iteration/asset counts, and reservation reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, and change/removal equivalence tests are prepared.
The ignored benchmark runs in one Runtime crate release command; commit integration, terminal p95
values, and WeCom delivery remain coordinator-owned.
