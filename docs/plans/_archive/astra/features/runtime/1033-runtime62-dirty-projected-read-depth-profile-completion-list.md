---
doc_type: feature-completion-list
status: candidate_static_review_complete_managed_validation_pending
runtime: Runtime62
related_plan:
  - docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime62-dirty-projected-read-depth-profile.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
---

# Runtime1033: dirty projected-read depth compatibility profile

| Item | Implemented | Evidence | State |
|---|---|---|---|
| Immediate dirty-read behavior | At depths 1/32/1,024, mutate a chain root, query before the next public tick, then verify the published derived component and query agree after the tick. | `runtime62_dirty_projected_reads_match_flushed_values_after_ancestor_changes` covers root transform to leaf `WorldMatrix` and root `ActiveSelf` to leaf `ActiveInHierarchy`. | candidate_static_review_complete_managed_validation_pending |
| Dirty compatibility-path samples | Add an ignored Windows Release integration profile with 31 batches of 256 queries per read domain and depth; emit raw latency and current-thread heap request samples plus p50/p95/p99. | `runtime62_dirty_projected_read_depth_compatibility_profile` asserts zero request calls and gross requested bytes for each measured batch. It has not run. | candidate_static_review_complete_managed_validation_pending |
| RSH-G19 acceptance | Keep G19 open. Dirty ordinary reads still project through ancestors in O(depth); these samples do not establish O(1), and no latency ceiling is defined. | The profile measures the existing immediate compatibility semantics only. A published-snapshot contract or write-side affected-subtree publication requires separate product and implementation review. | open |

- [x] Add a standalone profile binary and new records without editing v10 frozen paths.
- [ ] Run the dirty behavior regression in managed grouped validation.
- [ ] Run the serialized Windows Release profile and retain raw output.
- [ ] Resolve whether G19 requires O(1) ordinary reads while derived state is pending, and define the corresponding product contract.
