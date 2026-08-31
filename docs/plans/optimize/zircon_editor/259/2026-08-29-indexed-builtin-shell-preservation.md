# Editor259 Indexed Builtin Shell Preservation

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime314-editor259-performance-batch-20260829an-v1`

## Problem

Builtin shell restoration scanned every open view instance for every builtin candidate to preserve
single-instance descriptors. Startup and capability-driven shell rebuild work therefore grew
quadratically with open and builtin instance counts.

## Optimization

- Build one hash index of open descriptor identities before processing builtin candidates.
- Maintain the index as restored instances are inserted or replace an existing instance id.
- Preserve registry multi-instance policy and restore/reuse behavior.

## Regression Contract

The `optimization_batch_20260829an_` Editor tests compare indexed and linear preservation and guard
the production index lookup. The ignored paired release benchmark emits
`EDITOR259_INDEXED_BUILTIN_SHELL_PRESERVATION_BENCH_V1`. It includes hash-index construction for
1,024 open descriptors and 1,024 absent candidates per build, replaces 1,048,576 worst-case linear
comparisons with 1,024 hash lookups, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
