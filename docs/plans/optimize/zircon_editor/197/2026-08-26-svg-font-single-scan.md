# Editor197 SVG Font Single Scan

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime251-editor197-performance-batch-20260826he-v1`

## Problem

SVG font detection searched the complete UTF-8 document separately for `<text`, `<tspan`, and
`font-family`. The common no-font icon path therefore traversed a large SVG three times before
parsing could continue without the system font database.

## Optimization

- Preserve the initial UTF-8 validation and case-sensitive marker contract.
- Traverse SVG bytes once.
- Check marker prefixes only at `<` and `f` candidate bytes, returning on the first match.

## Regression Contract

The `optimization_batch_20260826he_` Editor tests preserve all three markers, case sensitivity, and
invalid UTF-8 rejection; enforce the single byte scan; and provide an ignored paired release
benchmark emitting `EDITOR197_SVG_FONT_SINGLE_SCAN_BENCH_V1`. It repeatedly scans a 64 KiB no-font
SVG and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
