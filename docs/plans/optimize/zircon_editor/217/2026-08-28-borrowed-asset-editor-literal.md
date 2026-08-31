# Editor217 Borrowed Asset Editor Literal

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime271-editor217-performance-batch-20260828hy-v1`

## Problem

Asset-editor component mutation cloned the decoded literal before every manager setter even though
those APIs accept borrowed string-like values. Projection patch construction then cloned the same
literal twice and dropped the original, producing three deep copies on every successful mutation.

## Optimization

- Borrow the decoded literal across every asset-editor manager mutation route.
- Clone once for the patch attribute and move the original allocation into the final state value.
- Preserve target routing, host mutation errors, patch values, transaction IDs, and changed state.

## Regression Contract

The `optimization_batch_20260828hy_` Editor tests prove that the final state value receives the
original literal allocation and that the manager mutation block contains no literal clone. The
ignored paired release benchmark emits `EDITOR217_BORROWED_ASSET_EDITOR_LITERAL_BENCH_V1`. It
projects 128 64-KiB literals per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
