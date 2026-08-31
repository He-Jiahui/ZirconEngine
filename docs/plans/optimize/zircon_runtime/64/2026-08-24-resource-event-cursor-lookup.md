---
title: Runtime64 Resource Event Cursor Lookup Optimization
category: zircon_runtime
report_id: Runtime64-resource-event-cursor-lookup-2026-08-24
date: 2026-08-24
session_id: root-runtime64-resource-event-cursor-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime64 Resource Event Cursor Lookup Optimization

## Scope

This slice optimizes retained resource-event cursor lookup without changing publication,
coalescing, capacity, TTL, gap, or disconnect policy. It does not claim the parent plan's typed
asset identity, async load, version lease, reload transaction, cache, or publication-generation
milestones are complete.

## Implementation

Retained resource events are stored in ascending sequence order. Coalescing and eviction can create
sequence gaps but do not reorder the remaining entries. `ResourceEventReceiver::len` and
`take_next` now share `first_sequence_index`, which uses `VecDeque::partition_point` to locate the
first retained sequence at or after a cursor in logarithmic time.

The gap contract remains unchanged. A focused regression uses retained sequences `2, 4, 9` and
proves the cursor reports gaps `3 -> 4` and `5 -> 9`, while the exact sequence `4` is delivered once.
A source contract prevents either consumer path from returning to full retained-window scans.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 250,000 tail lookups over 4,096 retained events | 1,024,000,000 comparisons | <= 3,250,000 comparisons; <= 500 ms | >= 99.68% comparison reduction |
| Receiver `len` and `take_next` cursor search | O(retained events) | O(log retained events) | one shared partitioned lookup |

The ignored Windows-native release evidence prints `RESOURCE_EVENT_CURSOR_BENCH_V1` with the exact
optimized elapsed nanoseconds, comparison bound, and reduction. Runtime elapsed values are accepted
only from coordinator terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, sparse-gap behavior, source complexity contract,
  and ignored release performance evidence are submitted as one multi-task coordinator batch with
  Editor48.
- No local Cargo lane is launched and no compilation is monitored in real time.
- Final validation ticket, terminal marker values, and commit integration remain pending.

## Remaining Parent-plan Work

The event stream still uses fixed global retention budgets and `wrapping_add` sequence generation;
publication generation, checked exhaustion, typed version receipts, and resync remain parent-plan
work. Publish-side coalescing still scans backward through the retained window and is not changed by
this cursor-only slice.
