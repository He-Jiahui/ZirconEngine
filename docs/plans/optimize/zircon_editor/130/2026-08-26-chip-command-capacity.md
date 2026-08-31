# Editor130 Chip Command Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime184-editor130-performance-batch-20260826eo-v1`

## Problem

A full retained-host Chip emits one surface, two leading-icon strokes, one label, and ten delete
dots. Starting from an empty or tight caller vector caused repeated growth while appending the
fixed 14-command maximum.

## Optimization

- Define the 14-command Chip upper bound beside the primitive builder.
- Reserve all 14 additional slots after root/slot and positive-layout validation.
- Preserve avatar precedence, optional icon/label/delete behavior, command order, z values,
  clipping, opacity, and rejected-path allocation behavior.

## Regression Contract

The shared `optimization_batch_20260826eo_` filter owns three Editor tests: full Chip behavior,
capacity source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR130_CHIP_COMMAND_CAPACITY_BENCH_V1`, builds 14 real `HostPaintCommand` values 32,768 times
per sample, replaces growth-driven allocation with one reserve, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
