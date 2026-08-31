# Runtime285 Owned Migrator Schema Header

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime285-editor231-performance-batch-20260828im-v1`

## Problem

Runtime UI schema migration parsed an owned asset header, borrowed it for source-version rejection,
then cloned its ID into the error before discarding the header. Unsupported inputs therefore paid
for an avoidable allocation at the migration entry.

## Optimization

- Consume the parsed header at the owned migration entry.
- Move its asset ID into unsupported-version errors and return valid headers unchanged.
- Preserve borrowed tree-document validation, flat/tree routing, migration reports, and errors.

## Regression Contract

The `optimization_batch_20260828im_` Runtime tests prove header ID allocation identity and prevent
the borrowed validation call from returning at the owned entry. The ignored paired release
benchmark emits `RUNTIME285_OWNED_MIGRATOR_SCHEMA_HEADER_BENCH_V1`. It validates 512 64-KiB IDs per
sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
