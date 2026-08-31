---
title: Runtime44 Borrowed Active-state Hot-path Reads
category: zircon_runtime
report_id: Runtime44-borrowed-active-state-hotpath-2026-08-25
date: 2026-08-25
session_id: root-runtime44-two-task-diagnostic-performance-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime44 Borrowed Active-state Hot-path Reads

## Scope

This slice addresses the process diagnostic-log read path under `R44-G08`. It removes avoidable
reference-count traffic from filter checks, direct writes, lazy writes, and sink snapshots. It does
not change sink lifetime ownership, flush/shutdown fencing, queue admission, durability, or
rotation policy.

## Implementation

`ProcessLogController::with_active_state` holds the `ArcSwapOption` guard while a closure borrows
the published `DiagnosticLogState`. The four short synchronous read paths use that borrowed view
instead of `load_full`, so they no longer increment and decrement the state's `Arc` strong count on
every call. Lifecycle operations that must retain state beyond a guard continue to use the owned
`active_state` path.

A deterministic regression test locks the ownership contract by checking the strong count inside
and after a borrowed read. Existing behavior for an unpublished state remains unchanged.

## Performance Evidence

| Evidence | Before | After / target | Reduction |
| --- | ---: | ---: | ---: |
| 1,000,000 active-state reads | 1,000,000 `Arc` increment/decrement pairs | 0 pairs | 100% |
| Release evidence wall time | not gated | <= 3 s | pending terminal evidence |
| Hot paths covered | filter, direct write, lazy write, snapshot | same four paths | behavior preserved |

The ignored Windows-native release evidence emits
`RUNTIME44_BORROWED_ACTIVE_STATE_BENCH_V1`, including read count, legacy and borrowed reference-count
pairs, reduction basis points, elapsed nanoseconds, and the elapsed-time ceiling. Exact timing is
accepted only from coordinator terminal evidence.

## Validation

- Exact-file Rustfmt, scoped diff validation, the ownership regression, lifecycle regressions, and
  ignored release evidence are prepared as one managed Runtime44 two-task batch.
- `runtime44_batch_borrowed_active_state_reads_do_not_clone_the_published_arc` and
  `runtime44_batch_borrowed_active_state_evidence` run with the schedule task in one Cargo release
  invocation; no local Cargo lane is launched.
- Terminal validation evidence, performance marker values, integration commit, and automatic WeCom
  delivery remain pending.

## Remaining Parent-plan Work

This change does not complete Runtime44's typed diagnostic batches, sample-time/completeness
schema, router authority, queue byte budgets, sink isolation, durability receipts, rotation,
shutdown, or crash-recovery milestones.
