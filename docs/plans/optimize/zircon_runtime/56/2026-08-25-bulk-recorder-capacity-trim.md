---
title: Runtime56 Bulk Recorder Capacity Trim
category: zircon_runtime
report_id: Runtime56-bulk-recorder-capacity-trim-2026-08-25
date: 2026-08-25
session_id: root-runtime56-bulk-button-release-20260825
implementation_status: implementation_complete
validation_status: managed_validation_queued
validation_ticket: 2a6e907071354f28b114db0c90a9074c
---

# Runtime56 Bulk Recorder Capacity Trim

## Scope

This slice removes the per-record loop used when an enabled input-event recorder is reconfigured to
a smaller capacity. It preserves the newest records, FIFO drain order, saturating discarded-record
accounting, enabled and disabled transitions, sequence handling, public status values, and serialized
record contents. It does not change steady-state event recording or the Runtime56 input authority.

## Implementation

The retired path repeatedly called `VecDeque::pop_front` and updated the saturating discard counter
once for every excess record. The optimized path computes the excess once, drains that complete prefix
as one range, and applies one equivalent saturating counter update. An empty or growing-capacity trim
returns without touching the deque or counter.

The regression compares retired and optimized records and counters while forcing counter saturation,
then verifies the retained sequence is the newest suffix. A source contract requires one ranged drain
and one counter update and rejects the retired per-record loop.

## Performance Contract

| Evidence per 16,384-to-256 record trim | Retired path | Optimized gate |
| --- | ---: | ---: |
| Front-pop operations | 16,128 | 0 |
| Range-drain calls | 0 | 1 |
| Saturating counter updates | 16,128 | 1 |
| Alternating release benchmark | 11 samples x 16 trims | optimized P95 <= 80% of retired P95 |

The benchmark emits `RUNTIME56_BULK_RECORDER_CAPACITY_TRIM_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/record/capacity counts, front-pop operations, range-drain
calls, and counter updates.

## Validation

The scoped TDD source probe first observed the missing bulk helper and retained per-record loop, then
observed one ranged drain, one saturating counter update, and no loop on the optimized path. Rust
1.94.1 formatting and scoped static checks passed before batching. One managed Runtime56 batch covers
this slice together with scratch-free binding sorting, including equivalence, source contracts, and
both ignored release benchmarks. Dynamic P95 evidence, integration SHA, automatic commit, and
automatic WeCom performance delivery remain coordinator-owned and pending.
