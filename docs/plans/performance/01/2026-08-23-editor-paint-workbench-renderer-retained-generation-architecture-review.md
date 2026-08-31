---
title: Editor paint workbench renderer retained-generation architecture review
date: 2026-08-23
module: zircon_editor retained-host paint_workbench_renderer
priority: MVP-P0 basic editor window, dock, pane, menu and Welcome paint
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate invalidation, cached element lists and Project Browser retained item source
---

# Goal

Replace the immediate whole-workbench paint fan-out with generation-owned layout, item and command
ranges. Damage must select an owner before leaf preparation. Stable Welcome, chrome, dock, pane, menu
and overlay content must not rescan template models, rematerialize state or rebuild unchanged commands.

# Current-source coverage

`paint_workbench_renderer.rs` and `paint_workbench_renderer/**` were reconciled file by file against
the current worktree. This review supersedes the 2026-07-17 102-file snapshot for coverage purposes.

| owner | Rust files | current review evidence |
| --- | ---: | --- |
| root dispatcher, root frames, skeleton, style and text | 9/9 | current source reread; root paint-frame ownership review |
| docks, panes and dock-owned projectors | 24/24 | dock-pane review plus six child projector reviews |
| menu bar, popup, submenu, geometry and rows | 12/12 | menu visible-row/state-generation review |
| native panes, hierarchy, assets, viewport and scrollbar | 20/20 | native-pane metadata/scrollbar review |
| scene chrome, docks, floating, resize and overlays | 10/10 | scene-layer damage/paint-state review |
| Welcome root, main column, form, frame fallback and recent rows | 29/29 | current source reread in this review |
| total | **104/104** | **9,690 lines, 318,401 bytes after M1** |

Pre-M1 sorted normalized path, NUL, raw bytes, NUL SHA256 was
`968f2b6cc1fd31af36cbd8e8e5ca6339c5e1575f8ce884c0ec18c2943b4e6548` over 9,698 lines and
318,307 bytes. Post-M1 SHA256 is
`485bc035a2d0bbe64b84a855a6e3f8adfa8a455c39a35a2e5b5bc126db5e40b6`.

The review also traced the Welcome producer through `WelcomePaneSnapshot`, view data, runtime layout,
host-contract conversion, sparse dispatch patches, direct test fixtures, template paint/hit owners and
recent-project presentation. The M1 source-and-test scope before implementation is 33 files, 3,900
lines, 128,301 bytes, SHA256
`d220f4d1f23caad79a296415d96ef2b310fe845ff09dded9349658584030a999`.
Post-M1 it is 33 files, 3,971 lines, 131,283 bytes, SHA256
`2545cf713928b9ba1e6b28815e3848742a9a8cc4618e6e4063e836c3967539ed`.

# Foundations to retain

1. Root-frame selection is O(1) and consumes typed committed layout frames.
2. Dock, floating-window, hierarchy and menu work already contain useful leaf-level damage or visible-
   range gates from the recent M1 changes.
3. `ModelRc::iter` provides borrowed row access, sparse row patches retain unchanged row owners, and
   paint primitives reject disjoint damage before raster or command recording.
4. Production presentation uses command-only frames; CPU RGBA allocation is not the normal GPU paint
   path.
5. Welcome recent rows are bounded by their visible-row count and reject rows outside the exact clip.
6. Existing host-painter scopes distinguish root resolve, skeleton and scene work.

# Structural findings

## P0: root damage is only a primitive clip, not an owner dispatch plan

`draw_host_workbench_window` always prepares interaction state, resolves the root, enters the complete
skeleton and then enters the complete scene fan-out. Leaf primitives may reject pixels, but damage
does not select retained root/chrome/dock/pane/overlay command ranges before state and traversal work.
The separate paint-recording and paint-workbench reviews own the required `Full | Regions | Empty`
damage state and the typed `WorkbenchPaintPlan` hard cut.

## P0: stable scene content is rebuilt as one immediate command stream

The renderer has no generation-owned range table. Chrome, dock, pane, floating and overlay builders
rediscover state and emit new command structs in fixed order on every accepted recording. Local M1
gates reduce wasted leaves but cannot make unchanged ranges reusable. The scene-layer and root paint-
frame plans own the shared command arena, stable z/range table and exact dirty mask.

## P0: Welcome paint performs twelve independent fixed-control model lookups

The Welcome root resolves three controls, main-column frame resolution dynamically invokes the same
lookup for seven controls, and recent-project header/list resolve two more. Each lookup traverses
`welcome.nodes` from row zero, calls cloning `row_data`, compares a string control id and stops only
after finding a visible match. One paint is therefore up to twelve O(N) traversals and repeated DTO
copies for layout data that was already stable when the pane generation was published.

This is not a reason to add a paint-local HashMap. `to_host_contract_welcome_pane` already traverses
the same generation with borrowed `nodes.iter()` to patch the four interactive controls. M1 compiles
a typed pane-local `WelcomePaneLayoutData` in that existing traversal and publishes it beside the node
model. Paint translates the prepared optional frames to the body origin in O(1). Expected steady paint
node visits change from at most `12N` to `0`; generation visits remain `N`, not `2N`; only the first
valid frame for each of twelve recognized controls is copied once per generation.

## P0: Welcome still lacks subregion damage and retained text ownership

Even after M1, damage intersecting a small Welcome child enters root/main/recent layout and stable
labels/status/action text are measured or shaped again. M2 assigns typed child bounds and retained
command/text ranges to the Welcome generation. Damage selects only intersecting children; source,
layout, theme, text-preference and interaction generations independently invalidate their ranges.

## P1: theme and text snapshots are requested by many sibling leaves

The recent paint-theme M1 makes metric access a direct Copy and paint-text M1 captures run context
once, so this is no longer the former scaling/font-capture hotspot. The final paint context should
still publish one immutable theme/text snapshot for the complete accepted owner range instead of
using hidden thread-local reads at every leaf.

# Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp`
- `dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElements.h`

`SProjectBrowser::Construct` retains one `STileView` bound to `FilteredProjectItemsSource`
(`SProjectBrowser.cpp:367-393`). Discovery/filter changes rebuild the source and explicitly request a
list refresh (`809-919`); ordinary paint does not repeatedly find named layout children in the entire
project model. `FSlateInvalidationRoot::PaintInvalidationRoot` selects slow rebuild only when required,
uses a cached fast path otherwise, and does not repaint widgets when the retained update list is empty
(`SlateInvalidationRoot.cpp:356-424`). `FWidgetProxy` carries persistent index/state and pushes only
invalidated owners into update lists (`WidgetProxy.cpp:199-257`). `FSlateCachedElementData` retains
per-widget cached element lists and changed-list pointers (`DrawElements.h:219-230`).

The transferable rules are: compile stable identity/layout at generation change, retain direct item/
widget owners, and dispatch invalidated ranges. Zircon should keep Rust value ownership and its own
typed generation receipts rather than copy Unreal's pointer or threading implementation.

# Target architecture

1. Presentation publication compiles immutable root, layer, pane and Welcome layout artifacts with
   stable typed identities and exact source/layout/style/text/resource generations.
2. Redraw produces tri-state typed damage and a `WorkbenchPaintPlan` containing ordered owner/range
   selections; `Empty` never becomes a full repaint.
3. A per-window retained command arena maps owner generations to stable z ranges. Dirty ranges patch;
   unchanged ranges reuse commands and shared text/resource handles.
4. Pane and list generations own viewport/visible-row metadata. Paint consumes O(V) item ranges and
   never scans string control ids for geometry.
5. CPU raster, command recording and GPU replay consume the same owner/range plan, preserving pixel,
   text, clip and ordering parity without duplicated policy.
6. Background discovery, asset/plugin preparation and shaping may use the task system; window state
   publication and final command submission remain explicit bounded main-thread commits.

# Instrumentation and acceptance

Matrix: workbench/template nodes `1/100/10,000`, panes `1/16/128`, Welcome controls present/missing/
duplicate/invalid, recent rows `0/1/100/10,000`, damage `empty/outside/one-child/full`, generations
`stable/layout/text/theme/content`, backend `GPU/softbuffer/CPU snapshot`.

| evidence | acceptance |
| --- | --- |
| Welcome generation node visits and captured frames | exactly N borrowed visits; at most 12 frame copies |
| Welcome paint node visits / `row_data` clones | 0 / 0 after M1 |
| root/layer/pane owner visits | proportional to selected dirty owners, not total owners |
| command/text/resource builds and allocated bytes | zero for stable retained ranges |
| row visits | visible rows plus declared overscan only |
| CPU, allocation, RSS, input latency, context switches and package energy | same executable/workload before/after, median/p95/max reported |
| RenderDoc GPU/draw/batch plus screenshot/pixel/text parity | current-source launchable GPU backend only |

WPR/ETW, power and RenderDoc artifacts must stay on D/E/F. No dynamic claim is accepted from static
contracts. Current managed Cargo remains unavailable because validation Session
`019ffe1c-46d5-7933-97cb-65996b76f552` is archived, so executable/profile evidence is pending.

# Milestones

| milestone | work | gate |
| --- | --- | --- |
| M0 | Add owner/range, generation build/reuse, node visit/clone, command bytes and power attribution counters. | attributable baseline |
| M1 | Compile typed Welcome layout during the existing generation traversal; hard-cut paint model scans. | generation N, paint 0, focused RED-to-GREEN contract |
| M2 | Add Welcome child damage/ranges and retained text artifacts. | stable child rebuild/measure/shape = 0 |
| M3 | Publish root/scene/pane `WorkbenchPaintPlan` from tri-state damage. | empty/outside owner visits = 0 |
| M4 | Reuse per-window command ranges and shared resource/text handles. | rebuild proportional to dirty ranges |
| M5 | Hard-cut immediate whole-scene dispatch and duplicated CPU/recording policy. | one plan and one range authority |
| M6 | Run managed behavior/scale tests, WPR/power and RenderDoc/pixel parity. | quantified accepted milestone |

# M1 implementation result

`WelcomePaneData` now owns a typed pane-local `WelcomePaneLayoutData`: twelve optional fixed-control
frames plus an explicit node-presence bit. The projection rejects non-finite or sub-pixel frames and
keeps the first valid occurrence, preserving the former lookup semantics for invalid then valid
duplicates.

`welcome_nodes_with_native_dispatch` still performs exactly one borrowed `nodes.iter()` traversal.
That traversal now captures layout before applying sparse patches to the four interactive rows, so no
second generation scan or paint-local map was introduced. Both direct host test-fixture constructors
compile the same layout instead of relying on an empty compatibility fallback.

The Welcome root, main-column sequence and recent-project header/list now translate typed prepared
frames in O(1). The old string-parameter lookup function and all paint reads of
`welcome.nodes.{row_count,row_data}` were hard-cut.

| static work | pre-M1 | post-M1 | change |
| --- | ---: | ---: | ---: |
| fixed-control lookup traversals per Welcome paint | 12 | 0 | eliminated |
| worst-case template-node visits per Welcome paint | 12N | 0 | eliminated |
| layout `row_data` DTO clones per Welcome paint | up to 12N | 0 | eliminated |
| added generation traversals | 0 | 0 | capture fused into existing N traversal |
| prepared frame copies per generation | 0 | at most 12 | bounded |

Focused source contracts moved RED 3/3 to GREEN 3/3; the existing sparse-dispatch contract remained
GREEN 1/1. `rustfmt --check` and scoped `git diff --check` pass. The complete discovered performance-
contract suite is 115/120: the five unchanged failures are two missing test-owner files, missing
`available_slots`, and two existing `.roots.clone()` violations. No new broad failure was introduced.

Changed implementation/test scope: 15 files, 2,534 lines, 84,578 bytes, SHA256
`c18987a639f32c31161c17a45b786b8ec834698a0a9fc9297b4ab83d11b7aa53`.
This fingerprint includes the two direct fixture owners and the new Python source contract.

# Current disposition

All 104 Rust files are source-reviewed and M1 is statically implemented, but the module remains
dynamically unaccepted. Managed Cargo cannot run under the archived validation Session, and no
current-source executable exists for WPR, power or RenderDoc capture. M2-M6 must not be replaced by
micro-optimizations or false validation. No commit or WeCom milestone notification is permitted until
current-source dynamic acceptance succeeds.
