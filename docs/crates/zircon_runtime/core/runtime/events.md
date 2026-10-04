---
related_code:
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/events/admission.rs
  - zircon_runtime/src/core/runtime/events/close.rs
  - zircon_runtime/src/core/runtime/events/frozen.rs
  - zircon_runtime/src/core/runtime/events/diagnostics.rs
  - zircon_runtime/src/core/runtime/events/prune.rs
  - zircon_runtime/src/core/runtime/events/publish.rs
  - zircon_runtime/src/core/runtime/events/subscribe.rs
  - zircon_runtime/src/core/runtime/events/subscriber.rs
  - zircon_runtime/src/core/runtime/events/topic.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/framework/events.rs
implementation_files:
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/events/admission.rs
  - zircon_runtime/src/core/runtime/events/close.rs
  - zircon_runtime/src/core/runtime/events/frozen.rs
  - zircon_runtime/src/core/runtime/events/diagnostics.rs
  - zircon_runtime/src/core/runtime/events/prune.rs
  - zircon_runtime/src/core/runtime/events/publish.rs
  - zircon_runtime/src/core/runtime/events/subscribe.rs
  - zircon_runtime/src/core/runtime/events/subscriber.rs
  - zircon_runtime/src/core/runtime/events/topic.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
plan_sources:
  - user: 2026-06-12 runtime architecture implementation from docs/plans/zircon_runtime/runtime
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/core/runtime/tests/events
  - zircon_runtime/src/core/runtime/events/admission/tests.rs
  - zircon_runtime/src/core/runtime/tests/events/benchmark_evidence.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
doc_type: module-detail
---

# Runtime Events

## Ownership and public contract

`CoreRuntimeInner` owns `EventBus`. Framework events defines neutral producer DTOs,
bounded policies, receipts and lease-bearing received handles. Runtime events owns
the registry, immutable frozen envelopes, queue admission, retention ledger and close.

`EventBus::try_publish(EngineEvent)` and `CoreHandle`/`CoreRuntime::try_publish_event`
return `Result<EngineEventPublishReceipt, EngineEventPublishRejected>`. Every
rejection contains the original owned input. `NoSubscribers` is an explicit
rejection without replay or payload preparation. Accepted means admitted to every
current subscriber queue, not consumed or durable. Retrying a rejected event
cannot duplicate a partial fanout: admission succeeds for all recipients or none.

Subscriptions return `Result` and require `Reliable`, `DropOldest` or `Latest`
with finite count and retained-byte limits. Reliable rejects pressure. Lossy
policies evict only queued deliveries and report replacement count and bytes;
they cannot evict a handle already returned to a consumer. Oversized candidates
leave existing deliveries intact. No unbounded policy or old void publication
entry remains.

## Retention and concurrency

The bus freezes producer JSON once into an exact-length `Box<[u8]>`, with a
`Box<str>` topic. It does not retain the producer's `Value`, spare String capacity,
or decoded JSON trees. Bytes count exact owned frozen buffers conservatively per
subscriber delivery; shared payloads are charged once for each recipient. Fixed
envelope, lease and queue metadata is separately bounded by delivery and registry
count limits. Allocator metadata and process RSS are not encoded-buffer bytes.

`EngineEventDelivery::clone` shares both immutable payload and a retention lease.
`recv` releases queue depth but keeps count/bytes charged until the last clone
drops. Unsubscribe and close drain queued deliveries outside queue locks; consumer
handles survive and remain charged. JSON decoding creates caller-owned data and
returns a typed parse result. The handle exposes borrowed topic/payload bytes,
not a detachable raw payload Arc. `event_bus_retention` remains mandatory even
when optional timing/traffic diagnostics are disabled.

Lock order is topic delivery, global admission commit, subscriber queues in ID
order, then retention ledger. Registry locks are released before delivery waits;
serialization runs without these locks. Replacement preflights all limits,
detaches old values, releases queue/ledger guards and destroys them while commit
capacity is reserved. Only after old leases release their actual retained buffers
does the already-admitted fanout enqueue. No old envelope loses its charge before
destruction. A topic held before acquiring commit does not block another topic's
admission. Lease Drop acquires only the ledger.

`close_event_admission` is terminal and idempotent. It rejects new publishers and
subscribers, waits for earlier topic commits, drains queues and wakes receivers.
It does not revoke received handles or claim a complete runtime/DLL unload barrier.
Queue, ledger or interrupted delivery poison closes admission instead of assuming
an interrupted transaction is safe to resume.

## Validation scope

`core::runtime::tests::events`, runtime admission tests and App profile-bootstrap
cover bounded policies, count/byte boundaries, owned rejections, shared fanout,
retained clone lifetimes, replacement, unsubscribe, close and relevant races.
Existing source guards follow the folder-backed owners and new failure contract.

Managed Runtime07 benchmarks still need current-source execution. Serialization,
lease accounting and global commit serialization change costs; old Arc-clone and
replacement-RMW claims are not qualification for this contract. Measure publish
and receive P50/P95/P99, allocations, throughput and mandatory retained bytes in
the declared Windows batch. Large teardown fixtures declare explicit finite test
budgets. Pending/static/rustfmt results are not a Cargo or performance pass.

RT02-P1-6 remains open: this bounded JSON control boundary does not implement
frame-local `Messages<T>`, generic `ControlBus<T>`, async scheduler wake/cancellation
or the versioned plugin event envelope split.
