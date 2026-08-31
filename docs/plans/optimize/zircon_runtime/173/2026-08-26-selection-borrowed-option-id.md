# Runtime173 Selection Borrowed Option Id

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime173-editor119-performance-batch-20260826ed-v1`

## Problem

Multiple-selection array updates constructed a fresh `UiValue::Enum(String)` for every candidate
comparison. Duplicate detection and removal therefore allocated once per visited enum instead of
borrowing the stored and requested identifiers.

## Optimization

- Compare enum payloads directly against the borrowed option id.
- Preserve enum-only equality so existing string values retain their prior distinct behavior.
- Reuse the same zero-allocation predicate for selection and removal scans.

## Regression Contract

The shared `optimization_batch_20260826ed_` filter owns three Runtime tests: enum-only semantics,
borrowed source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME173_SELECTION_BORROWED_OPTION_ID_BENCH_V1`, scans 256 values 8,192 times per sample,
reduces allocations per compared enum from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
