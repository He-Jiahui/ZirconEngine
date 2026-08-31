# Editor154 MUI Icon Name Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime208-editor154-performance-batch-20260826fm-v1`

## Problem

MUI ligature icon name normalization grew its PascalCase output string from empty even though the
ASCII-normalized result cannot exceed the input name byte length.

## Optimization

- Reserve the input name byte length before appending normalized name parts.
- Preserve prefix stripping, separator handling, PascalCase conversion, hyphen rejection, alias
  validation, and final ASCII module-name constraints.

## Regression Contract

The `optimization_batch_20260826fm_` Editor tests normalize 256 ligature name parts, verify output
content, character constraints and capacity, enforce the production source shape, and provide an
ignored paired release benchmark emitting `EDITOR154_MUI_ICON_NAME_CAPACITY_BENCH_V1`. It copies a
4,096-byte name 512 times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
