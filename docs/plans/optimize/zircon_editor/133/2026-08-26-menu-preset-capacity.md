# Editor133 Menu Preset Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime187-editor133-performance-batch-20260826er-v1`

## Problem

The synthesized Window menu started with two entries, extended a variable preset list, and then
grew again for the debug entry despite the exact output count being available up front.

## Optimization

- Allocate once to `preset_count + 3` before constructing menu items.
- Preserve save/reset/load/debug action order, action IDs, and the three-item empty-preset path.

## Regression Contract

The `optimization_batch_20260826er_` Editor tests cover 256 preset actions, empty capacity, source
shape, and an ignored paired release benchmark emitting `EDITOR133_MENU_PRESET_CAPACITY_BENCH_V1`.
It writes 259 menu items 2,048 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
