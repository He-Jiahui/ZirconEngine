---
doc_type: feature-completion-list
status: candidate_static_review_complete_managed_validation_pending
runtime: Runtime62
related_plan:
  - docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime62-depth-bounded-propagation-stack.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
---

# Runtime1030: depth-bounded derived propagation stack

| Item | Implemented | Evidence | State |
|---|---|---|---|
| Active and matrix propagation traversal | Replace wide reverse-push sibling copying with an iterator only for each branching ancestor, preserving stable depth-first order and visited/written publication. | A nested-branch behavior regression replaces the obsolete exact stack-spelling assertion; pinned Rustfmt and scoped diff checks pass. New and existing behavior tests are selected for successor grouped validation. | candidate_static_review_complete_managed_validation_pending |
| RSH-G24 performance acceptance | A new ignored Release profile covers 1/1K/100K/1M wide stars and shorter single-child chains, but has not run. Allocation-byte samples and a numeric ceiling remain missing. | No measured latency or allocation result is claimed. | open |

- [x] Make pending traversal storage proportional to branching depth rather than sibling width or single-child chain depth.
- [ ] Run behavior tests and the Release profile in managed grouped validation.
- [ ] Capture allocated-byte samples and show raw 1K/100K/1M results below a defined ceiling.
