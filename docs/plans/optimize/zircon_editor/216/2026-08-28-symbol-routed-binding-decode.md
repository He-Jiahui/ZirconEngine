# Editor216 Symbol-Routed Binding Decode

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime270-editor216-performance-batch-20260828hx-v1`

## Problem

Editor binding payload decode cloned every owned `UiBindingCall` while probing seven unrelated
command codecs. Custom payloads consequently performed seven complete copies of their symbol and
argument graph before returning the original owned call.

## Optimization

- Classify command symbols by their exact namespace before invoking a codec.
- Probe at most one command family and retain one compatibility clone only for that selected codec.
- Move unclassified custom calls directly into the payload without a probe clone.
- Preserve malformed known-command errors and custom fallback for unknown symbols inside a known
  namespace.

## Regression Contract

The `optimization_batch_20260828hx_` Editor tests prove allocation identity for custom payloads,
cover all seven namespace routes, and retain malformed-command errors. The ignored paired release
benchmark emits `EDITOR216_SYMBOL_ROUTED_BINDING_DECODE_BENCH_V1`. It decodes 32 custom calls with
256 one-KiB string arguments per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
