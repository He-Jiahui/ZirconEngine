# Editor221 Reused Round-Robin Cursor

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime275-editor221-performance-batch-20260828ic-v1`

## Problem

Editor Runtime-event consumer scheduling cloned the next consumer ID into a fresh `String` every
time it advanced the round-robin cursor. Repeated single-consumer pumps therefore allocated and
copied the same identifier on every budget cycle.

## Optimization

- Borrow the next consumer ID from the active snapshot slice through cursor selection.
- Leave the cursor untouched when the selected ID is unchanged.
- Clear and append into the existing cursor buffer when selection changes, preserving capacity.
- Preserve next-index calculation, zero-visit behavior, poison recovery, and lock scope.

## Regression Contract

The `optimization_batch_20260828ic_` Editor tests prove allocation identity for equal and shorter
replacement IDs and prevent snapshot-ID cloning from returning. The ignored paired release
benchmark emits `EDITOR221_REUSED_ROUND_ROBIN_CURSOR_BENCH_V1`. It performs 8,192 repeated cursor
updates with approximately 5-KiB IDs per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
