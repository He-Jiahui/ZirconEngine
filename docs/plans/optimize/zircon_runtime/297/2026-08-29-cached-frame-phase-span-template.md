# Runtime297 Cached Frame Phase-Span Template

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime297-editor243-performance-batch-20260829x-v1`

## Problem

Every combined geometry/sprite frame summary rebuilt the same 10 phase-order span groups from the
13 canonical phases. The builder performed 58 phase-order comparisons, regenerated 13 diagnostic
strings, and incrementally grew the same fixed outer and inner vectors before projecting the
per-frame counts and queue bounds.

## Optimization

- Build the immutable ordered phase-span skeleton once from adjacent canonical phase groups.
- Clone that template into an independent span vector for each frame summary.
- Project only geometry/sprite counts, index bounds, and ordering keys on each frame.
- Preserve the legacy result for every public span field and keep per-summary mutation isolated.

## Regression Contract

The `optimization_batch_20260829x_` Runtime tests compare every projected span field against the
legacy builder and verify that mutating one result cannot affect a later summary. The ignored
paired release benchmark emits `RUNTIME297_CACHED_FRAME_PHASE_SPAN_TEMPLATE_BENCH_V1`. It performs
20,000 complete frame-span builds per sample, reduces steady-state phase-order comparisons from 58
to zero and diagnostic builds from 13 to zero after one process-wide template initialization, and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
