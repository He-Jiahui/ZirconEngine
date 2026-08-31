# Runtime257 Borrowed IME UTF-8 Validation

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime257-editor203-performance-batch-20260826hk-v1`

## Problem

Dynamic-runtime IME surrounding-text conversion copied every payload into a new byte vector before
checking UTF-8 validity. Malformed host input therefore allocated and copied its complete payload
only to reject it immediately, amplifying invalid-input bursts at the ABI boundary.

## Optimization

- Validate UTF-8 directly against the borrowed payload slice.
- Allocate an owned `String` only after validation succeeds.
- Preserve the existing invalid-payload status and cursor/anchor boundary checks.

## Regression Contract

The `optimization_batch_20260826hk_` Runtime tests preserve valid multibyte text and invalid UTF-8
rejection; enforce borrowed validation before ownership conversion; and provide an ignored paired
release benchmark emitting `RUNTIME257_BORROWED_IME_UTF8_VALIDATION_BENCH_V1`. It rejects a 32 KiB
payload with an invalid final byte 512 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
