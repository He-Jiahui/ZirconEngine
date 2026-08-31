# Editor07 Play Output Streaming Decode Optimization Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plan: `docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md`, E-PLAY-P1-32
- Status: implementation and release performance gate complete; batched managed validation pending

## Problem

Each Play stdout/stderr read decoded the complete 8 KiB chunk into a temporary
`Vec<DecodedOutputLine>` before attempting to enqueue any line. A newline-dense
chunk therefore grew and moved a large temporary element array even though the
reader immediately consumed the array in order. A disconnected consumer was
also discovered only after the complete chunk had been decoded.

## Change

- `BoundedLineDecoder::push` emits each completed line through a caller-owned
  callback instead of materializing a per-chunk line vector.
- The reader enqueues each line as soon as it is decoded and exits immediately
  when the consumer is disconnected.
- Existing CRLF normalization, UTF-8 replacement, maximum-line truncation,
  queue item/byte budgets, and full-queue drop accounting remain unchanged.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| One 8 KiB newline-dense read | one temporary line vector holding 8,192 elements | no temporary line vector | 100% of per-chunk collection storage removed |
| Consumer disconnect during a chunk | finish decoding the chunk before enqueue observes disconnect | stop at the rejected line | bounded by the accepted prefix |
| Line ordering | vector insertion order | callback emission order | byte-for-byte equivalent |

The ignored release gate runs 17 alternating legacy/streaming sample pairs on
an 8 KiB, 8,192-line chunk. Acceptance requires streaming nearest-rank P95 to
be at most 75% of legacy P95, a minimum 25% reduction. Exact Windows timing
values remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826_editor07_play_output_streaming_decode_preserves_line_order`
  locks CRLF,
  multi-line, and unterminated-tail order.
- `optimization_batch_20260826_editor07_play_output_streaming_decode_stops_on_consumer_rejection`
  locks
  early termination.
- `optimization_batch_20260826_editor07_play_output_streaming_decode_has_no_per_chunk_line_vector`
  locks
  the production callback shape.
- `optimization_batch_20260826_editor07_play_output_streaming_decode_performance_evidence`
  emits
  `EDITOR07_PLAY_OUTPUT_STREAMING_DECODE_BENCH_V1`, all raw samples, the exact
  temporary-vector count, and the 25% P95 reduction threshold.
- Exact-file Rust 1.94.1 rustfmt, source contract, and scoped diff checks are
  required before managed validation.

## Remaining Plan Work

This slice does not close Editor07. The typed process control transport,
startup/runtime heartbeat, multi-instance authority, Game View presentation,
and large-scene/long-run Play performance matrix remain open.
