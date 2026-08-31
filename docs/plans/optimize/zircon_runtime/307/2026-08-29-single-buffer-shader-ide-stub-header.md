# Runtime307 Single-Buffer Shader IDE Stub Header

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime307-editor253-performance-batch-20260829ah-v1`

## Problem

Shader IDE stub header construction formatted the primary header into one string, formatted an
optional source URI into a second temporary string, and appended that temporary to the first
buffer. Headers with source metadata therefore required two independent string buffers and could
also grow the destination buffer during append.

## Optimization

- Reserve one destination buffer using the import path and a safe upper bound for the URI display.
- Write both header lines directly through `fmt::Write`.
- Preserve the exact no-source and source-with-label output text.

## Regression Contract

The `optimization_batch_20260829ah_` Runtime tests cover exact text with and without a source URI
and guard the single-buffer source contract. The ignored paired release benchmark emits
`RUNTIME307_SINGLE_BUFFER_SHADER_IDE_STUB_HEADER_BENCH_V1`. It builds 100,000 source-bearing
headers per sample, reduces independent string buffers from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
