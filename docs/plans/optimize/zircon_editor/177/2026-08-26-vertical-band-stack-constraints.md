# Editor177 Vertical Band Stack Constraints

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime231-editor177-performance-batch-20260826gk-v1`

## Problem

Workbench vertical-band layout created a three-element constraint vector and then always appended
the status band, forcing an input-buffer growth before every solver call.

## Optimization

- Select four- or five-element stack constraint slices from bottom-panel visibility.
- Keep the owned solver result and all band ordering, gap, compact-bottom, and frame semantics.
- Eliminate the temporary input vector and its mandatory growth from the repeated layout path.

## Regression Contract

The `optimization_batch_20260826gk_` Editor tests compare both visibility branches with the legacy
solver inputs, enforce stack slices, and provide an ignored paired release benchmark emitting
`EDITOR177_VERTICAL_BAND_STACK_CONSTRAINTS_BENCH_V1`. It resolves 262,144 layouts per sample and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
