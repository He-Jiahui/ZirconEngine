---
title: Editor pointer redraw-result region promotion performance review
date: 2026-08-23
module: zircon_editor retained-host native_pointer redraw_result, resize_damage and template_hover_damage
priority: MVP-P0 editor pointer-to-present repaint scope
status: source_reviewed_no_local_change_dynamic_pending
reference_engine: Unreal Engine Slate retained path transitions and reason-coded invalidation
---

# Goal

Project pointer, hover, resize and tab-drop mutations into typed retained-owner effects and a bounded
multi-region damage result. Old/new hover rows, remote template/reference regions, pane-local changes
and layout changes must not be collapsed into a large bounding rectangle merely because the native
dispatch result has only one frame slot.

## Reviewed source

- owner Rust files: 14/14
- lines: 420
- bytes: 16,626
- source-only SHA256 over lexicographically sorted owner files:
  `43ab4b52499589279abefd6460ee0d642db65a792b182efdd2c2d71fa0fa1507`
- owning commit at review: `cb62fe090eb917ebd59fc3aea5d3c01d52093782`

Owner scope is `native_pointer/redraw_result.rs + redraw_result/**`, `resize_damage.rs` and
`template_hover_damage.rs`. All files were read in full. Direct callers, pane interaction state,
presentation generations, frame-geometry union/visibility, `NativePointerDispatchResult`,
`HostRedrawRequest`, redraw merge and presenter damage consumers were traced. The current source has
no local diff in this owner.

## Correct foundations to retain

1. Pane/workbench move callers retain before/after presentation generations and pass borrowed
   interaction state. They do not deep-clone the full presentation for redraw calculation.
2. Exact unchanged interaction state returns idle. Viewport movement can remain idle when no template
   or reference hover changed.
3. Hierarchy scroll and row-hover paths are distinguished; row damage uses direct row arithmetic and
   old/new indices rather than scanning hierarchy rows.
4. Redraw results distinguish paint-only region work from a region that also requires a frame update.
   This is the correct semantic boundary to carry typed reasons later.

## Structural findings

### P0: every disjoint change becomes one bounding rectangle

Template old/new hover, browser/activity reference hover, hierarchy row plus template damage, chrome
plus cleared text input, and resize/tab-drag extra damage all use `union_frame`. A switch between two
distant rows repaints the space between them. Independent panes or overlay/content regions can expand
to most of the window before redraw coalescing or the presenter sees the request.

The owner cannot fix this locally because `NativePointerDispatchResult` contains one
`HostRedrawRequest::Region { frame }`. M1 must introduce the shared bounded `DamageRegionSet`; the
result and redraw queue preserve separate regions/reasons until a measured backend promotion policy.

### P0: flat interaction equality replaces typed mutation receipts

`pointer_move_redraw` and workbench redraw first compare all 26 pane-interaction fields, including
four Strings, then infer changed pixels from a subset. When a non-template/non-reference pane state
changes, the generic fallback repaints the complete `pointer.frame`. This is fixed-size CPU work but
it hides mutation ownership and makes damage proportional to pane area instead of the changed row or
control.

M1 makes pointer handlers return a typed hover/scroll/control transition receipt with old/new stable
owner ids and reason bits. Redraw projection consumes that receipt directly; it does not rediscover
meaning by comparing a flat global state DTO.

### P0: resize has no old/new layout transaction damage receipt

`resize_damage_frame` clones the committed `host_layout.center_band_frame` captured before the resize
callback. The callback schedules window-metrics work, but the redraw result does not receive the new
drawer/center frames or identify which region changed. An absent/invalid frame promotes to full-frame.
The current broad center band may be conservative, but its completeness and amplification cannot be
proved from the effect.

M1 resize generation publishes the final scalar patch. M2 layout apply returns old/new affected owner
frames and a layout reason; exact regions are then generated after the new layout is committed.

### P1: damage geometry is duplicated and visibility is inconsistent

`template_hover_damage` carries private union helpers while other paths use shared frame geometry.
Some hover frames are unioned without finite/visible validation. This duplication is not a measured
runtime bottleneck, and moving it alone would not change repaint area. Delete it during the shared
damage-set hard cut instead of creating a cosmetic milestone.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`

Slate retains the previous and current widget paths and emits `OnDragLeave/Enter/Over` only along
path transitions. Widget invalidation carries explicit layout/paint/visibility/child-order reasons;
proxy update repaints flagged widgets and propagates layout work only where required. The transferable
invariant is mutation reason plus retained owner/path identity before spatial damage projection.
Zircon should not copy Slate proxy classes, and no Unreal timing value is inferred from source.

## Target architecture

1. Pointer handlers return typed transition receipts: old/new owner ids, paint/layout/order/status
   reasons, changed scalar fields and committed generation id.
2. `HostPresentationGeneration` resolves owner ids to exact frames/ranges without scanning or flat
   state comparison.
3. `NativePointerDispatchResult` carries `DamageRegionSet` plus frame-update/reason bits.
4. Redraw merge, event-loop state, retry, paint extraction and presenter preserve regions. Promotion
   is backend-owned, bounded and reports useful versus submitted area.
5. Resize/drop transaction receipts contain both old and new affected owner geometry.

## Instrumentation and acceptance

Matrix: hover distance `adjacent/opposite pane`; rows `1/100/10K`; simultaneous regions `1/2/8/64`;
action `hover/scroll/template/reference/resize/drop`; input `125/500/1000 Hz`; backend
`GPU/softbuffer/snapshot`; scale `1x/1.5x/2x/4K`.

| Evidence | Acceptance |
| --- | --- |
| transition receipts/reasons/owner ids | every redraw attributable; no flat-state inference fallback |
| input regions/useful/union/submitted area | disjoint regions preserved; amplification measured |
| full-frame and region promotion | explicit bounded backend reason only |
| interaction fields compared | O(changed receipt), not all 26 fields/sample |
| resize old/new owner frames | exact changed owners after committed layout |
| CPU p50/p95/p99, allocation, context switches and power | same executable/workload before and after |

WPR owns CPU/scheduling/power evidence. RenderDoc is required only for GPU scissor/draw/resource/pixel
parity after a current-source editor can launch. All artifacts remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add reason/owner/region/useful/union/submitted-area telemetry and scale workloads. | attributable baseline |
| M1 | Emit typed pointer/hover transition receipts and retained owner-frame lookup. | no flat-state damage inference |
| M2 | Carry shared bounded region sets through dispatch/redraw/presenter; add layout receipts. | no default bounding union/full promotion |
| M3 | Delete single-frame result APIs and duplicate union helpers after consumers migrate. | one damage authority |
| M4 | Run WPR/power and RenderDoc parity matrices. | quantified acceptance and closeout |

## Validation state

- Owner source review: passed, 14/14 current Rust files.
- Direct move/clear/capture callers, state/generation, redraw merge and presenter boundaries: traced.
- Unreal path-transition and invalidation sources: read and mapped.
- No Rust change was made because exact local regions cannot be represented by the current result/
  redraw contract; a leaf-only change would risk under-damage.
- Managed Rust tests, M0-M4, current-source launch, WPR and RenderDoc remain pending. Managed Cargo is
  unavailable because Session `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal
  `archived`; raw Cargo is not an allowed bypass.
- This owner remains in `pending.md` until M0-M4 pass on one source/executable fingerprint.
