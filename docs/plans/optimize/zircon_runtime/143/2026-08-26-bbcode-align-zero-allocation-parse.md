# Runtime143 BBCode Align Zero-Allocation Parse

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime143-editor89-performance-batch-20260826cz-v1`

## Problem

Every BBCode paragraph alignment attribute allocated an ASCII-lowercase `String` before matching a
small fixed keyword set. Rich-text documents can repeat alignment tags across many blocks, making
the temporary allocation proportional to parsed block count.

## Optimization

- Trim the attribute once and compare borrowed text with `eq_ignore_ascii_case`.
- Preserve `left`, `center`, `right`, `fill`, `justify`, `start`, and `end` aliases across mixed
  case, including the shared `fill/justify` result and unknown-value rejection.
- Leave indent, nesting, list marker, and block-stack parsing unchanged.

## Regression Contract

The shared `optimization_batch_20260826cz_` filter owns three Runtime tests: full keyword behavior,
zero-owned-lowercase source shape, and an ignored paired release P50/P95 benchmark. The benchmark
emits `RUNTIME143_BBCODE_ALIGN_ZERO_ALLOCATION_PARSE_BENCH_V1`, executes 262,144 alignment parses
per sample, records keyword allocations from 262,144 to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
