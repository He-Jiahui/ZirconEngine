---
title: Editor49 Listener Registry Capacity
category: zircon_editor
report_id: Editor49-listener-registry-capacity-2026-08-26
date: 2026-08-26
session_id: root-editor49-listener-filter-index-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor49 Listener Registry Capacity

## Scope

Listener descriptor snapshots and delivery-route rebuilds copied the bounded listener order into
fresh vectors without reserving the known listener count.

## Implementation

Both projections now reserve `listener_order.len()` before extending. Registration order, enabled
filtering, immutable route snapshots, and listener ownership are unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Descriptor/route start capacity | 0 | listener count bound |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `EDITOR49_LISTENER_REGISTRY_CAPACITY_BENCH_V1` with legacy/optimized
p95, listener count, route capacity bound, and start-capacity reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, order/route equivalence tests, and an ignored release
benchmark are prepared. Commit integration, terminal p95 values, and WeCom delivery remain
coordinator-owned.
