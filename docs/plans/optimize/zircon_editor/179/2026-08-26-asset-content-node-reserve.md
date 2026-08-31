# Editor179 Asset Content Node Reserve

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime233-editor179-performance-batch-20260826gm-v1`

## Problem

Assets Activity appended five large view nodes for every visible folder or asset without reserving
the exact additional count, repeatedly reallocating and moving accumulated nodes.

## Optimization

- Reserve one node for the empty state or five nodes for each visible content row.
- Account for the caller's existing Vec length and capacity through `Vec::reserve`.
- Preserve folder-before-asset order and every generated node, control ID, label, and visual state.

## Regression Contract

The `optimization_batch_20260826gm_` Editor tests cover empty, folder, asset, and mixed counts and
enforce reserve placement, and provide an ignored paired release benchmark emitting
`EDITOR179_ASSET_CONTENT_NODE_RESERVE_BENCH_V1`. It appends 4,096 eight-field node payloads across
64 projections per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
