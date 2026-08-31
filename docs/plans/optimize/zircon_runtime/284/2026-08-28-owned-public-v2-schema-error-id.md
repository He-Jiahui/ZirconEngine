# Runtime284 Owned Public V2 Schema Error ID

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime284-editor230-performance-batch-20260828il-v1`

## Problem

The public Runtime v2 UI loader cloned a parsed asset ID into an unsupported-schema error and then
discarded the invalid document. Invalid large IDs therefore caused a redundant allocation in the
public string/file loading path.

## Optimization

- Consume the parsed v2 document during version validation.
- Move the asset ID directly into unsupported-version errors and return valid documents unchanged.
- Preserve parse/I/O behavior, version reporting, and subsequent ZUI profile validation.

## Regression Contract

The `optimization_batch_20260828il_` Runtime tests prove error asset-ID allocation identity and
prevent the document ID clone from returning. The ignored paired release benchmark emits
`RUNTIME284_OWNED_PUBLIC_V2_SCHEMA_ERROR_ID_BENCH_V1`. It converts 512 64-KiB IDs per sample and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
