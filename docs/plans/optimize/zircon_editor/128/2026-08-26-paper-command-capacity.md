# Editor128 Paper Command Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime182-editor128-performance-batch-20260826em-v1`

## Problem

An elevated retained-host Paper can append three shadow layers, its surface, and a dark overlay.
Starting from an empty or tight command vector, the fifth `HostPaintCommand` crosses Rust's initial
four-element growth and reallocates the full command buffer.

## Optimization

- Define the five-command Paper upper bound beside the primitive builder.
- Reserve all five additional slots after identity/layout validation and before emitting layers.
- Preserve outlined/elevation branching, command order, z values, clipping, and opacity.

## Regression Contract

The shared `optimization_batch_20260826em_` filter owns three Editor tests: five-layer behavior,
capacity source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR128_PAPER_COMMAND_CAPACITY_BENCH_V1`, builds five real `HostPaintCommand` values 32,768
times per sample, reduces empty-buffer allocations from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
