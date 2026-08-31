# Runtime178 Style Direct Hex Color

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime178-editor124-performance-batch-20260826ei-v1`

## Problem

The shared Runtime UI style parser decoded fixed six- and eight-digit colors with three or four
generic radix calls over UTF-8 string slices. Theme, token, button, and inline style resolution all
paid general-purpose parsing overhead for an ASCII-only format.

## Optimization

- Decode each color channel directly from two ASCII nibbles.
- Remove UTF-8 string slicing and general radix parsing from the shared style path.
- Preserve RGB/RGBA byte order, named style handling, uppercase/lowercase digits, and invalid-input
  rejection.

## Regression Contract

The shared `optimization_batch_20260826ei_` filter owns three Runtime tests: color/invalid behavior,
direct-decoder source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME178_STYLE_DIRECT_HEX_COLOR_BENCH_V1`, performs 524,288 parses per sample, replaces four
generic radix calls with four direct byte decodes, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
