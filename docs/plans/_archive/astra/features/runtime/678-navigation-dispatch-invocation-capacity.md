---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
related_code:
  - zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs
tests:
  - zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs
---

# Navigation Dispatch Invocation Capacity

The navigation dispatcher now reserves the candidate-count upper bound for its
invocation result before traversing the published route. Each candidate can
produce at most one recorded handled/focus invocation, and the existing early
stop and visited-node semantics are unchanged. Empty routes and handler-free
dispatches still return the same empty result without entering the allocation
path.

## Plan Completion List

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime11A / navigation dispatch | Reserve `candidates.len()` for the bounded invocation output | implemented_pending_validation | Focused source regression locks the reservation and candidate upper bound. Rustfmt, scoped diff checks, and the Runtime + Editor performance-contract discovery passed `1727/1727`. Managed Cargo and product input CPU/allocation/p50/p95/p99 evidence remain pending. |

## Complexity Boundary

The route walk, handler ordering, visited membership, effect handling, and
result ownership are unchanged. The reservation removes geometric growth when
handlers record one invocation per candidate; it does not claim that every
candidate has a handler or that product latency/allocation bytes improve by a
fixed percentage.

## Source Snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs` | `0FDFBCF1838DE2BED71D07644830CCCB847B057C3DB971F1F9CAEBB62D8C8333` |

## Managed Gate

The first submission was rejected because the immutable Session had no live
attribution; after lease/attribute, the Cargo submission was rejected by the
external `E:\\Git\\zr_vm` dirty-worktree gate. A separate static contract
ticket was accepted asynchronously, but it does not compile this production
path. No Cargo process or status polling was started. This record therefore
remains `implemented_pending_validation` until an owner-attributed multi-task
ticket can run the focused navigation regression with the existing
Runtime/Editor batch and collect current-source release evidence.
