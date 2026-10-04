---
title: Editor native pointer routing generation and hit-path performance review
date: 2026-08-23
module: zircon_editor retained-host native_pointer routing
priority: MVP-P0 editor input correctness, route scale and allocation
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine Slate window routing and FHittestGrid
---

# Goal

Resolve every pointer fact against one coherent structure/interaction generation and one retained
hit path. Candidate traversal must borrow rows, topmost ordering must match paint/window ordering,
and route cost must be independent of unrelated pane/node counts.

## Reviewed source

- pre-M0 owner Rust files: 53/53
- pre-M0 lines: 1,958
- pre-M0 bytes: 61,630
- pre-M0 source-only SHA256 over lexicographically sorted owner files:
  `59aabc3259ea25b6f98e9e92d092796a0683ab04aa83fc99d8f4046c95e045aa`
- post-M0 owner Rust files: 53/53
- post-M0 lines: 2,075
- post-M0 bytes: 65,014
- post-M0 source-only SHA256 over lexicographically sorted owner files:
  `f8ad87d6022018fe8c7d2e7b347c2e538fc9d598ecc78075a60ddb44ac1d7a00`
- owning commit at review: `08094b9b9e17f6c80372e15c17b01204038b305b`

Owner scope is `native_pointer/routing.rs + routing/**`. All files were read in full. Direct callers
in button/move/scroll dispatch, generation/state publication, Console hit testing/paint metadata,
Workbench hit index, runtime surface hit grid and `ModelRc` storage/iteration were traced.

The 2026-07-17 report is materially stale: the owner grew from 48 files/1,166 lines to 53 files/
1,958 lines, asset content/reference/tree routing now uses retained `AssetContentPaintMetadata`, and
Workbench routes use a generation-owned hit index. Those improvements are retained; remaining
`row_data` and String findings are current-source observations.

## Correct foundations to retain

1. Top-level chrome priority is explicit, and floating pane content prevents fall-through into
   covered local docks.
2. Workbench template hits use `HostWorkbenchHitIndex` instead of rebuilding/scanning the complete
   Workbench model.
3. Asset content/reference/tree panel frames come from retained paint metadata; routing no longer
   scans wide asset template nodes to rediscover known panels.
4. Viewport toolbar hit testing uses the runtime `UiSurfaceFrame` hit grid.
5. Console output viewport metadata clips scroll routing, and generic template hit testing has a
   retained pane index.
6. `ModelRc::iter/get` supports borrowed access across contiguous, shared-row and overlay storage.

## Structural findings

### P0: split-generation routing reads reset interaction state

`HostState::replace_host_presentation` removes interaction data from the structural presentation
and publishes it through a separate retained `Arc<HostPaneInteractionStateData>`. Button and move
route callers pass `generation.structure()`, but `route_pointer_to_pane_with_mode` reads
`presentation.pane_interaction_state.console_scroll_px`. That field is the reset structural copy,
normally 0, so Console click/move template hits can disagree with scrolled paint geometry.

M0 changes pane routing to accept structure and interaction explicitly. It does not materialize a
combined presentation and does not reacquire another generation.

### P0: six candidate loops deep-clone rows

Floating chrome, floating panes, activity rails, document tabs, drawer tabs and host-page tabs use
`ModelRc::row_data`. `row_data` is `get(row).cloned()`, and these DTOs carry Strings/models/panes.
Every rejected candidate therefore clones before containment testing. M0 replaces all six with
borrowed `iter/enumerate/rev` traversal and clones only the selected route identity that still owns
String data.

### P0: floating chrome and pane paths disagree on z order

Floating pane routing visits rows in reverse, while floating header routing visits forward. For
overlapping floating windows they can target different windows. Unreal locates child/top-level
windows from the last entry toward the first; Zircon's pane route already encodes the same topmost
rule. M0 makes header routing reverse too and adds an overlap regression test.

### P0: one button fact can build independent Workbench and pane routes

Button dispatch first asks the Workbench hit index and, after a miss, starts pane routing. Popup,
chrome and menu use additional geometry routes. These are separate ownership domains rather than
one retained path/reply, so route work and priority knowledge are distributed across dispatch.

M1 publishes one `HostPointerHitPath` per generation with overlay/window/pane/control ancestry and
lets dispatch bubble until handled.

### P1: closed route identities allocate Strings

`ChromePointerRoute` and `PanePointerTarget` own `String` values for left/right/document/activity/
browser/references/used_by, surface keys and control ids. `SharedString` is currently an alias for
`String`, so each `.into()`/`.clone()` copies bytes. M2 introduces typed side/surface/list enums and
stable ids; editable labels remain owned text. A permanent global interner is not acceptable.

### P1: remaining candidate cost scales with unrelated siblings

Borrowed loops remove clone cost but floating windows, tabs and rail buttons remain O(candidate
count). M2 extends generation-owned hit metadata to chrome/panes using cell or interval indexes,
then proves visited candidates are bounded independently of 10K/1M unrelated nodes.

### P1: pane kind/mode policy is distributed string matching

`PaneRouteMode`, target mapping, asset routing and viewport routing repeatedly match pane-kind
Strings. This obscures which surfaces permit template hits and makes new panes easy to route
inconsistently. M3 hard-cuts to a typed pane route policy produced during projection.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp`
  - `LocateWindowUnderMouse` first accepts the OS/native window, otherwise visits windows and child
    windows from `Num()-1` to 0 and returns one `FWidgetPath`.
  - button and wheel routes reuse that path rather than separately searching feature domains.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp`
  - `FHittestGrid::GetBubblePath` maps the point to one 128x128 cell, finds the frontmost candidate,
    and reconstructs the parent path.
  - `GetHitIndexFromCellIndex` visits cell candidates back-to-front with clip and transformed
    geometry checks; it does not scan all widgets.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/HittestGrid.h`
  - hit testing returns a bubble path and supports custom paths, retaining ownership/ancestry.

The relevant standard is one coherent topmost path backed by paint-time hit data. Zircon's existing
Workbench/runtime-surface indexes are useful foundations but must converge into one generation.

## Target architecture

1. Capture `{ structure, interaction, hit-index }` from one generation and never read reset fields
   from the structure owner.
2. Query one retained topmost hit path with overlay/window/pane/control ancestry.
3. Store closed route categories as enums/stable ids and borrow immutable route payloads.
4. Bubble/tunnel one typed reply; do not rediscover Workbench/pane/chrome paths.
5. Update spatial indexes only for changed projected rows/geometry.
6. Make route counters report cells, candidates, path depth, clones, String bytes and generation id.

## Instrumentation and acceptance

Matrix: event `move/scroll/press/release`; windows/tabs/rails/nodes `0/1/100/10K/1M`; overlap
`none/2/100`; Console scroll `0/18/max`; target `popup/chrome/workbench/pane/viewport/none`; scale
`1x/1.5x/2x/4K`.

Acceptance requires:

- Console click/move hit ids and frames equal painted ids/frames at every scroll offset;
- rejected-candidate DTO clones = 0 after M0;
- topmost floating header and pane identify the same window;
- final route builds = 1 per uncaptured fact;
- steady closed-id String allocations = 0;
- final visited candidates are bounded by addressed cells/path depth, not total nodes;
- p95 route time below 0.10 ms at 10K nodes and below 0.20 ms at 1M nodes on the recorded host.

WPR owns CPU/allocation/power evidence. RenderDoc is used only after current-source launch to prove
the route/index changes preserve draw/scissor/pixel results. All artifacts remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Pass split interaction explicitly; borrow six row loops; reverse floating header z order. | applied; focused RED 0/4 to GREEN 4/4 plus Rust regression source |
| M1 | Publish one generation-owned `HostPointerHitPath`; route all pointer facts once. | one path/fact and exact ancestry |
| M2 | Add chrome/pane spatial indexes and typed route ids. | bounded candidates and zero closed-id allocation |
| M3 | Hard-cut string pane policy and independent feature-route APIs. | one routing authority, no compatibility shims |
| M4 | Run scale/WPR/power and RenderDoc parity matrices. | quantified acceptance and closeout |

## Validation state

- Owner source review: passed, 53/53 current Rust files.
- Direct dispatch callers, generation/interaction state, Console hit/paint metadata, Workbench and
  runtime hit indexes, and ModelRc access: read and mapped.
- Unreal window/path/grid sources: read and mapped.
- M0 implementation: applied. Pane routing now receives split interaction state from the captured
  generation; all six candidate loops borrow rows; floating header/pane traversal is topmost-first.
- Focused static contract:
  `tools/tests/test_editor_native_pointer_routing_generation_performance_contract.py`, 54 lines,
  2,177 bytes, SHA256
  `bda6b2db2de48db6dcf29c1442364d4453a62908b6298e553c6a26abe50e7d84`; RED 0/4, GREEN 4/4.
- Adjacent button/move/scroll/drag/damage/hit-route/overflow/workbench/viewport contracts: GREEN
  33/33. Scoped `git diff --check` passed and routing-owner `row_data(` search returned zero.
- Rust regression sources cover Console click at `console_scroll_px=18` and overlapping floating
  header topmost selection. They are formatted but not claimed passing until managed Cargo runs.
- Managed Rust tests, M1-M4, current-source launch, WPR and RenderDoc remain pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; raw Cargo is not an
  allowed bypass.
- This owner remains in `pending.md` until M0-M4 pass on one source/executable fingerprint.
