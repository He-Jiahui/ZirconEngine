---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_runtime/301-runtime-core-lifecycle-taskgraph-session-shutdown-review.md
  - docs/plans/astra/performance/01-bounded-hotpaths.md
---

# Combined fence admission

## Current source and repair

JobHandle::combine, shared by JobScheduler::wait_all and TaskHandle::wait_all,
allocated a continuation and cloned two handles for every dependency before
checking whether it was already terminal. The rejected registration then read
the failure and cancellation outcome using two additional locks.

Read terminal state and either capture its immutable outcome or register the
continuation under one dependency lock. Allocate callbacks only for pending
dependencies; aggregate terminal outcomes after releasing that lock. Pending
callbacks, dispatcher ownership, failure-over-cancellation precedence, and the
requirement to wait for every input are preserved.

Reference: Unreal Core/Public/Tasks/TaskPrivate.h AddPrerequisites/AddSubsequent
only retains dependency state after successful registration against the closed
subsequent list. Zircon keeps its existing mutex and callback dispatcher, without
introducing a different scheduler or memory-reclamation scheme.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M24 | Avoid rejected continuation allocation and redundant outcome locks for terminal dependencies | implemented_pending_validation | Three correctness regressions and paired101 release comparison; next combined batch pending |

Correctness covers empty/1/1k/10k sets, mixed failed/cancelled/completed states,
duplicate pending dependencies, and registration racing completion. Release
sampling alternates the exact previous combine implementation and the public
new caller for 101 pairs after eight warmups. Test 0/50/100 percent completed
inputs, report p50/p95/p99, and keep input creation and final destruction outside
timing. Fully completed 1k/10k p95 must improve at least 20%; other p95 workloads
must stay within 5%. Allocation elimination is currently a source observation,
not a measured global allocator result. All execution/performance gates pending.

Static independent review found no correctness issue. Samples print the actual
completed count as well as the requested ratio: a single input cannot represent
a half-completed workload. Formatting and scoped diff checks passed.
