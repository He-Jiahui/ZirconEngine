# Editor91 Command Entry Direct Append

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime145-editor91-performance-batch-20260826db-v1`

## Problem

Nested command-palette values recursively returned a new `Vec` from every array and leaf before
flattening those temporary collectors into their parent. Deep declarative command trees therefore
created allocation and growth work proportional to the number of parse nodes.

## Optimization

- Traverse nested arrays into one caller-owned output vector.
- Append valid string and table entries directly while preserving depth-first source order.
- Preserve invalid-value rejection, empty-ID rejection, aliases, labels, descriptions, and disabled
  state parsing.

## Regression Contract

The shared `optimization_batch_20260826db_` filter owns three Editor tests: nested output behavior,
single-collector source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR91_COMMAND_ENTRY_DIRECT_APPEND_BENCH_V1`, parses 1,024 entries per iteration, records
collector instances from 1,365 to one per parse, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
