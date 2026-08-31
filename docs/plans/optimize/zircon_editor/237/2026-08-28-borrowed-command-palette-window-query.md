# Editor237 Borrowed Command-Palette Window Query

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime291-editor237-performance-batch-20260828is-v1`

## Problem

Every command-palette window request copied the query tail into a new String during parsing. The
copy was used only for comparison with the bridge's already-owned query; the request input remains
alive throughout dispatch, so the temporary allocation never escaped the call.

## Optimization

- Return the fourth request field as a slice tied to the request input.
- Compare the borrowed request query with the bridge query without constructing a temporary String.
- Preserve `splitn(4, '|')` so query text containing separators remains intact.

## Regression Contract

The `optimization_batch_20260828is_` Editor tests prove the query slice points into the request and
guard the allocation-free parser shape. The ignored paired release benchmark emits
`EDITOR237_BORROWED_COMMAND_PALETTE_WINDOW_QUERY_BENCH_V1`. It performs 100,000 parses of a
610-byte request with a 600-byte query per sample, reduces query allocations from one to zero, and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
