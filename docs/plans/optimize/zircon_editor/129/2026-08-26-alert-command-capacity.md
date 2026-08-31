# Editor129 Alert Command Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime183-editor129-performance-batch-20260826en-v1`

## Problem

A full retained-host Alert emits one surface, three icon commands, one message, and ten close-mark
commands. Starting from an empty or tight command vector caused repeated growth while appending
the fixed 15-command maximum.

## Optimization

- Define the 15-command Alert upper bound beside the primitive builder.
- Reserve all 15 additional slots after identity and positive-layout validation.
- Preserve slot/root handling, invalid-layout zero output, command order, z values, clipping, and
  opacity.

## Regression Contract

The shared `optimization_batch_20260826en_` filter owns three Editor tests: full Alert behavior,
capacity source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR129_ALERT_COMMAND_CAPACITY_BENCH_V1`, builds 15 real `HostPaintCommand` values 16,384 times
per sample, replaces growth-driven allocation with one reserve, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
