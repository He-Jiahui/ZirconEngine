---
related_code:
  - zircon_editor/src/ui/control
  - zircon_editor/src/ui/host/editor_event_control_requests.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/workbench/reflection/route_registration
  - zircon_runtime/src/ui/event_ui/manager
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandList.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/UICommandList.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/FastUpdate/SlateInvalidationRoot.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
---

# Protected plan routing: Editor UI control, reflection and routes

## Reason for routing

The main performance plan, `review.md`, `pending.md`, Runtime UI 11A and numbered owner plans are
protected or foreign dirty. Broad Runtime/Editor source is leased by the active `mvp00` session. This
record routes the current 3/3-file evidence without overwriting those owners. Evidence source:
`2026-08-19-editor-ui-control-reflection-route-generation-architecture-review.md`.

## Requested Performance01 updates

### PERF-MVP-076 and PERF-MVP-099

Extend full-reflection ownership through the control demand boundary:

- every control request drains pending refresh before request classification;
- a query can synchronously build complete chrome/model/routes/reflection;
- `replace_tree` scans old/current nodes and rebuilds the index for all trees;
- tree/node/property results deep-clone owned artifacts under the shell mutex;
- mirror property writes do not update the live surface and can report false success.

Required target: one committed surface/reflection generation, read-only shared artifact, target-tree
persistent index and explicit surface transactions for writes. Queries must never trigger projection;
stable generation build/diff/index work is zero and one-tree updates do not scan unrelated trees.

### PERF-MVP-252

Keep the dead-subscriber finding and promote acceptance to a transport-bound subscription lifecycle.
`SubscribeDiffs` must not return success after discarding the only receiver. Require subscription
owner receipt, bounded entries/bytes/age, tree-generation coalescing, send-failure cleanup, resync
cursor and terminal leak census. Broadcast cannot clone a wide payload once per dead/slow subscriber.

### PERF-MVP-572

Add the full route-generation evidence. Reflection scans every menu/activity action and calls
`route_id_for_binding()`, which formats complete native binding Strings. Direct duplicate registration
can overwrite the binding index while retaining the old route/handler, and no route has owner-scoped
unmap or unload retirement.

Required target: typed stable binding identity, dense `UiRouteId`, immutable shared binding/default
arguments, one compiled generation, duplicate policy and owner retirement receipt. Native strings are
authoring/serialization/error boundaries only. Do not copy Unreal's recursive parent/child search as a
global dynamic hot path; use its paired map/unmap and shared command identity principles.

### New P0 child item: reflection artifact and remote demand isolation

Add one Runtime09/EditorUI08 child covering the mirror-authority defect and remote demand behavior.
The live surface transaction is the sole writer; reflection is an immutable generation artifact.
Remote queries read a committed generation without UI build/lock inversion, while remote writes route
through the same authorized surface transaction and receipt path used by local input.

Acceptance must measure queries/trees/nodes/changed ratios, build/diff/index counts, scanned nodes,
shell/command lock wait/hold, cloned bytes, mirror/live parity, query p95 and package power.

### Direct P1 stopgap for Runtime09

When the broad Runtime lease is released, change `query_property()` to borrow through
`node_index -> tree -> node -> property` and clone only the returned property. Add a wide-node
regression proving node properties/actions are not cloned. This is a useful independent reduction but
does not close the P0 artifact cutover.

## Requested owner-plan updates

### Runtime09

Own `UiRuntimeService`, `UiReflectionArtifact`, compiled route generations and bounded subscription
transport. Remove writable mirror authority, global index rebuild and naked route/subscription
lifetimes. Publish counters for build/diff/index/route/queue/root ownership.

### EditorUI08

Stop treating remote control as a synchronous Workbench materialization request. Coalesce dirty
domains once per frame, publish one committed reflection handle, release shell/command locks before
projection, and serve read queries from the immutable handle. This plan also consumes the Activity
registry generation defined by the companion review.

### Editor08

Provide stable command/binding identity, authorization and provenance for compiled UI routes. Native
binding text remains a bounded external representation; normal input/remote execution passes typed
handles. Plugin disable/reload retires owner routes before unload.

### Optimize Runtime UI 11A

Retain product-architecture ownership for the single runtime UI service, surface transaction,
reflection read model, routes and subscriber lifecycle. Performance01 supplies the scale, lock,
allocation, latency and power gates; no second Editor-only authority is introduced.

## Requested protected index state

- `pending.md`: add or retain one concise row for `zircon_editor/src/ui/control/**` with 3/3 files,
  138 lines, 0 in-module tests, fingerprint `c199375a...`, and
  `static_complete / structural_cutover_required / dynamic_pending`.
- `review.md`: do not add the module. Require Runtime09/EditorUI08 generation cutover, route and
  subscription retirement, mirror/live parity, managed Cargo, scale counters and 31-run F0/F4
  WPR/ETW/allocator/RSS/package-power evidence.
- Keep module-level accounting. The three external control tests and delegated Runtime manager are
  supporting evidence, not separate completion rows for this module.

## Validation and milestone rule

All artifacts stay on D/E/F. RenderDoc is conditional on rendered UI state/resource changes and cannot
replace CPU, lock, allocation or queue evidence. Commit and send quantified WeCom results only after
the owner cutovers and product gates pass; this static routing record is not a completed milestone.
