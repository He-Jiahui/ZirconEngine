---
related_code:
  - zircon_editor/src/ui/control
  - zircon_editor/src/ui/host/editor_event_control_requests.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/workbench/reflection/route_registration
  - zircon_runtime/src/ui/event_ui/manager
tests:
  - zircon_editor/src/tests/ui/control
  - zircon_runtime/src/ui/tests/event_manager.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandList.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/UICommandList.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/FastUpdate/SlateInvalidationRoot.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
doc_type: current-architecture-performance-review
status: static_complete_structural_cutover_required_dynamic_pending
created_at: 2026-08-19
---

# Editor UI control, reflection and route generation architecture review

## Status

- Result: `static_complete / structural_cutover_required / dynamic_pending`.
- MVP priority: P0. Every remote control request can become a demand boundary for complete Workbench
  reflection, and the stored reflection mirror, routes and subscriptions have independent lifetime
  authorities.
- Accounting: retain `zircon_editor/src/ui/control/**` in `pending.md`. It cannot enter `review.md`
  before Runtime09 and EditorUI08 complete the shared generation cutover and dynamic scale matrix.
- Code disposition: no Rust source changed. The current `mvp00` session owns broad
  `zircon_runtime/src` and `zircon_editor/src` scopes. A safe local `query_property()` clone reduction
  is identified below, but source ownership must be respected.

## Exact scope

| scope | files | physical lines | in-module tests | raw bytes | ordered path-and-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/ui/control/**` | 3/3 | 138 | 0 | 4,585 | `c199375ada4e70f07f4fa853b6d7525e4de1007737853f538964ea49484cd56f` |

The fingerprint is SHA256 over sorted normalized path, NUL, raw bytes, NUL. All three production
files were read in full. The three external control test files contain 169 lines and three tests.
The nine directly delegated Runtime manager files were also read in full: 512 lines and four current
event-manager tests.

## File acceptance record

| file | current-source performance verdict |
|---|---|
| `error.rs` | Two descriptor duplicate errors plus invocation passthrough. No hotspot; duplicate-only semantics expose the append-only registry defect. |
| `mod.rs` | Re-export only. No independent runtime cost. |
| `service.rs` | Thin code, wide responsibility: Activity registry, route registry, reflection store, mutable remote property mirror, query service and subscriber transport all share one shell-owned object. Most cost is delegated, but this facade makes the complete manager run under the Workbench shell mutex. |

## Structural bottlenecks

### P0: every control request can force complete Workbench materialization

`handle_control_request()` calls `drain_pending_view_refreshes()` before distinguishing a query from
an invocation. Any pending non-structure dirtiness can synchronously enter full reflection while the
requesting thread waits. That path locks shell then commands, rebuilds descriptors, chrome, Inspector,
Activity Log, capabilities, command evaluation, view model, routes and the complete reflection tree.

After the drain, tree/node/property requests lock the shell again. A headless inspection query can
therefore inherit user-input, plugin callback and presentation work rather than read a stable artifact
at a declared generation. This is priority inversion and an observability hazard: the profiler query
itself changes the work being profiled.

### P0: `replace_tree` rebuilds global indexes and owns a second mutable UI tree

`UiEventManager::replace_tree()` stores the new owned snapshot, compares every current node against
the previous tree, scans previous IDs for removals, then clears and rebuilds the path index by scanning
every node in every tree. Replacing one tree is O(previous tree + current tree + all stored trees),
before subscriber fanout. Queries then deep-clone complete snapshots or nodes.

`set_property()` mutates only this reflection snapshot and reports success; it does not transact with
the live `UiSurface`. Reflection patches similarly mutate the mirror and can later be replaced by a
full projection. The mirror is consequently a second state authority, not merely an immutable read
artifact. Optimizing its BTreeMap probes would preserve the wrong system.

### P0: subscriptions are unbounded and remote subscribe leaks dead senders

`subscribe()` creates an unbounded crossbeam channel. Broadcast clones every notification for every
subscriber and ignores send failure. `UiControlRequest::SubscribeDiffs` immediately drops the returned
receiver but retains its sender, so every such request permanently adds a dead subscriber until an
explicit ID-based unsubscribe arrives from a transport that never received the receiver.

Wide reflection and invocation payloads can therefore accumulate without entry, byte or age limits.
This confirms PERF-MVP-252 and is also a lifecycle leak: teardown has no owner receipt or automatic
disconnect.

### P0: routes are string-compiled at lookup time and lack generation retirement

`route_id_for_binding()` calls `native_binding()` for every lookup, formatting path, action and
arguments into a new String. Full reflection revisits all menu/activity actions and performs these
lookups. Invocation repeats formatting, then clones arguments, binding, context and result before
subscriber fanout.

Stable Workbench routes usually reuse the existing ID through the string index, which prevents one
specific per-reflection leak. The underlying registry still allows direct duplicate registration to
overwrite `routes_by_binding` while retaining the old handler in `routes_by_id`; there is no unmap,
owner token, generation publication or plugin-unload retirement. Stub routes are retained even though
direct manager invocation must fail without a handler. This confirms PERF-MVP-572 and Optimize11A
P1-21 rather than justifying another facade cache.

### P1: query APIs return wide owned values

`query_tree()` clones a complete reflection snapshot. `query_node()` clones a complete descriptor.
`query_property()` currently calls `query_node()` and therefore clones the node, all properties and
actions before cloning the one requested property. The last behavior is a valid isolated stopgap:
resolve path/tree/node by reference and clone only the final property. It must not be represented as
the reflection architecture fix.

### P2: Activity registry is append-only and its full getters are dead

Activity view/window maps reject duplicate IDs, have no replace/remove/generation operation, and full
catalog getters deep-clone every descriptor. Current production uses only point lookup and register;
no production caller was found for either full getter. Stable reflection still reconstructs candidate
descriptors before point-checking them, while changed metadata is silently ignored. The companion
Activity review owns the detailed registry cutover.

## Reference-engine evidence

- Unreal `UICommandList.h:90-125` provides paired map, unmap and mapped-state operations over shared
  command identity. `UICommandList.cpp:54-100` stores actions in the command map and removes both action
  and context entries on unmap. Zircon needs the paired generation/owner lifecycle, not append-only
  string routes.
- Unreal `UICommandList.h:251-260` uses the shared command object as the binding-map key;
  `UICommandList.cpp:254-275` first performs a direct map lookup. Zircon should compile a typed
  `UiRouteId` handle once at route-generation publication instead of serializing a native string per
  event. Unreal's recursive parent/child fallback is explicitly not a model for an unbounded dynamic
  route graph; Zircon's hot path should remain direct/indexed.
- Unreal `SlateInvalidationRoot.h:161-203` separates slow-path rebuild from dirty pre/post/prepass
  queues. `SlateInvalidationRoot.cpp:299-340` accumulates reasons on a widget proxy and inserts unique
  dirty entries. This supports generation-bound changed-node processing rather than replacing and
  reindexing the complete reflection mirror for every demand.

These sources establish data ownership and invalidation shape. They do not prove Zircon's latency or
power target. Same-hardware measurements remain mandatory.

## Required architecture cutover

1. Runtime09 owns one `UiRuntimeService`. A committed surface generation publishes an immutable
   `UiReflectionArtifact` containing shared node records, per-tree generation and a persistent/path
   index. Reflection is read-only; writes become surface transactions and publish the resulting delta.
2. EditorUI08 consumes dirty domains once per frame and publishes the Workbench artifact once. Remote
   queries read the latest committed handle or return an explicit stale/generation result; they never
   synchronously cause chrome/model/reflection construction.
3. Replace global `rebuild_node_index()` with target-tree index construction/reuse and apply changed
   nodes to a persistent artifact. Diff work is proportional to changed nodes plus removed ranges,
   while structural replacement is once per source generation.
4. Editor08/Runtime09 publish `CompiledUiRouteGeneration`: stable typed binding identity to dense
   route handle, immutable binding/default arguments, duplicate policy and owner retirement receipt.
   Normal invocation does not format/parse native strings. Plugin disable/reload revokes routes before
   code unload.
5. Bind subscriptions to real transport/session lifetime. Use bounded per-subscriber queues with
   coalescing by tree/generation, entry/byte/age policy, send-failure cleanup, resync cursor and explicit
   drop/disconnect diagnostics. `SubscribeDiffs` cannot succeed without a consumer channel.
6. Split Activity registry from remote UI control. Publish one immutable descriptor generation with
   add/update/remove semantics; delete unused full-clone getters after call-site verification.

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| demand boundary | queries `1/1k/100k`, pending dirtiness `0/1/1k`, trees `1/100` | query-triggered chrome/model/reflection builds `=0`; shell lock excludes projection; explicit observed generation; query p95 and wait bounded |
| reflection | trees `1/100`, nodes/tree `1/1k/100k`, changed `0/1/1%/100%` | stable build/diff/index `=0`; one-tree change does not scan other trees; work near changed nodes; mirror/live divergence `=0`; node/path owner count `=1` |
| query clone | node properties/actions `1/100/10k`, query tree/node/property | property query clones only returned property; normal narrow query full-tree/node copied bytes `=0`; explicit full snapshot export remains bounded/paged |
| routes | routes `1/100/10k`, invokes `1/1M`, reload/unload | stable native String format/parse `=0`; direct typed lookup near O(1); route build `=1/generation`; stale handlers/roots `=0` after retirement |
| subscriptions | subscribers `1/100/10k`, slow/dead, 100k diffs/invocations | queues have entry/byte/age limits; dead sender removed on failure; stable generations coalesce; retained bytes and delivery age bounded; resync exact |
| product | F0/F4 cold, warm, idle, query and input storms, 31 runs | WPR/ETW CPU/waits/wakeups/locks/allocations/RSS/UI p95/package power on identical hardware/assets/settings; artifacts only on D/E/F |

RenderDoc is not a control-plane profiler and is not required for a route/query-only cutover. If the
reflection transaction changes rendered UI state, capture submitted UI draw/resource/output parity;
use WPR/ETW and allocator counters for the bottlenecks above.

## Static gates executed

- Read 3/3 control Rust files, all three external control tests, all nine delegated Runtime manager
  files, all four event-manager tests and the named Editor production callers.
- Reproduced 4,585 raw bytes and source fingerprint `c199375a...` after caller/reference review.
- Confirmed control requests drain before request classification; full reflection runs under shell and
  command locks; replace scans old/new nodes and rebuilds every tree index; queries return owned data.
- Confirmed unbounded notification channels, discarded remote receiver, ignored send failure and
  per-subscriber payload clones. Confirmed string route lookup and missing duplicate retirement.
- `rustfmt --edition 2021 --check` passed for all 3/3 production files and all three external control
  test files. Scoped `git diff --check`, 18/18 routed-path existence and
  `python -m tools.session_coordinator --repo-root . --json plan audit` passed.
- The documentation convention gate reports 0 violations owned by the four current Activity/control
  records. Its global baseline remains red at 692 violations across 242 of 2,709 documents.
- The source fingerprint was recomputed after documentation edits and remains `c199375a...`.
- Read the cited Unreal primary sources and current Runtime UI 11A review. Managed Cargo, scale
  counters, F0/F4 launch, WPR/ETW, allocator and package-power evidence remain pending. This is not an
  accepted milestone, so no commit or WeCom notification is due.
