---
title: Runtime11A Direct Property Query Projection
category: zircon_runtime
report_id: Runtime11A-direct-property-query-projection-2026-08-26
date: 2026-08-26
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A Direct Property Query Projection

## Scope

This slice removes whole-node cloning from reflected property queries. It preserves the existing
owned `UiPropertyDescriptor` response, missing-tree/node/property behavior, node index authority,
and all public runtime contracts. It does not treat the reflection mirror as UI mutation authority
or claim the larger Runtime11A reflection-store boundary is complete.

## Implementation

`query_property` previously called `query_node`, cloned the complete `UiNodeDescriptor`, looked up
one property in that temporary node, and cloned the selected property again. The optimized query
uses `node_index` and `trees` through borrowed lookups and clones only the requested property at the
API ownership boundary.

The regression covers an existing property, a missing property, and a missing node. The ignored
release benchmark compares the retired whole-node projection with the direct property projection
on a 256-property node.

## Performance Contract

| Evidence for 64 repeated queries on a 256-property node | Retired path | Optimized gate |
| --- | ---: | ---: |
| Node properties cloned per query | 256 | 0 |
| Requested properties cloned per query | 2 | 1 |
| Alternating release benchmark | 21 paired samples | optimized P95 <= 50% of retired P95 |

The benchmark emits `RUNTIME11A_DIRECT_PROPERTY_QUERY_BENCH_V1` with property/value/iteration
counts, structural clone counts, paired P50/P95 timings, and all raw samples for coordinator-owned
WeCom reporting.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, and source-structure gates are required before
submission. One managed Runtime11A Cargo invocation filtered by `runtime11a_` covers this regression
and ignored release benchmark together with borrowed patch-target resolution. Dynamic P95 evidence,
integration SHA, and automatic WeCom performance delivery remain coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime11A still requires a live-surface reflection generation, bounded notification fanout,
transactional write routing, incremental indexes/deltas, and native accessibility integration. This
focused projection optimization does not claim those milestones complete.
