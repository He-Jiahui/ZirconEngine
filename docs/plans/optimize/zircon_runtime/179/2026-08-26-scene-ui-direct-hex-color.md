# Runtime179 Scene UI Direct Hex Color

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime179-editor125-performance-batch-20260826ej-v1`

## Problem

The graphics scene UI renderer decoded fixed RGB/RGBA colors with three or four generic radix calls
over UTF-8 slices before converting channels to normalized floats. Every scene UI color repeated
general-purpose parsing work on an ASCII-only format.

## Optimization

- Decode each channel directly from two ASCII nibbles.
- Remove general radix parsing and UTF-8 slicing from scene UI color conversion.
- Preserve RGB/RGBA ordering, opacity multiplication, and malformed-input rejection.

## Regression Contract

The shared `optimization_batch_20260826ej_` filter owns three Runtime tests: color/opacity behavior,
direct-decoder source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME179_SCENE_UI_DIRECT_HEX_COLOR_BENCH_V1`, performs 524,288 parses per sample, replaces four
generic radix calls with direct byte decoding, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
