---
related_code:
  - zircon_editor/src/core/notifications/decision
  - zircon_editor/src/core/notifications/presentation.rs
implementation_files:
  - zircon_editor/src/core/notifications/decision/model.rs
  - zircon_editor/src/core/notifications/decision/center.rs
  - zircon_editor/src/core/notifications/presentation.rs
plan_sources:
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
tests:
  - zircon_editor/src/core/notifications/decision/tests.rs
  - zircon_editor/src/core/notifications/presentation_tests.rs
  - zircon_editor/src/ui/host/play_pending_decision/tests.rs
  - tools/tests/test_editor17_decision_notification_center_contract.py
doc_type: module-detail
---

# Decision Notification Center

## Ownership

`DecisionNotificationCenter` is the core authority for notifications that require an explicit user choice. Producers publish validated `DecisionNotification` values; retained UI, headless drivers, and replay consumers read the same snapshots and submit typed tickets. Presentation code does not own pending state or execute callbacks inside the center.

`NotificationId` identifies the producer-defined logical prompt. `DecisionTicket` additionally identifies the owning center instance and one published incarnation, so an old UI surface cannot resolve a prompt in a replacement center or a later prompt that reuses the same logical ID. Tickets, cursors, and receipts can only be created by the center. Center instance IDs are process-local authority epochs; this in-memory service does not define persisted tickets across process restarts.

## Model

A decision carries a builtin or plugin source, localization keys, at least two uniquely identified options, and optional default/cancel options that must refer to declared options. It can additionally carry up to eight immutable `u64` message facts under static lowercase-ASCII underscore names of at most 64 bytes. Constructors validate all identifiers and required strings; model fields remain private so callers cannot bypass those invariants. Notification IDs are limited to 192 bytes, option IDs to 64 bytes, source IDs to 128 bytes, localization keys to 256 bytes, and one notification to 16 options. The message-fact API deliberately excludes producer text, callbacks, and dynamic format expressions.

`DecisionNotification` stores its validated immutable payload behind `Arc`; snapshots clone that shared payload instead of duplicating producer strings and option vectors. `publish` returns a ticket. `pending_snapshot` returns only unresolved entries, while `snapshot` also exposes retained resolved entries until their receipts leave bounded history. Both snapshots include the ticket required by `resolve` or `cancel`.

## Presentation

`notifications::present_decision` is the read-only localization edge for a decision snapshot. It captures one active locale before resolving the title, message, and option label keys through an explicit `EditorI18nService`, then substitutes only the declared message facts into matching `{name}` placeholders; a concurrent locale change therefore cannot mix languages inside one immutable projection. The captured locale, ticket, option IDs, notification source, default/cancel option IDs, and any receipt remain available to consumers. Toast and Progress use equivalent read-only projections; expiry, severity, and the authoritative job snapshot remain unchanged. No localized projection is stored in the notification authority or accepted as an action input.

## Resolution

Resolution is serialized by the center and produces one monotonic receipt. Repeating the same ticket and option returns that receipt with `newly_resolved = false`; requesting a different option returns `AlreadyResolved`. Window-close behavior must call `cancel`, which only succeeds when the producer declared a cancel option.

No callback is stored or invoked. A consumer starts from `center.initial_cursor()`, observes `receipts_since(cursor)`, and routes newly observed receipts to the owning feature. This keeps UI teardown, headless operation, and event replay on one auditable data path. Consumers must persist their cursor only after their own command dispatch succeeds.

The Play pending-edit adapter is the production consumer for its own tickets. `EditorHostEventController::pump_runtime_event_consumers` drains that adapter before polling the backend; retained UI resolves through the same drain, while headless and replay callers invoke `pump_pending_play_decision_receipts`. A failed later receipt leaves prior successful effects recorded until the batch cursor commits, so retry never repeats an earlier apply/discard command. The adapter ignores receipts owned by other features and advances over them without becoming their second consumer.

An expired Play receipt cursor is never silently resumed. The host first republishes the still-pending edit prompt, then advances the adapter past the stale retained range and returns an explicit recovery error. The old Apply/Discard choice is not inferred or replayed after eviction; the replacement Decision requires a new explicit choice. If republishing fails, the adapter cursor remains unchanged and the error stays observable.

## Bounds And Failure Semantics

Pending and receipt capacities are explicit and non-zero. Combined with per-field and option-count limits, they bound the retained payload. The center uses deterministic keyed storage for entries and bounded FIFO receipt history. Capacity and sequence exhaustion reject before partial insertion. A foreign-center ticket or cursor returns a typed authority error. When a cursor predates retained history, `CursorExpired` includes both the oldest sequence and a directly reusable `resume_cursor`; retrying from it includes the oldest retained receipt instead of silently skipping decisions.

Evicting a receipt also retires its resolved entry. A later publication may reuse the logical notification ID, but receives a new incarnation. Any stale ticket is rejected with `StaleTicket` and cannot mutate the new entry.

## Integration Boundary

This module is the Editor17 M3.2 core Decision authority. It intentionally does not mount UI or invoke feature callbacks. The Editor04 pending-edit adapter publishes its prompt through this authority and routes apply/discard receipts to `PlaySessionController`; UI and headless consumers therefore act on the same ticket and receipt data. The retained notification center is a current-snapshot projection of Decision, Toast, and Progress authority, not a second notification history. The cross-plan failure remains open only until the current-source managed integration gate is accepted.
