---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_runtime/203-runtime-preference-config-storage-authority-durability-migration-multiprocess-product-integration-current-working-tree-review.md
---

# Cancellation authority identity

## Current source and repair

Every BoundedKeyedIoLane starts ticket ids at 1. Cancellation authority checked
only that numeric id, allowing a capability from one lane to cancel another
lane's same-id ticket. Equal generation values do not identify an execution
instance either. This is a shared owner-identity defect below preference storage
consumers, within the Runtime203 bounded IO authority review scope.

Bind authority to a clone of the actual ticket and compare its Arc state identity.
Cloned tickets and capabilities retain authorization; different instances with
the same numeric id/generation are rejected before taking the state lock. The
authority retains only ticket state, not lane, work closure, or backend resources.
No new allocation, global id allocator, registry, or public constructor is added.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M25 | Bind pre-start cancellation capability to its actual ticket instance | implemented_pending_validation | Three regressions include colliding real lanes and post-rejection activation; combined batch pending |

Regression scope: two real lanes with colliding ids/generations, ticket and
authority clones, retired same-id instance, fence-pinned and already-started
outcomes, and authorized cancellation idempotence. Activation proves a rejected
foreign authority leaves target work executable. Constant-time pointer checking
is a source property; no measured performance improvement is claimed.
