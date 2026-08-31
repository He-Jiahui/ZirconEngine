# Runtime186 Shader Resource Binding Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime186-editor132-performance-batch-20260826eq-v1`

## Problem

Named shader-resource validation appended every valid ABI binding to a growth-driven vector even
though successful output cannot exceed either declared or requested resource count.

## Optimization

- Allocate once to `min(declared_resources, requested_bindings)` before validation.
- Preserve missing/unknown/mismatch diagnostics, declaration order, ABI group/binding numbering,
  and zero allocation when either input is empty.

## Regression Contract

The `optimization_batch_20260826eq_` Runtime tests cover 256 real bindings, source shape, and an
ignored paired release benchmark emitting `RUNTIME186_SHADER_RESOURCE_BINDING_CAPACITY_BENCH_V1`.
It writes 256 real bindings 2,048 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
