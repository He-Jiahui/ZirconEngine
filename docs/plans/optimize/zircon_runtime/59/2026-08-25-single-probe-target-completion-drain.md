---
title: Runtime59 Single-probe Target Completion Drain
category: zircon_runtime
report_id: Runtime59-single-probe-target-completion-drain-2026-08-25
date: 2026-08-25
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime59 Single-probe Target Completion Drain

## Scope

This slice removes repeated target-key hash-set probes while startup prewarm synchronizes through
one required asynchronous pipeline. It preserves the initial ready drain, target-absent fast return,
FIFO completion publication, target-inclusive completion count, later-work retention, disconnected
worker failure publication, and all public runtime contracts.

## Implementation

`PipelineAsyncCompiler::finish_pending_through` previously used
`while self.pending.contains(target)`, hashing and probing the target after every received
completion. The optimized path checks target membership once after `drain_ready`, then compares each
received completion key directly with the borrowed target. The target completion is removed and
published before the loop exits, so later queued work remains pending exactly as before.

The regression holds the first worker job, queues the target last in a 64-item FIFO, and uses a
hash-counted key to prove the complete synchronization performs one target membership probe plus one
pending-set removal per completion.

## Performance Contract

| Evidence for a 4,096-completion target-last stream | Retired path | Optimized gate |
| --- | ---: | ---: |
| Target membership hash probes | 4,096 | 1 |
| Completion removals | 4,096 | 4,096 |
| Alternating release benchmark | 11 samples x 32 drains | optimized P95 <= 75% of retired P95 |

The benchmark emits `RUNTIME59_SINGLE_PROBE_TARGET_COMPLETION_DRAIN_BENCH_V1` with both P95
timings, reduction basis points, sample/iteration/completion counts, and target hash-probe counts.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, source-structure gates, and the focused FIFO
boundary regression are required before submission. One managed Runtime59 Cargo invocation filtered
by `runtime59_async_pipeline_` covers this regression and ignored release benchmark together with
the admission clone optimization. Dynamic P95 evidence, integration SHA, and automatic WeCom
performance delivery remain coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime59 still owns execution-runtime lifecycle, task scopes, typed results, cancellation,
dependency validation, thread budgets, timer convergence, shutdown, and product diagnostics. This
micro-optimization does not claim those milestones complete.
