# Editor243 Single-Allocation Plugin Success Message

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime297-editor243-performance-batch-20260829x-v1`

## Problem

A successful live-plugin action with diagnostics first joined every diagnostic into a temporary
string and then formatted the final status string. The full diagnostic payload was therefore
allocated and copied twice before the Editor status line received it.

## Optimization

- Compute the exact final byte capacity from the plugin ID, action label, diagnostics, and separators.
- Write the fixed prefix and all dynamic segments directly into one `String`.
- Preserve the no-diagnostic and multi-diagnostic output bytes and separator order.

## Regression Contract

The `optimization_batch_20260829x_` Editor tests compare the optimized and legacy output for empty
and eight-entry diagnostic sets and guard removal of the intermediate `join`. The ignored paired
release benchmark emits `EDITOR243_SINGLE_ALLOCATION_PLUGIN_SUCCESS_MESSAGE_BENCH_V1`. It performs
40,000 eight-diagnostic message builds per sample, reduces result allocations per message from two
to one, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
