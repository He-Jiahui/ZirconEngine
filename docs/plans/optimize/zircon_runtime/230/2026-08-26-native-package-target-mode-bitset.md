# Runtime230 Native Package Target Mode Bitset

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime230-editor176-performance-batch-20260826gj-v1`

## Problem

Runtime native-package projection linearly searched the target-mode output for every declaration in
runtime modules, even though the target-mode domain contains three exhaustive variants.

## Optimization

- Track ClientRuntime, ServerRuntime, and EditorHost membership in a three-bit mask.
- Keep the output Vec demand-grown and retain the Runtime-module filter.
- Preserve first-occurrence order, duplicate suppression, and exclusion of Editor/Native/VM module
  declarations from the native runtime package selection.

## Regression Contract

The `optimization_batch_20260826gj_` Runtime tests cover runtime-only filtering and first-occurrence
order and enforce bitset membership, and provide an ignored paired release benchmark emitting
`RUNTIME230_NATIVE_PACKAGE_TARGET_MODE_BITSET_BENCH_V1`. It projects 512 lists with 4,096 mode
declarations per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
