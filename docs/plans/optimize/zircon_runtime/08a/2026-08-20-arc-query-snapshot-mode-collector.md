# Runtime08A Arc Query Snapshot and Mode Collector Optimization Record

- Date: 2026-08-20
- Owner: `optimize-physics-query-batch-r2-01a00797-20260820`
- Source plan: `docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md`, P1-6 and P1-7
- Status: implementation and 21-pair release-gate definition complete; combined managed execution pending

## Problem

Every manager query cloned the complete `PhysicsWorldSyncState`, including all body, collider, joint, and material vectors. Query filters then ran `excluded_entities.contains` for every collider, making a large exclusion set an O(colliders * exclusions) scan. `Closest` collected and sorted every hit even though only one result survived, while `First` still visited every candidate before truncation.

## Change

- Manager-owned synchronized worlds are immutable `Arc<PhysicsWorldSyncState>` snapshots. Internal query reads clone the `Arc`; the existing public convenience snapshot keeps its owned return contract.
- Query filters compile exclusion membership once. Up to eight exclusions retain an allocation-free slice scan; larger sets use a `HashSet` for expected O(1) membership.
- A shared mode collector stops `First` after its first hit, computes `Closest` with a linear minimum, and sorts only `All`.
- Ray, overlap, and sweep paths reuse one prepared filter and the shared mode semantics. Backend APIs accept their explicit filter without cloning the complete query DTO.

## Deterministic Performance Evidence

| Operation | Before | After | Reduction |
|---|---:|---:|---:|
| Query snapshot with 4,096 colliders | 4,096 collider values deep-cloned per read | 1 `Arc` refcount increment | 100% of collider deep copies removed |
| Exclusion lookup, 4,096 colliders x 2,048 exclusions | up to 8,388,608 linear comparisons | 4,096 expected O(1) probes after one build | 99.95% fewer worst-case membership probes |
| `First`, 100,000 eligible candidates | 100,000 candidates collected | 1 candidate consumed | 99.999% fewer candidate visits |
| `Closest`, N hits | collect + O(N log N) sort | O(N) minimum | sort and N-result allocation removed |

The ignored release gate emits `PERF_RESULT physics_query_snapshot`, `PERF_RESULT physics_query_filter`, and `PERF_RESULT physics_query_mode`, each with all 21 alternating legacy/optimized sample pairs and nearest-rank P95 values. Acceptance requires Arc snapshot P95 <= 25% of deep clone, prepared-filter P95 <= 50% of linear exclusion, and linear-closest P95 <= 75% of full sort.

## Acceptance

- `physics_query_snapshot_clones_the_arc_instead_of_the_world`
- `ray_query_modes_and_large_exclusions_preserve_contracts`
- `query_modes_preserve_first_closest_and_sorted_all_contracts`
- `first_mode_stops_after_the_first_candidate`
- `first_mode_truncates_prefilled_output_to_its_first_result`
- `prepared_filter_hashes_large_exclusion_sets_and_preserves_membership`
- `physics_query_snapshot_filter_and_mode_release_benchmark_evidence` with `sample_pairs=21`
- Rust formatting, compile/test execution, and release measurements: pending the combined coordinator validation; no direct Cargo run was started.

## Remaining Plan Work

This batch does not close Runtime08A P1-6 or P1-7. Jolt broad-phase query ownership, fixed-generation tickets, caller-owned bounded buffers, async/batch queries, richer filters, and explicit approximate fallback capability remain open.
