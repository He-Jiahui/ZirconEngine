# Editor186 Finite Workbench Preset Normalization

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime240-editor186-performance-batch-20260826gt-v1`

## Problem

Workbench preset normalization collected the complete input, sorted it, and deduplicated it even
though the domain contains only four ordered variants. Repeated declarations made the temporary Vec
grow with input size and paid sorting cost for values whose canonical order is already fixed.

## Optimization

- Record input membership in a four-entry presence table during one pass.
- Allocate the result once at the exact number of present variants.
- Emit the four variants in the same canonical enum order as the previous sort and dedup path.

## Regression Contract

The `optimization_batch_20260826gt_` Editor tests cover empty, duplicated, and unordered inputs,
enforce the finite-presence-table source contract, and provide an ignored paired release benchmark
emitting `EDITOR186_FINITE_WORKBENCH_PRESET_NORMALIZATION_BENCH_V1`. It repeatedly normalizes a long
duplicate-heavy declaration and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
