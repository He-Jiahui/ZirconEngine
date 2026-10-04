---
doc_type: feature-completion-list
status: candidate_static_review_complete_managed_validation_pending
runtime: Runtime62
related_plan:
  - docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime62-derived-propagation-scale-profile.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
---

# Runtime1029: derived propagation scale profile

| Item | Implemented | Evidence | State |
|---|---|---|---|
| Isolated propagation timing | Added an ignored Windows Release profile for 1/1K/100K/1M star hierarchies and 1/32/1,024-node chains, with 31 raw elapsed-time, visited, and written samples per matrix and active pass. | The profile calls one internal propagation system per sample after root mutation and publishes pending dirty state outside the timer; pinned Rustfmt and scoped diff checks pass. | candidate_static_review_complete_managed_validation_pending |
| RSH-G24 scale acceptance | The new profile has not run and contains no allocation-byte measurement or numeric ceiling. | A successor grouped Release validation must run and retain raw output; allocation and threshold work remains. | open |

- [x] Add a separate ignored Release latency and work-counter harness.
- [ ] Run the exact profile in managed grouped validation and retain raw samples.
- [ ] Measure allocated bytes, define a numeric ceiling, and show 1K/100K/1M pass against it.
