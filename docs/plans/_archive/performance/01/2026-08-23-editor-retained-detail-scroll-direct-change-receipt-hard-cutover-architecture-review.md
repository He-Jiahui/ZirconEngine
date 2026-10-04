---
title: Editor retained detail scroll direct change receipt hard cutover performance review
date: 2026-08-23
module: zircon_editor retained-host detail_pointer and ScrollSurfaceHostState
priority: MVP-P0 Inspector Console and Asset Details scrolling
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine SScrollBox with Godot Range notification gate
---

# Goal

Make the committed native pane route plus scalar viewport/content layout the only detail-scroll input
authority. Console, Inspector and Asset Details must clamp one retained offset directly, return an
explicit changed receipt, and avoid UI property writes for zero, boundary, non-finite or out-of-
viewport input. A generic two-node runtime surface must not be reconstructed to implement one
vertical scalar range.

## Reviewed source

- owner Rust files: 23/23
- physical lines: 418
- bytes: 15,242
- LF owner-relative-path-tab-raw-file-SHA manifest SHA256:
  `f658ef22cb00fdb59fc193d168ac340e8fcc42d75e3030437d73f56726429419`
- owning commit at review: `cb62fe090eb917ebd59fc3aea5d3c01d52093782`
- post-M0 owner Rust files: 19/19
- post-M0 physical lines: 275
- post-M0 bytes: 9,872
- post-M0 LF owner-relative-path-tab-raw-file-SHA manifest SHA256:
  `1c721e2c32e33b4ef8c1fb8bce1e8ef6d6d7ea5aa7e2805ac2a0b467428faba0`

All owner files were read in full. The review also traced the three app scroll adapters,
`ScrollSurfaceHostState`, pointer-layout sync, startup construction, native pane scroll routing,
callback wiring, root/floating fallback tests, retained detail-pointer tests, route-intent ownership,
current performance-capture contracts and the 2026-07-17/2026-07-31 reports.

The prior reports correctly show that scroll no longer rebuilds the surface on every wheel event and
that fixed Asset Details extent arithmetic no longer allocates a temporary section vector. Those
improvements remain present. The remaining issue is the obsolete surface authority itself and the
lost changed/unchanged result above it.

## Structural findings

### P0: three generic surfaces implement one scalar vertical range

Each `ScrollSurfacePointerBridge` owns `UiSurface`, `UiPointerDispatcher` and
`EditorRouteIntentMap`. Construction formats root/viewport paths and builds a two-node scrollable
tree. Every layout or external state change rebuilds that tree, its scroll state and route binding.
The only semantic result consumed by app code is a clamped `f32` offset; the viewport route is
already bounded by scalar pane geometry and has no child controls.

Native pane routing has already identified Console, Inspector or Asset Details and supplies local
coordinates. M0 must replace the generic tree with direct viewport containment plus
`clamp(old + delta, 0, max)`. Layout sync remains O(1) and becomes allocation-free.

### P0: changed state is erased before the UI publication boundary

`handle_scroll` reads the runtime-mutated viewport offset, but `ScrollSurfaceHostState` converts the
dispatch into `Result<(), String>`. All three app adapters therefore call a Slint/global scroll
setter for every successful callback, including delta 0, clamped top/bottom overscroll, header hits
and unchanged/non-finite input. This creates avoidable property publication and potential redraw or
invalidation work at precisely the high-frequency wheel boundary.

M0 must return `changed: bool` from the owner that applies the clamp and call the pane setter only
when true. Focus-window behavior and a real size/layout sync remain separate required actions.

### P1: host and bridge duplicate scroll state

`ScrollSurfaceHostState` stores a `ScrollSurfacePointerState` while its bridge stores another copy.
Every sync clones state into the bridge and reads it back; every scroll copies dispatch state back
again. M0 must keep one state in the direct bridge and let host tail-follow policy read/update that
single owner.

### P1: constructor identity strings exist only for deleted mirror nodes

The three startup call sites pass `tree_id` and `path_prefix` strings solely to construct the generic
surface and formatted node paths. They do not identify paint, data or commands. M0 removes these
parameters rather than preserving compatibility shims.

## Zircon and reference-engine source basis

Direct Zircon source read:

- native `scroll_dispatch/pane/native/{browser,panels}.rs` invokes the specific pane callback only
  after `PanePointerRoute` has identified that committed pane.
- `ScrollSurfacePointerLayout` already contains the complete scalar authority: pane size, viewport
  origin and content extent.
- `ScrollSurfaceHostState` is the retained owner used by all three panes and is the correct place to
  preserve tail-follow policy and expose changed/unchanged publication.

Direct Unreal source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Layout/SScrollBox.cpp`
  `OnMouseWheel` calls `ScrollBy` and returns handled only from its boolean result.
- `SScrollBox::ScrollBy` saves `PreviousScrollOffset`, clamps the new offset to content bounds and
  returns whether the desired offset changed (unless configured to always consume wheel input).

Supplementary Godot source read:

- `dev/godot/scene/gui/range.cpp` `Range::set_value` compares previous and clamped values and emits
  the value-changed notification only when they differ.

The transferable rule is one retained scalar offset, one clamp owner and an explicit changed result
controlling downstream publication. A second generic hit tree is unjustified when no child target
or dynamic layout participates.

## Target architecture

1. `ScrollSurfacePointerBridge` owns only Copy layout and state.
2. `handle_scroll` validates finite point/delta and viewport containment, clamps once and returns a
   Copy dispatch with route, state and `changed`.
3. `ScrollSurfaceHostState` owns no duplicate state; tail-follow sync reads and writes bridge state.
4. Console, Inspector and Asset Details publish their UI scroll property only when `changed` is true.
5. The constructor has no tree/path identity arguments and no compatibility overload.
6. `EditorRouteIntent::Detail`, its lookup helper and all detail generic surface files are removed.

## Instrumentation and acceptance

Matrix: panes `console/inspector/asset-details`; viewport `0/1/96/4K`; content extent
`0/<viewport/=viewport/10K`; delta `0/+1/-1/+1M/-1M/NaN`; point `inside/header/outside/non-finite`;
window `root/floating`; input rate `10/125/500 Hz`; tail policy `at-tail/away/shrink/grow`.

Acceptance requires:

- generic mirror surface/dispatcher/route-map instances per detail owner: `1 -> 0` at M0;
- formatted tree/path strings and surface rebuilds per layout sync: `>0 -> 0` at M0;
- retained scroll-state copies per event: `>=1 -> 0` at M0;
- UI scroll property writes for unchanged input: `1 -> 0` at M0;
- changed input publishes exactly one offset and one local paint/damage decision;
- clamp, Asset Details header exclusion, root/floating size fallback and Console tail-follow parity;
- p95 callback-to-receipt below 0.005 ms with no allocator samples at 500 Hz;
- WPR shows no runtime surface dispatch/hash lookup/path allocation on detail scroll stacks.

RenderDoc is relevant only to final pixel/draw parity because M0 changes no paint geometry. WPR,
allocator, executable and capture artifacts must remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Direct scalar route/clamp, changed receipt, single state owner, conditional UI setter, delete generic mirror surface and route intent. | focused RED/GREEN, Rustfmt, managed tests when available |
| M1 | Carry exact local damage from changed scroll publication and prove unchanged input produces no invalidation. | setter/invalidation/damage counters |
| M2 | Recheck Console extent generation and Inspector/Asset Details content changes for generation-gated projection. | stable recompute allocation/counter gate |
| M3 | Run storm/WPR/allocator/power plus interaction and RenderDoc pixel/draw parity. | quantified acceptance and closeout |

## Validation state

- Owner review: complete, 23/23 Rust files.
- App/native callback, host state, startup, tests and route-intent chain: read and mapped.
- Unreal `SScrollBox` changed result and Godot `Range` notification gate: read and mapped.
- Architecture report: recorded before implementation.
- M0 implementation: applied. The bridge now owns Copy scalar layout/state only, direct scroll
  validates viewport containment and clamps once, and its receipt carries changed/unchanged. Host
  state no longer duplicates the offset. The three app adapters and stable layout-sync paths publish
  scroll properties only when the retained offset changes. Four generic surface/default support
  files plus the Detail route-intent branch are deleted; constructor identity strings are removed.
- Exact static owner delta: files `23 -> 19`, physical lines `418 -> 275` (-143, 34.2%), bytes
  `15,242 -> 9,872` (-5,370, 35.2%). Generic mirror dispatches per callback are `1 -> 0`, mirror
  surface rebuild/path allocations per layout change are `>0 -> 0`, retained state clones are
  `>=1 -> 0`, and unchanged scroll/property publications are `1 -> 0`. These are source/operation-
  count facts, not timing claims.
- Focused static contract:
  `tools/tests/test_editor_retained_detail_scroll_change_receipt_performance_contract.py`, 155
  lines, 6,211 bytes, SHA256
  `557b3c3dd8b4e4b2d1755837fa0c531568427c22aee1be8b0cf2fd9ce21540ea`; initial RED 0/9 to GREEN
  9/9, then stable-sync publication gate RED 9/10 to GREEN 10/10.
- Retained-host Python performance contracts: GREEN 59/59. Rustfmt and scoped `git diff --check`
  passed. Rust behavior tests were updated for header exclusion, zero delta and repeated clamped
  overscroll, but cannot execute until managed Cargo is available.
- Broad current-worktree performance discovery ran 235 tests: 234 passed and one failed in the
  separate active asset-browser preview-materialization contract because `ShellPresentation` still
  builds eager `asset_surface_presentation` models. It does not touch detail-scroll paths and is not
  counted as detail acceptance.
- M1-M3 and dynamic evidence remain pending; this owner stays in `pending.md`.
- Managed Cargo remains unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is archived and returns
  `cargo_session_not_executable`; raw Cargo is not an allowed bypass.
