---
title: Editor48 Plugin Lifecycle Pending-queue Backpressure
category: zircon_editor
report_id: Editor48-plugin-lifecycle-pending-backpressure-2026-09-28
date: 2026-09-28
implementation_status: implemented_pending_validation
validation_status: managed_validation_pending
---

# Editor48 Plugin Lifecycle Pending-queue Backpressure

## Scope

This is a narrow partial repair for E-MSG-P1-36 in the [Editor48 integration review](../48-editor-message-bus-topic-subscription-inbox-retention-admission-dispatch-request-dirty-projection-shutdown-product-integration-review.md).
The source-confirmed retained-delivery failure path is a concurrent plugin-manager mutation:
`dispatch_lifecycle_event_to_active` returns `MutationInProgress`, and the bridge retains the
delivery. Plugin callback failures are reported by the manager and counted by the bridge; they do
not enter this retry queue.

Before this change, every `pump` drained the bus before retrying retained deliveries. Repeated
mutation deferrals could therefore move each bounded bus batch into the separate `pending` queue,
leaving the bus free to admit another batch and allowing the shadow queue to grow without a bound.

## Implementation

The bridge now checks its retained queue before draining. If a previous batch is pending, the pump
retries that batch first and does not drain newer bus deliveries during the same pump. Once the
pending batch completes, the next pump may drain from the bus. This preserves FIFO order and limits
the detached shadow queue to one drained batch; the bus keeps newer deliveries under its own
configured inbox limits and reports backpressure when those limits are reached.

This does not make the private pending queue part of the bus subscription's count/byte/age
reservation. While the bridge is stalled, memory can still include one detached batch plus the
bounded bus inbox. A shared acknowledgement window, explicit retry/quarantine policy, and complete
P1-36 closure remain open.

## Regression Coverage

The bridge integration regression uses a real `SharedEditorMessageBus`, the installed bridge
subscriber, and an active plugin manager blocked by a concurrent lifecycle callback. It defers an
older Play transition, queues a newer transition, and verifies that another deferred pump leaves the
newer event in the bus. A one-message lossless inbox then reports backpressure for another publish.
After the manager resumes, the older transition retries first, the newer one drains on the following
pump, and republishing after inbox space is freed is accepted. The test checks reports, inbox depth,
and callback order.

## Validation

- The focused regression and scoped static checks are authored for managed execution; Cargo was not
  run in this implementation slice, so compilation and dynamic behavior remain unverified.
- No performance result is claimed. Allocation, latency, retained-byte, and managed runtime gates
  remain pending.
- This slice does not close plugin callback retry/quarantine, the shared acknowledgement window, or
  the broader Editor48 message lifecycle and delivery contracts.
