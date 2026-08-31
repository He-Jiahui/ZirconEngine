---
related_code:
  - zircon_editor/src/core/runtime_event_consumer/**
  - zircon_editor/src/ui/host/editor_host_event_controller/runtime_event_consumers.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/core/gateway/**
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_runtime_interface/src/runtime_api/session/**
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
status: pending
---

# Runtime event consumer closure

This is a current-source static closure for the retained editor play-session
event path. It remains pending: the path is dynamically reachable from the
retained-host tick, but the current-source Cargo/product gates are not green.
No production code was changed and no file is accepted into `review.md` from
this record alone.

## Scope and source state

- `zircon_editor/src/core/runtime_event_consumer/**`: 16 Rust files, 3,754
  physical lines, 3,416 nonempty lines, 134,786 bytes, 16 inline tests, one
  ignored test and one include site. Sorted raw-content SHA256:
  `4a2024eec6a2cc655519f541223d6b238b8fc0d76385f7024a32bf34f2947add`.
- The current foreign work is directionally positive and remains preserved:
  fault receipts, lifecycle/retirement helpers, pending-page restoration,
  round-robin cursor reuse, callback panic containment, slow-consumer health,
  bounded one-page pending storage, and callback execution outside the active
  consumer map lock.
- The producer contract currently exposes 64 deliveries and 256 KiB encoded
  bytes per drain page (`ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1` and
  `ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1`). The runtime queue and
  host budgets remain separate owners.
- Focused formatting and behavior gates are not a current-source acceptance:
  the repository still has the known UI/text compiler failures, SDF cfg(test)
  mismatch, stale compiled-scene/OIT source guards and graphics feature
  reexport issue. No accepted executable, WPR/Tracy/RenderDoc capture or
  benchmark result is claimed.

## Product boundary and positive behavior

The path is default retained-host work during an active play session. The
controller reconciles capabilities and calls the host pump before UI sync on
each retained tick. Runtime event production is owned by the dynamic-session
event mirror; the gateway transports one bounded page; the host owns pending
delivery, callback execution, quarantine and lifecycle state.

Current behavior worth preserving:

1. Runtime event pages are bounded by delivery count and encoded bytes. The
   runtime retains a pending page until editor-side output commit and can roll
   back an output attempt.
2. The host retains at most one decoded page per consumer and applies a global
   retained-byte limit. Pump work has max-event and elapsed-time checks,
   round-robin visitation and p95 drain/decode telemetry.
3. Active consumer snapshots are copied under the map lock and callbacks run
   after that lock is released. Begin, consume and end callback panics are
   contained; repeated slow or faulty consumers can be quarantined.
4. Subscription and consumer generations are checked, stale gateway leases
   are retired, lifecycle execution prevents overlapping pump/reconcile work,
   and contribution retirement attempts remote cleanup with an explicit error.
5. Pending batches have a restore guard, so an incomplete callback budget does
   not silently discard the unexecuted tail. The reused round-robin cursor also
   avoids rebuilding its consumer-id allocation on every rotation.

## Findings

### P0: destructive producer drain precedes host admission

`pump_execution.rs` drains a page whenever a consumer has no pending batch,
then `append_drained_deliveries` attempts the host pending-byte reservation
after the page has been decoded. A reservation failure drops the decoded
deliveries and only increments a host report counter. The producer-side
`commit_plugin_event_output` may already have removed that prefix from its
pending page, so host pressure can lose events with no producer retry or
terminal receipt. The 64-delivery page can also exceed the remaining host
budget even when the callback loop itself is within its event limit.

Admission must reserve the exact wire, decoded, object and callback/result
capacity before a destructive drain. Prefer a producer `peek/lease/commit`
protocol that keeps the page until host acknowledgement; an inadmissible page
returns typed `Backpressured` and performs zero drain/commit work.

### P0: retained-byte accounting measures the wrong representation

`PendingDeliveryBatch::from_page` divides `encoded_bytes` across deliveries and
uses that value as retained bytes. The retained host actually owns decoded ABI
objects, `VecDeque` storage, event and schema identifiers, boxed raw values,
metadata and allocator overhead. Synthetic pages can report zero encoded bytes
while carrying arbitrary decoded payload. During decode, wire bytes and
decoded objects coexist, so the peak is higher than either value. Fault
receipts and quarantine metadata are likewise outside the raw-payload counter.

Publish separate admitted fields for wire bytes, decoded payload bytes,
object/label overhead, current bytes and candidate/peak bytes. Count all owners
that can be retained by a pending or fault generation before allocation.

### P0/P1: stable capability reconciliation and snapshot cloning

The active controller clones `enabled_capabilities().to_vec()` and calls
`reconcile_enabled_capabilities` every active tick. Reconciliation clones the
full registration map, quarantine and disabled sets, scans enabled capability
names for every registration, and retries remote cleanup. The subsequent pump
clones every active consumer identity, registration, origin and subscription
into another snapshot vector even when capability and gateway generations are
unchanged. Empty consumers still issue a gateway drain attempt when no pending
page exists.

Compile one immutable eligibility generation keyed by registry, capability,
quarantine, session, gateway and device generations. Stable pump slots borrow
dense IDs/registrations and use an event-ready bitset or notification so an
empty consumer performs no FFI/JSON drain. Reconcile only when an input
generation changes.

### P0: arbitrary callback work runs on the retained host tick

Typed consume parses JSON and then invokes arbitrary plugin code while the
retained host tick is active. The 4 ms pump budget is checked only between
callbacks; a callback can exceed it without cancellation or preemption. Begin
and end session callbacks have the same synchronous property. Three consecutive
slow callbacks can quarantine a consumer after three frame stalls.

Declare callback affinity and execution class in the provider generation. Use a
shared task-graph/worker lane and bounded mailbox for callbacks that may leave
the main thread; keep only bounded delivery/result commit on the host tick.
Main-thread-affine callbacks need a typed deadline, cancellation/quarantine
outcome and an explicit admission budget because in-process code cannot be
preempted safely.

### P1: lifecycle reconciliation is not one atomic generation

`reconcile_enabled_capabilities_inner` retires removed consumers before all
new registrations are prepared. A retirement error leaves removals applied;
an addition failure can leave the old set only partially restored because
rollback covers newly added rows, not every retired subscription and callback
state. Remote subscription cleanup is therefore not equivalent to publishing
one accepted active set.

Build a candidate subscription/session generation, validate retirements and
prepare additions first, then publish one old-or-new active generation. Every
remote subscription and callback state must have a terminal cleanup receipt on
failure, cancellation or stale gateway identity.

### P1: delivery terminal diagnostics are incomplete

Host reservation drops, decode failures, stale leases, callback faults, slow
quarantine and pending-tail cancellation are reported through different
counters. There is no end-to-end delivery ledger tying a producer sequence to
an owner, session/subscription generation and one terminal state. A pump can
process some consumers and return only its first error; the UI controller then
reduces the result to status text.

Every drained sequence needs exactly one `Applied`, `Rejected`, `Backpressured`,
`Poison`, `Cancelled` or `Stale` receipt. Aggregate outcomes may preserve all
consumer errors while still returning a bounded first-error compatibility view.

### P1: session identity and selected diagnostics

The play-session generation uses unchecked `fetch_add`, so zero/repeated values
remain possible at exhaustion. Explicit fault/report queries clone retained
receipts and pump reports allocate sample vectors and sort p95 data on every
call. `finish_pump_report` scans all pending consumers each pump. These are
selected or fixed-size costs, but their rows and byte budgets are not part of a
diagnostic proposal, and diagnostics-disabled operation has no explicit zero
work gate.

Use checked, device/session-qualified generations. Diagnostics Disabled should
avoid snapshots, p95 vectors, sorting and empty drains; Counters/Sampled/Full
should use fixed dense IDs and pre-admitted row/byte capacities.

## Architecture hard cut

M0 adds RED tests for producer/host backpressure, page loss, byte accounting,
empty-consumer no-work, stale gateway, lifecycle failure, callback deadline,
terminal delivery outcomes and session-generation exhaustion.

M1 seals one `RuntimeConsumerEligibilityGeneration` and one provider/session/
subscription generation. It contains capability schemas, callback affinity,
maximum rows/bytes, queue age and deadline limits, dense consumer slots and the
accepted gateway/device epoch.

M2 introduces an end-to-end `DeliveryGeneration` lease. It reserves wire,
decoded/object, callback and result capacity before drain; producer commit,
host retention and callback publication share that lease. Capacity failure is
typed `Backpressured` with no destructive work, and every accepted sequence
terminalizes exactly once.

M3 runs accepted callbacks through the shared task scheduler or an explicit
affinity mailbox. The retained tick only performs bounded snapshot/commit work;
no third-party callback executes under the framework or active-map lock.

M4 makes lifecycle reconcile transactional: prepare the candidate subscription
set, quiesce and clean old rows, then compare-and-swap one active generation.
Failure retains the old generation exactly and publishes cleanup receipts for
every remote/session child.

M5 makes diagnostics explicit and generation-qualified. Disabled owns zero
labels, snapshots, p95 arrays and empty gateway drains. Counters/Sampled/Full
borrow compiled IDs and admit their report rows, bytes, callback time and
terminal-reason histograms before work.

M6 runs current-source Cargo, retained-host F0/F4 play sessions, task/lock/RSS
and queue traces, then WPR/Tracy/RenderDoc only where the product path reaches
graphics. Unreal's `TickTaskManager` and `MessageRouter` are retained as
evidence for declared task prerequisites, recipient affinity and queued
dispatch; they do not define Zircon's ABI, quotas or recoverable errors.

## Acceptance matrix and hard gates

- Consumers: 0/1/4/16/64/cap+1; capabilities stable/change/quarantined;
  registration add/remove/reload and stale gateway/session/device generations.
- Pages/events: 0/1/64/256/1k/16k deliveries; payloads 0/64 B/2 KiB/256 KiB;
  encoded/decoded/object bytes below/equal/above budget; producer commit,
  rollback and host `Backpressured` at every boundary.
- Callbacks: empty/valid/malformed payload, 0/1/4/16 ms duration, panic,
  timeout, retry, quarantine and main-thread/worker affinity.
- Lifecycle: begin/prepare/reconcile/retire/end success, partial failure,
  cancellation, device loss and shutdown; old generation must remain usable
  when the candidate is rejected.
- Diagnostics: Disabled/Counters/Sampled/Full; report rows, labels, p95 samples
  and pending/fault bytes at 0/1/64/1k/cap+1.

Hard gates are: no event loss on host backpressure; every accepted delivery and
remote subscription reaches exactly one terminal receipt; wire/decoded/object
and peak bytes are pre-admitted; stable eligibility and empty consumers do no
reconcile/clone/FFI work; callbacks obey affinity and deadline ownership;
failed lifecycle or callback preparation publishes no partial active generation;
session/device identities never repeat; diagnostics match actual work and
Disabled adds zero retained-event overhead.

This closure supersedes the stale unbounded-host-pending portion of PERF-MVP-069
with the current bounded implementation and retains the new cross-owner loss,
generation and callback findings. No microbenchmark or production micro-fix is
warranted before the delivery lease and lifecycle owner are unified.
