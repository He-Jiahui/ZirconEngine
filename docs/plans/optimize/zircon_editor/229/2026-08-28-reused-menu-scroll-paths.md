# Editor229 Reused Menu Scroll Paths

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime283-editor229-performance-batch-20260828ik-v1`

## Problem

Editor menu scroll routing replaced retained hovered/open submenu vectors with freshly cloned item
paths. Repeated wheel and trackpad movement over the same popup depth discarded usable capacity
and allocated on the input hot path.

## Optimization

- Clear and extend retained menu path vectors from borrowed route slices.
- Reuse the same helper for submenu and leaf hover updates.
- Preserve hover indices, submenu rebuild decisions, route projection, and dispatch snapshots.

## Regression Contract

The `optimization_batch_20260828ik_` Editor tests prove retained vector allocation identity and
prevent the three item-path clone assignments from returning. The ignored paired release benchmark
emits `EDITOR229_REUSED_MENU_SCROLL_PATHS_BENCH_V1`. It performs 65,536 representative path updates
per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
