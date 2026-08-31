# Runtime248 Child Segment Direct Write

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime248-editor194-performance-batch-20260826hb-v1`

## Problem

Every UI template child path first collected a sanitized control or component id into one owned
string, then formatted that complete string and the child index into a second allocation. Large
template trees repeated the intermediate allocation and full-buffer copy for every child.

## Optimization

- Reserve the final string from the source byte length and decimal index width.
- Sanitize the source characters directly into that final buffer.
- Append the separator and write the child index in place without an intermediate string.

## Regression Contract

The `optimization_batch_20260826hb_` Runtime tests preserve source priority, separator replacement,
component fallback, and the default node segment; enforce direct final-buffer construction; and
provide an ignored paired release benchmark emitting
`RUNTIME248_CHILD_SEGMENT_DIRECT_WRITE_BENCH_V1`. It repeatedly builds segments from a long control
id and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
