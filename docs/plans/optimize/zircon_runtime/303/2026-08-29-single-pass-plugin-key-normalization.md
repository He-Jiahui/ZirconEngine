# Runtime303 Single-Pass Plugin Key Normalization

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime303-editor249-performance-batch-20260829ad-v1`

## Problem

Runtime plugin key normalization separately validated every byte, re-read the first byte, and then
scanned again for uppercase input. Valid mixed-case extension IDs could traverse the complete key
twice before allocating their lowercase representation.

## Optimization

- Validate the first byte and the remaining key in one iterator traversal.
- Record uppercase input during the same validation pass.
- Preserve borrowed lowercase keys, owned lowercase conversion, aliases, and rejection rules.

## Regression Contract

The `optimization_batch_20260829ad_` Runtime tests cover borrowed, owned, and rejected keys and
guard the single-pass source contract. The ignored paired release benchmark emits
`RUNTIME303_SINGLE_PASS_PLUGIN_KEY_NORMALIZATION_BENCH_V1`. It normalizes a long valid key with its
only uppercase byte at the end 10,000 times per sample, reduces full validation scans from two to
one, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
