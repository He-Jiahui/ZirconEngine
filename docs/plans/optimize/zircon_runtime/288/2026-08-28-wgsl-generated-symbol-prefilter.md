# Runtime288 WGSL Generated-Symbol Prefilter

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime288-editor234-performance-batch-20260828ip-v1`

## Problem

Generated-material anchor diagnostics copied every material WGSL source while stripping comments,
even when the source did not contain any generated `zr_mat_` symbol. Ordinary shaders therefore
paid a source-sized allocation and multiple full scans before producing no diagnostic.

## Optimization

- Reject sources without the generated-symbol prefix through a borrowed substring scan.
- Preserve comment stripping for sources whose raw text may contain a real or commented symbol.
- Preserve include-anchor and nested-comment semantics before emitting the diagnostic.

## Regression Contract

The `optimization_batch_20260828ip_` Runtime tests prove diagnostic semantics and guard the symbol
check ahead of source copying. The ignored paired release benchmark emits
`RUNTIME288_WGSL_GENERATED_SYMBOL_PREFILTER_BENCH_V1`. It checks a 51-KiB ordinary shader 512 times
per sample, removes all 512 source copies, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
