---
title: Editor retained activity rail stable tab route performance review
date: 2026-08-23
module: zircon_editor retained-host activity_rail_pointer
priority: MVP-P0 editor drawer navigation input
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine FTabManager and SDockingTabStack
---

# Goal

Route an activity-rail click through one explicit coordinate space and one compact retained tab
identity. Stable clicks must not repeat hit dispatch or clone drawer/tab Strings merely to rediscover
payload already owned by the immutable pointer layout.

## Reviewed source

- pre-M0 owner Rust files: 23/23
- pre-M0 physical lines: 550
- pre-M0 bytes: 20,142
- pre-M0 LF path-tab-file-SHA manifest SHA256:
  `db17bbf99abeaf718d4b46ebc5e4d96ead1e65da7f2507061358a4d24a0d859b`
- post-M0 owner Rust files: 23/23
- post-M0 physical lines: 527
- post-M0 bytes: 21,232
- post-M0 LF path-tab-file-SHA manifest SHA256:
  `cdd714a5ee1760bf64ce701009702c111380c653b461d3cc0f4d3e209052f814`
- current post-drawer-command owner Rust files: 22/22
- current post-drawer-command physical lines: 564
- current post-drawer-command bytes: 20,753
- current post-drawer-command LF path-tab-file-SHA manifest SHA256:
  `180badbdaf06e333ef1702ce6e99fff87856e21caf731530b10ebf847553db75`
- owning commit at review: `99bc83322336d234a6177a44cf104ef5b5d1007a`

All owner files were read in full. The review also traced host recompute, pointer-layout sync, native
button dispatch, host callback, shared drawer-toggle dispatch, Workbench model/tool-window build,
typed drawer and view-instance identities, route-intent lookup and all five retained activity-rail
test files. The July 17 report was reread and remains directionally correct, but it did not identify
the coordinate fallback or per-route payload duplication.

## Correct foundations to retain

1. Pointer dispatch itself runs against a retained surface and stable node ids.
2. Layout equality prevents a full surface rebuild after an equal layout reaches `sync`.
3. Shared workbench template frames, rather than stale legacy shell geometry, own rail placement.
4. Drawer activation ultimately goes through the typed layout command/runtime transaction path.

## Structural findings

### P0: one native local click can execute two complete hit dispatches

The native route explicitly passes rail-local coordinates. `handle_click` first translates them to
global coordinates and dispatches. If the result is not a button, it dispatches the original point
again as if the caller might have supplied global coordinates. A miss or strip hit therefore pays
two surface hit/path/handler/typed-route lookups. The fallback exists only to satisfy a test that
calls the same ambiguous API with global coordinates.

M0 freezes the product API as local coordinates and performs one dispatch. A separately named test
and diagnostic API accepts global coordinates explicitly. M1 carries the native hit receipt through
the callback and removes this editor mirror dispatch entirely.

M0 applied: the product entry translates the documented local point once and executes exactly one
dispatch. The global-point entry is test-only and explicitly named; the speculative fallback and
its second event construction/dispatch are gone.

### P0: each button route duplicates tab Strings three times

Layout projection allocates `slot: String` and clones `instance_id` for every tab. Surface rebuild
clones both into `HostActivityRailPointerRoute`; route-intent lookup clones the route again on every
click. The route already has stable `side + item_index`, so these payload copies are redundant.

M0 stores typed `ActivityDrawerSlot` and `ViewInstanceId` only in the layout. The retained route is
the Copy pair `(side, item_index)`; shared dispatch borrows the matching layout target and converts
to the external binding contract only at the command boundary. Per-tab layout projection drops one
slot String allocation, surface rebuild drops two String clones, and route lookup drops two String
clones per click.

M0 applied exactly this boundary. `EditorRouteIntentMap` now copies the compact activity route, and
the callback resolves `side + item_index` against the bridge-owned immutable layout before calling
the existing drawer command boundary.

### P0: every host recompute builds before equality rejection

`sync_recompute_viewport_and_pointer_layouts` always calls the activity layout builder. It walks four
drawer slots, allocates two Vecs, allocates a slot String and clones every view-instance String, then
`sync` deep-compares the completed layout. `WorkbenchViewModel` has no topology generation for tool
windows, so a local cache key cannot be proved fresh.

M0 adds projection visit/build and sync reuse/rebuild counters but does not invent a weak cache.
M1 publishes one immutable activity projection under the authoritative workbench layout generation;
layout, paint and pointer consumers share it.

M0 counters now publish `ui.activity_rail.projection_batch_count`,
`ui.activity_rail.projection_visit_count`, `ui.activity_rail.sync_reuse_count`,
`ui.activity_rail.sync_rebuild_count`, `ui.activity_rail.surface_rebuild_count` and
`ui.activity_rail.surface_rebuild_button_count` under the `editor` stream.

### P1: any layout delta rebuilds all input topology

A frame or tab change creates a new `UiSurface`, root, both strips, every button path, dispatcher and
route map, then calls full `surface.rebuild`. Index-derived node ids also make insertion/reorder look
like identity replacement. M2 assigns stable tab handles and applies atomic changed-node topology
updates through runtime incremental hit/layout support.

### P1: drawer toggle clones the complete workbench layout

After pointer resolution, `dispatch_builtin_host_drawer_toggle` calls `runtime.current_layout()`,
then dispatches a command that locks the authority again. The drawer-header M0 follow-on removed the
redundant template-binding scan, slot parse, active-drawer-map clone and selected-drawer clone for
both callers, and changed `target_for_button` to return typed identities directly. The complete
layout snapshot clone remains. M1 moves active/collapse/switch evaluation into one typed layout
transaction rather than making an owned read snapshot followed by a write command.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/LevelEditor.cpp`
  - the level editor retains one `FUICommandList` and one `FTabManager`; tab spawners and commands are
    registered against stable names instead of rebuilt per click.
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp`
  - `TryToggleSidebarTab` resolves the existing tab and asks its retained docking area to toggle the
    drawer; `TryInvokeTab` reuses the manager-owned tab identity.
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabStack.cpp`
  - `BringToFront` operates on the retained `SDockTab`; `OpenPersistentTab` changes the state of an
    existing persistent `FTabId` before adding a new one.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp`
  - the retained hit grid resolves one widget path and updates unchanged widget coverage in place.

The transferable rule is persistent tab identity and an explicit event coordinate/path contract,
not repeated String payload projection or speculative second hit tests.

## Target architecture

1. Workbench authority publishes a generation-owned immutable activity projection containing typed
   slot/tab ids and committed frames.
2. Paint, native hit routing and command dispatch consume that same projection handle.
3. Native input carries a stable hit/tab receipt; the host does not repeat hit testing.
4. Drawer toggle is one typed read/modify/write layout transaction with a generation receipt.
5. Tab add/remove/reorder/frame updates patch only changed stable nodes.

## Instrumentation and acceptance

Matrix: tabs per side `0/1/16/100/1K`; event `button/strip/miss`; coordinates `local/global`;
topology `stable/add/remove/reorder/frame-only`; drawer state `collapsed/pinned/autohide`; click rate
`10/125/500 Hz`.

Acceptance requires:

- product hit dispatches per click `max 2 -> exactly 1` at M0 and `1 -> 0` mirror dispatches at M1;
- retained route payload String clones `2 per click -> 0`;
- per-tab surface rebuild route String clones `2 -> 0`;
- stable recompute projection Vec/String allocations and surface rebuilds = 0 after M1;
- tab reorder preserves stable identity and changed-node visits scale with the changed suffix;
- p95 click route below 0.03 ms at 100 tabs and below 0.10 ms at 1K tabs on the recorded host;
- WPR shows no repeated pointer rebuild/allocation wakeups and drawer behavior remains equivalent.

RenderDoc is irrelevant to this CPU/input owner except final product pixel parity. WPR, allocator,
build and capture artifacts stay on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Compact Copy route, typed layout payload, explicit coordinate APIs and counters. | focused RED/GREEN, Rustfmt, managed tests when available |
| M1 | Publish/share generation-owned activity projection and carry native hit receipt. | zero stable projection allocations and mirror hit tests |
| M2 | Stable tab node ids and atomic incremental topology; typed drawer transaction. | changed-node-only reports and no full layout clone |
| M3 | Scale/storm/WPR/power and interaction/pixel parity. | quantified acceptance and closeout |

## Validation state

- Owner review: complete, 23/23 current Rust files.
- Upstream model/recompute/native/callback/layout-command chain: read and mapped.
- Unreal tab manager/docking stack/command list and Slate hit-grid source: read and mapped.
- Focused static contract:
  `tools/tests/test_editor_retained_activity_rail_stable_route_performance_contract.py`, 43 lines,
  2,341 bytes, SHA256
  `e1b669dfbb1a6b167a98ad52ac8082f2c701c2b4e68039ce0de84c24d7c5aae8`.
- Focused RED/GREEN: 0/4 before M0, 4/4 after M0.
- All retained-host performance contracts: 17/17.
- Follow-on current fingerprint: 22/22 files, 564 lines, 20,753 bytes, manifest
  `180badbdaf06e333ef1702ce6e99fff87856e21caf731530b10ebf847553db75` after deleting the now-unused
  slot-to-string helper and returning typed drawer targets to the shared command boundary.
- Broad `test_*performance_contract.py` on the current shared worktree: GREEN 199/199.
- Direct `rustfmt --edition 2021` on all touched Rust files and scoped `git diff --check`: pass.
- Managed Cargo remains blocked by archived Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552`; raw Cargo is not an allowed bypass.
- No Rust compile/test, WPR, power or RenderDoc claim is made. M1-M3 and dynamic evidence remain
  pending; this owner stays in `pending.md`.
