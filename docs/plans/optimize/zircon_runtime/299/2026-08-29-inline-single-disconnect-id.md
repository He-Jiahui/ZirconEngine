# Runtime299 Inline Single-Disconnect ID

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime299-editor245-performance-batch-20260829z-v1`

## Problem

Event publication stored disconnected subscriber IDs in an optional `Vec`. The first disconnected
subscriber therefore allocated a heap buffer even though the usual cleanup case contains one ID,
and the topic removal API can already consume a borrowed one-element slice.

## Optimization

- Keep the first disconnected subscriber ID inline and allocate the additional-ID vector only when
  a second disconnected subscriber is observed in the same publication.
- Pass the inline ID to topic cleanup through `std::slice::from_ref`.
- Preserve sorted batch removal for multiple disconnected subscribers and all publication ordering.

## Regression Contract

The `optimization_batch_20260829z_` Runtime tests deactivate a real EventBus subscriber and verify
that publication still removes the empty topic, then guard the single-ID inline representation.
The ignored paired release benchmark emits `RUNTIME299_INLINE_SINGLE_DISCONNECT_ID_BENCH_V1`. It
performs 200,000 single-ID records per sample, reduces result allocations per record from one to
zero, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
