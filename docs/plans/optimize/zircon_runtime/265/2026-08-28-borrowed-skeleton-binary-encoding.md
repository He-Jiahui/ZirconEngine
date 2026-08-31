# Runtime265 Borrowed Skeleton Binary Encoding

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime265-editor211-performance-batch-20260828hs-v1`

## Problem

Animation skeleton encoding passed the complete owned asset to the shared binary envelope. The
envelope cloned that payload before serialization, duplicating every bone name and transform only
to discard the clone after producing bytes.

## Optimization

- Serialize the skeleton name and bone slice through a borrowed payload view.
- Preserve current binary field order and the owned decode schema.
- Keep the change local while the shared binary encoder is under separate ownership.

## Regression Contract

The `optimization_batch_20260828hs_` Runtime tests require byte-for-byte parity with the legacy
owned payload, preserve the decode round trip, enforce borrowed bones, and provide an ignored
paired release benchmark emitting `RUNTIME265_BORROWED_SKELETON_BINARY_ENCODING_BENCH_V1`. It
encodes 512 bones with 4 KiB names eight times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
