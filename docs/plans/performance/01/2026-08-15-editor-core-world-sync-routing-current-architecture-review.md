# Editor core WorldSync routing current-architecture review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-08-15.
- MVP priority: P0 for retained-editor idle cost, gateway replacement correctness and hierarchy
  change scaling; P1 for currently unused generic world-fact publication and watch-map diagnostics.
- Owners: Editor02 owns WorldSync routes and gateway-generation application; Runtime07/08 own the
  World mutation/change-set authority; Runtime10 owns the dynamic ABI; Runtime11 owns serialized
  session execution; Editor05/Layout09 own hierarchy operations; Render17 owns measurements.
- Accounting: keep this module in `pending.md`. Do not add it to `review.md` before current managed
  Cargo, route-scale counters, F4 WPR and CPU/RSS/power evidence pass.
- Code disposition: no Rust source changed. All five files are foreign modified or untracked current
  work and were preserved. A structural hard cut is required before small allocation edits.

## Exact scope

| scope | files | physical lines | tests | ignored | ordinal path-and-raw-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/core/sync/**` | 5/5 | 1,342 | 20 | 0 | `da7589d8e6371d120447e460d377a5d3a719221d7ceba56c15af815b87afcea7` |

The fingerprint streams every ordinal-sorted normalized workspace-relative path, NUL, raw file
bytes and NUL into SHA256. All five files were read in full. Reachability was traced through the
retained tick, hierarchy watch lifecycle, editor message bus, scene-inspection publication, gateway
and dynamic ABI, runtime level/subscription table, direct external tests and Unreal Scene Outliner.

The 2026-07-30 report's 3 files/450 lines/8 tests is obsolete. `pump.rs` and its tests now make the
module live in the retained editor, and the watch-map implementation/tests have expanded.

## Current positive baseline

- Runtime subscriptions use typed indexes, one reused ancestry scratch and a sorted pending dirty
  set. Facts coalesce by entity/scene/reload and have 4,096-entry, 512 KiB estimated and eight-world-
  generation retention limits (`scene/inspection/subscription.rs:14-57,147-191,206-313,415-464`).
- `InvalidationBatch::has_canonical_dirty_tokens` selects a no-dedup-tree editor path for strictly
  increasing runtime tokens. Malformed duplicate/unordered input remains observable.
- Projection visits dirty tokens rather than all registered watches. Dirty view identities are
  borrowed and one projected set crosses one message-bus lock per batch.
- Repeated exact view/key/mask registration reuses its token. Watch tokens are retained with the
  issuing gateway generation and stale unwatch does not target a replacement session.
- Runtime batches and owned FFI outputs have explicit error handling, and the gateway rejects world
  responses above 1 MiB.

These are useful support-layer repairs. They do not solve the route ownership, idle cadence or UI
execution architecture below.

## Structural bottlenecks

### P0: stable retained frames still synchronously poll the whole boundary

Every retained tick calls `ensure_hierarchy_world_watch`, `pump_edit_world_invalidations` and then
the other runtime consumer pump (`retained_host/app/host_lifecycle/tick.rs:18-24`). The WorldSync
pump takes its editor mutex, reads the gateway generation and calls `drain_world_invalidations`
unconditionally (`ui/host/editor_world_sync.rs:29-37`, `core/sync/pump.rs:190-207`).

For a dynamic gateway, one empty poll enters the FFI action guard, locks the complete mutable
session, locks the Level World and subscription table, constructs `Vec::new()`, serializes it as the
allocated JSON payload `[]`, transfers/releases the owned buffer and decodes another empty Vec
(`dynamic_api/session/registry/session_store.rs:62-86`, `session/ffi.rs:460-471`,
`dynamic_api/frame.rs:79-83`, `gateway/session/world_sync.rs:89-118`). In-process transport avoids
JSON but still takes the World and subscription locks. The editor pump then parses and allocates the
static `EditorTopic` even when there are zero batches (`pump.rs:217,271-273`), and the retained
wrapper drains the message bus on every frame regardless of the report
(`scene_hierarchy_refresh.rs:37-47`).

At frame frequency `F`, idle cost is therefore at least `F` pump mutex acquisitions, gateway loads,
session/World/subscription lock chains for dynamic transport, two-byte encode/decode transactions,
topic allocations and bus drains. No ready generation or wake contract suppresses this work.

### P0: one remote token per view scales transport with UI fanout

`WorldWatchMap` owns `by_token` and `by_view`, but no index groups equal `WatchKey` routes. Exact
reuse only scans tokens already owned by the same view (`watch_map.rs:124-166`). Two views watching
the same `WorldStructure` or component key create two runtime subscriptions. A matching mutation
inserts every token into the runtime `pending_dirty` BTreeSet, transports every token over JSON and
maps them back to views in the editor.

For `B` local bindings sharing one key, current mutation and wire work is approximately
`O(B log D) + O(B encoded tokens)`, followed by `O(B log registered_tokens)` projection. It should
be one runtime route/change identity plus local fanout: wire work near `O(1)` for that key and editor
work near the affected local views. Per-view token idempotence is not route convergence.

### P0: typed mutation semantics are built, discarded and reconstructed

Runtime already creates `WorldFact::Spawned/Despawned/Reparented/...` and the inspection subsystem
maintains generation-owned hierarchy anchors. The editor pump nevertheless converts every decoded
fact into a fresh `serde_json::Value`, wraps it in a custom schema message and publishes it under
`editor.world_fact` with one bus transaction per fact (`pump.rs:224-232`). There is no production
subscriber for that topic; its only subscribers are pump tests.

The dirty token then causes `drain_pending_view_refreshes` to lock the editor shell/World and call
`observe_scene_inspection_publication`, creating a second change projection before the retained
hierarchy consumes a `SceneInspectionMessage` (`editor_event_runtime_reflection.rs:114-146`,
`scene_inspection_publication.rs:178-260`). The architecture therefore pays for a typed runtime fact,
dynamic JSON batch, decoded fact, second JSON `Value`, unused message route and a separate inspection
observation, while the hierarchy applies only the latter.

World mutation semantics and inspection generation must be one producer-owned change authority.
`WorldFact` JSON publication and dirty-token-triggered re-observation are parallel delivery paths,
not independent product requirements.

### P0: gateway replacement can consume the first new-session batch as stale

`WorldSyncPump::pump` reads `gateway.generation()` and then independently snapshots the gateway for
the drain (`pump.rs:190-207`, `gateway/handle.rs:139-143`). Replacement can occur between those two
operations. The pump can therefore drain a new session while retaining the old gateway generation,
watch map and `last_generation`. A lower new World generation is rejected only after the destructive
drain; a coincidentally non-regressing batch can be published against old bindings. The next frame
notices replacement and clears state, too late to recover that first batch.

Watch/unwatch currently avoids the inverse race by holding the gateway replacement mutex across
foreign transport and editor-map mutation (`pump.rs:135-187`). That preserves identity but makes
replacement/stop wall time depend on an arbitrary provider call. The correct contract is a strong
generation lease with foreign work outside the replacement mutex, a short current-generation
commit and stale-result compensation/cancellation.

### P0: destructive whole-batch drain has no editor work budget

The runtime normally returns at most one batch, but `WorldSyncPump` consumes every returned batch,
converts every fact and projects every dirty token without count/time paging. Runtime facts have a
producer limit; dirty tokens do not have a count/byte page limit, and the gateway's 1 MiB cap is
checked only after runtime serialization and destructive flush. An oversized output is released and
rejected after its data has left the accumulator. Pump and projection diagnostics are returned, but
the retained caller discards the report and no central profiler records queue wait, lock time,
serialization, projection or hierarchy apply.

### P1: remaining local structures optimize the wrong ownership boundary

- `token_for` linearly scans a view's token set and compares an allocated one-element
  `depends_on: Vec<WatchKey>` even though `WatchRegistration` contains one key.
- Gateway replacement calls `drain_tokens`, allocating a Vec that is immediately discarded.
- Canonical projection still linearly validates ordering; malformed input builds three BTree-based
  diagnostic structures. Linear validation is acceptable at a bounded page, while malformed work
  needs a separate diagnostic budget.
- `mark_view_dirty_set` is called for empty projections and locks the bus. These are measurable
  residuals, but route/demand convergence removes or bounds most of them.

Do not optimize these details while retaining per-view remote subscriptions, empty synchronous
polling and duplicate fact/inspection delivery.

## Required architecture hard cut

1. Runtime07/08 publishes one immutable `WorldChangeGeneration` from the mutation commit. It carries
   typed added/removed/reparented/component/asset operations, canonical ordering, affected route
   identities, explicit overflow/resync and the inspection artifact generation. Facts and dirty
   tokens must not become two independently replayed authorities.
2. Editor02 compiles one immutable `WorldSyncRouteGeneration` grouped by distinct `WatchKey`. The
   runtime owns at most one subscription per distinct route; editor view/tool bindings fan out
   locally from shared route payloads. Registration changes publish one generation, not per-frame
   scans or per-view remote tokens.
3. Runtime10 exposes a bounded page/cursor and a ready generation. Empty-to-ready transitions signal
   the existing session wake channel. Stable no-ready frames issue no FFI call, allocate no `[]`,
   parse no topic and lock no WorldSync queue.
4. Runtime11's existing ordered per-session lane seals WorldSync pages under a short session lock,
   serializes/decodes outside that lock and returns a generation-tagged immutable completion. It is
   the same lane selected for tick/plugin-event work; no WorldSync-private pool is allowed.
5. Gateway replacement captures a generation/route lease, performs foreign work outside the
   replacement mutex and commits only if both generations remain current. Replacement cancels or
   discards old completions without consuming new-session data and without waiting on slow providers.
6. Editor05/Layout09 consumes typed hierarchy operations directly. Added/removed/moved rows enter a
   bounded pending operation queue; broad invalidation, overflow or generation gap alone requests an
   authoritative reflow. Remove the unused `editor.world_fact` JSON route.
7. Apply work is count/time/byte budgeted and resumable. Render17 records route, generation, queue
   age/peak, bytes, seal/encode/decode/project/apply time, session/World lock wait/hold, cancellation,
   overflow, stale completion and authoritative reflow reason.

## Unreal primary-source comparison

- Unreal's `ActorHierarchy.cpp:39-67` creates one hierarchy event source and subscribes it to engine
  actor/level/folder delegates. It does not allocate one engine subscription per Outliner row or
  view.
- `ActorHierarchy.cpp:697-756` turns actor add/delete/attach/detach directly into typed
  `Added/Removed/Moved` hierarchy changes; the added path checks visibility before constructing an
  item because label creation can be expensive. Broad partition/list changes alone call full
  refresh (`ActorHierarchy.cpp:796-829`).
- `SceneOutlinerStandaloneTypes.h:228-253` carries typed items or stable item IDs and explicit
  `Added/Removed/Moved/FolderMoved/FullRefresh` semantics. `SSceneOutliner.cpp:2111-2221` queues those
  operations and requests refresh only when work exists.
- `SSceneOutliner.cpp:725-886` processes pending operations incrementally. Its configured processing
  budget is 5 ms/frame and is checked every 100 items (`SSceneOutliner.cpp:40-44,778-817`); tick calls
  populate only when `bNeedsRefresh` (`SSceneOutliner.cpp:2432-2468`).

Zircon should adopt the event-source ownership, typed incremental operation and demand/budget
principles. The exact Rust DTOs, budgets and session-lane mechanics remain Zircon-specific; source
comparison is not measured performance parity.

## Acceptance and measurement plan

| case | matrix | required result |
|---|---|---|
| idle | in-process/dynamic; 60/120/240Hz; 10/300s | ready generation stable; WorldSync FFI, session/World/subscription/bus locks, JSON bytes, topic alloc and route scans=0 |
| route sharing | keys 1/100/10K; bindings per key 1/2/16; change 0/1/10/100% | runtime subscriptions and wire dirty identities=distinct affected keys, not views; stable registration work=0 |
| hierarchy operations | entities 1/1K/100K; add/remove/move mixed; storm 1/10K/1M | accepted op loss/dup/reorder=0; queue count/bytes/age hard bounded; apply respects frame budget; full reflow only typed reason |
| replacement | provider delay 0/10ms/10s; replace before/during/after seal/decode/apply | first new-session change applied once; old completion apply=0; replacement wait independent of provider delay; compensation exact |
| malformed/overflow | dirty/op rows 0/64/1K/100K; duplicate 0/50/99%; bytes 1KiB/1MiB/64MiB | bounded page validation; no destructive oversize loss; one resync marker; diagnostic work bounded separately |
| product | F4 hierarchy with 1/1K/100K nodes; idle/rename/spawn/reparent/delete/reload | current Cargo plus WPR CPU/thread/wake/lock p50/p95, allocation/RSS/package power and operation counters GREEN |

Run Zircon and an available local Unreal editor build on the same machine, scene scale, frame cap,
foreground state and power plan. Compare CPU, wakeups, p50/p95, RSS and package power. Unreal's 5 ms
source budget is architecture evidence, not an empirical claim that Zircon has reached parity.
RenderDoc is not applicable to this CPU/synchronization slice.

## Per-file review

| file | current-source result |
|---|---|
| `mod.rs` | Export-only owner; now exposes live pump and watch map. |
| `pump.rs` | Correct basic lifecycle and projection, but owns synchronous idle polling, duplicate fact JSON, generation race and unpaged UI work. |
| `pump/tests.rs` | 10 lifecycle/behavior tests cover in-process facts, reuse, collision and replacement blocking. No ready-only, route sharing, page budget or nonblocking replacement test. |
| `watch_map.rs` | Dirty-only/canonical projection and borrowed view coalescing are sound. Per-view remote identity and one-element Vec metadata are the wrong scale boundary. |
| `watch_map/tests.rs` | 10 deterministic index/projection tests cover malformed batches and cleanup. No multi-view same-key route convergence or scale counters. |

## Static gates executed

- Read 5/5 production/module-test files in full; traced the full retained/runtime/ABI/inspection chain
  and relevant Unreal source.
- `rustfmt --edition 2021 --check --config skip_children=true` passed all five files.
- Scoped `git diff --check` passed; Git emitted only existing LF-to-CRLF checkout warnings.
- Two Python WorldSync suites ran 13 tests: 12 passed and one foreign source-shape guard errored on
  its retired `gateway.watch_world(...)` anchor. The separate failure handoff records this drift.
- Managed Cargo and WPR product capture remain blocked by the recorded build-helper approved-root
  separator defect. No output artifact was written to C:.
- Protected plans/indexes were not modified. This static review is not an accepted milestone, so no
  commit or WeCom notification is due.
