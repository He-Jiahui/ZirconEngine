---
doc_type: optimization-implementation
status: candidate_static_review_complete_managed_validation_pending
runtime: Runtime62
gate: RSH-G19
related_code:
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/derived_state_clean_world_matrix_profile.rs
  - zircon_runtime/tests/runtime_scene_projected_read_performance.rs
tests:
  - clean_world_matrix_reads_match_the_published_cache
  - missing_or_dirty_world_matrix_cache_keeps_projection_behavior
  - runtime62_clean_world_matrix_profile
  - runtime62_clean_world_matrix_allocation_profile
---

# Runtime62 RSH-G19: clean world-matrix cache reads

## Candidate change

`project_world_matrix_for_read` now returns the existing `WorldMatrix` row when
the hierarchy/transform frontier is clean. This makes ordinary clean reads use
the published component lookup instead of walking every ancestor. A pending
hierarchy or transform frontier, a pending render-component mutation, or a
missing cache row retains the prior streaming projection and Floyd cycle check.
The fallback still composes each local matrix child-to-parent by pre-multiplying
it, treats self-parent and missing-parent edges as chain termination, and fails
closed on a multi-node parent cycle. Active-hierarchy reads are unchanged.

This optimizes only the clean cached case. Dirty and cache-missing reads remain
O(depth) with O(1) auxiliary space. The code change defines no latency ceiling
and does not establish a measured speedup or a G19 acceptance pass.

## Behavior and performance gates

`clean_world_matrix_reads_match_the_published_cache` checks that a clean public
read matches the component value produced by `World::new`. The fallback
regression checks projection before the first commit, then checks a dirty parent
transform returns the new projected value instead of the stale published row.
The clean behavior assertion checks result compatibility; it does not by itself
prove complexity. Existing projected-read cycle tests continue to cover dirty
multi-node cycles and fail-closed behavior.

The ignored Windows Release unit profile commits each hierarchy fixture and
verifies the leaf cache exists before sampling. It measures `world_matrix` at
depths 1, 32, and 1,024 with 31 samples of 256 reads per depth. Raw batch
latency, per-query latency, and nearest-rank p50/p95/p99 are printed under
`RUNTIME62_CLEAN_WORLD_MATRIX_LATENCY_PROFILE_V1`.

The ignored integration profile reuses the existing test-binary allocator in
`runtime_scene_projected_read_performance.rs`. It measures the clean active
camera cache published by `World::new` at depth 1, with 31 samples of 256 reads,
and prints raw latency and allocation count/requested-byte samples with
nearest-rank p50/p95/p99 under
`RUNTIME62_CLEAN_WORLD_MATRIX_ALLOCATION_PROFILE_V1`. The fixture setup and
cache/value assertions are outside the measured window. The managed invocation
must select this test exactly with `--exact --test-threads=1`; allocation
counting uses the existing process-wide test-binary counter while its profile
window is active. This single-depth allocation profile is not evidence for
depths 32/1,024 or G24. Both profiles have no latency threshold and have not
been run; no allocation or performance pass is claimed.

## Provenance and validation

The frozen v3 source manifest remains unchanged. Its exact `derived_state.rs`
preimage for this successor was
`f86f2f04b7d6978a00eb2b1a53b5ee0ada083738ac5b99bbf2e101a3f57a4573`; the
existing integration profile's v3 preimage was
`5ff8549cad60fdff58a31ebdd83b8bbf78c66a95da5ee1c2ba99c59723e5b2c2`.
Candidate postimage hashes are sealed in
`.codex/state/session-coordinator/async-validation-batches/2026-09-28-runtime62-clean-world-matrix-read-v4-source-manifest.json`.

Pinned Rustfmt 1.94.1 edition 2021 and direct final-newline/trailing-whitespace
checks are the static gates for this slice. Cargo was not run. Managed compile,
behavior tests, and the ignored Windows Release profile remain pending. RSH-G19
and G24 remain open until their managed behavior and performance evidence is
accepted; no latency pass is claimed.
