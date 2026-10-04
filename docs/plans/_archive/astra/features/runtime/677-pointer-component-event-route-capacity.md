---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
related_code:
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
tests:
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
---

# Pointer Component Event Route Capacity

The pointer component-event producer now reserves the known route lower bound
before walking entered and left nodes. The reservation covers one possible
event per entered node, one per left node, and the two activation slots used by
the existing press/release branches. Binding-driven fan-out remains dynamic and
continues to append through the same compiled or metadata source path.

## Plan Completion List

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime11A / pointer component event route | Reserve the existing entered/left route lower bound before event fan-out | implemented_pending_validation | Focused source regression locks the saturating route bound and capacity constructor. Scoped Rustfmt and diff checks pass; the combined Runtime/Editor static-contract batch and managed Rust/product evidence remain pending. |

## Complexity Boundary

The route walk and binding fan-out remain unchanged. The lower-bound reserve
removes geometric growth for the common one-event-per-route-node case without
assuming a fixed binding count or changing event ordering. It makes no claim
about product CPU, allocation bytes, RSS, input-to-present latency, or p50/p95/p99
until managed release evidence is available.

## Source Snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/surface/pointer_component_events.rs` | `4787D498E4E2F4A4982FF0DA18ED682323667AB245E65AD4D95CA922A0AD4566` |

## Managed Gate

No Cargo process was started for this slice: the attempted Cargo submission was
rejected by the external `E:\\Git\\zr_vm` dirty-worktree admission gate. A
separate two-module static contract ticket was accepted asynchronously, but it
does not compile this production path. No coordinator status was polled. The
next owner-attributed multi-task ticket should compile the pointer
component-event regression with the existing Runtime/Editor batch and collect
the input-route allocation/time comparison. Until that batch succeeds, this
record remains `implemented_pending_validation` and makes no product
performance claim.
