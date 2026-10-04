---
title: Runtime158 Timer Ready Bucket Buffer Reuse
category: zircon_runtime
report_id: Runtime158-timer-ready-bucket-reuse-2026-09-27
date: 2026-09-27
session_id: astra-optimize-20260926-batch-a
implementation_status: implemented_pending_validation
validation_status: managed_tests_and_release_pending
performance_status: release_measurement_pending
related_code:
  - zircon_runtime/src/core/runtime/tasks/timer.rs
tests:
  - zircon_runtime/src/core/runtime/tasks/timer/ready_bucket_reuse_tests.rs
  - zircon_runtime/src/core/runtime/tasks/timer/astra_interval_tests.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/158-runtime-core-events-tasks-timer-event-bus-task-graph-current-source-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
---

# Runtime158 timer ready bucket buffer reuse

## Current source and change

`next_callbacks` removes an owned `Vec<Arc<TimerRegistration>>` from the timer's
deadline tree, then allocates another vector with the same requested length
to collect ready callbacks. This occurs for every nonempty due bucket,
including buckets whose periodic deliveries are all already pending.
`run_timer` uses this path for production callbacks; bounded keyed I/O uses
the same timer for deadline expiration.

The candidate filters the removed vector in place and returns it. It retains
registration order, removes the same scheduled IDs, renews each accepted
interval before checking delivery coalescing, and preserves cancellation
and closing behavior. The owned input buffer becomes the output buffer.
Interval rescheduling, callback captures, dispatcher envelopes and persistent
timer storage retain their existing allocation needs.

Foreign changes include graph-owned timer shutdown and Astra M26 interval
overflow handling. Shutdown and first-deadline overflow code remain byte
identical. The recurring overflow check and cancellation store are unchanged;
its loop `continue` becomes the equivalent `return false` inside the retain
predicate. The existing shutdown and general timer tests remain byte identical.
The recurring M26 worker test is corrected to match that existing overflow
contract; production overflow behavior is unchanged.

## Executable behavior and ownership evidence

Five new regressions exercise:

- One-shot order and exact registration `Arc` identity; the output must reuse
  the input vector pointer and spare capacity.
- Mixed cancellation, ready one-shots, pending intervals and ready intervals;
  scheduled IDs, renewed buckets, callback order and atomic flags are checked.
- Entirely coalesced intervals returning an empty vector while preserving its
  buffer, followed by resumed production delivery after releasing pending flags.
- A closed timer leaving its buckets and registration flags untouched.
- Real timer-worker delivery of one shared deadline after production
  subscription cancellation. The fixture moves that admitted bucket to due
  under the timer lock, avoiding a scheduling sleep or cancellation race.

The first three cases and every performance sample compare the actual
production `next_callbacks` with the complete frozen original function.
Only the baseline function name changes. Output IDs and the full timer-map
shape, registration flags, capacity and next-ID state must agree. Absolute
renewed deadlines are checked against each call's start/end bounds because
both real functions sample `Instant::now`; the original future bucket is
checked exactly.

The preexisting recurring M26 gate expected one callback even though the
worker retires an overflowing interval before dispatch. The repaired gate
waits for the real worker to set `cancelled`, asserts zero callbacks, and
admits a replacement into the capacity-one timer while the original
subscription is still alive. After dropping the replacement, it joins the
worker through the existing bounded shutdown API and checks zero callbacks
again before dropping the original subscription. The first-deadline
typed-rejection test is unchanged. Both M26 gates still require managed
execution.

Pointer/capacity checks require production to return the already-owned
buffer and require the old function to return a distinct buffer. These are
executable evidence for removing the second callback-vector allocation.
They do not measure total allocator traffic or resident memory.

## Pending Release profile and local guard

Ignored test: `runtime158_timer_ready_bucket_reuse_release_profile`.
Marker: `RUNTIME158_TIMER_READY_BUCKET_REUSE_BENCH_V1`.

| Dimension | Runnable coverage |
|---|---|
| Registrations per due bucket | 1, 8, 64, 512 |
| Workloads | Ready one-shots; mixed cancellation/coalescing; all intervals already pending |
| Samples | Five warmup pairs, 31 measured pairs, alternating old/new order |
| Sample operation | 64 calls to the actual due-bucket function on prepared real `TaskTimerInner` state |
| Report | Raw batch nanoseconds, nearest-rank p50/p95/p99, median-derived buckets/second, OS/architecture/processor/crate version |
| Local guard | New p95 <= 110% of old p95 for every case; 10% is an explicit timing-noise tolerance |

Fixture construction, output-container reservation, correctness checks and
output/fixture destruction are outside timing. The existing inline dispatcher
allows these nonblocking due-bucket calls without extra workers or a new clock
abstraction. The real-worker regression separately covers dispatch integration.

The local guard is pending and is not an original product latency budget.
Any measured excess requires diagnosis. Runtime158 M3/M7 and G03/G09/G12
remain open. Runtime02 section 7.4 still requires complete allocation, CPU,
throughput, platform/core-count profiles and the corresponding product
budgets. This slice makes no measured latency, total-memory or engine-comparison
claim before managed validation.

## Provenance and remaining validation

| Source | SHA-256 |
|---|---|
| Production preimage | `202190f8213f3ec084c88f3fca5ddb069d9c29ee942148a716d45088ef9488d1` |
| Production candidate | `af499c03298708add7cecb3687a3a804f34e1e04bf851a126bdc7ec9ed018133` |
| New regression/profile candidate | `538ed7eea4e4af3f9110d69eb24e72fb0b35841bc6f5e7913a10000ca700d4a0` |
| Corrected M26 worker gate candidate | `94daf09a94123566890a88ea88eb54bba1b4d509e4377ac47ff5a17509054b1f` |

Original four-path claim: `cab4ee2a4f06481886b3d569d907df94`. Supplemental M26
test claim: `656bc6ccfa3646f181a3b2452343b2b2`. Preimages, full frozen function,
protected support files and final static checks use operational prefix
`.codex/state/session-coordinator/async-validation-batches/2026-09-27-astra-optimize-batch-p-runtime158-timer-ready-reuse`.
The `-source-manifest.json` records all five candidate paths and hashes.
The original `-support-preimage.json` is preserved; `-m26-preimage.json`
records the test ownership change and the test/record handoff bytes.
Static evidence includes exact inverse reconstruction of the scoped changes.

The original four-path read-only review preceded the supplemental gate repair.
The parent completed independent review of the repaired test and revised
records, including shutdown join before the final zero-callback assertion.
Managed results and performance acceptance remain open.

Managed compilation, the new regressions, existing cancellation/slow callback/
panic/shutdown/M26 overflow tests and Release measurements remain pending the
parent's grouped asynchronous batch. No Cargo or validation submission was
run by this implementation agent.
