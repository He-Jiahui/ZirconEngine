---
title: Runtime60 Specialized Event Batch Extend
category: zircon_runtime
report_id: Runtime60-specialized-event-batch-extend-2026-08-28
date: 2026-08-28
session_id: root-runtime60-single-write-conflict-probe-20260828
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime60 Specialized Event Batch Extend

## Scope

This slice improves the typed event queue batch-write path and contributes local throughput and
allocation evidence toward RECS-P1-72. It does not close event entry/byte budgets, subscription
lifecycle, generation exhaustion, whole-scene profiling, or the Runtime60 parent plan.

## Implementation

`Events::send_batch` now delegates insertion to `Vec::extend` and derives the exact written count
from the queue length delta. This lets `Vec` use its iterator-specific extension path instead of
the owner manually repeating `size_hint`, `reserve`, and one `push` per element. High-water
tracking still runs once after a non-empty batch, and empty batches remain metadata no-ops.

Three Rust regressions preserve exact batch count and order, non-exact filtered iterators, and the
empty-batch high-water behavior. Existing event-store and system tests continue to cover observer
inspection, multiple batch ordering, current/next publication, and next-queue high-water reserve.

## Performance Evidence

The conservative release model writes 65,536 `u64` events per round as 128 batches of 512 for 64
rounds. Every batch passes through an `inspect` adapter that observes each event, matching the
`EventStore` observer wrapper. The queue retains its high-water capacity across rounds. Results use
31 alternating legacy/optimized sample pairs after five warmups and verify an identical checksum.

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| P50 per 64 rounds | 13,137,800 ns | 8,793,300 ns | -33.069% |
| P95 per 64 rounds | 15,614,300 ns | 12,416,600 ns | -20.479% |
| Cold capacity growth operations | 8 | 8 | unchanged |
| Cold requested allocation bytes | 1,044,480 | 1,044,480 | unchanged |

The timed implementations retained checksum `144703424`. A preceding observed-iterator run
measured P50 `13,857,500 -> 9,681,600 ns` (-30.134%) and P95
`19,731,100 -> 16,537,800 ns` (-16.184%). A separate cold allocation profile retained checksum
`196607` for both implementations. A bare-range diagnostic was faster still, but is intentionally
not used as the acceptance result because production wraps batches for observer inspection.

## Validation

- Source contract: 3/3 passed after a confirmed 0/3 initial state.
- Exact Rust formatting and Python contract compilation: passed.
- Scoped `git diff --check`: passed for the exact three candidate paths.
- This task is queued in one Runtime60 five-task asynchronous validation batch. The batch runs 15
  source contracts, 15 `runtime60_batch_` Rust regressions, and six release models for five exact
  performance rows; no local Cargo lane was launched.
- Commit and WeCom publication remain pending independent review and managed validation.

## Remaining Parent-plan Work

RECS-P1-55 through RECS-P1-66 and G25 through G32 remain governed by their identity, budget,
lifecycle, and semantics requirements. RECS-P1-72 still needs fixed product scenes, p99, RSS,
cache, worker utilization, and cross-engine evidence; this local event insertion benchmark does not
substitute for that qualification.
