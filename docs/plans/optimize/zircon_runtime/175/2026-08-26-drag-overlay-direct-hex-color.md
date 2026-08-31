# Runtime175 Drag Overlay Direct Hex Color

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime175-editor121-performance-batch-20260826ef-v1`

## Problem

Drag-overlay style resolution validated every CSS hex string with a complete byte pass, then called
the generic radix parser three or four times over two-character slices. Each overlay color therefore
paid multiple parsing passes despite accepting only fixed six- and eight-byte forms.

## Optimization

- Decode each hexadecimal byte directly from two ASCII nibbles.
- Combine validation and conversion without string slicing or general radix parsing.
- Preserve trimming, required `#`, uppercase/lowercase digits, and six/eight-digit semantics.

## Regression Contract

The shared `optimization_batch_20260826ef_` filter owns three Runtime tests: supported/invalid
forms, direct-decoder source shape, and an ignored paired release P50/P95 benchmark. The benchmark
emits `RUNTIME175_DRAG_OVERLAY_DIRECT_HEX_COLOR_BENCH_V1`, performs 524,288 parses per sample,
replaces one validation pass plus four radix calls with direct byte decoding, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
