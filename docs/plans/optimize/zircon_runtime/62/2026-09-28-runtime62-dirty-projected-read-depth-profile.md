---
doc_type: optimization-implementation
status: candidate_static_review_complete_managed_validation_pending
runtime: Runtime62
gate: RSH-G19
related_code:
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/tests/runtime62_dirty_projected_read_depth_profile.rs
tests:
  - runtime62_dirty_projected_reads_match_flushed_values_after_ancestor_changes
  - runtime62_dirty_projected_read_depth_compatibility_profile
---

# Runtime62 RSH-G19: dirty projected-read compatibility profile

## Scope

The standalone integration binary constructs public `World` chain fixtures at
depths 1, 32, and 1,024, publishes their initial derived components through
public `LevelSystem::tick` calls, then mutates an ancestor while the derived
state is pending. It separately samples `world_matrix` after a root transform
change and `active_in_hierarchy` after a root `ActiveSelf` change. In each
case, the query result must differ from the prior published component where
expected and then match the component published by the next public tick.

The non-ignored behavior test repeats this at all three depths and checks that
the immediate dirty projection remains consistent with the subsequently
flushed value. The ignored Windows Release profile captures 31 batches of 256
queries per domain and depth. Each timer and thread-local allocation window
starts inside `LevelSystem::with_world`, after lock acquisition, and encloses
only the public query loop. Fixture creation, initial ticks, mutations,
semantic assertions, post-sample ticks, percentile calculation, and reporting
are outside the measured window.

The allocator records current-thread `alloc`, `alloc_zeroed`, and `realloc`
request calls and gross requested bytes. It does not subtract deallocations,
measure peak or net memory, or observe allocations on other threads. Every
measured dirty query batch asserts zero request calls and zero gross requested
bytes.

## Results and limits

No Release samples have been run. The profile prints all 31 raw batch latency,
request-call, and gross-requested-byte samples, plus nearest-rank p50/p95/p99.
It has no latency ceiling and establishes no performance pass.

This is a measurement of the immediate compatibility projection while a
derived-state frontier is pending. The query walks the ancestor chain, so this
profile does not demonstrate O(1) dirty reads and does not close RSH-G19. It
does not change the current flush-before-published-component behavior or
select a new public snapshot contract. Managed Windows Release execution
remains pending.

## Candidate source hash

| Path | SHA-256 |
|---|---|
| `zircon_runtime/tests/runtime62_dirty_projected_read_depth_profile.rs` | `6bcdcb11f82b3bb1d09613c54d7acc89b4777198b9f28332efa37df0e651b32e` |

## Verification

Only pinned Rustfmt and direct byte/document-structure checks are in scope.
Cargo compilation and managed execution remain pending. The ignored profile
filter is `runtime62_dirty_projected_read_depth_compatibility_profile` with
`--exact --ignored --nocapture --test-threads=1`.
