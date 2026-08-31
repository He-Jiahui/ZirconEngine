# Editor106 Commandlet Capability Single Buffer

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime160-editor106-performance-batch-20260826dq-v1`

## Problem

Commandlet admission joined unavailable capability names into an intermediate string and then
formatted that value into the final error. Every rejected commandlet built and copied the complete
capability list twice before publishing its structured failure report.

## Optimization

- Sum capability bytes and separators before allocating the error message.
- Reserve the exact final capacity and append the prefix and capability names directly.
- Preserve capability ordering and the existing empty-list message.

## Regression Contract

The shared `optimization_batch_20260826dq_` filter owns three Editor tests: output behavior,
single-buffer source and capacity shape, and an ignored paired release P50/P95 benchmark. The
benchmark emits `EDITOR106_COMMANDLET_CAPABILITY_SINGLE_BUFFER_BENCH_V1`, formats 65,536 reports
per sample, reduces formatter allocations from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
