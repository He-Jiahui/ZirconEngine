# Runtime252 Labeled URI Direct Rebuild

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime252-editor198-performance-batch-20260826hf-v1`

## Problem

Missing-label lookup recovery formatted a canonical labeled `AssetUri` into a new string, split the
label from that string, copied the source text, and parsed it back into another locator. The
original locator already retained the validated scheme and canonical path needed by the source URI.

## Optimization

- Keep the existing early return for unlabeled locators.
- Preserve the owned missing-label value returned to the error path.
- Rebuild the source locator directly from the borrowed scheme and path with no display, split, or
  parse round trip.

## Regression Contract

The `optimization_batch_20260826hf_` Runtime tests preserve labeled URI splitting across every
resource scheme and the unlabeled fallback; enforce direct construction from borrowed parts; and
provide an ignored paired release benchmark emitting
`RUNTIME252_LABELED_URI_DIRECT_REBUILD_BENCH_V1`. It repeatedly strips a label from a locator with a
32 KiB canonical path and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
