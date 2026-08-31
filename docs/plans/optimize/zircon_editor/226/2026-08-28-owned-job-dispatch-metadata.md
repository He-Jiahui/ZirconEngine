# Editor226 Owned Job Dispatch Metadata

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime280-editor226-performance-batch-20260828ih-v1`

## Problem

Editor job promotion owned the removed pending job but cloned its label, mutex group, and
cancellation token into dispatch state. The original spec was then discarded, adding a bounded
String allocation and redundant Arc reference-count operations to each promoted job.

## Optimization

- Consume the owned `EditorJobSpec` after dependency and admission bookkeeping completes.
- Move label, mutex group, and cancellation token into dispatch state.
- Preserve category accounting, dependency scheduling, completion guards, and mutex-group tails.

## Regression Contract

The `optimization_batch_20260828ih_` Editor tests prove label and mutex-group allocation identity
and prevent the three promotion clones from returning. The ignored paired release benchmark emits
`EDITOR226_OWNED_JOB_DISPATCH_METADATA_BENCH_V1`. It converts 16,384 representative specs per
sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
