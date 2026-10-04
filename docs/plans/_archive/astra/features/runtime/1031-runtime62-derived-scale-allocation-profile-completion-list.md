---
doc_type: feature-completion-list
status: candidate_static_review_complete_managed_validation_pending
runtime: Runtime62
related_plan:
  - docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime62-derived-scale-allocation-profile.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
---

# Runtime1031: derived scale allocation profile

| Item | Implemented | Evidence | State |
|---|---|---|---|
| Public tick behavior regression | Add a non-ignored 1K star integration regression for changed child world matrix/active state and exact visited/written counts. | `runtime62_derived_scale_tick_publishes_matrix_and_active_state_for_small_star`; managed execution pending. | candidate_static_review_complete_managed_validation_pending |
| 1K/100K/1M scale profile | Add an ignored Windows Release profile with 31 root-transform and 31 root-active samples per size. | `runtime62_derived_scale_allocation_profile` records current-thread TLS allocator request counts, gross requested bytes, full public tick-envelope latency, and visited/written counts. It has not run. | candidate_static_review_complete_managed_validation_pending |
| RSH-G24 acceptance | Keep G24 open until managed raw samples, a product-grounded ceiling, and an acceptance run exist. | No numeric ceiling or measured pass is claimed; 1M fixture peak memory is unverified. | open |

- [x] Add the standalone profile and behavior assertions without changing production or v6 frozen paths.
- [ ] Run the small integration regression and ignored profile in serialized managed Windows Release validation.
- [ ] Retain raw 1K/100K/1M visited, written, allocation-request, gross-byte, and tick-envelope latency samples.
- [ ] Establish a defensible numeric ceiling from the product frame budget or comparable baseline evidence, then evaluate G24.
