# Editor238 Borrowed Scene-Picker Window Query

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime292-editor238-performance-batch-20260828it-v1`

## Problem

Every scene-picker window request copied its query tail into a new String. Dispatch used the copy
only while the request input was alive: once for comparison with the bridge query and once as a
borrowed argument to rebuild the visible scene window.

## Optimization

- Return the query tail as a slice tied to the request input.
- Compare the borrowed slice against the bridge's existing query string.
- Pass the same slice directly to scene-picker state generation.
- Preserve `splitn(4, '|')` so separators inside the query remain part of the query.

## Regression Contract

The `optimization_batch_20260828it_` Editor tests prove the query points into the request buffer and
guard the allocation-free parser shape. The ignored paired release benchmark emits
`EDITOR238_BORROWED_SCENE_PICKER_WINDOW_QUERY_BENCH_V1`. It performs 100,000 parses of a 611-byte
request with a 600-byte query per sample, reduces query allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
