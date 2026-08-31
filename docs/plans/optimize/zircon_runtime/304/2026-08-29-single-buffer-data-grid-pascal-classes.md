# Runtime304 Single-Buffer DataGrid Pascal Classes

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime304-editor250-performance-batch-20260829ae-v1`

## Problem

DataGrid template compilation built dynamic density, mode, spacing, and slot classes by first
allocating a PascalCase value and then allocating the final prefixed class. A representative class
therefore required at least two result buffers, while the shared PascalCase implementation also
allocated per segment.

## Optimization

- Reserve the complete final class buffer from the known prefix, infix, and value lengths.
- Skip separators and apply ASCII capitalization while writing directly into that buffer.
- Reuse the builder across six dynamic DataGrid class paths without changing emitted bytes.

## Regression Contract

The `optimization_batch_20260829ae_` Runtime tests cover separator runs, camelCase, Unicode, and
all migrated call sites and guard the single-buffer source contract. The ignored paired release
benchmark emits `RUNTIME304_SINGLE_BUFFER_DATA_GRID_PASCAL_CLASS_BENCH_V1`. It builds 100,000
multi-segment classes per sample, reduces result buffers from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
