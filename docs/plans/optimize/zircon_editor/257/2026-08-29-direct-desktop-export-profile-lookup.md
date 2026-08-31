# Editor257 Direct Desktop Export Profile Lookup

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime312-editor257-performance-batch-20260829al-v1`

## Problem

Looking up one built-in export profile first constructed the complete eight-profile catalog and
then discarded seven owned profiles. Each discarded profile allocated names, output names, and a
packaging strategy vector.

## Optimization

- Match the stable profile name before constructing an `ExportProfile`.
- Construct only the requested profile while keeping unknown-name behavior unchanged.
- Reuse the same direct constructor for ordered full-catalog generation.

## Regression Contract

The `optimization_batch_20260829al_` Editor tests compare every direct lookup with the ordered
catalog and guard against rebuilding the catalog inside the single-item path. The ignored paired
release benchmark emits `EDITOR257_DIRECT_DESKTOP_EXPORT_PROFILE_LOOKUP_BENCH_V1`. It performs
10,000 last-profile lookups per sample, changes eight profile constructions to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
