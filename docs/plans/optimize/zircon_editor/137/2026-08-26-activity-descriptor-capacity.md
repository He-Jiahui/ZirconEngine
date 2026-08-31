# Editor137 Activity Descriptor Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime191-editor137-performance-batch-20260826ev-v1`

## Problem

Workbench reflection projected view descriptors into separate ActivityView and ActivityWindow
vectors without reserving either output, despite all descriptor kinds being available beforehand.

## Optimization

- Count both descriptor kinds in one read-only pass with saturating counters.
- Allocate each output vector exactly once, preserving descriptor order and projection fields.

## Regression Contract

The `optimization_batch_20260826ev_` Editor tests cover 128 views plus 128 windows, exact kind
counts and order, source shape, and an ignored paired release benchmark emitting
`EDITOR137_ACTIVITY_DESCRIPTOR_CAPACITY_BENCH_V1`. It writes 256 real descriptors 2,048 times per
sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
