---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-24-runtime-events-and-task-hotpaths.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/core/runtime/events/prune.rs
  - zircon_runtime/src/core/runtime/events/subscriber.rs
tests:
  - zircon_runtime/src/core/runtime/events/prune.rs
---

# Runtime882 Event Unsubscribe Lock-Scope Optimization

## 完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Topic delivery critical section | Subscriber deactivation and snapshot removal still occur under the topic delivery lock, preserving the publish/unsubscribe ordering barrier. | `EventSubscriber::deactivate` returns the owned queue only after atomically marking the subscriber inactive; `EventTopic::remove_subscribers_while_delivery_locked` remains inside the same lock scope. | implemented |
| Queue-age diagnostics | Batched queue drain accounting now runs after the delivery lock is released, so a large lossless queue no longer makes publishers wait for per-record age aggregation. | `prune.rs` source contract asserts `drop(_delivery)` precedes `record_deactivated_queue(queued)`; existing `record_drained` counters and disconnected semantics are retained. | implemented |
| Existing teardown paths | Direct subscription drop, event-bus drop and test-only deactivation retain the `deactivate_and_drain` wrapper and therefore keep their previous behavior. | Wrapper delegates to the split deactivate/accounting operations; no production event payload or policy was changed. | implemented |

## Validation boundary

Rustfmt and scoped diff checks pass for the two Runtime event files. This source
repair was made after the previous Runtime source snapshot and still needs one
grouped managed Runtime/Editor Cargo check, library test batch and applicable
ignored Release evidence. No per-file Cargo run or tooling change was made;
performance acceptance is not inferred from the source guard alone.

The current grouped Runtime request is included in development job
`42db99a91bce4a0ca4b29d389670f4d8`, which was still `running` in the single
post-review ledger read. No Cargo run receipt is available yet; validation and
performance remain pending without per-test retries.

After the later source stabilization, a new package-wide managed Runtime
library check/test wrapper was launched together with the Editor wrapper. It
was intentionally left unpolled, so this record still claims no Cargo stage,
test count, or performance result. The Runtime02 Release admission attempted in
the same wave returned coordinator `request_overloaded` before creating a job;
Release evidence remains pending.
