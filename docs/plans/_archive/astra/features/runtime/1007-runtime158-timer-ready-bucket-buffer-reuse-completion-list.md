---
related_code:
  - zircon_runtime/src/core/runtime/tasks/timer.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/158/2026-09-27-timer-ready-bucket-buffer-reuse.md
tests:
  - zircon_runtime/src/core/runtime/tasks/timer/ready_bucket_reuse_tests.rs
  - zircon_runtime/src/core/runtime/tasks/timer/astra_interval_tests.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Runtime158 timer ready bucket buffer reuse completion list

| Work | Source evidence | Remaining acceptance |
|---|---|---|
| Return the owned due-bucket buffer after filtering. | `next_callbacks` no longer creates a second callback vector. New tests check exact pointer/capacity reuse for ready, mixed and fully coalesced buckets. | Grouped managed compilation and executable storage checks pending. |
| Preserve registration and delivery semantics. | Five regressions cover full prior-function comparisons, ordering, Arc identity, cancellation, renewal, coalescing, closing and real-worker dispatch. Existing shutdown tests and production M26 overflow handling are preserved. | Original four-path source review completed before the gate repair; combined behavior execution pending. |
| Repair the preexisting recurring-overflow test expectation. | The real worker must mark the interval retired, free capacity while the original subscription remains alive, and finish its bounded shutdown join with zero callbacks. The first-deadline typed-rejection gate is unchanged. All five owned paths and hashes are frozen in the Runtime158 record's operational source manifest, with exact inverse evidence and the original support snapshot retained. | Parent independent source review is complete, including worker join before the final zero-callback check; grouped managed execution remains pending. |
| Supply a real Release comparison. | 1/8/64/512 registrations, three workloads, five warmup pairs, 31 alternating sample pairs, raw p50/p95/p99 and bucket throughput. | Local new p95 <= old 110% guard pending; any excess needs diagnosis. Original allocation, CPU, throughput and product qualification budgets remain open. |
