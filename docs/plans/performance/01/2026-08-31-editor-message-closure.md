---
related_code:
  - zircon_editor/src/core/editor_message/**
  - zircon_editor/src/tests/editor_message/**
  - zircon_editor/src/core/runtime_event_consumer/**
  - zircon_editor/src/core/gateway/**
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
write_scope: []
status: pending
---

# Editor message closure

This is a current-source static closure for the editor message bus, inbox and
UI-delta boundary. It remains pending: the current workspace does not reach a
green `zircon_editor` Cargo test because compilation stops in `zircon_runtime`
with unrelated production errors. No Rust source was changed and this scope
does not enter `review.md` from static evidence alone.

## Scope and source state

- `zircon_editor/src/core/editor_message/**`: 38 Rust files, 4,244 physical
  lines, 3,733 nonempty lines, 139,714 bytes, 31 tests, six ignored tests and
  six include sites. Sorted path/raw-content SHA256:
  `0859642a5ac30d909e53fcb7d61a5bf809433eacf84a58e4185405ae6125a6b4`.
- `zircon_editor/src/tests/editor_message/**`: 14 Rust files, 1,841 physical
  lines, 1,646 nonempty lines, 66,110 bytes, 40 tests and two ignored tests.
  Sorted path/raw-content SHA256:
  `f370931ff8daa1bde875d7f0956eaa0bae21bbe2436e6a91e10a8f692ce0d74f`.
- Isolated `rustfmt --check --edition 2024 --config skip_children=true`
  passes 44/52. The eight failures are current foreign import/assert
  formatting only. Managed `cargo test -p zircon_editor --lib editor_message
  --no-run --message-format=short` reached `zircon_runtime` and failed before
  editor tests; no module pass is claimed.

## Positive work to preserve

- `SharedEditorMessageBus` prepares under the bus mutex, dispatches to inbox
  mutexes outside it, and invokes request handlers outside both locks.
- Delivery payloads share one `Arc`; latest/bounded inbox lanes use indexed
  count-only eviction, fixed delivery and retained-byte limits, and O(1) depth
  counters. Lossless fanout preflights every target before enqueueing.
- Scene-inspection property paths and selection payloads use shared storage in
  the current foreign work. Built-in and plugin schema IDs validate namespace
  and a 256-byte protocol limit; topic parsing performs one byte scan.
- The external tests cover typed routing, request re-entry, lossless
  backpressure, latest coalescing, dirty masks, hierarchy deltas and payload
  sharing. Ignored performance tests remain evidence fixtures only.

## Retained findings

1. `EditorUiDeltaQueue` has no entry, property, byte, age or deadline cap.
   `push_patch` grows a `HashMap<UiNodePath, EditorUiNodeDelta>` and barriers
   move pending rows into an unbounded `Vec`; both operations run under the
   global message-bus mutex. `EditorUiDeltaBatch::reflection_patches` then
   clones every patch into a second vector. A UI storm can therefore hold the
   bus lock while materializing unbounded owned data.
2. Inbox retention is bounded by configured depth and an estimated byte total,
   but `estimate_retained_bytes` counts slice descriptors and selected string
   lengths rather than all nested allocations. `drain` moves the full retained
   map while holding the inbox mutex, with no page, wall-time or cancellation
   contract. A legal 4,096-delivery/16 MiB drain can monopolize a consumer
   boundary and its result vector.
3. Publication builds target and report vectors proportional to subscriber
   count. Lossless fanout locks every target inbox and holds all guards while
   preflighting and enqueueing; best-effort fanout serially waits on each inbox,
   so one stalled subscriber creates head-of-line delay. There is no aggregate
   proposal for target count, report bytes or callback capacity before dispatch.
4. The message bus, runtime-event consumer and gateway each own separate
   pending/delivery lifetimes. A plugin/runtime bridge can drain one queue into
   another and invoke arbitrary callbacks without one request generation that
   reserves decoded bytes, output rows, callback capacity and terminal outcome.
   Cross-module overflows can consequently become silent loss or stale pending
   state even though each local queue is bounded.
5. Stable routed messages still mint delivery sequences and report vectors, and
   built-in topic/schema constructors own strings on selected paths. These are
   smaller than the delta/drain risks, but no diagnostics mode currently gates
   all label/report ownership. Full scene-selection coalescing also allocates
   temporary hash/sorted projections for each accepted composition.
6. Subscriber and delivery IDs use checked increments, which is positive, but
   the retained rows and dirty/view projections are not qualified by one
   editor-session/device/runtime generation. Reconnect or replacement can leave
   an old consumer holding a valid-looking ID while its successor owns the
   same logical route.

## Architecture handoff

1. Compile one `EditorMessageGeneration` containing session/consumer identity,
   topic and schema IDs, retention lanes, target/report limits, decoded/object
   bytes, callback budget and deadline. Reject cap+1 before queue mutation.
2. Make UI deltas a bounded latest-property journal plus ordered barrier
   segments. Admit count/bytes/age per segment, publish a cursor/receipt, and
   expose an explicit bounded export instead of cloning all patches by default.
3. Replace full `drain` with page/cursor delivery. Keep the inbox mutex hold
   bounded, terminalize eviction/replacement/shutdown as Delivered, Dropped,
   Backpressured or Cancelled, and measure actual owned payload bytes rather
   than slice metadata.
4. Unify message, runtime-event, gateway and plugin callback capacity under one
   end-to-end request/delivery lease. Every accepted identity reaches exactly
   one terminal state, including consumer loss, panic, replacement and device
   or session shutdown. Callback execution uses an affinity/deadline-aware
   scheduler outside producer locks.
5. Publish immutable session-qualified topic/schema/dirty/delta generations;
   stable consumers borrow dense IDs and labels. Diagnostics Disabled performs
   zero report vectors, label strings, scans and callback drains; Counters and
   Full modes own only pre-admitted rows.

## Evidence and gates

The local Unreal reference keeps `FMessageBus` on a dedicated router thread and
has `FMessageRouter::Run` process commands before waiting on a synchronization
event (`dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageBus.cpp`
20-27, `MessageRouter.cpp` 53-61). Slate's fast invalidation path explicitly
includes TaskGraph support. This supports separate routing/scheduling and
event-driven waiting, not a claim that Zircon should copy Unreal's ABI.

M0 adds RED tests for UI delta cap+1, page/deadline drains, subscriber fanout,
cross-queue eviction and generation replacement. M1-M3 implement the shared
proposal/lease, bounded journal and terminal delivery cursor. M4-M5 add
session-qualified diagnostics and managed scale tests before any acceptance.

Hard gates: current-source Cargo builds; cap+1 produces zero queue/callback
work; UI delta and inbox drains are bounded and generation-qualified; no
consumer loss leaves orphan payloads; callback/panic/shutdown paths terminalize
exactly once; stable/Disabled routes allocate no labels or reports; and
diagnostics match actual delivery, bytes, waits and drops. No benchmark artifact
or micro-fix is warranted before these ownership corrections.
