# Runtime161 Window Optional Diagnostics Direct Format

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime161-editor107-performance-batch-20260826dr-v1`

## Problem

Window descriptor diagnostics converted optional primary-window and scale-factor values into owned
strings before interpolating them into their final lines. Each populated or absent value therefore
allocated and copied an intermediate string during diagnostic snapshot generation.

## Optimization

- Branch on optional window values before formatting their complete diagnostic lines.
- Format present numeric values directly into the final string.
- Return the complete absent-value line from one allocation while preserving existing text.

## Regression Contract

The shared `optimization_batch_20260826dr_` filter owns three Runtime tests: present/absent output,
direct-format source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME161_WINDOW_OPTIONAL_DIAGNOSTICS_DIRECT_FORMAT_BENCH_V1`, formats 262,144 default absent
overrides per sample, reduces allocations from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
