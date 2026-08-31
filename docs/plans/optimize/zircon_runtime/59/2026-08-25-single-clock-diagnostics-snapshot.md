---
title: Runtime59 Single-Clock Diagnostics Snapshot
category: zircon_runtime
report_id: Runtime59-single-clock-diagnostics-snapshot-2026-08-25
date: 2026-08-25
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime59 Single-Clock Diagnostics Snapshot

## Scope

This slice reduces bounded keyed IO diagnostics observation overhead and gives every entry age in a
snapshot one coherent time origin. It preserves oldest-entry selection, empty-state zero duration,
all counters, shutdown reporting, and public task/runtime contracts. It does not claim to close
Runtime59's remaining scheduler, cancellation, timer, shutdown, or product-integration gaps.

## Implementation

`diagnostics_for_state` previously called `Instant::elapsed()` separately for every queued,
suspended, and active entry. A large diagnostics snapshot therefore read the system monotonic clock
once per entry and assigned slightly different observation times within one report.

The optimized path captures `Instant::now()` once, streams all enqueue timestamps through a pure
`oldest_age_at` reduction, and uses `saturating_duration_since` against that shared instant. No
temporary timestamp collection is introduced.

The regression proves exact oldest-age and empty-input behavior at a fixed instant. A source contract
requires one snapshot clock read and rejects per-entry `elapsed()` calls.

## Performance Contract

| Evidence | Retired path | Optimized gate |
| --- | ---: | ---: |
| Monotonic clock reads per 4,096-entry snapshot | 4,096 | 1 |
| Snapshot age origin | one later origin per entry | one coherent origin |
| Alternating release benchmark | 11 samples x 64 scans | optimized P95 <= 65% of retired P95 |

The benchmark emits `RUNTIME59_SINGLE_CLOCK_DIAGNOSTICS_BENCH_V1` with both P95 timings, reduction
basis points, sample/iteration/entry counts, and retired/optimized clock-read counts.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped `git diff --check`, and production source guards passed before
submission (apart from the repository's existing CRLF notice). One managed Runtime batch covers
fixed-clock behavior, the single-clock source contract, and the ignored release benchmark. Dynamic
P95 evidence, integration SHA, and automatic WeCom performance delivery remain coordinator-owned
and pending.
