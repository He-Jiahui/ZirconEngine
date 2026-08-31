---
title: Runtime60 Shared Schedule Batch Systems Optimization
category: zircon_runtime
report_id: Runtime60-shared-schedule-batch-systems-2026-08-24
date: 2026-08-24
session_id: optimize-runtime60-shared-schedule-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime60 Shared Schedule Batch Systems Optimization

## Scope

This slice advances RECS-P1-44 for the compiled schedule control plane. It removes the system-id
copy performed when each compiled parallel batch is submitted. It does not claim the wider
scratch-buffer, system-box, timing, or command-buffer allocation work in RECS-P1-44 is complete.

## Implementation

`ScheduleParallelBatch` now freezes its compact single/pair/triple/vector system representation in
an `Arc`. Batch construction uses `Arc::make_mut`, so the compile path keeps the existing compact
promotion behavior while the execution path can transfer one shared owner into the scheduled job.

`ScheduleParallelExecutor` now borrows the system-id slice from that shared representation. The
previous `system_ids.to_vec()` cloned the outer vector allocation and every owned `String` once per
batch per execution frame.

## Performance Evidence

| Evidence for a 64-system batch | Before | After / target | Change |
| --- | ---: | ---: | ---: |
| Execution-frame heap allocations | 65 | 0 | 100% eliminated |
| Release benchmark P95 | pending coordinator | <= 50% of legacy | acceptance gate |

The deterministic allocation count is one `Vec` allocation plus 64 `String` allocations in the
legacy copy. The optimized path performs one `Arc` reference-count pair and no heap allocation.
The ignored release test uses 21 alternating sample pairs and 5,000 batch transfers per sample. It
prints `PERF_RESULT runtime60_shared_schedule_batch_systems` with nearest-rank P50/P95 timings.
Dynamic timing is not accepted until the coordinator returns terminal evidence.

## Validation

- Source contract: 3/3 passed after a confirmed 3/3 initial red state.
- Exact rustfmt, Python bytecode compilation, and scoped `git diff --check`: passed locally.
- Parallel-schedule behavior batch and ignored release performance evidence: pending the managed
  coordinator batch.
- No local Cargo lane was launched and no Cargo process was terminated.

## Remaining Parent-plan Work

RECS-P1-44 still requires persistent system-box, timing, command-buffer, and scratch storage across
stable frames. The other Runtime60 P0/P1/P2 and acceptance items remain governed by the parent plan
and are not hidden by this allocation slice.
