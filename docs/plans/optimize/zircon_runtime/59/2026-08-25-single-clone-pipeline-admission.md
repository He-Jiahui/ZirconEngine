---
title: Runtime59 Single-clone Pipeline Admission
category: zircon_runtime
report_id: Runtime59-single-clone-pipeline-admission-2026-08-25
date: 2026-08-25
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime59 Single-clone Pipeline Admission

## Scope

This slice removes one redundant key clone from every accepted asynchronous graphics-pipeline
compile admission. It preserves pending-key deduplication, in-flight capacity, bounded channel
submission, full/disconnected rollback, worker ownership, completion routing, and all public runtime
contracts.

## Implementation

`PipelineAsyncCompiler::try_queue` previously cloned the key once for the pending set and a second
time for `PipelineCompileRequest`, retaining the original only so failed `try_send` calls could undo
the pending entry. The optimized path still clones once for the set but moves the original key into
the request. `TrySendError` returns the rejected request, whose key now drives the exact rollback for
both full and disconnected channels.

The regression uses a clone-counted key to prove an accepted admission performs exactly one clone
and still publishes the expected completion.

## Performance Contract

| Evidence for 256 admissions with 2,048-byte keys | Retired path | Optimized gate |
| --- | ---: | ---: |
| Key clones | 512 | 256 |
| Cloned payload bytes | 1,048,576 | 524,288 |
| Alternating release benchmark | 11 samples x 32 batches | optimized P95 <= 70% of retired P95 |

The benchmark emits `RUNTIME59_SINGLE_CLONE_PIPELINE_ADMISSION_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/admission/key-byte counts, clone counts, and cloned payload
bytes.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, source-structure gates, and the focused completion
regression are required before submission. One managed Runtime59 Cargo invocation filtered by
`runtime59_async_pipeline_` covers this regression and ignored release benchmark together with the
target-completion drain optimization. Dynamic P95 evidence, integration SHA, and automatic WeCom
performance delivery remain coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime59 still owns execution-runtime lifecycle, task scopes, typed results, cancellation,
dependency validation, thread budgets, timer convergence, shutdown, and product diagnostics. This
micro-optimization does not claim those milestones complete.
