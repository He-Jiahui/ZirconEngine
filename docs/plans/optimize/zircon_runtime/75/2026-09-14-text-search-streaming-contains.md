---
title: Runtime75 Text Search Streaming Unicode Contains
category: zircon_runtime
report_id: Runtime75-text-search-streaming-contains-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Text Search Streaming Unicode Contains

## Scope

The shared Runtime UI text-search helper used an allocation-free ASCII contains path, but its
non-ASCII fallback materialized a full lowercase candidate string for every command-palette and
menu filter probe. This slice compares the same Unicode lowercase scalar stream directly against
the normalized query, including scalar-expansion and overlapping-match cases.

## Implementation

- `contains_lowercase_query` delegates its non-ASCII path to
  `unicode_lowercase_contains`.
- The helper walks each candidate scalar in the lazily lowercased stream and clones only iterator
  state for a potential suffix comparison; it does not construct a candidate lowercase buffer.
- ASCII matching, trimming, empty-query behavior, Unicode scalar matching, and caller ordering
  remain unchanged.
- Added a lower regression and ignored marker
  `RUNTIME774_TEXT_SEARCH_STREAMING_CONTAINS_BENCH_V1`.
- Updated the existing shared-search source contract so it requires both streaming Unicode helpers
  instead of treating the retired `value.to_lowercase()` fallback as an invariant.

## Deterministic work model

For a non-ASCII contains probe, the implementation uses constant-size iterator state plus the
already-authored query iterator. It eliminates the full temporary lowercase candidate allocation.
This is allocation-shape evidence, not a claim about allocator, CPU/RSS, or product
input-to-present latency percentiles.

## Validation

- TDD source contract: `tools/tests/test_runtime_text_search_contains_streaming_performance_contract.py`
  passes `4/4` after production and lower-test wiring.
- The related prefix contract and updated ASCII shared-search guard pass in the focused `14/14`
  source-contract batch.
- A standalone module compile probe ran the seven lower tests: `5` passed and `2` Release markers
  remained intentionally ignored.
- `rustfmt --edition 2021 --check` passes for the helper and both text-search regressions.
- The current combined Runtime/Editor source-contract batch passes `120/120` in `0.080s`; managed
  Cargo, Release allocation, and product p50/p95/p99 evidence remain pending.

## Remaining acceptance

The owner-attributed Windows Runtime/Editor lane must compile the current source, execute both
lower regressions and their ignored markers, and report the plan-specific allocation and latency
gates.
