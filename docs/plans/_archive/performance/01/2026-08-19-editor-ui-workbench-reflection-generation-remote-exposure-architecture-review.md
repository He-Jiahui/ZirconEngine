---
related_code:
  - zircon_editor/src/ui/workbench/reflection
  - zircon_editor/src/ui/reflection
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/control/service.rs
  - zircon_runtime_interface/src/ui/event_ui
tests:
  - zircon_editor/src/tests/workbench/reflection
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/49-editor-event-runtime-envelope-listener-registry-journal-replay-snapshot-dirty-lifecycle-product-integration-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_code:
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/RemoteControl/Source/RemoteControl/Public/RemoteControlPreset.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/RemoteControl/Source/RemoteControl/Public/IRemoteControlModule.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/RemoteControl/Source/WebRemoteControl/Private/WebRemoteControl.cpp
  - dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/WidgetSnapshotService.h
  - dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/WidgetSnapshotService.cpp
  - dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/Widgets/SWidgetReflector.cpp
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Public/ToolMenuEntry.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandList.h
doc_type: current-architecture-performance-review
status: static_complete_structural_cutover_required_dynamic_pending
created_at: 2026-08-19
---

# Editor UI Workbench reflection generation and remote exposure architecture review

## Status

- Result: `static_complete / structural_cutover_required / dynamic_pending`.
- MVP priority: P0 for deny-by-default remote exposure, generation-owned reflection and route
  registration; P1 for transient-state deltas and the diagnostic widget reflector.
- Accounting: retain `zircon_editor/src/ui/workbench/reflection/**` in `pending.md`. Do not add it to
  `review.md` before remote policy, stable-generation zero-work, scale, current-source Cargo and F4
  product-trace gates pass.
- Code disposition: no Rust source changed. The active MVP00 session holds the Editor source tree,
  ten scoped files contain foreign changes, and the required correction crosses command, control,
  reflection and runtime-interface ownership. A local cache or clone removal would preserve the
  wrong authority.

## Exact scope

| scope | files | physical lines | tests | raw bytes | ordered path-and-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/ui/workbench/reflection/**` | 27/27 | 1,324 | 2 in-module | 48,357 | `5b19374416825d920df20d19591203579aef0bed9056ad536e5696f0ad5cad40` |
| external Workbench reflection tests | 5/5 | 663 | 10 | 25,729 | `0b69bfdab4965b833549c8b51a239718ba6b3b36c134cb99119195ec35aa0d00` |

The fingerprint is SHA256 over sorted normalized path, NUL, raw bytes, NUL. All 27 production files
and all five external test files were read in full. Nine production files and
`tests/workbench/reflection/action_dispatch.rs` contain foreign changes, so this fingerprint is a
current-source review anchor, not a clean-tree baseline. Implementation must re-read and re-hash the
scope after those owners converge.

## Module acceptance record

| module | files/lines | current-source performance verdict |
|---|---:|---|
| `activity_actions/**` | 6 / 182 | Rebuilds common and activity-specific owned `UiActionDescriptor` vectors for every activity in every full reflection. Viewport contributes nine descriptors, including raw pointer and resize input. |
| activity collection/descriptors/name/drawer | 4 / 239 | Recursively returns and extends owned activity vectors, clones string-bearing host and payload state, recreates descriptor strings, and recomputes name mappings on each full projection. |
| animation/asset/docking/draft/inspector routes | 5 / 189 | Converts domain actions to owned bindings and routes on every model registration. The route builder later marks all actions remotely callable. |
| `viewport_route/**` | 3 / 118 | Maintains a parallel string-to-command map and exposes pointer move, press/release, scroll and resize through the generic reflected action plane. |
| `route_registration/**` | 5 / 159 | Recreates bindings for lookup and again on miss, clones menu bindings, revisits every activity action, and mutates every route to `callable_from_remote=true`. |
| `model_build.rs` | 1 / 126 | Clones full page/drawer/floating activity models and menu fields; one menu leaf constructs its complete binding twice. |
| `transient_ui_state.rs` | 1 / 139 | Clones node paths for hover/focus/pressed updates and returns no changed bit; full projection scans every snapshot node and upserts four transient properties. |
| `widget_reflector.rs` | 1 / 148 | Recursively materializes and sorts a complete owned row set with no depth/node/time/byte budget or virtualization. |
| facade | 1 / 24 | Re-export only. |

## Structural bottlenecks

### P0: one logical state is rebuilt into three owned generations

`refresh_reflection()` first constructs Workbench layout/activity/menu models while holding the shell
and command-registry locks. `model_build.rs` then copies those rows into
`EditorWorkbenchReflectionModel`. `EditorUiReflectionAdapter::build_snapshot()` finally formats new
paths, clones labels/actions/properties and inserts every node into a fresh `BTreeMap`.

An activity copies instance, descriptor, title, host and recursive JSON properties before
`activity_node()` clones the action vector and JSON values again. Menu reflection converts a binding
to the runtime form, clones its symbol, formats a node path and creates a new action descriptor. The
adapter also hard-codes menu actions as remotely callable when a route ID exists. Stable UI therefore
still pays layout-to-model-to-snapshot ownership, string formatting, ordered-map insertion and route
lookup; lock scope covers much of the producer work.

The target is one immutable `ReflectionGeneration` whose static nodes, action metadata and route
handles are shared by presentation, control and diagnostics. State changes publish a typed delta
against that generation. A fully owned wire snapshot is materialized only for initial subscription,
explicit resync or export, outside shell/command locks and under a byte budget.

### P0: route registration is incorrectly coupled to snapshot publication

`register_model()` traverses all menus and activities each time. `register_stub_route()` first calls
`binding.as_ui_binding()` for lookup and repeats the conversion on a miss. Menu registration clones
the complete binding before lookup. Activity routes are therefore rediscovered and rebound even when
the command definition set has not changed.

Routes belong to a command/control definition generation, not a visual snapshot. Optimize08's
InvocationGateway must compile stable handles once per owner generation, revoke them atomically on
unload, and let reflection reference those handles. Stable layout, selection or transient changes
must perform zero route lookup, registration, binding conversion and route-table mutation.

### P0: all reflected actions are remote-open, including realtime input

`register_action_route()` sets `callable_from_remote=true` for both existing and newly registered
routes. The reflection adapter independently marks routed menu actions remote. Consequently docking,
Inspector batch edits, draft changes, asset import, animation track changes and raw viewport pointer
move/press/release/scroll/resize share one generic remote surface.

The external test `workbench_reflection_routes_mark_activity_actions_as_remotely_callable` requires
this behavior, so the current test suite actively preserves the defect. High-frequency input can
enter the serialized binding and main-thread command path without an explicit exposure catalog,
principal policy, per-action payload limit, rate budget or coalescing contract. This is both a
security boundary failure and a denial-of-service/performance amplifier.

Remote control must be deny-by-default. Only semantic commands explicitly exposed in a versioned
automation preset may cross the remote gateway. Pointer motion, button edges, scroll and resize use
the typed realtime input plane and are never generic reflected remote actions. Tests must invert the
current default and separately prove authorized semantic automation.

### P0: static action descriptors are rebuilt per activity and per publication

Every activity creates at least two common owned descriptors. Inspector adds three, Assets two, and
Scene/Game nine viewport descriptors. These IDs, symbols and parameter schemas are definition-static
but are recreated for every tab in every full reflection. Route registration then walks them again,
and `activity_node()` clones them into the runtime snapshot.

Compile action schemas once into an immutable activity-kind catalog keyed by a stable descriptor or
command handle. Activity instances carry only instance state and references. External serialization
may expand handles at a boundary, but the local hot path does not own another descriptor vector per
tab.

### P1: activity traversal can become quadratic on adversarial split shape

`collect_workspace_activities()` recursively allocates a `Vec` for each subtree and extends the left
result with the right result. A right-deep tree repeatedly moves the already-large right subtree,
giving O(N squared) element moves and repeated growth in the worst shape. It also recursively follows
unbounded layout depth.

The generation cutover should project through one iterative sink, reserve from the validated tab
count, enforce a hard depth/node budget and retain stable instance slots. The fallback full builder
must remain O(N) time and O(N) output space for balanced, left-deep and right-deep trees.

### P1: transient state lacks no-op and sparse-delta semantics

Hover/focus setters clone paths before proving the state changed; pressed insertion clones before
duplicate detection. `apply()` reports no changed bit. Full projection scans every node and upserts
the same four transient properties even if only one node changed.

Common retained events already have some patch paths, so this review does not claim every pointer
event triggers a full reflection. The remaining contract is still wrong: transient state must return
a precise changed set, coalesce pointer levels, preserve lossless button edges and patch only affected
node flags. Repeated identical state produces zero allocation, publication and wakeup.

### P1: the widget reflector is an unbounded diagnostic full-tree export

`WidgetReflector::rows()` recursively visits the full snapshot, clones path/class/display strings,
uses an ordered visited set and materializes every row. Its two tests contain only two nodes. There is
no maximum depth, node count, elapsed time, output bytes, paging or cancellation.

Keep this feature off the normal frame path. It must be explicitly requested, bounded, cancellable
and virtualized. A production reflection update must never build reflector rows. Deep or cyclic-invalid
input fails with a typed partial/truncated result rather than stack overflow or unbounded memory.

## Reference-engine evidence

- Unreal `RemoteControlPreset.h:427-436` requires explicit `ExposeProperty()` or
  `ExposeFunction()` calls and stores exposed entities in the preset; `IsExposed()` checks that
  catalog. `IRemoteControlModule.h:477-501` resolves preset functions/properties/presets explicitly.
  `WebRemoteControl.cpp:1136-1239` implements the preset call route through that resolution before
  `InvokeCall()`. This supports a separate, explicit remote exposure catalog and rejects Zircon's
  rule that every reflected local UI action becomes remote callable.
- Unreal `WidgetSnapshotService.h:22-26` models a remote widget snapshot as an explicit request with
  an abort handle. `WidgetSnapshotService.cpp:51-62` takes and serializes a snapshot only while
  servicing such a request. `SWidgetReflector.cpp:100-108,411-427` exposes a console/button action,
  pending state and cancellation. This supports an on-demand diagnostic snapshot, not continuous
  full-tree row materialization in the product update path.
- Unreal `ToolMenuEntry.h` and `UICommandList.h` bind entries through shared command identity and
  command-list lookup. That supports generation-owned action metadata and stable handles rather than
  recreating action strings and routes per reflected instance.

These are ownership and control-flow references. They do not establish Zircon timing targets.
Same-hardware WPR/ETW, allocation, latency and package-power evidence remains mandatory.

## Required architecture cutover

1. Optimize08/Editor08 defines the canonical command identity, owner generation and
   `InvocationGateway`. Route registration happens once per definition generation and returns stable
   handles with atomic revoke/stale semantics.
2. Add a separate versioned `RemoteExposureCatalog`. Default policy is deny. Exposure records carry
   semantic command handle, principal/capability policy, payload schema and byte/rate/concurrency
   budgets. Remove `with_callable_from_remote(true)` and blanket route mutation in the same hard cut.
3. EditorUI08 creates one immutable `ReflectionGeneration`: stable node slots, activity/action
   catalogs, menu generation and route handles. Shell/layout/command locks are released before wire
   serialization or subscriber fanout.
4. Normal updates publish typed node/property/action deltas with generation and causal receipt.
   Stable/no-op events publish nothing. Full owned snapshots are reserved for cold subscribe,
   explicit resync and bounded export.
5. Replace recursive activity collection with a budgeted iterative traversal and exact/precomputed
   capacity. Static action schemas are shared by kind; per-instance state does not clone them.
6. Split transient pointer levels from lossless edges. Update only changed node flags and coalesce
   levels per frame. Repeated hover/focus/pressed state is allocation-free and publication-free.
7. Move `WidgetReflector` behind an explicit diagnostics request with node/depth/time/byte budgets,
   cancellation, pagination/virtualization and typed truncation telemetry.
8. Migrate tests from blanket remote-open expectations to default-deny, explicit exposure,
   unauthorized early rejection, realtime-input exclusion, owner revoke and budget enforcement.

## Milestones

| milestone | deliverable | dependency |
|---|---|---|
| M0 | Add counters for generation builds, node/action/property clones, formatted/String bytes, route lookup/register/mutation, shell/command lock wait+hold, full/delta publications and remote reject stage. | current source re-read |
| M1 | Hard-cut blanket remote flags; introduce explicit exposure catalog and invert tests. Raw viewport input is absent from generic remote actions. | Optimize08 policy and migration contract |
| M2 | Compile command/action/route metadata once per owner generation; stable route handles and atomic owner revoke. | M1 |
| M3 | Publish immutable reflection generations and sparse typed deltas; full wire snapshot only for cold/resync/export outside locks. | M2 + EditorUI08 |
| M4 | Linear budgeted activity traversal, no-op transient updates and on-demand bounded widget diagnostics. | M3 |
| M5 | Current-source Cargo, F4 product, scale, WPR/ETW, power and conditional RenderDoc acceptance; only then move the module to `review.md`. | M0-M4 |

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| stable generation | tabs/actions/menu nodes `1/100/10k/100k`; 1k stable frames | generation builds, node/action/property clones, path/String bytes, route lookups/registers/mutations, full snapshots and publications `=0` after warm-up |
| changed state | affected nodes `1/10/1k`, total nodes `1/10k/100k` | visits and allocation proportional to affected set plus bounded index lookup; one causal delta; shell/command locks exclude serialization/fanout |
| activity shape | balanced/left-deep/right-deep, depth `1/64/max/max+1`, tabs `1/1k/10k/100k` | accepted traversal O(N), no recursive stack growth, one output allocation plan; over-budget fails before unbounded work |
| route lifetime | owners `1/1k`, actions `1/100/10k`, reload/revoke/stale | one registration per definition generation; stable visual updates do zero route work; revoke atomic; stale handle cannot dispatch |
| remote policy | every reflected action; authorized/unauthorized/stale principal; payload/rate/concurrency boundary | default callable count `=0`; raw pointer/scroll/resize remote routes `=0`; unauthorized rejected before domain dispatch; only explicit semantic exposure succeeds; all rejects metered |
| transient input | pointer levels `1/1k/1M`, button edges, repeated hover/focus | identical levels allocate/publish/wake `=0`; latest level coalesces; edges preserve order; work bounded per frame |
| diagnostics | nodes `1/1k/100k`, depth/cycle-invalid, local/remote, cancel | off-state cost `=0`; request respects node/depth/time/byte budgets; cancel bounded; rows paged/virtualized; typed truncation |
| product | F4 cold/warm/idle/activity storm/remote abuse, 31 runs | WPR/ETW CPU, allocation, lock, wakeup, queue, RSS, input-to-effect p50/p95/p99 and package power on identical hardware/assets/settings; artifacts on D/E/F |

RenderDoc is conditional. Require it if the cutover changes rendered UI geometry, clipping, resources,
draw ordering or visible output; then capture event/draw/resource and pixel parity. It is not the proof
for CPU route/reflection ownership, which belongs to WPR/ETW and explicit counters.

## Static gates executed

- Read 27/27 production files and 5/5 external tests; reproduced 1,324 production lines, 48,357
  bytes, two inline tests, ten external tests and current-source fingerprint `5b193744...`.
- Traced full reflection through shell/command-locked model construction, route registration,
  `EditorUiReflectionAdapter`, snapshot publication and transient projection.
- Confirmed blanket remote enablement in both action-route registration and menu snapshot adaptation;
  confirmed the external test suite requires remote access to raw viewport and mutating domain
  actions.
- Read the cited Unreal Remote Control, WebRemoteControl, Slate Reflector, ToolMenus and command
  primary sources. The source evidence supports explicit remote exposure and on-demand diagnostic
  snapshots.
- `rustfmt --edition 2021 --check` passed for all 27 production and five external test files. Scoped
  `git diff --check`, 28/28 routed-path existence and
  `python -m tools.session_coordinator --repo-root . --json plan audit` passed. The production
  fingerprint remains `5b193744...` after the documentation write.
- The documentation convention gate reports zero violations owned by these two records. The
  unrelated repository baseline remains 692 violations across 242 documents out of 2,721 scanned.
- Dynamic Cargo, scale counters, F4 launch, WPR/ETW, package power and conditional RenderDoc evidence
  remain pending. This is not an accepted milestone, so no commit or WeCom notification is due.
