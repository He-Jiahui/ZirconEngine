# Editor227 Owned Job Cancel Metadata

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime281-editor227-performance-batch-20260828ii-v1`

## Problem

Editor pending-job cancellation owned the job removed from its queue but cloned the job label and
cancellation token before discarding the original spec. Each cancellation therefore performed
redundant Arc reference-count increments and decrements on the synchronous terminal path.

## Optimization

- Consume the owned job spec after it is removed from scheduler state.
- Move the label and cancellation token into the cancel task context.
- Preserve cancellation ordering, terminal accounting, observer delivery, and promotion.

## Regression Contract

The `optimization_batch_20260828ii_` Editor tests prove label allocation identity, single label
ownership, and token propagation while preventing both clones from returning. The ignored paired
release benchmark emits `EDITOR227_OWNED_JOB_CANCEL_METADATA_BENCH_V1`. It converts 65,536 pending
specs per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
