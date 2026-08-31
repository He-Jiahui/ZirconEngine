---
title: Runtime99i Streaming Single Mutable Query
category: zircon_runtime
report_id: Runtime99i-streaming-single-mut-query-2026-08-27
date: 2026-08-27
session_id: root-runtime99i-contiguous-transition-validation-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime99i Streaming Single Mutable Query

## Scope

`QueryState::single_mut_with_ticks` previously collected every stable candidate location into a
temporary vector before checking whether zero, one, or multiple entities matched. Candidate
inspection is read-only; the mutable component fetch already occurs only after the iterator is
finished and exactly one entity has been selected.

The implementation now loops over `stable_query_location_iter` directly. It retains the same one
component-location scratch vector, returns `MultipleEntities` at the same second match, returns
`NoEntities` for zero matches or a failed final revalidation, and performs the same unique mutable
fetch after the loop. Stable order, query-plan selection, filter/tick semantics, and error mapping
are unchanged.

## Behavior Evidence

- `query_state_single_mut_reports_zero_one_many_and_mutates_match` covers empty, unique mutable
  update, and multiple-match outcomes on direct `QueryState`.
- `system_query_single_mut_reports_zero_one_many_and_uses_run_window` covers the same outcomes
  through `SystemState` and its run-window tick policy.
- `test_runtime99i_streaming_single_mut_query_performance_contract.py` requires direct stable
  location iteration, one reused component-location scratch, unique selection before mutable fetch,
  and removal of the candidate projection vector.

## Deterministic Performance Model

The release model uses 131,072 production-layout stable locations. Each location is 32 bytes on x64:
stable entity id, generational internal id, archetype id, and table row. The timed workload has one
match at the final location, so both implementations inspect every candidate. Separate parity cases
cover zero and multiple matches.

| Metric | Collected candidates | Streaming candidates | Reduction |
|---|---:|---:|---:|
| allocations per selection | 1 | 0 | 100.000% |
| allocated bytes per selection | 4,194,304 | 0 | 100.000% |

Each run uses five warmups and 31 alternating sample pairs. Every sample batches eight scans to
amortize Windows scheduling noise:

| Run | Legacy P50 ns | Streaming P50 ns | Reduction | Legacy P95 ns | Streaming P95 ns | Reduction |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 15,060,400 | 1,591,600 | 89.430% | 30,265,100 | 3,517,500 | 88.380% |
| 2 | 14,803,200 | 1,232,600 | 91.670% | 28,792,000 | 2,114,200 | 92.660% |
| 3 | 14,081,600 | 1,392,800 | 90.110% | 19,696,200 | 2,816,000 | 85.700% |
| 4 | 14,511,800 | 1,680,600 | 88.420% | 26,216,100 | 3,554,700 | 86.440% |

The four-run worst case reduces P50 by 88.420% and P95 by 85.700%. Result checksum
`5934484393051454980` and timing checksum `16004514007522166795` are exact and nonzero. The managed
gate requires zero streaming projection allocations/bytes, at least 80% lower P50, at least 75%
lower P95, and exact parity for zero/one/multiple selection and both checksums.

This isolated model measures stable-location projection and selection only. It does not claim
end-to-end component validation/fetch, system scheduling, frame time, power, or external-engine
performance.

## Validation

Passed locally without Cargo:

- 3/3 Python source/performance contracts;
- Rust formatting and scoped diff checks;
- four independent release-model runs with exact outcome parity and every gate met.

Managed validation must run both focused `single_mut_reports_` Rust tests, the three Python
contracts, formatting, scoped diff, and a fresh release model in one coordinator ticket. Cargo
validation is not claimed until that asynchronous ticket reaches a passing terminal state.

## Remaining Parent-Plan Work

Runtime99i still owns world-qualified query plans, sound mutable query composition, chunk leases,
parallel iteration, stable-frame scratch reuse across broader APIs, schedule conflict proofs, event
and command throughput, and product-scale comparison. This slice only removes the stable-location
projection from the existing single-mutable-query path.
