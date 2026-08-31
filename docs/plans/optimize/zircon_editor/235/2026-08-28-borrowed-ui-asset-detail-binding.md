# Editor235 Borrowed UI Asset Detail Binding

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime289-editor235-performance-batch-20260828iq-v1`

## Problem

UI asset detail edit routing split a borrowed binding ID and copied its instance, detail, and
action segments into three Strings. The parsed binding is consumed synchronously, so those owned
copies were allocated and freed for every edit event without escaping the dispatch call.

## Optimization

- Tie the parsed binding lifetime to the input binding ID.
- Retain borrowed segment slices through synchronous dispatch.
- Preserve prefix, field-count, empty-segment, and item-index validation.

## Regression Contract

The `optimization_batch_20260828iq_` Editor tests prove parsed field semantics and guard all three
fields as borrowed slices. The ignored paired release benchmark emits
`EDITOR235_BORROWED_UI_ASSET_DETAIL_BINDING_BENCH_V1`. It performs 100,000 valid parses per sample,
removes 300,000 field allocations, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
