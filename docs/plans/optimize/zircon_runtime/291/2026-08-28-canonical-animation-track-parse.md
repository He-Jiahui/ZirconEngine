# Runtime291 Canonical Animation-Track Parse

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime291-editor237-performance-batch-20260828is-v1`

## Problem

Parsing a canonical animation track path built an owned `EntityPath`, an owned
`ComponentPropertyPath`, and then a third formatted path string. The two validation objects and
their segment storage were discarded immediately even though canonical asset paths need no
normalization.

## Optimization

- Stream entity and property segments to prove that each segment is non-empty and already trimmed.
- Own the original canonical track path exactly once when no normalization is required.
- Preserve the original object-based parser as the fallback for whitespace and empty-segment input.

## Regression Contract

The `optimization_batch_20260828is_` Runtime tests cover canonical and normalized results and guard
the fast-path ordering. The ignored paired release benchmark emits
`RUNTIME291_CANONICAL_ANIMATION_TRACK_PARSE_BENCH_V1`. It performs 100,000 parses of a 103-byte
canonical track path per sample, reduces owned path objects from three to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
