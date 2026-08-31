# Runtime213 Particle Velocity Vertex Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime213-editor159-performance-batch-20260826fr-v1`

## Problem

Runtime particle velocity rendering appended six vertices for every accepted current sprite into a
vector grown from empty even though the current sprite count gave a constant-time output upper
bound before filtering and previous-frame matching.

## Optimization

- Reserve `current sprite count x 6` vertices with saturating arithmetic before the hot projection
  loop.
- Preserve all camera-layer, depth-test, size, alpha, ambiguity, stable-key, previous-state, and
  billboard-basis filtering plus vertex order; rejected sprites only leave unused capacity.

## Regression Contract

The `optimization_batch_20260826fr_` Runtime tests verify six vertices per sprite, saturation,
capacity, ordering, and production source shape, and provide an ignored paired release benchmark
emitting `RUNTIME213_PARTICLE_VELOCITY_VERTEX_CAPACITY_BENCH_V1`. It fills 128 vectors for 4,096
sprites and 24,576 vertices per build and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
