# Editor194 Watched Asset Id Direct Join

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime248-editor194-performance-batch-20260826hb-v1`

## Problem

The UI asset watcher converted a root-relative path into one lossy string, allocated another string
while replacing Windows separators, and then formatted that normalized path into the final asset id.
Every accepted filesystem event therefore materialized multiple complete path strings.

## Optimization

- Check the `.zui` suffix directly on the platform-encoded file name.
- Reserve the final `res://` buffer once from the relative path byte length.
- Stream the lossy relative path into the final buffer while mapping backslashes to slashes.

## Regression Contract

The `optimization_batch_20260826hb_` Editor tests preserve unique-root selection, extension
rejection, and normalized asset ids; enforce direct final-buffer construction; and provide an
ignored paired release benchmark emitting `EDITOR194_WATCHED_ASSET_ID_DIRECT_JOIN_BENCH_V1`. It
repeatedly resolves a long asset path and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
