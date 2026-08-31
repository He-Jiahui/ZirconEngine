# Editor runtime-event consumer semantic-routing current-architecture review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-08-15.
- MVP priority: P0 for active Play retained-frame cost, debug-snapshot latency and plugin callback
  isolation.
- Owners: Editor02 owns the editor consumer route/pump; Editor04 owns Play session lifetime;
  Editor12 and Plugins01/11 own registration semantics; Runtime10 owns the event ABI and serialized
  session; Runtime11 owns off-thread preparation; AI06 and Navigation05 own product stream policies;
  EditorUI08 owns retained application.
- Accounting: keep production and external tests in `pending.md`. Current managed Cargo, the two
  ignored scale tests, F4 WPR and same-machine CPU/RSS/power evidence are absent.
- Code disposition: no Rust source changed. Five of eight current production files and three of four
  current external test files have pre-existing modifications or are untracked; all bytes were
  preserved.

## Exact scope

| scope | files | physical lines | tests | ignored | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/core/runtime_event_consumer/**` | 8/8 | 1,526 | 2 | 0 | `10231d12f9ebb68b2b9a5d493058c9357cd6912948cfc908713d9d077d931c46` |
| direct external tests | 4/4 | 1,844 | 22 | 2 | `2b79526d98ff90b4da9e9590dde76813654a3d938651a95f0e5bd1b74fd87e8f` |

Direct external tests are `tests/runtime_event_consumer.rs`,
`tests/runtime_event_consumer_bounded_pump.rs` and the latter's `real_runtime_abi.rs` and
`round_robin.rs` children. Each fingerprint streams ordinal-sorted normalized workspace-relative
path, NUL, raw bytes and NUL into SHA256. Every exact-scope Rust file was read in full. Supporting production
callers in the retained host, runtime dynamic session, event mirror, AI editor/runtime and Navigation
editor/runtime were traced but are not counted as accepted scope.

The 2026-07-30 report's six files/1,180 lines and 16 external tests is obsolete. The host execution
and round-robin owners are now separate files, two more fairness/real-ABI test files exist, and the
pending-page, raw-payload and empty-page architecture has changed materially.

## Current-source corrections

1. Editor pending is now hard-bounded to one decoded page per active consumer. The host drains only
   when `has_pending` is false, retains at most 64 deliveries/128 KiB encoded upper bound, and exposes
   pending bytes/oldest age plus runtime remaining/oldest age (`host.rs:396-431,643-673`;
   `pump.rs:56-224`). The old “unbounded editor pending” finding is closed.
2. The old per-event double active-map lock is closed. A whole consumer page is moved out once,
   processed locally and restored once with generation/subscription validation
   (`host.rs:433-482,675-727`). A panic restore guard retains the unprocessed tail and last successful
   sequence.
3. Empty runtime pages now return `ZrOwnedByteBuffer::empty()` before JSON construction
   (`zircon_runtime/src/dynamic_api/session/event_mirror.rs:97-108`), and SessionGateway returns an
   empty page without decode. Empty serialization/decode is zero; an empty FFI/session/queue-lock call
   per visited subscription remains.
4. Delivery payload is now `Box<RawValue>`. Runtime writes retained producer JSON bytes directly into
   the ABI batch and the editor performs typed `from_str` only at the consumer boundary
   (`zircon_runtime_interface/src/runtime_api/session/plugin_event_mirror.rs:59-88`;
   `registration.rs:44-95`). The old producer `Value -> wire Value -> editor Value` description is
   stale.
5. Round-robin resumes at the first unvisited consumer, gateway failure does not starve later
   consumers, lifecycle mutation is atomically excluded from a pump, and callbacks execute outside
   the host active-map lock. These contracts and their tests must remain.

## Architecture verdict

The current code has bounded storage and credible lifecycle correctness, but it still uses the wrong
semantic model for high-frequency state streams. `PluginEventConsumerManifest` contains only
consumer id, event id, payload schema and required capability
(`zircon_runtime/src/plugin/package_manifest/plugin_module_manifest.rs:8-35`). It cannot declare
whether a delivery is lossless, latest-state or bounded/coalescible, nor its key or execution
affinity. Every event therefore becomes a per-subscription lossless FIFO.

That default is incorrect for current first-party product streams:

- AI sends a full `AiBehaviorDebugSnapshot` every Update
  (`zircon_plugins/ai/runtime/src/plugin/registration.rs:480-503`). The editor registers two separate
  consumers for the same event/schema (`ai/editor/src/runtime_mirror.rs:349-379`): one replaces the
  debug mirror, while the other builds an active-node set and prunes a second result map
  (`runtime_mirror.rs:238-279,309-340`).
- Navigation sends a full `NavigationOverlayFrame` every Update
  (`zircon_plugins/navigation/runtime/src/plugin.rs:143-165`) and the editor replaces its retained
  frame (`navigation/editor/src/runtime_mirror.rs:85-106,146-172`).

Both snapshots are latest-state streams: once generation N+1 exists, applying N later is wasted work
and increases visible latency. Current runtime subscriptions instead each own a 16K-entry/64 MiB FIFO
with 64-entry/128 KiB pages
(`zircon_runtime/src/scene/event_mirror/subscription.rs:16-19,100-173`). Each subscription installs its
own observer and serializes the event independently (`subscription.rs:48-63,91-112`). The two AI
snapshot consumers can therefore serialize the same snapshot twice and retain up to 128 MiB across
their two runtime queues before editor pending, ABI batch and typed objects. At a 60 Hz maximum-size
128 KiB snapshot, those two routes alone can generate about 15 MiB/s of duplicate queued payload;
the byte cap is reached after about 512 frames per route. This is bounded failure, not an optimal
algorithm.

## Structural bottlenecks

### P0: event semantics are absent, so latest snapshots backlog as lossless work

Hard-cut the registration and ABI contract to require one delivery policy:

- `Lossless`: ordered commands/results; bounded backpressure is an explicit error and no accepted
  delivery may be coalesced.
- `Latest { key }`: one pending value per stable key; producer publication replaces the older value,
  increments a generation and preserves newest age. AI debug snapshot and Navigation overlay use
  world/session keys.
- `Bounded { entries, bytes, overflow }`: diagnostics/event streams with an explicit drop/coalesce
  policy and counters.

The policy must exist at the producer queue, not only in the editor. A latest snapshot that has
already been serialized into a FIFO and crossed ABI is too late to optimize. Tests must prove stale
snapshot application is zero, not merely that memory is capped.

### P0: transport subscriptions are per consumer instead of per event route

Build one immutable `RuntimeEventRouteGeneration` grouped by stable `(event_id, payload_schema,
policy, key)` and attach multiple editor endpoints to that route. Runtime owns one observer/queue and
serializes each accepted payload once. The session lane drains one page into a shared immutable
message/page owner; route fanout shares that owner. Same-schema endpoints must agree on a registered
typed decoder identity, decode once, and receive an `Arc` typed artifact or borrowed immutable view.

AI's two debug-snapshot consumers should either consume the same decoded artifact or converge into
one preparation that publishes both derived projections atomically. They must not create two runtime
subscriptions, two producer JSON buffers, two ABI pages and two typed decodes for one snapshot.

### P0: arbitrary typed consumer work executes on the retained UI owner

`EditorRuntimeEventConsumerRegistration::typed` decodes the full payload, locks plugin-owned state
and invokes arbitrary `consume` work inline (`registration.rs:44-95`). The 4 ms pump budget checks time
between callbacks and cannot preempt one slow callback. Current AI snapshot pruning allocates a
`BTreeSet`, scans retained results and clones node ids during membership checks; Navigation replaces
a potentially large frame. These are real product callbacks, not synthetic risks.

Split each endpoint into explicit preparation and application:

1. the Runtime11-owned session/event lane performs wire decode and declared thread-safe preparation;
2. preparation produces an immutable, generation-tagged projection or small editor command;
3. the editor owner applies only current projections under the existing count/time budget;
4. heavy maps are rebuilt off the UI thread and published by Arc/generation swap;
5. lifecycle stop/reload cancels stale preparation and never calls plugin code while a host lock is
   held.

The route declares affinity. Main-thread work is allowed only for the small final commit. This
matches Unreal's requirement that AnyThread receivers be thread-safe and sufficiently fast; slow
work must not block the router.

### P0: stable Play ticks rebuild capability and route state

Before every active Play pump, the controller clones the complete capability snapshot and then clones
the enabled list (`ui/host/editor_host_event_controller.rs:218-226`). Reconcile then clones all
registrations, performs a linear capability search for each, builds a desired `BTreeMap`, scans active
consumers, builds an existing `BTreeSet` and computes deltas (`host.rs:226-357`). With `C`
capabilities and `R` registrations, stable work is approximately `O(R*C + R log R)` plus String/Arc
clones, locks and allocations even though no generation changed.

Editor12 must publish an immutable capability generation; this module must own a registration/route
generation. Reconcile only at Play begin or when either generation changes, and update affected route
keys. Stable ticks perform zero capability/registration clones, Map/Set builds and subscribe calls.

### P1: every pump clones all active metadata and computes unused p95 diagnostics

`snapshot_active_consumers` clones every consumer id and registration; registration clone deep-copies
the manifest Strings, then performs a linear cursor search and Vec rotation (`host.rs:611-641`). Each
pump allocates two duration vectors, sorts both for p95 and scans all active consumers again for queue
diagnostics (`host.rs:386-394,475-480,729-775`). No production caller reads `last_pump_report`; all
current readers are tests.

Store active route metadata in a shared immutable generation and use a numeric/generation-safe
round-robin cursor. Move latency samples to Render17's bounded profiler/histogram and enable detailed
collection only when profiling is active. Production queue counters remain O(changed routes) or
incremental; stable empty frames must not allocate/sort diagnostic vectors.

### P1: empty consumers are polled once per active Play frame

For every visited consumer with no local pending page, the host calls `drain_plugin_events` before
the next budget check (`host.rs:396-431`). Empty JSON work is fixed, but the call still enters the
runtime session action guard, locks the whole mutable session and locks the route queue. With `A`
empty active consumers at `F` frames/s, the idle lower bound remains `A*F` foreign/session/queue calls.

Publish a route-ready generation/wake bit as part of the session tick/lane contract. Drain only ready
routes and preserve round-robin fairness among ready work. Polling is the compatibility behavior to
delete, not a second authority.

## Per-file production review

| file | current-source performance result |
|---|---|
| `error.rs` | Typed lifecycle/protocol/apply errors; allocations occur on failures. |
| `host.rs` | One-page pending, batch commit and lifecycle guard are fixed. Stable reconcile, metadata snapshot, synchronous drain/callback and full diagnostic scan remain. |
| `host/execution_support.rs` | Atomic pump/lifecycle ownership and validation are sound. p95 sorts caller-owned vectors; descriptor mismatch errors clone Strings only on failure. |
| `host/round_robin.rs` | Correct first-unvisited rotation; cursor is still an owned String found linearly in a rebuilt snapshot. |
| `manifest.rs` | Re-export only; underlying manifest lacks delivery policy/key/affinity. |
| `mod.rs` | Module/export wiring only. |
| `pump.rs` | Hard count/time/callback limits and queue/backlog metrics are useful. A budget cannot preempt one foreign call or callback. |
| `registration.rs` | Registry batch extension is atomic but clones the full registry on registration. Typed apply decodes and runs arbitrary state work inline; no generation, route grouping or affinity. |

## Per-file external test review

| file | coverage result |
|---|---|
| `tests/runtime_event_consumer.rs` | Capability bind/unbind, session/schema/sequence rejection, raw typed decode and retryable lifecycle cleanup covered; several lifecycle checks are source-shape assertions. |
| `tests/runtime_event_consumer_bounded_pump.rs` | One-page deferral, byte/age/backlog metrics, panic recovery, slow callback, reentry, concurrency and gateway failure covered. One synthetic 1K/10K benchmark is ignored. |
| `tests/runtime_event_consumer_bounded_pump/real_runtime_abi.rs` | Real linked runtime 1K/10K page/order metrics exist but are ignored and run with a one-second budget, not the F4 4 ms product budget. |
| `tests/runtime_event_consumer_bounded_pump/round_robin.rs` | First-unvisited behavior and fairness across 64 consumers covered. No ready-only/wake-driven route test. |

## Unreal primary-source comparison

- Unreal `Messaging/Public/IMessageBus.h:35-95` defines transport-neutral structured messages and a
  router that fans publish/subscribe messages to registered recipients. The same message context is
  passed as a `TSharedRef` to every recipient in
  `Messaging/Private/Bus/MessageRouter.cpp:118-180`; payload ownership is shared rather than
  re-serialized per local endpoint.
- `MessagingCommon/Public/MessageEndpoint.h:47-63` supports async arrival or a synchronously polled
  inbox and keeps message ownership in the bus. Lines 54-60 and 180-190 require explicit recipient
  thread selection and warn that time-consuming AnyThread handlers block the router.
- `MessageRouter.cpp:167-181` directly invokes only AnyThread recipients and otherwise schedules a
  `FMessageDispatchTask`; `MessageTracer.cpp:118-137` records dispatch latency, dispatch type and
  recipient thread. Zircon needs the equivalent route affinity and central profiling, not ad hoc
  per-frame p95 vector sorting.
- Unreal messaging also supports expiration/interception/resequencing (`IMessageBus.h:84-95`), which
  demonstrates that delivery semantics belong in the route/message contract. Zircon's exact
  `Lossless/Latest/Bounded` policies are an engine-specific hard cut driven by its current snapshot
  producers, not a claim that Unreal uses those enum names.

## Acceptance and measurement plan

| case | matrix | required result |
|---|---|---|
| stable route | capabilities/registrations/routes 1/100/10K; 60/120/240Hz; stable/1% changes | stable snapshot/String clone, locks, Map/Set/Vec builds and subscribe calls=0; changed reconcile<=1/generation and near affected routes |
| latest streams | snapshots 1/1K/10K; 1/60/240Hz producer; consumer stall 0/1/60s; 1/2/16 endpoints | producer retained values<=1/key, stale snapshot apply=0, serialize/decode once per accepted route payload, duplicate runtime subscriptions=0 |
| lossless streams | 0/1/64/1K/10K rows; 64B/2KiB/128KiB | accepted loss/dup/reorder=0, queue entries/bytes/age hard bounded, overflow typed and retry/stop semantics explicit |
| callback isolation | prepare/apply 0/1/4/16ms/10s; 1/64 routes | UI decode/prepare wall=0, one main-thread commit cannot include heavy map rebuild, stale completions=0, callback-in-host-lock=0 |
| idle ready routing | active routes 1/4/64; 30/60/120Hz; no events | empty ABI/session/queue calls approach ready-generation changes, stable allocations/sorts=0, wake latency recorded |
| product | F4 Play with AI+Navigation off/on, 1/1K agents/nodes, start/idle/storm/stop/reload | WPR CPU/thread/wake/lock/queue p50/p95, RSS/package power, current-source Cargo, two managed benchmarks and lifecycle behavior GREEN |

Measure Zircon and an available local Unreal editor build on the same machine, scene scale, frame cap,
foreground state and power plan. Compare CPU, wakeups, p50/p95, RSS and package power; do not infer
parity from source architecture. RenderDoc is not applicable to this CPU/message-routing slice.

## Static gates executed

- Read 8/8 production files and 4/4 direct external test files in full; traced current retained host,
  runtime producer/ABI and AI/Navigation consumers.
- `rustfmt --edition 2021 --check` passed all 8 production and all 4 external files.
- Scoped `git diff --check` passed; Git emitted only existing LF-to-CRLF checkout warnings.
- Managed Cargo and both ignored benchmarks were not run because the recorded Windows build-helper
  separator defect still rejects valid non-C target roots. No output artifact was written to C:.
- Protected `review.md`, `pending.md`, Performance01 and owner plans were not modified. This static
  review is not an accepted milestone, so no commit or WeCom notification is due.
