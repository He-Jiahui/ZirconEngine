# Runtime251 Renderer Feature Borrowed Capability

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime251-editor197-performance-batch-20260826he-v1`

## Problem

Renderer feature capability queries called `descriptor()`, which deep-cloned a plugin descriptor and
all of its string and pass vectors before checking one capability requirement. Repeated compile and
selection queries therefore copied immutable descriptor data on every lookup.

## Optimization

- Check asset-local capability requirements first.
- Borrow `descriptor_override` capability requirements without cloning the descriptor.
- Preserve builtin descriptor generation and the empty plugin-without-override fallback.

## Regression Contract

The `optimization_batch_20260826he_` Runtime tests preserve local, descriptor, and missing capability
sources; enforce borrowed override access; and provide an ignored paired release benchmark emitting
`RUNTIME251_RENDERER_FEATURE_BORROWED_CAPABILITY_BENCH_V1`. It repeatedly queries a plugin descriptor
with 512 retained strings and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
