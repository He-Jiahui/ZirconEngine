---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
related_code:
  - zircon_runtime/src/ui/surface/navigation_index.rs
  - zircon_runtime/src/ui/surface/navigation_index/tests.rs
tests:
  - zircon_runtime/src/ui/surface/navigation_index/tests.rs
---

# Runtime Navigation Rebuild Candidate Stream

The retained UI navigation index now populates its spatial and tab candidate
lists directly while iterating the retained `nodes` map. The previous rebuild
first allocated a temporary `Vec<UiNodeId>` for every focus candidate and then
looked each node up again before publishing the same lists. The new pass keeps
the existing candidate membership, modal scopes, ordering keys, and subsequent
sorting unchanged while removing that snapshot allocation and one map lookup
per candidate.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime11A/P1-14 | Stream navigation candidates from the retained node map during rebuild | implemented_pending_validation | Batched Runtime/Editor navigation, surface, asset, and logging static contracts pass `82/82`; implementation-owner Rustfmt and scoped diff checks pass. The navigation test module retains pre-existing formatting drift outside this slice. Managed Runtime Cargo and release navigation timing/allocation evidence remain pending. |

## Validation boundary

This change establishes a deterministic allocation reduction in the rebuild
hot path: no temporary candidate-id snapshot is created and candidate nodes are
not re-looked up after iteration. It does not claim product p50/p95/p99 timing,
RSS, or power improvements until the managed validation path records those
measurements. Coordinator state was not polled.
