# Runtime44 Bounded Critical Log Admission Optimization Record

- Date: 2026-08-19
- Owner: `runtime44-bounded-critical-admission-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_runtime/44-process-diagnostic-log-router-filter-record-queue-sink-durability-rotation-crash-multi-session-product-integration-review.md`, R44-P1-19 / R44-G13 / R44-G35
- Inherited failure: `docs/plans/zircon_runtime/runtime/07/failure-2026-07-19-diagnostic-log-synchronous-sink.md`
- Status: implementation complete; combined managed validation pending

## Problem

When the bounded sink queue was full, `warn` and `error` producers fell back
from `try_send` to an unbounded blocking `send`. A stalled file or console sink
could therefore stop a frame, UI, job, or runtime owner thread indefinitely.

## Change

- `DiagnosticLogSinkSettings` now exposes `critical_enqueue_timeout`, with a
  2ms default and stable settings diagnostics.
- Full-queue Warn/Error admission uses `send_timeout` instead of unbounded
  `send`.
- An expired wait returns failed admission and reuses the existing per-level
  drop counters plus `critical_backpressure_count`; it does not claim that a
  record accepted into RAM is durable.
- Flush and shutdown retain their separate caller-provided fence deadlines.

## Deterministic Performance Evidence

| Saturated critical producer | Before | After | Bound |
|---|---:|---:|---:|
| Warn/Error enqueue wait | unbounded | configured timeout | 2ms default |
| Caller-side file I/O / flush | none | none | unchanged |
| Timeout observability | backpressure only | backpressure + per-level drop | complete |

The release evidence runs 20 alternating Warn/Error admissions against a
deliberately blocked sink. It reports P50/P95/max caller duration and requires
P95 <= 50ms while the configured wait is 2ms. The raw 20-value admission
distribution is emitted so the managed validator can recompute nearest-rank
P50/P95/max and lock the 10/10 drop plus 20-backpressure counters. Exact
Windows timing values are pending the combined coordinator batch.

## Acceptance

- `full_queue_bounds_critical_producer_wait_and_records_drops` covers both
  Warn and Error with a blocked sink and asserts bounded return, failed
  admission, per-level drops, and backpressure count.
- `critical_enqueue_timeout_is_configurable_and_visible` locks the public
  setting and diagnostic projection.
- `critical_admission_timeout_release_benchmark_evidence` emits 20 release
  timing samples and the managed P95 gate.
- Existing best-effort queue, flush, shutdown, durability, and full
  PERF-MVP-434 tests run in the same managed batch rather than per-task Cargo
  invocations.
- The managed PERF-MVP-434 validator requires the complete unique 54-case
  Cartesian product and its deterministic queue/output/RSS fields. It requires
  caller P95 <= 50ms for the 36 cases with 1,000 or 100,000 admission samples.
  The 18 one-log cases are not represented as percentile evidence; their single
  caller sample is independently required to remain <= 50ms. All 54 matrix
  cases therefore have an explicit caller-latency bound.
- Exact-file Rustfmt and scoped diff checks: passed.
- Cargo regressions and release timing: pending.

## Remaining Scope

This slice removes the unbounded producer wait. It does not add a reserved
critical lane, emergency sink, byte quotas, per-owner fairness, durability
receipts, isolated sink workers, or rotation. R44-P1-19 remains partially open
for product policy and R44-P1-20 through R44-P1-39 remain open.
