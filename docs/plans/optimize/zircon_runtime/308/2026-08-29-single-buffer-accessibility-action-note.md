# Runtime308 Single-Buffer Accessibility Action Note

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime308-editor254-performance-batch-20260829ai-v1`

## Problem

Accessibility action diagnostics formatted a base status string, then appended optional code and
reason fields. Long diagnostics could grow the buffer during each append.

## Optimization

- Compute the exact byte capacity from status, code, and reason fields.
- Write all note fields directly into one `String` through `fmt::Write`.
- Preserve field ordering and omission behavior for absent code/reason values.

## Regression Contract

The `optimization_batch_20260829ai_` Runtime tests cover minimal and fully populated notes and
guard the single-buffer source contract. The ignored paired release benchmark emits
`RUNTIME308_SINGLE_BUFFER_ACCESSIBILITY_ACTION_NOTE_BENCH_V1`. It builds 150,000 long diagnostic
notes per sample, reduces independent string buffers from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
