# Editor222 Owned Inspector Apply Arguments

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime276-editor222-performance-batch-20260828id-v1`

## Problem

Editor inspector Apply retrieved an owned editor snapshot but cloned name, parent, translation,
plugin field ID, and plugin value strings while constructing binding arguments. The owned inspector
snapshot was discarded immediately after those copies.

## Optimization

- Move name, non-empty parent, and all three translation strings into binding values.
- Consume plugin components and properties while retaining customization and editability filters.
- Move editable plugin field IDs and values into their output pairs.
- Preserve binding key order, selected-entity target, empty-parent null behavior, and field filters.

## Regression Contract

The `optimization_batch_20260828id_` Editor tests prove allocation identity across base inspector
and plugin binding values and prevent field cloning from returning. The ignored paired release
benchmark emits `EDITOR222_OWNED_INSPECTOR_APPLY_ARGUMENTS_BENCH_V1`. It converts 128 owned snapshots
with 32-KiB base and plugin strings per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
