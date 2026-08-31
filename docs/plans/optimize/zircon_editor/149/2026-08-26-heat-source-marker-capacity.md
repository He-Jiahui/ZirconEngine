# Editor149 Heat Source Marker Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime203-editor149-performance-batch-20260826fh-v1`

## Problem

Editor weight heatmap painting appended one command per source without reserving the known source
count in the caller-owned command vector.

## Optimization

- Reserve the source count after confirming the heatmap geometry is drawable and before emitting
  the one-command-per-source marker loop.
- Preserve selected marker size/color, source order, coordinate mapping, clipping, opacity, and the
  collapsed geometry zero-allocation path.

## Regression Contract

The `optimization_batch_20260826fh_` Editor tests cover 256 source markers, one-command-per-source
cardinality, final capacity, collapsed zero output and zero capacity, source shape, and an ignored
paired release benchmark emitting `EDITOR149_HEAT_SOURCE_MARKER_CAPACITY_BENCH_V1`. It appends 256
lightweight commands 2,048 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
