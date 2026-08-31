# Editor225 Owned Panic Payload String

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime279-editor225-performance-batch-20260828ig-v1`

## Problem

Editor job panic handling inspected an owned panic payload by reference and cloned an embedded
`String`. Because the payload Box is consumed by message extraction, the original allocation was
discarded immediately after the redundant copy.

## Optimization

- Consume the panic payload while downcasting its supported string forms.
- Move an owned `String` directly out of its Box without reallocating.
- Preserve static-string conversion, the non-string fallback, and job failure event text.

## Regression Contract

The `optimization_batch_20260828ig_` Editor tests prove panic-message allocation identity and lock
the static and non-string fallbacks. The ignored paired release benchmark emits
`EDITOR225_OWNED_PANIC_PAYLOAD_STRING_BENCH_V1`. It extracts 512 owned 64-KiB panic messages per
sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
