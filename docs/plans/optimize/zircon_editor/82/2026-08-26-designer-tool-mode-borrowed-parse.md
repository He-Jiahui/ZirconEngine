# Editor82 Designer Tool Mode Borrowed Parse

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime136-editor82-performance-batch-20260826cs-v1`

## Problem

Every UI designer tool-mode parse lowercased its token into a temporary `String` before matching
seven fixed ASCII aliases. This path is used when reconstructing editor interaction state and did
not need owned normalization.

## Optimization

- Trim once, bucket aliases by their exact byte lengths, and compare with
  `eq_ignore_ascii_case`.
- Remove the normalized `String` allocation from every successful and rejected parse.
- Preserve select, resize-slot, preview-interact aliases, whitespace handling, and safe rejection
  of non-ASCII near-matches.

## Regression Contract

The shared `optimization_batch_20260826cs_` filter owns three Editor tests: alias behavior, source
shape, and an ignored paired release P95 benchmark. The benchmark emits
`EDITOR82_DESIGNER_TOOL_MODE_BORROWED_PARSE_BENCH_V1`, performs 240,000 mixed alias parses per
sample, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
