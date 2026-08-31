---
title: Runtime60 Single-state Event Read Iterator
category: zircon_runtime
report_id: Runtime60-single-state-event-read-2026-08-28
date: 2026-08-28
session_id: root-runtime60-single-write-conflict-probe-20260828
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime60 Single-state Event Read Iterator

## Scope

This slice improves the per-event cursor commit path behind RECS-P2-07 and preserves the existing
G27 partial-consumption behavior. It does not close message cursor convergence, subscription
lifecycle, cursor generation exhaustion, or the Runtime60 parent plan.

## Implementation

`EventReadIter` remains a public opaque struct, but its two independent optional fields are replaced
by one private `EventReadState`: either `Empty` or `Events { inner, cursor }`. This removes the
impossible `inner without cursor` and `cursor without inner` states and reduces every successful
`next()` from two option checks plus an `expect` to one state match.

The cursor now uses direct increment after `inner.next()` succeeds. `EventCursor::read` clamps its
start to the event length, and the slice iterator can only yield while unread elements remain, so a
successful yield proves the cursor is below the bounded slice end. Three Rust regressions cover a
permanently empty iterator, exact commit after partial consumption, and repeated exhaustion after
the tail.

## Performance Evidence

The release model consumes 65,536 `u64` events for 256 rounds per sample. It uses 31 alternating
legacy/optimized sample pairs after five warmups and checks identical cursor plus payload results.
The acceptance result uses the final conservative rerun.

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| P50 per 256 reads | 5,699,000 ns | 2,104,400 ns | -63.074% |
| P95 per 256 reads | 9,449,400 ns | 6,379,600 ns | -32.487% |
| Iterator size | 24 bytes | 24 bytes | unchanged |
| Iterator heap allocations | 0 | 0 | unchanged |

Both implementations retained checksum `549764202496`. A preceding independent run measured P50
`5,165,900 -> 1,682,700 ns` (-67.427%) and P95 `7,239,900 -> 3,401,200 ns` (-53.021%). The result
qualifies the in-memory event-read loop only; it is not an event-store, observer, schedule, or
product-scene benchmark.

## Validation

- Source contract: 3/3 passed after a confirmed 0/3 initial state.
- Exact Rust formatting and Python contract compilation: passed.
- Scoped `git diff --check`: passed for the exact three candidate paths.
- This task is queued in one Runtime60 five-task asynchronous validation batch. The batch runs 15
  source contracts, 15 `runtime60_batch_` Rust regressions, and six release models for five exact
  performance rows; no local Cargo lane was launched.
- Commit and WeCom publication remain pending independent review and managed validation.

## Remaining Parent-plan Work

RECS-P2-07 still requires a unified event/message naming and contract suite. G27 still requires the
explicit acknowledgement decision across Event, Message, and Removed streams. The P1 event
identity, budget, lifecycle, and observer requirements remain open.
