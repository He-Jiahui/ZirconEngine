---
title: Editor49 Listener Filter Index
category: zircon_editor
report_id: Editor49-listener-filter-index-2026-08-25
date: 2026-08-25
session_id: root-editor49-listener-filter-index-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor49 Listener Filter Index

## Scope

This slice implements the exact operation-group lookup portion of E-EVT-P1-41 and the compiled
filter direction in M4. It does not claim the parent plan's filter budgets, typed topics,
generation fences, delivery protocol, replay safety, or production consumer work is complete.

## Implementation

`EditorEventListenerFilter::normalized` now canonicalizes, sorts, and deduplicates operation-path
prefixes and exact operation groups once when the registry accepts a filter. Sources are
deduplicated in stable order with a fixed five-entry seen table, so untrusted duplicate-heavy JSON
does not introduce quadratic registration work.

Exact operation-group acceptance now uses binary search over the compiled contiguous group table.
Prefix matching remains a borrowed `starts_with` scan because prefix predicates cannot use exact
binary membership, and source matching remains a maximum-five-variant scan.

## Performance Evidence

| Evidence | Before | After / target | Reduction |
| --- | ---: | ---: | ---: |
| 10K groups x 100K last-group queries | 1,000,000,000 string comparisons | <= 1,400,000 comparisons; <= 500 ms release | 99.86% comparison reduction |
| Duplicate prefixes/groups/sources | retained and rechecked for every event | normalized once on filter update | redundant route work removed |
| Exact group lookup allocation | 0 | 0 | contiguous lookup remains allocation-free |

The ignored Windows-native release evidence prints `EDITOR49_LISTENER_FILTER_BENCH_V1` with group
and query counts, legacy and indexed comparison bounds, reduction basis points, and elapsed
nanoseconds. Exact elapsed time is accepted only from coordinator terminal evidence.

## Validation

- Static RED proved the tests referenced missing production operation-group APIs before the
  implementation was added.
- Normalization, deduplication, exact hit/miss behavior, source order, the hot-path source guard,
  and the ignored release performance gate are prepared for a shared Runtime/Editor coordinator
  batch.
- Scoped `rustfmt --check`, `git diff --check`, and the binary-search/source contract pass locally.
- No local Cargo lane is launched and no compilation is monitored in real time.
- Final validation ticket, terminal marker values, commit integration, and WeCom delivery remain
  pending.

## Documentation Decision

`docs/zircon_editor/core/editor_event.md` already states that listener prefixes are normalized
once and that per-record matching is borrowed. The group index preserves that contract and does
not make the retained architecture document false, so this scoped optimization record is the only
documentation change.

## Remaining Parent-plan Work

Filter item/byte budgets, canonical operation-path parsing, typed topic indexing, generation-aware
mutation receipts, observer backpressure, ABI projection, replay safety, and a production listener
consumer remain open under Editor49.
