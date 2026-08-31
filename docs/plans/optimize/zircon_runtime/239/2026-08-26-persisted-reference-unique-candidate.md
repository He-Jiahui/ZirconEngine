# Runtime239 Persisted Reference Unique Candidate

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime239-editor185-performance-batch-20260826gs-v1`

## Problem

Persisting one runtime asset reference collected every matching project root into a Vec even though
the result contract only distinguishes missing, unique, and ambiguous matches. The common unique
case therefore allocated a candidate container for one item.

## Optimization

- Retain only the first matching root and track whether any later match makes it ambiguous.
- Continue resolving every root so later path I/O errors keep their existing precedence.
- Preserve missing, unique, and ambiguous result behavior without a candidate Vec allocation.

## Regression Contract

The `optimization_batch_20260826gs_` Runtime tests cover first-candidate retention and repeated
ambiguity, enforce the no-candidate-Vec source contract, and provide an ignored paired release
benchmark emitting `RUNTIME239_PERSISTED_REFERENCE_UNIQUE_CANDIDATE_BENCH_V1`. It repeatedly stores
the common single candidate and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
