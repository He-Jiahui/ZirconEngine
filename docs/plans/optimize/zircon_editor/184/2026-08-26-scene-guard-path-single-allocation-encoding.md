# Editor184 Scene Guard Path Single-Allocation Encoding

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime238-editor184-performance-batch-20260826gr-v1`

## Problem

Every Editor scene path guard collected an exact-capacity UTF-16 path and then appended the NUL
terminator required by `CreateFileW`, forcing a second allocation and a full code-unit copy for each
protected path component.

## Optimization

- Collect encoded path units and the NUL terminator through one exact-size iterator chain.
- Reuse the helper for each reparse-safe handle open without changing Win32 flags or share modes.
- Preserve all UTF-16 code units and emit exactly one trailing NUL.

## Regression Contract

The `optimization_batch_20260826gr_` Editor tests cover UTF-16 content and exactly one trailing NUL,
enforce the single-collection source contract, and provide an ignored paired release benchmark
emitting `EDITOR184_SCENE_GUARD_PATH_SINGLE_ALLOCATION_ENCODING_BENCH_V1`. It repeatedly encodes a
long Windows scene path and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
