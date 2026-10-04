---
doc_type: feature-completion-list
status: candidate_static_review_complete_managed_validation_pending
runtime: Runtime62
related_plan:
  - docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime62-clean-projected-read-depth-allocation-profile.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
---

# Runtime1032: clean projected-read depth allocation profile

| Item | Implemented | Evidence | State |
|---|---|---|---|
| Clean read behavior at depth | Build and commit public LevelSystem chain fixtures at depths 1/32/1,024; compare clean query values with published WorldMatrix/ActiveInHierarchy components and expected world translation. | `runtime62_clean_projected_reads_match_published_values_at_chain_depths`; the stable follow-up tick must report zero active and world-matrix propagation passes. | candidate_static_review_complete_managed_validation_pending |
| Clean read allocation samples | Add an ignored Windows Release standalone integration profile with its own TLS allocator and 31 batches of 256 exact query calls for each depth and query. | `runtime62_clean_projected_read_depth_allocation_profile` asserts zero request calls and gross requested bytes in each batch and emits raw batches plus p50/p95/p99. It has not run. | candidate_static_review_complete_managed_validation_pending |
| RSH-G19 acceptance | Keep G19 open pending managed behavior/Release evidence and resolution of the dirty-read scope. | This covers clean committed read allocations at all three depths; dirty compatibility reads remain O(depth). No latency ceiling or full G19 pass is claimed. | open |

- [x] Add a new test binary and records without changing v7 paths or production code.
- [ ] Run the behavior regression in managed grouped validation.
- [ ] Run the ignored serialized Windows Release profile and retain all raw output.
- [ ] Resolve whether dirty ordinary reads are included in the RSH-G19 O(1) requirement.
