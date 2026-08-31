# Editor153 MUI Icon Path Fast Path

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime207-editor153-performance-batch-20260826fl-v1`

## Problem

MUI icon JavaScript path parsing copied every unescaped SVG path value one byte at a time into a
string that repeatedly grew, although ordinary path data ends at the first quote and needs no
escape decoding.

## Optimization

- Detect when the first quote-or-backslash delimiter is an unescaped closing quote and copy the
  complete path slice into its final string in one operation.
- Preserve the existing byte decoder for escaped quotes, slashes, control escapes, malformed input,
  opacity parsing, and path element order.

## Regression Contract

The `optimization_batch_20260826fl_` Editor tests cover a 4,096-byte plain path and an escaped-quote
fallback, enforce the fast-path-before-decoder source contract, and provide an ignored paired
release benchmark emitting `EDITOR153_MUI_ICON_PATH_FAST_PATH_BENCH_V1`. It parses a 4,096-byte
path 512 times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
