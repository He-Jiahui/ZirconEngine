# Editor245 Single-Buffer Presentation Summary

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime299-editor245-performance-batch-20260829z-v1`

## Problem

The softbuffer presentation diagnostic formatted seven frame rectangles into temporary `String`
values and then formatted the final summary. A single diagnostic snapshot therefore allocated eight
result buffers and copied every frame string into the final output.

## Optimization

- Represent a borrowed frame as a `Display` adapter so rectangle bytes can be written directly into
  the final diagnostic string.
- Reserve one summary buffer using a conservative fixed numeric allowance plus all dynamic string
  lengths, then write the complete output through `fmt::Write`.
- Preserve the standalone `frame_summary` contract by routing it through the same display adapter.

## Regression Contract

The `optimization_batch_20260829z_` Editor tests compare the complete optimized diagnostic with the
legacy eight-allocation builder and verify standalone frame bytes. The ignored paired release
benchmark emits `EDITOR245_SINGLE_BUFFER_PRESENTATION_SUMMARY_BENCH_V1`. It performs 10,000 real
presentation summaries per sample, reduces result allocations per summary from eight to one, and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
