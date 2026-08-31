# Editor185 Export Job Status Direct Format

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime239-editor185-performance-batch-20260826gs-v1`

## Problem

Every export queue status projection formatted two or three temporary Strings, stored them in a Vec,
and then allocated the final newline-joined String. Repeated UI polling paid for all intermediate
allocations even though it consumes one final label.

## Optimization

- Format the complete status directly in the progress and no-progress branches.
- Preserve output path display, phase wording, progress wording, and newline placement.
- Keep the standalone progress diagnostic formatter available to existing callers.

## Regression Contract

The `optimization_batch_20260826gs_` Editor tests cover queued and running progress text, enforce the
direct-final-format source contract, and provide an ignored paired release benchmark emitting
`EDITOR185_EXPORT_JOB_STATUS_DIRECT_FORMAT_BENCH_V1`. It repeatedly formats a long active export
status and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
