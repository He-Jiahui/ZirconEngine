# Runtime283 Owned UI Schema Error ID

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime283-editor229-performance-batch-20260828ik-v1`

## Problem

Runtime UI document loaders cloned parsed asset IDs into unsupported-schema errors and then
discarded the invalid document. Large or malformed asset identifiers therefore incurred an
unnecessary allocation on both current and v2 schema rejection paths.

## Optimization

- Consume v2 documents during version validation and return them unchanged on success.
- Move current and v2 asset IDs directly into unsupported-version errors.
- Preserve parse errors, version fields, expected/current constants, and valid document output.

## Regression Contract

The `optimization_batch_20260828ik_` Runtime tests prove error asset-ID allocation identity and
prevent the document ID clone from returning. The ignored paired release benchmark emits
`RUNTIME283_OWNED_UI_SCHEMA_ERROR_ID_BENCH_V1`. It converts 512 64-KiB IDs per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
