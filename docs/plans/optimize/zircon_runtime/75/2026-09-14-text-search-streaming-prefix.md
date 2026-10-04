---
title: Runtime75 Text Search Streaming Unicode Prefix
category: zircon_runtime
report_id: Runtime75-text-search-streaming-prefix-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Text Search Streaming Unicode Prefix

## Scope

The shared Runtime UI text-search helper already had an allocation-free ASCII prefix fast
path, but its non-ASCII prefix fallback lowercased the complete candidate into a temporary
`String` for every menu/typeahead match. This slice keeps the existing trim and case semantics
while comparing Unicode lowercase scalar streams directly against the already-normalized query.

## Implementation

- `starts_with_lowercase_query` now delegates its non-ASCII branch to
  `unicode_lowercase_starts_with`.
- The helper consumes `char::to_lowercase` output lazily, preserving multi-scalar lowercase
  expansions without materializing the candidate string.
- ASCII matching, empty-query behavior, Unicode expansion semantics, and all caller ordering
  remain unchanged.
- Added a lower regression and ignored marker
  `RUNTIME773_TEXT_SEARCH_STREAMING_PREFIX_BENCH_V1`.

## Deterministic work model

For a non-ASCII prefix probe, the candidate's lowercase stream is consumed only until the query
ends or a mismatch occurs. The old full lowercase buffer is no longer required. This is an
allocation-shape improvement; it is not a claim about allocator, CPU/RSS, or product input
latency percentiles.

## Validation

- TDD source contract: `tools/tests/test_runtime_text_search_streaming_performance_contract.py`
  passes `4/4` after the production and lower-test wiring.
- Lower Rust Unicode expansion/order regression and ignored Release marker are present.
- `rustfmt --edition 2021 --check` passes for the helper and regression source.
- The current combined Runtime/Editor source-contract batch passes `120/120` in `0.080s`; managed
  Cargo, Release allocation, and product p50/p95/p99 evidence remain pending.

## Remaining acceptance

The owner-attributed Windows Runtime/Editor lane must compile the current source, execute the
lower regression and ignored marker, and report the plan-specific allocation and latency gates.
