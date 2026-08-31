# Runtime238 Windows Path Single-Allocation Encoding

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime238-editor184-performance-batch-20260826gr-v1`

## Problem

Runtime atomic file operations collected an exact-capacity UTF-16 Windows path and then appended the
required NUL terminator. The append forced a second allocation and copied every code unit on common
short absolute paths used by rename and replace operations.

## Optimization

- Collect encoded path units and the NUL terminator through one exact-size iterator chain.
- Keep embedded-NUL and prefix checks on the path-unit slice before the terminator.
- Preserve short path, absolute path, verbatim path, device path, and long UNC prefix behavior.

## Regression Contract

The `optimization_batch_20260826gr_` Runtime tests cover UTF-16 content and exactly one trailing NUL,
enforce the single-collection source contract, and provide an ignored paired release benchmark
emitting `RUNTIME238_WINDOWS_PATH_SINGLE_ALLOCATION_ENCODING_BENCH_V1`. It repeatedly encodes a long
Windows path and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
