---
title: Editor native pointer drag-session capture and damage performance review
date: 2026-08-23
module: zircon_editor retained-host native_pointer drag_resize and tab_drag_damage
priority: MVP-P0 editor docking, drag latency, resize latency and repaint scope
status: source_reviewed_m0_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate stateful drag operation and retained widget-path routing
---

# Goal

Make tab drag and drawer resize explicit retained input sessions. Static tab identity and source data
must be captured once, pointer motion must update compact scalar state without cloning owned strings,
target transitions must be change-proportional, and release must consume the committed route/layout
generation without rebuilding the complete workbench model. Damage must be derived from typed mutation
effects and remain multi-region until presenter policy deliberately promotes it.

## Reviewed source

- owner Rust files: 33/33
- lines: 648
- bytes: 23,391
- source-only SHA256 over lexicographically sorted owner files:
  `9a4cecd400799973ede5a265752a4ac8741d1bf67feaa33b6904596a0f8e81f1`
- owning commit at review: `7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`

| Owner group | Files | Lines | Bytes | SHA256 |
| --- | ---: | ---: | ---: | --- |
| `native_pointer/drag_resize.rs + drag_resize/**` | 21/21 | 436 | 15,383 | `4bdaeab12750c7c72f5302c5465c35e220a9ed7fed43c6e8104e2d99b132572b` |
| `native_pointer/tab_drag_damage.rs + tab_drag_damage/**` | 12/12 | 212 | 8,008 | `41fd4217e12451f226a88365d10506dada54ac6eea5375c88c9bdee25e93a2d3` |

All owner files were read in full. Direct boundaries read include `globals/ui_context.rs`, drag/resize
state DTOs, native press/move/release callers, workspace docking drag/drop and drawer-resize handlers,
the retained shell-pointer bridge and drag surface, redraw projection, `ModelRc`, and drag group-key
encoding. This record supersedes the matching 2026-07-17 native-pointer drag/resize review and links
the separate drag-overlay/world-surface generation review rather than duplicating that owner.

## Correct foundations to retain

1. Tab drag starts only after the four-pixel threshold and active move does not request repaint by
   itself. Pointer routing stays on committed bridge frames instead of rebuilding presentation in a
   native callback.
2. The shell-pointer surface retains its tree, dispatcher, route map and indexed hit geometry across
   stable topology. Geometry patching is change-gated, and hit routing uses the runtime grid.
3. Repeated target groups already avoid republishing state after comparison. Drawer resize writes a
   transient extent only when the preferred value changes and commits persistent layout on release.
4. The drag payload is resolved only while arming, not on every move. Release damage is isolated from
   callback execution, which is the correct boundary for later typed effect projection.

## Structural findings

### P0: high-frequency motion clones owned drag strings twice

`SharedString` is currently a type alias for `String`, not a shared reference-counted string. Each
active move first calls `get_drag_state()` in the native host, cloning five strings with the scalar
coordinates, then `sync_drag_target_group()` clones the same complete state again before comparing
one field. A stable non-empty drag can therefore perform up to ten String allocations/copies per
pointer sample, in addition to constructing an owned group key before discovering that it did not
change. At 125/500/1000 Hz this is input-frequency allocation work with no state transition.

M0 replaces full-state motion reads/writes with scalar pointer-session methods, rejects duplicate
coordinates, compares the typed route against the stored group without materializing a key, and
allocates a group string only on an actual target transition. M1 replaces the flat global DTO with a
typed `DragSessionGeneration` whose immutable payload and mutable latest-wins pointer/target patch
have separate ownership.

### P0: release rebuilds workbench authority instead of consuming the drag generation

Pointer up synchronizes the target and then `resolve_drag_drop_route_from_pointer()` performs another
route hit. Before the detach fast path can return, it clones current layout, builds chrome and command
evaluation context, locks commands and constructs a complete `WorkbenchViewModel`. This is one-shot,
not move-frequency work, but its cost scales with the entire workbench and is on pointer-to-drop
latency. It also permits route, layout and model facts from different generations.

M1 captures the committed route/layout generation at drag begin and updates only the typed target.
Release reuses the final hit and generation-owned drop index; generation drift causes one explicit
re-resolve or cancellation. Detach returns before layout/model/command work. M2 applies attach,
split, reorder, detach and drawer-mode changes as one typed atomic layout transaction.

### P0: drag payload and floating damage clone model rows

Arm-time payload lookup uses `row_data()` for root/drawer tabs and scans floating windows using cloned
window rows before cloning the selected tab. Only tab id/title/icon and source group survive into the
session. Release damage likewise clones each floating-window row merely to compare group keys and
read a fixed-size frame. M0 changes these paths to borrowed `ModelRc::get/iter` access and clones only
the strings that the session must own. Complexity remains O(W) for a floating lookup until M1 adds a
stable tab/group/window owner index.

### P0: release damage is a broad single bounding rectangle

Same-group release unions the target region with three status frames. Cross-group local drops union
the complete center band and status, while floating transfers may add a distant window. The API can
return only one `FrameRect`, so disjoint areas become bounding-space repaint and no effect reason says
whether layout, order, tab strip, content or status actually changed.

M2 consumes the typed layout transaction receipt, resolves affected retained owners and emits a
bounded `DamageRegionSet` with reason bits. The shared redraw/frame-paint plans own transport and
promotion; this module must not introduce another private region-set type.

### P1: resize remains raw-input-frequency mutation

Changed resize points still dispatch through the resize surface, update a transient map, publish
window-metrics invalidation and request redraw for every native sample. Existing coalescing prevents
immediate full recompute per callback, but input rate still governs state writes and callback work.
M0 rejects identical coordinates. M1 stores a latest-wins resize patch and consumes it at most once
per display frame while preserving ordered down/up/cancel and the final release point.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/DragAndDrop.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabWell.cpp`

Slate creates one stateful `FDragDropOperation`, stores it on the user, and carries that same shared
operation in every `FDragDropEvent`. Pointer move locates the retained widget path, diffs it against
the previous weak path, sends `OnDragLeave/Enter`, and bubbles `OnDragOver`; it does not reconstruct a
workbench/view model on each sample. `SDockingTabWell` retains the dragged tab and grab offset, updates
only the dragged offset during `OnDragOver`, and performs the final insertion in `OnDrop`.

The transferable invariants are one explicit drag-session lifetime, retained path/target transition
tracking, compact motion updates and a drop transaction at the owning dock. Zircon should keep its
runtime hit grid and Rust ownership model; it should not copy Slate widget classes or infer an Unreal
timing budget from source structure.

## Target architecture

1. EditorUI01 owns `DragSessionGeneration`: immutable tab/source/policy payload; ordered begin/drop/
   end/cancel receipts; latest pointer and typed target; committed route/layout generation id.
2. The shell-pointer generation returns typed owner ids and transition receipts. Stable move performs
   one indexed hit, zero String allocation and zero state publication when point/target are unchanged.
3. EditorUI08 owns an indexed docking generation and one atomic `ApplyTabDrop` transaction. Release
   never rebuilds chrome/context/workbench state for a stable generation.
4. Resize uses an analogous capture record plus latest-wins frame patch, consumed at display cadence.
5. Transaction effects project exact owner/range damage into the shared bounded region pipeline.

## Instrumentation and acceptance

Matrix: pointer `125/500/1000 Hz`; moves `1/1K/1M`; tabs/windows `1/10/100/1K/10K`; target `same/
local/document-edge/floating/floating-edge/detach/invalid`; resize `same/changed/latest-release`;
display `30/60/120 Hz`; backend `GPU/softbuffer/snapshot`; scale `1x/1.5x/2x/4K`.

| Evidence | Acceptance |
| --- | --- |
| drag state String allocations/bytes and full-state clones | stable move: zero |
| route hits and target publications | at most one hit/sample; publication only on transition |
| layout/chrome/context/model builds and command-lock time | stable release: zero rebuild; detach: zero |
| tab/window rows visited and cloned bytes | M0 zero row clone; M1 indexed O(1) lookup |
| resize samples, applied patches and recomputes | duplicate point: zero; changed apply at most once/frame |
| damage regions/useful/union/submitted area | exact affected owners; no implicit bounding promotion |
| CPU/allocation/RSS/input latency/context switches/power | same current-source executable before/after |

WPR owns CPU, allocation, scheduling, context-switch and power evidence. RenderDoc is used only after
a current-source GPU editor launches, for scissor/draw/resource/pixel parity. All artifacts remain on
D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Remove move-frequency full-state/string copies, duplicate-point work and model-row clones. | applied; static contract GREEN, managed Rust/dynamic pending |
| M1 | Introduce typed drag/resize session generations, indexed targets and final-hit reuse. | stable move allocations/publications zero; one hit/sample |
| M2 | Add atomic tab-drop transaction and exact reason-coded multi-region damage. | stable release rebuild zero; no single-rect default |
| M3 | Hard-cut flat string group/session authority and duplicate release resolver paths. | one session and one drop authority |
| M4 | Run scale/storm/WPR/power and RenderDoc parity matrices. | quantified acceptance and milestone closeout |

## M0 implementation result

The native context now exposes a compact `(active, x, y)` drag pointer snapshot plus scalar pointer,
activation and target-group mutation methods. Active move no longer clones or writes the complete
five-String state. `sync_drag_target_group` compares the typed route against the borrowed stored key
before materializing a String, then replaces only the target field on a transition. Duplicate drag
and resize coordinates return idle before callback/redraw work.

For stable local/document targets, source-structure work changes from two complete drag-state clones
plus one owned group-key construction, up to eleven non-empty String allocations/copies per move, to
zero String clones/allocations in these functions. Floating routes still clone one `MainPageId` when
the route-intent map returns its owned route; M1 must make that route identity borrowed/indexed before
claiming zero allocation across the complete target matrix.

Payload and floating damage lookup remove seven `row_data` calls across six source files. Arm now
borrows the selected tab/window and owns only id, title, icon and source group; the unused tab slot and
wide floating-window row are not cloned. Release damage borrows window rows while retaining current
O(W) lookup and broad single-rectangle semantics.

Post-M0 owner scope:

- Rust files: 33/33
- lines: 607
- bytes: 22,409
- source-only SHA256: `3af125afb9c7e0c743720d581d591e4f194a395c917aa6228c0cc5d5e8efd887`

| Owner group | Files | Lines | Bytes | SHA256 |
| --- | ---: | ---: | ---: | --- |
| `native_pointer/drag_resize.rs + drag_resize/**` | 21/21 | 398 | 14,525 | `be52d851ba71e64133e2a4ed0c22a86252fefdd660d09923a41c21398cef4def` |
| `native_pointer/tab_drag_damage.rs + tab_drag_damage/**` | 12/12 | 209 | 7,884 | `b3b2f1061d11776d32692fe7fd3b981e924cd5336504a7dea1775c3e5dbb17eb` |

Supporting changes are limited to scalar state access in `globals/ui_context.rs`, route matching in
`tab_drag/group.rs`, its export/import, and `app/workspace_docking/drag_drop.rs`. The typed-route Rust
regressions are present in `tab_drag/group.rs`. The focused static contract is
`tools/tests/test_editor_native_pointer_drag_session_performance_contract.py`, 82 lines, 3,491 bytes,
SHA256 `8c9c7f44d467a83827b8125f65c5cb1318486443b9845e06da8d103650d2403d`.

## Validation state

- Owner source review: passed, 33/33 current Rust files.
- Direct state, callback, retained hit-surface, drop resolver, resize and redraw boundaries: read.
- Unreal drag operation, application routing and docking tab-well sources: read and mapped.
- M0 focused static contract moved RED 0/4 to GREEN 4/4. The drag/resize/pointer/damage focused set is
  GREEN 25/25.
- Broad `test_*performance*.py` discovery is 151/157. The six failures are pre-existing external
  drift: two missing editor test files, one missing `available_slots` source anchor, two UI asset
  `.roots.clone()` findings and Runtime 07 documentation/source telemetry drift.
- Changed Rust files pass independent `rustfmt --check`; scoped `git diff --check` passes with only
  repository line-ending warnings.
- Typed route matching Rust regressions are written but are not claimed passing until managed Cargo
  is executable. Current-source launch, WPR and RenderDoc remain pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; raw Cargo is not an
  allowed bypass.
- This owner remains in `pending.md` until M0-M4 pass on one source/executable fingerprint.
