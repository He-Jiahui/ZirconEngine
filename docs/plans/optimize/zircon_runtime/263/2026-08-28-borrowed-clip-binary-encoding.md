# Runtime263 Borrowed Clip Binary Encoding

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime263-editor209-performance-batch-20260828hq-v1`

## Problem

Animation clip binary encoding first cloned the clip name, every bone track, every event track, and
all nested strings and channel key buffers into a temporary owned payload. Serialization then read
that duplicate once and discarded it, adding allocation and memory-copy cost proportional to the
complete authored clip.

## Optimization

- Serialize clip names, bone tracks, and event tracks through a borrowed payload view.
- Preserve the current binary field order and the owned decode schema, including the v1 fallback.
- Keep reference canonicalization local to the small skeleton reference boundary.

## Regression Contract

The `optimization_batch_20260828hq_` Runtime tests require byte-for-byte parity with the legacy
owned payload, preserve the decode round trip, enforce borrowed collection fields, and provide an
ignored paired release benchmark emitting `RUNTIME263_BORROWED_CLIP_BINARY_ENCODING_BENCH_V1`.
It encodes 1,024 event tracks with 1 KiB payloads eight times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
