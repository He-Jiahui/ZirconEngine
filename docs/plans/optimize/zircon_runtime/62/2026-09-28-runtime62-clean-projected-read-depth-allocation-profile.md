---
doc_type: optimization-implementation
status: candidate_static_review_complete_managed_validation_pending
runtime: Runtime62
gate: RSH-G19
related_code:
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/tests/runtime62_clean_projected_read_depth_allocation_profile.rs
tests:
  - runtime62_clean_projected_reads_match_published_values_at_chain_depths
  - runtime62_clean_projected_read_depth_allocation_profile
---

# Runtime62 RSH-G19: clean projected-read depth allocation evidence

## Scope

The standalone integration binary constructs 1-, 32-, and 1,024-node chains
with public `World` APIs, installs each into a public `LevelSystem`, and uses
two public ticks to publish derived state and confirm the stable tick runs no
active or world-matrix propagation pass. Before sampling, it compares the
public `world_matrix` and `active_in_hierarchy` results with their published
components and checks the chain's accumulated world translation.

The non-ignored behavior test takes one exact clean-query allocation window per
query and depth and asserts zero allocation requests and zero gross requested
bytes. The ignored Windows Release profile samples both reads at all three
depths with 31 batches of 256 calls each. The TLS `GlobalAlloc` window and
timer begin inside `LevelSystem::with_world`, around only the public query
loop; fixture construction, world replacement, both ticks, cache/behavior
assertions, mutex acquisition, and reporting are outside the sample window.
The allocator records current-thread `alloc`, `alloc_zeroed`, and `realloc`
requests and their gross requested sizes; it does not subtract deallocations or
observe allocations from other threads.

## Results and limits

No Release latency result is available yet. The profile prints raw per-batch
elapsed time, request calls, and gross requested bytes with nearest-rank
p50/p95/p99. Zero allocation requests/bytes is asserted for every measured
batch because it is the explicit G19 query contract; no latency ceiling or
before/after performance claim is added.

This slice tests clean committed reads only. It does not cover dirty reads,
which still take the O(depth) compatibility projection path, and it does not
close all of G19. The gate wording excludes diagnostics repair, so whether a
dirty ordinary read is in scope remains an acceptance question. Managed
Windows Release behavior and profiles remain pending.

## Candidate source hash

| Path | SHA-256 |
|---|---|
| `zircon_runtime/tests/runtime62_clean_projected_read_depth_allocation_profile.rs` | `9cc0f48c1c204ef3dd7a6f250991d2500268ec612768720c7553aad2c54f2540` |

## Verification

Only pinned Rustfmt and direct byte/document-structure checks are in scope.
Cargo compilation and managed execution remain pending. The ignored profile
filter is `runtime62_clean_projected_read_depth_allocation_profile` with
`--exact --ignored --nocapture --test-threads=1`.
