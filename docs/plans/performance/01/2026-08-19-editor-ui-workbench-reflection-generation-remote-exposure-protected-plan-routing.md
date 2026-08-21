---
related_code:
  - zircon_editor/src/ui/workbench/reflection
  - zircon_editor/src/ui/reflection
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/49-editor-event-runtime-envelope-listener-registry-journal-replay-snapshot-dirty-lifecycle-product-integration-review.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
source_evidence:
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/RemoteControl/Source/RemoteControl/Public/RemoteControlPreset.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/RemoteControl/Source/WebRemoteControl/Private/WebRemoteControl.cpp
  - dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/WidgetSnapshotService.h
  - dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/Widgets/SWidgetReflector.cpp
---

# Protected plan routing: Workbench reflection generation and remote exposure

## Reason for routing

The main performance plan, `pending.md`, `review.md`, Optimize08, Optimize49 and numbered owner plans
are protected or foreign dirty. The Editor source tree is leased by the active MVP00 session and ten
scoped files contain foreign changes. This record routes the current 27/27-file evidence without
editing those authorities. Detailed evidence source:
`2026-08-19-editor-ui-workbench-reflection-generation-remote-exposure-architecture-review.md`.

## Requested Performance01 updates

### PERF-MVP-076 and PERF-MVP-099

Extend the full-reflection finding through the complete Workbench reflection product chain. Current
code builds shell/activity/menu models under shell and command locks, copies them into an owned
reflection model, registers every menu/activity route, then expands them again into an owned runtime
snapshot. Static activity actions are rebuilt per tab and cloned into the snapshot.

Required target: immutable `ReflectionGeneration` with stable slots, shared activity/action catalog,
menu generation and route handles. Normal changes publish sparse typed deltas. Stable/no-op events do
zero model/snapshot build, path/String allocation, route work and publication. Full wire snapshots are
limited to cold subscribe, explicit resync and bounded export outside locks.

### PERF-MVP-278

Add the Workbench producer-side duplicate ownership. `EditorUiReflectionAdapter` formats paths,
clones labels/actions/JSON properties and inserts a fresh `BTreeMap` after the Workbench reflection
model has already cloned the same logical content. The runtime interface snapshot DTO must remain a
wire/resync artifact, not the internal per-event representation.

Required counters: source-generation reads, nodes/actions/properties materialized, cloned/formatted
String bytes, JSON bytes, ordered-map inserts, full/delta publications and subscriber fanout bytes.

### PERF-MVP-572, PERF-MVP-297 and PERF-MVP-314

Route registration currently recreates owned bindings for lookup and again on miss, and all activity
actions are forced remotely callable. Raw viewport move/press/release/scroll/resize therefore enter
the generic serialized binding plane.

Required target: command/control definition generations compile stable route handles once. Local
invocation carries typed handles. Raw realtime input remains on the typed coalesced/edge-preserving
input plane and has no generic remote route. Stable reflection performs zero binding conversion,
native String parse/format, route lookup or registration.

### PERF-MVP-101

Extend the widget-reflector gate. Current code recursively materializes and sorts all rows without
depth/node/time/byte budgets, paging or cancellation. Keep it completely off the normal reflection
path and expose it only as a bounded, cancellable diagnostics request with typed truncation.

## Requested Optimize and owner updates

### Optimize08 and Editor08

Attach a P0 correction to the remote-automation architecture: `register_action_route()` and the
snapshot adapter currently force remote access for all routes. Introduce a separate versioned
`RemoteExposureCatalog` with default deny, explicit semantic command exposure, principal/capability
policy and payload/rate/concurrency budgets. Delete both blanket remote flags in one hard cut.

Invert `workbench_reflection_routes_mark_activity_actions_as_remotely_callable`: raw viewport input
and mutating domain actions are denied unless a semantic command is explicitly exposed. Route handles
are compiled once per owner generation and revoked atomically.

### EditorUI08

Own the immutable `ReflectionGeneration`, stable node slots and sparse-delta publisher. Workbench
layout/activity/menu projection and runtime UI reflection must share generation-owned data rather than
copying it through three models. Move wire serialization and subscriber fanout outside shell/command
locks. Replace recursive per-subtree activity vectors with one linear, budgeted traversal.

### Optimize49 and Editor01

Reflection publication consumes typed causal receipts. No-op events and repeated transient levels do
not produce listener/journal/UI work. Realtime input stays in its coalesced/lossless edge planes;
remote semantic automation produces one authorized command receipt rather than replaying pointer
streams through reflected actions.

## Requested protected index state

- `pending.md`: add or retain one concise row for `zircon_editor/src/ui/workbench/reflection/**` with
  27/27 files, 1,324 lines, current fingerprint `5b193744...`, and
  `static_complete / structural_cutover_required / dynamic_pending`.
- `review.md`: do not add the module. Require default-deny remote exposure, removal of generic remote
  viewport input, immutable reflection generations, stable zero-work, linear budgeted traversal,
  current-source Cargo, F4 product traces, WPR/ETW and package-power evidence.
- Keep both protected indexes module-level and concise; detailed findings remain in the companion
  architecture review.

## Acceptance handoff

| owner | required proof |
|---|---|
| Optimize08 + Editor08 | default remote callable count `=0`; explicit semantic exposure only; raw viewport remote routes `=0`; stable route work `=0`; owner revoke/stale/budget parity |
| EditorUI08 | stable generation build/publication/String/clone bytes `=0`; changed work proportional to affected nodes; full wire build only cold/resync/export outside locks |
| Optimize49 + Editor01 | repeated transient levels produce no audit/UI publication; levels coalesce, edges preserve order; one authorized semantic receipt per automation command |
| Performance01 | scale counters plus 31-run WPR/ETW, allocation, lock, wakeup, latency, RSS and package-power matrix on identical hardware/assets/settings; D/E/F artifacts |

RenderDoc remains conditional on changed UI geometry, clipping, resource bindings, draw order or
visible output. CPU reflection, route and authorization acceptance uses WPR/ETW and domain counters.
