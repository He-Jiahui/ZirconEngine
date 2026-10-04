---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_runtime/192-runtime-task-execution-job-scheduler-task-graph-worker-domain-scope-cancellation-deadline-shutdown-diagnostics-product-adoption-current-working-tree-review.md
---

# Timer interval overflow

## Current source and repair

`TaskTimer::schedule_interval` used `checked_add(interval).unwrap_or(now)`, so
an unrepresentable first deadline became immediately due. Recurring intervals
used the same fallback and could repeatedly execute at once. The first schedule
now returns typed `CoreError::DeadlineOutOfRange`; a recurring registration that
cannot represent its next deadline is retired without another callback.

This closes RTASK-P2-09 without changing the zero-interval, cancellation,
capacity, or callback-dispatch contracts. The existing timer worker and bounded
registration store remain the owners.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M26 | Reject first-deadline overflow and retire recurring overflow safely | implemented_pending_validation | Two regressions cover typed admission, capacity release, and no immediate repeat; combined batch pending |

Regression scope: zero interval, ordinary interval, maximum representable
duration, first-deadline overflow, recurring deadline overflow, cancellation,
and capacity release. No performance improvement is claimed; this is a safety
and liveness correction.
