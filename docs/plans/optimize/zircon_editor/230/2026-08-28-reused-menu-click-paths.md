# Editor230 Reused Menu Click Paths

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime284-editor230-performance-batch-20260828il-v1`

## Problem

Editor submenu click routing replaced retained hovered/open submenu vectors with newly cloned item
paths. Repeated menu interaction discarded usable capacity on the pointer input path.

## Optimization

- Clear and extend retained click-path vectors from borrowed route slices.
- Reuse one helper for hovered and open submenu state.
- Preserve menu selection, popup rebuild conditions, action dispatch, and returned snapshots.

## Regression Contract

The `optimization_batch_20260828il_` Editor tests prove retained vector allocation identity and
prevent both click-path clone assignments from returning. The ignored paired release benchmark
emits `EDITOR230_REUSED_MENU_CLICK_PATHS_BENCH_V1`. It performs 65,536 representative updates per
sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
