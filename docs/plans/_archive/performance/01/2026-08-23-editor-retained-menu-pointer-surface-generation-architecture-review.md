---
title: Editor retained menu pointer surface generation performance review
date: 2026-08-23
module: zircon_editor retained-host menu_pointer
priority: MVP-P0 editor menu input, retained hit routing and stable recompute cost
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine Slate FMenuStack and FHittestGrid
---

# Goal

Make the retained menu pointer bridge consume one immutable menu/layout generation and one retained
open-menu hit authority. Unrelated host recomputes and stable pointer facts must not deep-copy the
menu tree, republish identical interaction state, rebuild popup path indexes, or discard the whole
runtime UI surface.

## Reviewed source

- pre-M0 owner Rust files: 28/28
- pre-M0 physical lines: 1,841
- pre-M0 bytes: 66,165
- pre-M0 path-and-file-SHA manifest SHA256:
  `33b57b0f3443c6391f3aee63f51d5cedec97ff6526394add952d2f1afddcba00`
- post-M0 owner Rust files: 28/28
- post-M0 physical lines: 1,875
- post-M0 bytes: 67,070
- post-M0 LF path-tab-file-SHA manifest SHA256:
  `fced1db7c4e6ece70c4c4f1a387bb77c5919a327279f86b1e545522a28d49853`
- owning commit at review: `08094b9b9e17f6c80372e15c17b01204038b305b`

Owner scope is `retained_host/menu_pointer/**`; all 28 current Rust files were read in full. Direct
callers in host recompute, committed pointer callbacks, shared menu action dispatch, host menu-state
publication and retained-menu tests were traced. Runtime `UiSurface::rebuild`, `rebuild_dirty`, tree
mutation tracking, incremental arranged-tree/hit-grid patching and pointer dispatcher registration
were also read because the editor bridge must not duplicate or bypass those owners.

This report supersedes the 2026-07-17 static report for the current 28-file owner and reconciles the
2026-08-22 shared menu-layout-generation and visible-row reports. It does not create a second menu
generation or duplicate their command compilation and paint owners.

## Correct foundations to retain

1. Idle ticks do not rebuild this owner: `recompute_if_dirty` returns before pointer-surface sync
   when no invalidation is pending.
2. Stable pointer move does not unconditionally rebuild `UiSurface`; topology changes are limited
   to open/close, top-level switching, submenu-path changes and menu-bar geometry scrolling.
3. Popup row hit projection is arithmetic and follows only the open submenu path, O(depth), rather
   than inserting one hit node or scanning every menu row per event.
4. Root popup scroll changes projection coordinates without rebuilding the popup surface frame.
5. `UiSurface` already owns dirty generations, node-local layout, arranged-tree, hit-grid and render
   patch paths. The missing work is adoption and topology mutation support, not another editor cache.

## Structural findings

### P0: stable host recompute builds before equality and deep-copies shared layout

Every slow host recompute calls `build_host_menu_pointer_layout` before the bridge equality guard.
It projects the chrome asset, measures top-level and popup text, recursively allocates a second
`MenuItemSpec` tree, clones action/preset strings and builds frame/width vectors. The host stores the
result, then passes `self.menu_pointer_layout.clone()` plus `self.menu_pointer_state.clone()` into the
bridge. `HostMenuPointerLayout::clone` deep-copies all vectors, strings and the complete menu tree.

M0 makes host and bridge retain the same `Arc<HostMenuPointerLayout>`, canonicalizes an equal
candidate to the existing Arc, and adds a borrowed-state shared sync entry. This removes duplicate
layout ownership and stable sync state cloning. It does not claim to solve build-before-equality;
M1 consumes the immutable menu/layout generation already required by the 2026-08-22 plan.

### P0: popup item/index cache is rebuilt for geometry-only layout changes

When any layout field changes, `sync` calls `refresh_popup_items(true)`. An open menu then deep-clones
its complete item tree into `popup_items` and rebuilds `HashMap<Vec<usize>, usize>`, even when only
shell size, button frames or popup widths changed. M0 compares the borrowed selected tree before
`into_owned` and path indexing, so geometry-only changes retain the item/index cache.

M1 splits structure, command-context and geometry generations. Menu structure changes rebuild only
the selected menu's route artifact; geometry changes only update frames; enabled/checked context
patches only the affected rows.

### P0: topology changes throw away runtime incremental state

`rebuild_surface` constructs a fresh `UiSurface`, `UiPointerDispatcher` and `EditorRouteIntentMap`,
formats every button/popup path, registers three handlers per node, runs full `surface.rebuild`, and
replaces all three owners. Opening one submenu therefore discards unchanged root/buttons/ancestor
popup nodes and bypasses runtime `rebuild_dirty`, incremental arranged-tree geometry patching,
incremental hit-grid patching and render-cache reuse.

M2 adds runtime-owned subtree insert/remove and handler/route unregistration transactions, then
keeps one surface generation and replaces only the changed open-stack suffix. Implementing ad hoc
node removal in this editor bridge is rejected because current `UiTree` and dispatcher APIs do not
offer one coherent topology transaction.

### P0: stable pointer facts still clone and republish owned interaction payloads

Each move/scroll/click returns a cloned `HostMenuPointerState` containing two path vectors. The host
always assigns it, clones both paths again into `HostMenuStateData`, borrows global state mutably and
performs an equality check even when hover/path/scroll did not change. Full pointer-layout sync also
publishes menu state unconditionally after a rejected bridge sync.

M0 compares the returned state before assignment/publication and publishes after layout sync only
when bridge state/layout actually changed. M1 returns a typed menu interaction receipt with changed
domains/generation so callers do not need an owned full-state reply on stable facts.

### P1: route identity allocates by path and String

Popup projection creates a path vector, hashes `Vec<usize>` in `popup_route_indices`, clones the path
into an internal route and clones the action String before converting to the public route. This is
O(depth), not O(total rows), but stable movement over a leaf still allocates owned route payloads.
M1 uses stable menu item ids/indices from the shared generation and returns path slices/compact stack
coordinates internally; owned command text exists only at the external command boundary.

### P1: surface rebuild cost is not exposed at the bridge boundary

Runtime records arranged, hit-grid, render, pool, text-cache and elapsed rebuild fields, but this
bridge exposes only a test-only surface generation/node count. M1 publishes layout build/reject,
popup item/index, surface suffix and stable-state suppression counters alongside the runtime rebuild
report so WPR samples can be attributed to exact menu generations.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/MenuStack.h`
  - `FMenuStack` retains `Stack` and `CachedContentMap` as the identity of the open menu chain.
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/MenuStack.cpp`
  - `PrePush` performs prepass, desired-size and fitted placement at the push boundary.
  - `PostPush` inserts after the parent and dismisses/removes only descendants after that point,
    updating the retained content map instead of reconstructing the complete menu stack.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/HittestGrid.h`
  - `FHittestGrid` retains a widget map, widget records and spatial cells with explicit add/remove.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp`
  - `AddWidget` finds an existing widget; when occupied cells are unchanged it updates ordering/user
    metadata only, and removes/reinserts only when coverage changes.

The transferable rule is persistent menu identity plus owner-level incremental hit-index mutation.
Zircon should use its existing `UiSurface` invalidation machinery, not port Slate widget objects or
place menu work on worker threads; native UI state and hit routing remain UI-thread-owned.

## Target architecture

1. Publish one immutable `{menu_structure, command_context, geometry}` generation shared by chrome,
   paint, native geometry and retained pointer routing.
2. Host and bridge retain the same Arc receipt; unrelated invalidations perform O(1) identity checks
   and zero menu projection/text measurement/tree allocation.
3. Retain one open-menu stack artifact containing stable item ids, popup frames, visible ranges,
   route indices and the longest-common-prefix update record.
4. Extend runtime UI with an atomic topology transaction that inserts/removes a subtree and updates
   dispatcher, route map, arranged tree, hit grid and render cache under one generation.
5. Opening/switching/closing a submenu mutates only the changed suffix; geometry-only updates patch
   changed frames; stable events query the retained topmost stack without owned payloads.
6. Return typed interaction receipts with state generation, changed domains, action id and exact
   redraw/damage need; host publication happens only when the receipt changed.

## Instrumentation and acceptance

Matrix: menu nodes `1/100/1K/10K`; top menus `1/7/100`; depth `0/1/10/100`; presets
`0/1/100/10K`; event `stable move/scroll/open/switch/submenu/close`; invalidation
`unrelated/context/geometry/structure`; scale `1x/1.5x/2x/4K`; backend `GPU/softbuffer`.

Acceptance requires:

- unchanged unrelated recompute: zero layout builds, chrome projections, text measures, item/action
  clones, popup reindexes and host menu-state publications;
- host and bridge layout receipt pointer identity is equal after sync;
- geometry-only change: zero popup item clones and route-index rebuilds;
- stable move/scroll: zero surface rebuilds, path/String allocations, state DTO clones, generation
  advances and redraws;
- submenu suffix change: only changed levels/nodes/handlers/routes are inserted or removed;
- runtime reports no full arranged/hit/render rebuild when topology/geometry patch eligibility holds;
- p95 stable menu hit below 0.05 ms at 10K rows and p95 depth-10 suffix update below 0.10 ms on the
  recorded host, with no sustained idle wakeup attributable to menu routing;
- hit, paint and damage generations, scroll/clamp, disabled rows, z order, actions and pixels match.

WPR owns CPU, allocation, wakeup and package-energy evidence. RenderDoc is used only after a
current-source launchable GPU executable exists and only for draw/scissor/resource/pixel parity.
Artifacts and build targets remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Share layout Arc, borrow stable sync state, suppress unchanged publication and retain equal popup item/index cache. | applied; focused RED 0/5 to GREEN 5/5 |
| M1 | Consume immutable menu/layout generation and return typed interaction receipts/counters. | zero stable build/clone/publication counters |
| M2 | Add runtime atomic topology transaction and update only open-stack suffix. | incremental arranged/hit/render reports |
| M3 | Run scale/WPR/power and RenderDoc parity matrices. | quantified acceptance and closeout |

## Validation state

- Owner review: complete, 28/28 current Rust files.
- Host recompute/event/global-state callers and runtime surface/invalidation dependencies: read and
  mapped.
- Unreal menu-stack and hit-grid implementations: read and mapped.
- M0 implementation: applied. Host and bridge now retain the same canonical layout Arc; product
  sync borrows menu state; stable callback state and rejected full-sync state are not republished;
  selected popup items are compared before ownership/index replacement.
- Static code-path delta: product sync deep layout clones `1 -> 0`, product sync state clones
  `1 -> 0`, stable callback host menu-state publication attempts `1 -> 0` per fact, and
  geometry-only equal popup tree ownership/index rebuilds `1 -> 0`. These are source-path counts,
  not elapsed-time or allocation-profiler claims.
- Focused static contract:
  `tools/tests/test_editor_retained_menu_pointer_surface_generation_performance_contract.py`, 80
  lines, 3,363 bytes, SHA256
  `7c7f381371b077a35f0b58280b60ba022880a024faea18690f5243f422ef8d6a`; RED 0/5 to GREEN 5/5.
- Adjacent menu geometry, visible-row, context-menu and host interaction contracts: GREEN 19/19.
  Rustfmt and scoped `git diff --check` passed.
- Broad performance-contract discovery: 168/173 passed. The five failures are existing external
  drift: two removed test-fixture paths, missing `available_slots`, preview resize
  `.roots.clone()` and UI asset root helper `.roots.clone()`. No menu-pointer contract regressed.
- Owner size increased by 34 physical lines and 905 bytes for explicit Arc/shared-sync and
  content-equality gates. The unchanged candidate layout is still built before canonicalization;
  that P0 is deliberately owned by M1 rather than hidden by this M0.
- Managed Rust tests and M1-M3 dynamic evidence: pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; raw Cargo is not an
  allowed bypass.
- This owner remains in `pending.md` until all milestones pass on one source/executable fingerprint.
