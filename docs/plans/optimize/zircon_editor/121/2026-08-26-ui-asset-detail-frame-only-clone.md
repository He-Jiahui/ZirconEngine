# Editor121 UI Asset Detail Frame-Only Clone

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime175-editor121-performance-batch-20260826ef-v1`

## Problem

UI asset detail projection cloned the complete retained `TemplatePaneNodeData` merely to snapshot
its four-value frame. When a section grew, the path cloned the full node a second time. The node
carries many shared strings, models, bindings, styles, and interaction fields unrelated to layout.

## Optimization

- Snapshot only `TemplateNodeFrameData` before calculating detail-row layout.
- Refresh only that frame after section-height growth.
- Preserve section growth, following-node displacement, row geometry, and generated node data.

## Regression Contract

The shared `optimization_batch_20260826ef_` filter owns three Editor tests: frame-value behavior,
frame-only source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR121_UI_ASSET_DETAIL_FRAME_ONLY_CLONE_BENCH_V1`, performs 65,536 projections per sample,
reduces full retained-node clones per projection from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
