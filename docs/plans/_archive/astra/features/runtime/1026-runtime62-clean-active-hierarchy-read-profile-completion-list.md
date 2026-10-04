---
doc_type: milestone-detail
status: candidate_static_review_complete_managed_validation_pending
plan_sources:
  - docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime62-clean-active-hierarchy-read-profile.md
related_code:
  - zircon_runtime/src/scene/world/derived_state_clean_world_matrix_profile.rs
  - zircon_runtime/tests/runtime_scene_projected_read_performance.rs
tests:
  - runtime62_clean_active_in_hierarchy_profile
  - runtime62_clean_active_in_hierarchy_allocation_profile
---

# Runtime1026: clean active-hierarchy read profiles

| Slice | Work | Status | Evidence |
|---|---|---|---|
| Runtime62 G19 clean active read | Add committed-chain latency samples at depths 1/32/1,024 and a depth-1 zero-allocation profile using the existing test-binary allocator. | `candidate_static_review_complete_managed_validation_pending` | Pinned formatting and scoped diff checks pass; managed Release samples are pending. |

- [x] Verify the published active cache and clean frontier before latency sampling.
- [x] Collect 31 batches of 256 public reads per depth and report raw latency plus p50/p95/p99.
- [x] Reuse the existing allocation counter without adding another global allocator.
- [ ] Run both ignored profiles through the managed Windows Release batch.
- [ ] Compare with the full G19/G24 acceptance evidence; no performance threshold or pass has been established.

This slice changes test and profile coverage only. It does not close G19/G24 or
claim a measured performance improvement. The frozen v4 validation request
received no ticket; these changed source hashes belong in a later successor
batch after static review and path attribution.
