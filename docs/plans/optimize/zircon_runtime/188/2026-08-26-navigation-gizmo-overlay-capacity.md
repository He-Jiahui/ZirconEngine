# Runtime188 Navigation Gizmo Overlay Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime188-editor134-performance-batch-20260826es-v1`

## Problem

Navigation gizmo extraction grew line and pick-shape vectors incrementally even though every
triangle always emits three lines and every off-mesh link emits one line plus one pick shape.

## Optimization

- Allocate the line vector once to `3 * triangle_count + link_count` using saturating arithmetic.
- Allocate the pick-shape vector once to `link_count` while preserving all geometry and ordering.

## Regression Contract

The `optimization_batch_20260826es_` Runtime tests cover 128 triangles plus 128 links, exact output
counts, source shape, and an ignored paired release benchmark emitting
`RUNTIME188_NAVIGATION_GIZMO_OVERLAY_CAPACITY_BENCH_V1`. It writes 512 lines and 128 pick shapes
1,024 times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
