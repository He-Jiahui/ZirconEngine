---
doc_type: milestone-detail
status: candidate_static_review_complete_managed_validation_pending
plan_sources:
  - docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime62-clean-world-matrix-read-cache-fast-path.md
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

# Runtime1025: clean world-matrix cache reads

| Slice | Work | Status | Evidence |
|---|---|---|---|
| Runtime62 G19 successor | Use the published `WorldMatrix` row for clean reads; preserve projection for dirty or missing rows; add behavior tests plus Release latency/allocation profiles | `candidate_static_review_complete_managed_validation_pending` | Static source and byte checks only. Managed compile/tests and both Windows Release profiles are pending. No measured allocation result, latency threshold, or performance pass is claimed. |

- [x] Add the clean-cache lookup at the shared world-matrix read projector.
- [x] Preserve dirty and missing-cache projection, composition order, and cycle behavior.
- [x] Add clean-read and dirty/missing fallback behavior tests.
- [x] Add an ignored Windows Release profile at depths 1, 32, and 1,024 with 31 raw latency samples and 256 reads per sample.
- [x] Add a separate clean allocation marker in the existing integration-test binary; it reuses that binary's allocator and samples the committed `World::new` active-camera cache at depth 1 with 31 samples of 256 reads.
- [x] Keep the existing dirty-read marker unchanged. The unit profile reports latency only; the integration allocation profile has not run, and its single-depth setup does not satisfy the full G19/G24 scale gate.
- [ ] Run managed compilation and the focused behavior tests.
- [ ] Run and retain both managed Windows Release raw profiles; compare latency against a suitable baseline before making a speedup claim.
- [ ] Complete G24's required 1K/100K/1M evidence. G19 and G24 remain open pending their managed acceptance gates.

The frozen v3 manifest is unchanged. The exact `derived_state.rs` preimage was
`f86f2f04b7d6978a00eb2b1a53b5ee0ada083738ac5b99bbf2e101a3f57a4573`; candidate
postimage hashes are sealed in the Runtime62 clean world-matrix v4 source
manifest. No Cargo command or coordinator status query was run for this slice.
