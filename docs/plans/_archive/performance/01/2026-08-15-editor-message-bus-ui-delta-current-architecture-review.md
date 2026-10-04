# Editor message bus and UI delta current-architecture review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-08-15.
- MVP priority: P0 for retained-host frame work and plugin lifecycle dispatch; P1 for generic bus
  routing and inbox consumption.
- Owners: Editor02 owns routing, inboxes and bounded pull contracts; EditorUI08 owns retained UI
  invalidation/delta application; Editor12 and Plugins01 own lifecycle callback dispatch; Runtime11
  may provide a bounded ticket only for callbacks with explicit non-main affinity.
- Accounting: keep `zircon_editor/src/core/editor_message/**` in `pending.md`. Do not add it to
  `review.md` until current-source managed Cargo, contention counters and an F4 retained-host trace
  are green.
- Code disposition: no Rust source was changed. The reviewed bus, tests, retained-host adapter and
  lifecycle bridge contain pre-existing modified/untracked work and were preserved.

## Exact scope

| scope | files | physical lines | tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/editor_message/**` | 35/35 | 2,935 | 10 inline | `f67e0c600b7e8352d91e21034906d20c07a3f546898ad6bc5b7bddc6b248652e` |
| `zircon_editor/src/tests/editor_message/**` | 13/13 | 1,646 | 32, including 1 ignored benchmark | `5a1088bcbe8a3630293aafff2204da920a5237a30a06bdb61b1411b601225e6c` |

The fingerprint is SHA256 over normalized sorted path, NUL, raw file bytes, NUL. Every production
and external-test Rust file in the two folders was read in full. Product chains were followed
through `ui/host/editor_event_runtime_reflection.rs`, `ui/host/scene_inspection_publication.rs` and
`core/plugin/lifecycle_message_bridge.rs`; these supporting files are evidence, not newly accepted
folder accounting.

## Architecture verdict

The July report is materially stale. The bus no longer keeps its global mutex while it enqueues
every subscriber: `SharedEditorMessageBus::publish` prepares under the bus lock, dispatches through
per-subscriber inbox mutexes, and reacquires the bus only to commit dirty state
(`shared.rs:38-47,162-167`). A delivery now owns one `Arc` payload and one cached retained-byte
estimate (`message/delivery.rs:16-54`), and lossless fanout locks targets in subscriber order,
preflights every inbox, then enqueues atomically (`bus.rs:386-410`). The old duplicate-delivery and
deep-fanout diagnosis must not drive another rewrite.

The P0 design problem has moved upward. UI invalidation now shares the message-bus owner with an
unbounded `EditorUiDeltaQueue`; barriers materialize pending patches into retained segments while
the global bus mutex is held. The host then drains the entire batch, clones all reflection patches,
and may perform a full reflection rebuild followed by a second patch pass. Separately, the plugin
lifecycle bridge still drains a complete inbox into a second unbounded queue and invokes arbitrary
plugin callbacks without an entry, byte or time slice. Those are frame-ownership problems, not
micro-optimization opportunities.

## Current-source corrections

1. `lossless`, `bounded` and `latest` lanes remain independently bounded at 4,096/256/256 entries,
   with a 2 MiB delivery ceiling and 16 MiB total retained-byte ceiling per inbox. Lane depth and
   retained-byte counters are O(1), and latest entries have key/order indexes.
2. Fanout clones share one immutable payload. The current benchmark explicitly checks a 1 MiB JSON
   publish for 1/5/100 subscribers and the behavior suite checks 100 paused subscribers over 10,000
   latest-state publishes.
3. Request handlers execute outside both the bus lock and inbox locks, reuse the prepared shared
   payload, and revalidate their target before response completion.
4. Lossless fanout remains all-or-nothing. Sorted target locking prevents cross-fanout lock-order
   inversion; this property must survive any routing split.
5. The old 29-file/25-test inventory is superseded by the 35-file/32-test current tree. The new
   files include structured scene-inspection deltas and `EditorUiDeltaQueue`.

## P0 structural bottlenecks

### UI delta retention is unbounded and owned by the global bus lock

- `EditorUiDeltaQueue` stores `entries: Vec<EditorUiDeltaEntry>` plus a pending `BTreeMap` without
  entry, byte, generation, age or deadline limits (`editor_ui_delta.rs:91-95`).
- Every Press, Release, Scroll, Focus, Geometry or Commit barrier calls `flush_pending`, moves and
  orders the complete pending map, allocates a delta vector, and appends another entry
  (`editor_ui_delta.rs:8-16,115-136`). Because `SharedEditorMessageBus::push_editor_ui_barrier` and
  `drain_view_updates` call these methods under the common bus mutex (`shared.rs:140-153`), unrelated
  topic preparation, subscription changes and dirty marking can wait behind UI batch materialization.
- A barrier deliberately splits coalescing segments. A high-frequency discrete-event burst can
  therefore retain one node segment plus one barrier per event until the host drains it; latest-wins
  coalescing cannot bound the accumulated `entries` vector.
- `EditorUiDeltaBatch::reflection_patches` deep-clones every retained patch into a second vector
  (`editor_ui_delta.rs:78-87`). The host applies that vector under the shell/control-service lock; on
  error it can rebuild the complete reflection and apply the same vector again
  (`editor_event_runtime_reflection.rs:114-145`). One malformed/stale sparse patch can turn a small
  input burst into full-tree work plus a duplicate patch pass.
- `view.invalidated` has no production subscriber in the current tree. `publish_view_invalidation`
  nevertheless constructs Custom JSON, parses a topic, prepares a delivery and dispatch report only
  to mark dirty (`editor_event_runtime_reflection.rs:98-111`). Direct dirty marking is the desired
  fast path, but the debug topic must first be declared non-observable or moved to an opt-in trace
  sink; deleting it silently would be a protocol change.

### Plugin lifecycle pump has no frame budget

- `EditorPluginLifecycleMessageBridge::pump` drains the subscriber's complete inbox, appends all
  deliveries to a second `VecDeque`, holds that queue's mutex, and loops through every lifecycle
  callback (`lifecycle_message_bridge.rs:36-70`). The second owner has no entry/byte/age ceiling.
- Callback failure requeues the current lossless edge at the front, preserving order, but a slow or
  faulting plugin can occupy the retained-host tick and delay every following plugin/event. Moving
  the loop to a private thread would violate declared callback affinity and deterministic editor
  commits; the fix is bounded pull plus lock-out callback and generation-checked completion.

## P1 routing and consumption bottlenecks

- `EditorMessageInbox::drain` clears all indexes, moves the complete ordered map and collects every
  delivery into one `Vec` while holding the inbox mutex (`inbox.rs:164-175`). There is no maximum
  entries, bytes or deadline per pull and no `remaining`/oldest wall-age result. The 4,096 lossless
  entry ceiling bounds count but still permits a large main-thread spike near the 16 MiB byte cap.
- Zero-target publish still allocates a sequence, creates the `Arc` delivery, estimates retained
  bytes and constructs a detailed report (`bus.rs:286-316`). Custom JSON estimation traverses the
  full tree and allocates a traversal vector (`message/delivery.rs:216-250`) even though no inbox can
  retain the message.
- Retained-byte estimation is approximate rather than a proven hard resident-memory bound. Scene
  property paths count owned strings but not all vector element/capacity overhead; JSON counts values
  and strings but not every map/array allocation detail. The 2 MiB/16 MiB limits therefore constrain
  an estimate, not exact heap ownership.
- Detailed dispatch always materializes delivered/coalesced/dropped/backpressured subscriber vectors.
  Correctness callers use them, but many fire-and-forget callers discard them. A summary-only mode is
  needed only after counters prove report allocation is material.
- Best-effort fanout acquires subscriber inboxes sequentially (`bus.rs:413-419`). A contended earlier
  subscriber can delay later independent subscribers even though the global route mutex is released.
  Measure this head-of-line effect before adding more scheduling machinery.
- Inbox age is delivery-sequence distance, not wall time. There are no route/global/per-inbox lock
  wait/hold, high-water entries/bytes, payload-size-walk or drain-wall counters, so the current test
  suite cannot locate main-thread stalls.
- Latest scene-inspection replacement composes selection deltas but not hierarchy generation deltas.
  The consumer can detect a generation gap and reflow, preserving correctness, but a stalled consumer
  can repeatedly convert sparse changes into full hierarchy work. Gap count and reflow cost need a
  product-scale gate.
- `take_retained_scene_inspection_message` drains all deliveries, clones matching messages and keeps
  the last (`scene_inspection_publication.rs:243-253`). The latest lane normally limits this topic to
  one row, so it is not independently unbounded; its wide clone should be removed only if counters
  show material cost.

## Unreal primary-source evidence

- `dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageBus.cpp:20-25` creates an
  independent router and router thread. Publish/Send create an immutable shared message context and
  enqueue it with sender-thread identity (`MessageBus.cpp:98-122,133-159`). This supports separating
  route ownership from caller/UI lock duration.
- `dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp:118-180` resolves
  recipients on the router, calls `AnyThread` receivers directly there, and submits named-affinity
  receivers through TaskGraph. Zircon should copy explicit affinity and lock-out callback, not the
  assumption that every local editor bus needs a dedicated OS thread.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp:299-329`
  merges reasons into a widget proxy and pushes the proxy through unique update structures.
  `:479-498` deduplicates while preserving paint order, and `:1281-1378` has named profiling scopes
  for pre-update, attributes, prepass and post-update. This supports a retained-owner invalidation
  index with stage counters rather than an unbounded bus-owned event log.
- Intentional divergence: Unreal `FMessageRouter::ProcessCommands` and delayed-message handling drain
  until empty (`MessageRouter.cpp:256-275`). Zircon's editor MVP requires explicit count+bytes+deadline
  frame slices, so this behavior is reference evidence for affinity, not an acceptance target.

## Optimization plan

### Milestone 1: observe current ownership before structural edits

- Add route preparation, retained-byte walk, target count and zero-target counters.
- Add global-bus and per-inbox lock wait/hold histograms, inbox high-water entries/estimated bytes,
  wall-clock oldest age, drain entries/bytes/wall and best-effort blocked-target counters.
- Add UI delta pending nodes, segments, barriers, estimated bytes, oldest event sequence/age,
  flush/materialization/clone/apply/full-fallback/retry wall; add plugin bridge bus/secondary backlog,
  callback-in-lock and callbacks-per-tick counters.
- Keep diagnostics bounded and disabled or sampled outside profiling builds.

### Milestone 2: hard-cut UI invalidation out of generic bus retention

- Make the retained host/window owner authoritative for UI patch retention. Store latest patch by
  stable node identity plus a bounded ordered discrete-edge lane; keep bus dirty state as a compact
  invalidation summary, not the patch journal owner.
- Publish `{generation, cursor, node-count, bytes, remaining}` pages. Coalesce continuous properties
  across a frame generation; preserve Press/Release/Commit order with a hard admission budget.
- Apply borrowed/shared patches once. On stale generation or apply failure, discard the incompatible
  page, rebuild once, advance generation, and do not replay the same stale patches.
- Replace zero-subscriber `view.invalidated` transport with direct dirty marking after an explicit
  observability test proves no consumer contract depends on that debug topic.

### Milestone 3: bounded bus and plugin consumption

- Add inbox `drain_page(max_entries, max_estimated_bytes, deadline)` returning `remaining`, oldest wall
  age and pressure counters. Keep accepted lossless order and never use truncation as silent drop.
- Return before sequence/delivery construction for zero targets. Add summary-only dispatch only if
  allocation evidence justifies its API cost. Calibrate byte accounting against allocator/RSS samples.
- Give the lifecycle bridge one bounded cursor rather than a second unbounded owner. Snapshot ordered
  active handles/generation under short locks, invoke callbacks outside bus/bridge/manager locks, then
  commit once if the generation still matches. Quarantine slow/faulted plugins with typed diagnostics.
- Schedule only explicitly non-main callbacks on a Runtime11 bounded single-flight ticket. Main-affinity
  callbacks remain on the editor owner but consume the same entry/byte/deadline budget.

### Milestone 4: prove sparse scene/UI behavior at product scale

- Make scene-inspection latest replacement either compose contiguous hierarchy deltas or emit an
  explicit gap token that triggers exactly one generation-scoped reflow.
- Exercise stable and bursty 10k/100k-node scenes through the retained host. No-change ticks must do no
  UI batch materialization, full reflection or hierarchy reflow.

## Quantified acceptance

| matrix | required measurements | acceptance |
|---|---|---|
| Bus: subscribers 0/1/100/10k; publishers 1/16; payload 64 B/2 MiB; lanes 0/1/256/4,096; stall 0/60 s | route/global/per-inbox lock wait+hold, target visits, delivery builds, size walks, report allocations, entries/bytes/oldest age, drain wall, RSS | zero target builds/sizes 0 deliveries; accepted payload owner=1 and size walk<=1; unrelated owners are not serialized by a stalled best-effort inbox; lossless atomicity/order unchanged |
| UI: patches 0/1/1k/100k; nodes 1/10k/100k; barriers 0/1/1k; stable/burst/error/stale generation | pending nodes/segments/barriers/bytes/age, flush/clone/apply/fallback/retry wall, full reflection and hierarchy materializations, UI frame p50/p95 | retained work has entry+byte+deadline bounds; stable frame materialization/rebuild=0; each accepted generation applies once; stale/error causes at most one rebuild and no stale replay |
| Plugins: messages 0/1/64/4,096; plugins 0/1/100/1k; callback 0/1/16 ms/10 s; error/reload/unload | bus/bridge entries+bytes+age, callback-in-lock, callbacks/tick, generation conflicts, UI p50/p95, RSS | callback-in-lock=0; per-tick entries/bytes/wall hard bounded; no accepted edge loss/dup/reorder; stale completion cannot mutate reloaded/unloaded generation |
| Scene: nodes 10k/100k; sparse changes 1/100/10k; consumer stalls 0/1/60 s | gap count, sparse rows materialized, reflows/generation, cloned bytes and frame wall | contiguous sparse cost scales with changed rows; any gap causes exactly one bounded reflow for the newest generation; stable tick performs zero scene publication work |

The existing ignored benchmark uses only 1/5/100 subscribers, a 4,096 lossless backlog, 10,000
latest publishes, a 1 MiB payload, a 50 ms publish p95 ceiling and a 64 MiB RSS-growth ceiling. It is
useful regression coverage for payload sharing, but too permissive and lacks multi-producer lock/UI
frame evidence; it is not acceptance data until run in the managed current-source environment.

## Static gates executed

- Read all 35 production files and all 13 external test files at the recorded fingerprints, plus the
  three product consumers and the Unreal primary-source files above.
- `rustfmt --edition 2024 --check` is formatting-only red on production `inbox.rs` and
  `message/delivery.rs`; the external suite is formatting-only red on
  `bus/backpressure/{behavior,performance}.rs` and `bus/fixture.rs`. No foreign source was formatted.
- Managed Cargo and the ignored performance benchmark did not run because the approved-root editor
  build helper still rejects valid Windows D:/E:/F: roots through its separator bug. The existing
  failure record is `failure-2026-08-15-build-editor-approved-root-separator.md`.
- WPR/xperf are available, but no launchable current-source editor binary exists for an F4 trace.
  RenderDoc is installed but is not applicable to this CPU/message-routing slice; no render claim is
  made.
- No performance improvement, power reduction or benchmark pass is claimed. The module remains
  pending until dynamic evidence closes the acceptance table.
