---
related_code:
  - zircon_editor/src/core/runtime_event_consumer
  - zircon_editor/src/tests/runtime_event_consumer.rs
  - zircon_editor/src/tests/runtime_event_consumer_bounded_pump.rs
  - zircon_runtime/src/scene/event_mirror
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_plugins/ai/editor/src/runtime_mirror.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/navigation/editor/src/runtime_mirror.rs
  - zircon_plugins/navigation/runtime/src/plugin.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageBus.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageDispatchTask.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageTracer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MessagingCommon/Public/MessageEndpoint.h
---

# Protected plan routing: runtime-event semantic routes

## Reason for routing

Performance01, `review.md`, `pending.md` and all owner plans are protected/foreign dirty in this
session. This record requests current-source corrections without overwriting their owners. Evidence is
`2026-08-15-editor-runtime-event-consumer-semantic-routing-current-architecture-review.md`.

## Requested Performance01 corrections

Replace the stale runtime-event summary with current scope: 8/8 production files, 1,526 lines and two
tests; four direct external files, 1,844 lines, 22 tests and two ignored managed benchmarks. Record
that all 12 current files pass rustfmt.

Correct existing task claims:

- `PERF-MVP-069`: editor pending is now at most one 64-row/128 KiB page per consumer; pending
  entries/bytes/age and runtime remaining/oldest age are observed; active-map take/commit is once per
  consumer batch, not twice per event. Retain the task for missing stream policy, ready-only polling,
  callback isolation and product acceptance.
- `PERF-MVP-432`: empty runtime pages now return an empty owned buffer with zero JSON encode/decode;
  payload stays raw JSON from producer queue through the ABI batch and is typed once at the editor
  boundary. Retain the task for repeated per-delivery descriptor bytes, per-consumer subscription
  serialization, duplicate same-schema typed decode and single shared payload ownership.
- `PERF-MVP-565`: stable Play still clones the complete capability snapshot plus enabled Vec, clones
  registrations, builds desired/existing maps/sets and computes deltas every tick. It also needs one
  immutable route generation so the pump does not deep-clone active manifests each frame.
- `PERF-MVP-597`: synchronous FFI/decode still belongs to the Runtime11 per-session lane described in
  the gateway review. This consumer plan adds endpoint preparation/application affinity; it must not
  create another pool.

Add the current P0 structural finding under `PERF-MVP-069/432`: consumer manifests have no
`Lossless/Latest/Bounded` policy, key or affinity. AI emits a full debug snapshot every Update and
registers two editor consumers for the same event/schema; Navigation emits an overlay frame every
Update. Each subscription installs its own observer, serializes independently and owns a 16K/64 MiB
FIFO. Latest snapshots therefore backlog and replay stale states; the two AI routes may retain 128 MiB
and duplicate producer serialization/ABI/decode.

## Required target architecture

1. Hard-cut manifest/ABI registration to explicit `Lossless`, keyed `Latest`, or configured
   `Bounded` policy. Apply policy at the producer queue; do not coalesce only after ABI crossing.
2. Publish one immutable `RuntimeEventRouteGeneration` grouped by event/schema/policy/key. One
   runtime observer/queue/serialized page fans a shared immutable payload to all local endpoints.
3. Enforce one typed decoder identity per schema route; decode once and share an Arc typed artifact.
4. Split endpoint preparation from editor commit. Runtime11's session/event lane performs decode and
   thread-safe heavy preparation; UI applies only a small current-generation projection/command.
5. Publish route-ready generation/wake state and poll only ready routes. Keep round-robin among ready
   work, lossless order and explicit overflow.
6. Replace per-frame duration Vec allocation/sort with Render17 bounded profiling and explicit
   recipient thread/dispatch-latency records.

## Requested owner-plan updates

### Editor02

Own route generation, ready-set fairness, shared page fanout and one-page editor retention. Preserve
session/sequence/schema checks, panic-tail restore and lifecycle atomicity. Stable generation work and
empty route polling must be zero.

### Editor04

Bind route/lane lifetime to Play instance and gateway generation. Stop/reload cancels stale
preparation, drains or rejects lossless work by explicit policy, and releases endpoints only after no
callback remains active.

### Editor12 and Plugins01/11

Extend plugin consumer registration with delivery policy, stable key, typed decoder identity and
prepare/apply affinity. Reject duplicate schema/type or incompatible policy declarations atomically.
Native/VM plugins may not inject arbitrary unbudgeted UI callbacks.

### Runtime10

Hard-cut the ABI/event mirror to one route subscription and shared page-level descriptor. Latest
queues store one generation per key; lossless/bounded queues retain current hard budgets and report
remaining/oldest/overflow. Expose ready generation to the session lane.

### Runtime11 and Render17

Run serialized drain/decode/heavy prepare on the existing per-session ordered lane. Record route id,
policy, queue wait/run/decode/prepare/apply, bytes, generation, affinity, cancellation and stale result
in the central profiler. No private executor and no unconditional per-frame p95 sort.

### AI06

Mark behavior debug snapshots keyed latest-state. Converge the debug mirror and node-result pruning
onto one shared decoded snapshot/preparation; build active indexes off the UI thread and publish both
derived states atomically. Node-result events remain lossless or explicitly bounded by their product
semantics.

### Navigation05

Mark overlay frames keyed latest-state. Build/publish one immutable overlay generation and replace
stale frames before serialization/ABI, not after UI application.

### EditorUI08

Apply current immutable route projections once per accepted generation. Retained tick must not clone
capability/registration state, perform FFI/decode or run heavy plugin map rebuilds.

## Acceptance additions

- Stable 1/100/10K capability/registration/routes at 60/120/240Hz: all snapshot/String clones,
  Map/Set/Vec builds, stable subscribe calls and diagnostic sorts zero.
- Latest snapshots 1/1K/10K at 1/60/240Hz with 0/1/60s consumer stall and 1/2/16 endpoints: retained
  values at most one/key, stale applies zero, serialization/typed decode once per accepted route
  payload, duplicate runtime subscription zero.
- Lossless 0/1/64/1K/10K rows and 64B/2KiB/128KiB: accepted loss/dup/reorder zero, queue hard-bound,
  overflow/retry/stop explicit.
- Preparation/apply 0/1/4/16ms/10s: UI decode/prepare wall zero, callback under host lock zero, stale
  completion zero and stop/reload bounded.
- F4 Play with AI and Navigation off/on at 1/1K agents/nodes: current Cargo, both managed benchmarks,
  WPR CPU/thread/wake/lock/queue p50/p95, RSS and package power required.

## Requested protected index state

- `pending.md`: one concise module row for `zircon_editor/src/core/runtime_event_consumer/**` plus its
  four direct external test files, `static_complete / dynamic_pending`, current counts and review
  link.
- `review.md`: do not add the scope until current Cargo, managed scale cases, F4 WPR and quantified
  CPU/RSS/power close the matrix.

## Milestone and notification state

This is a static architecture review, not an accepted performance milestone. No commit or WeCom
notification is due. Both become required only after the dynamic matrix is current and the protected
indexes are reconciled by their owner.
