---
related_code:
  - zircon_editor/src/core/play/edit_policy
  - zircon_editor/src/core/play/pending_edits
  - zircon_editor/src/core/play/edit_protection.rs
  - zircon_editor/src/core/play/controller.rs
tests:
  - zircon_editor/src/core/play/edit_policy/tests.rs
  - zircon_editor/src/core/play/pending_edits/tests.rs
  - tools/tests/test_editor04_play_edit_protection_contract.py
doc_type: module-detail
---

# Play Edit Protection

## Ownership

`PlayEditPolicy` is the single classification authority for editor operations while a Play session is active. `PlayEditProtection` serializes policy transitions with edit routing, while `PendingEditQueue` owns deferred `EditorOperationInvocation` values. The controller owns one protection instance and does not duplicate the policy in UI state.

The policy has three Playing tiers:

- play-domain operations are returned to the caller for immediate dispatch;
- edits targeting the document used to start Play are rejected as `RunningDocumentLocked`;
- other edit documents and workspace operations are queued until Play stops.

In Edit mode, edit-domain operations dispatch immediately. A play-domain operation has no valid target and is rejected as `PlayDomainUnavailable`.

## Lifecycle

Entering Play activates plugins, raises edit protection, then starts the selected backend. Backend start failure lowers protection before plugin rollback. A successful stop or terminal backend poll lowers protection and adds `PendingEditDecisionPrompt` to `PlayTransitionReport` when the queue is non-empty. Backend retirement and snapshot cleanup are separate terminal-owner steps: a cleanup failure enters `CleanupFailed`, retains the exact backend/session owner, and must be retried before a new Play start is admitted.

The next Play start is rejected with `PendingEditDecisionRequired` until the caller applies or discards the queue. While a decision callback is running, an explicit resolution barrier rejects both new Play starts and additional resolution attempts. The prompt is data-only; presentation belongs to the notification/dialog owner.

## Pending intents

The queue stores typed `EditorOperationInvocation` values with a monotonic `PendingEditId`, the registration-owned `EditOperationTarget`, and its declared retention policy. It does not store command closures or duplicate the transaction engine. Resolution is exposed by `PlaySessionController`; the controller does not expose its raw protection or queue owner. `apply_pending` drains a frozen batch, invokes the supplied operation dispatcher without holding the queue or controller transition lock, continues after individual failures, and returns each failed intent with its error. Callback panics requeue the in-flight intent before unwinding. `discard_pending` reports discarded queue state for audit or notification.

Entries enqueued concurrently after a batch is drained are not absorbed into that decision; reports expose `remaining_count`. Identifier exhaustion is a typed error and does not enqueue a partial entry.

## Boundaries

This M1 slice does not implement M4 play-domain history, keep-simulation-changes, or automatic asset hot reload. It only establishes the policy and pending-intent boundary that those consumers must use. `EditorOperationInvocation` remains the command-system source of operation identity and arguments; no compatibility command vocabulary was introduced.
