# Editor199 Stable Selection Alias Update

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime253-editor199-performance-batch-20260826hg-v1`

## Problem

Workbench selection-style projection allocated a new key and color string for every alias on every
projection, even when the retained map already contained the same value. Stable checked, radio,
and toggle controls therefore repeated allocations across unchanged UI frames.

## Optimization

- Skip aliases whose retained string already equals the requested value.
- Replace only the value for an existing alias that changed, retaining its allocated key.
- Preserve owned key and value insertion when an alias is absent.

## Regression Contract

The `optimization_batch_20260826hg_` Editor tests preserve missing, changed, and stable alias
semantics; enforce the borrowed lookup and unchanged-value guard; and provide an ignored paired
release benchmark emitting `EDITOR199_STABLE_SELECTION_ALIAS_BENCH_V1`. It repeatedly projects six
already stable aliases and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
