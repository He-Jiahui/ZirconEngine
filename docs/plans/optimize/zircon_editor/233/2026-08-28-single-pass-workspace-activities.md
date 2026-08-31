# Editor233 Single-Pass Workspace Activities

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime287-editor233-performance-batch-20260828io-v1`

## Problem

Workbench reflection recursively created a separate activity Vec for every tabs leaf and merged
those vectors at every split. Deep split layouts repeatedly moved already-built activities and
cloned the host at branch nodes before discarding the temporary vectors.

## Optimization

- Count workspace tabs before projection and reserve the final activity Vec once.
- Traverse split nodes into the same output Vec while preserving left-to-right activity order.
- Borrow the host through branch traversal and clone it only for retained activity records.

## Regression Contract

The `optimization_batch_20260828io_` Editor tests prove recursive count and activity ordering and
guard the single-output-Vec source contract. The ignored paired release benchmark emits
`EDITOR233_SINGLE_PASS_WORKSPACE_ACTIVITIES_BENCH_V1`. It collects a real 256-tab right-skewed
workspace 16 times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
