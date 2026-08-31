# Runtime261 Single-Pass Animation Snapshot

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime261-editor207-performance-batch-20260826ho-v1`

## Problem

`AnimationRuntimeStatus::sanitized_snapshot` deep-cloned the complete runtime status and then
replaced the cloned player vector with a second per-player sanitized clone. Every player string,
asset reference, and diagnostic allocation was therefore cloned twice per snapshot.

## Optimization

- Construct the sanitized snapshot directly from its fields.
- Clone and sanitize each player exactly once.
- Preserve one clone of rigs, last-tick state, and runtime diagnostics, plus source immutability.

## Regression Contract

The `optimization_batch_20260826ho_` Runtime tests preserve sanitization and source-state semantics;
enforce field-wise snapshot construction without a whole-status clone; and provide an ignored
paired release benchmark emitting `RUNTIME261_SINGLE_PASS_ANIMATION_SNAPSHOT_BENCH_V1`. It
snapshots 4,096 players 16 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
