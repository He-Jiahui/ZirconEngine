# Editor152 Render Command Conversion Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime206-editor152-performance-batch-20260826fk-v1`

## Problem

Retained-host runtime render command conversion grew its host paint command output from empty even
though the complete input command count was available before conversion.

## Optimization

- Use the runtime command count as the initial host paint command capacity.
- Preserve per-command paint element expansion, clipping, visibility filtering, command order, and
  support for commands that emit multiple host paint records.

## Regression Contract

The `optimization_batch_20260826fk_` Editor tests convert 256 real visible Group commands, verify
output count, order and capacity, enforce the production source shape, and provide an ignored
paired release benchmark emitting `EDITOR152_RENDER_COMMAND_CONVERSION_CAPACITY_BENCH_V1`. It
fills 256 host-command-sized records 2,048 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
