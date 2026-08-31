---
title: Runtime56 Incremental Context Ordering
category: zircon_runtime
report_id: Runtime56-incremental-context-ordering-2026-08-25
date: 2026-08-25
session_id: root-runtime56-three-task-input-performance-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime56 Incremental Context Ordering

## Scope

This slice removes repeated full sorting while adding contexts to `InputActionMap`. It preserves
descending priority, ascending ID tie order, duplicate-ID rejection, fluent builders, public context
storage, serde behavior, and compatibility with externally supplied unsorted context vectors. It does
not change Runtime56 context activation, binding evaluation, device state, recording, replay, or input
authority.

## Implementation

The retired `add_context` appended every new context and sorted the complete vector after each
insertion. Building a map incrementally therefore performed one full sort per accepted context. The
optimized path keeps the existing duplicate-ID scan, verifies that the publicly mutable vector still
obeys the canonical order, and uses `partition_point` plus `Vec::insert` on the normal sorted path. If
external construction or deserialization supplied an unsorted vector, the method repairs it once
before inserting the new context.

The regression compares retired and optimized ordering with mixed priorities, ID ties, and a duplicate
ID, then verifies repair of a deliberately unsorted public vector. A source contract requires the
conditional repair, stable insertion point, and shared comparator and rejects append-then-sort.

## Performance Contract

| Evidence per 512-context incremental build | Retired path | Optimized gate |
| --- | ---: | ---: |
| Full vector sorts on a maintained map | 512 | 0 |
| Sorted insertion searches | 0 | 512 |
| Alternating release benchmark | 11 samples x 8 builds | optimized P95 <= 60% of retired P95 |

The benchmark emits `RUNTIME56_INCREMENTAL_CONTEXT_ORDERING_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/context counts, and full-sort counts.

## Validation

The TDD source probe first observed all five append-and-sort indicators, then observed conditional
repair, partitioned insertion, and the shared comparator. Rust 1.94.1 formatting and scoped static
checks passed before batching. The managed `runtime56_batch_` release gate covers this slice together
with in-place action-value normalization and scratch-free binding sorting: 3 source contracts, 10 Rust
tests, and 3 performance rows in one Cargo invocation. Dynamic P95 evidence, integration SHA,
automatic commit, and automatic WeCom performance delivery remain coordinator-owned and pending.
