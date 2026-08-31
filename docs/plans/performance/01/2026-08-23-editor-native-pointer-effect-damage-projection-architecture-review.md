---
title: Editor native pointer effect damage projection performance review
date: 2026-08-23
module: zircon_editor retained-host native_pointer chrome/close/pane/viewport damage
priority: MVP-P0 editor pointer damage scope and main-thread repaint cost
status: source_reviewed_m0_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate reason-coded widget invalidation fast path
---

# Goal

Replace effect-agnostic bounding damage guesses with typed invalidation effects projected to retained
scene/widget owners and a bounded multi-region damage set. Pointer actions must not deep-clone model
rows merely to read frames, scan unrelated windows/nodes, or promote disjoint center/status/chrome
areas into a large bounding repaint without measured reason.

## Reviewed source

- owner Rust files: 31/31
- lines: 532
- bytes: 19,019
- source-only SHA256 over lexicographically sorted owner files:
  `d98e3d09b8fc4f7d314a82250948ab9964c36286e8c7d7b61be7f6c273548f23`
- post-M0 owner files/lines/bytes/SHA256: 31 / 526 / 18,550 /
  `07995775ee710ed6142456ed2f7a970e939a271a140d1a548ca1fdae0f6b4c87`
- owning commit at review: `7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`

| Owner group | Files | Lines | Bytes |
| --- | ---: | ---: | ---: |
| `native_pointer/chrome_damage.rs + chrome_damage/**` | 15/15 | 347 | 12,278 |
| `native_pointer/close_prompt_damage.rs + close_prompt_damage/**` | 4/4 | 44 | 1,346 |
| `native_pointer/pane_button_damage.rs + pane_button_damage/**` | 5/5 | 56 | 2,190 |
| `native_pointer/viewport_toolbar_damage.rs + viewport_toolbar_damage/**` | 7/7 | 85 | 3,205 |

All owner files were read in full. Typed chrome routes, button/toolbar callers, presentation models,
redraw request merging, frame geometry and paint/presenter damage consumers were inspected as direct
boundaries. Unreal invalidation root and widget-proxy sources were read directly. This record
supersedes the matching portions of the 2026-07-17 pointer damage review.

## Correct foundations to retain

1. `ChromePointerRoute` is a typed closed route and damage dispatch is constant-time once a direct
   dock/document route is known.
2. Visibility checks reject zero/non-finite frames. Close prompt returns only overlay/dialog bounds.
3. Activity-rail collapse distinguishes active center-band effects from inactive dock-only effects.
4. Toolbar-local actions can damage only their control frame; extra text-input damage is retained.
5. Damage computation is isolated from callback execution, which is the correct place to replace
   guesses with explicit action effects.

## Structural findings

### P0: effect-agnostic helpers promote broad bounding repaints

Every helper returns one `FrameRect`. Pane button actions unconditionally union pane, two center-band
frames and three status frames; viewport actions selected by a string heuristic do the same. Host-page
activation unions tab row, project path, every tab/close frame, every page template node, center band
and status. Floating-header activation unions every floating-window frame. Disjoint regions collapse
to bounding space here and again in redraw coalescing.

M1 changes callbacks/actions to publish typed invalidation effects such as target paint, selection
chrome, status text, center content, child order/z-order or layout. The committed presentation maps
owner ids to retained damage regions/ranges. M2 carries a bounded `DamageRegionSet` through redraw and
presenter; the existing redraw and frame-damage plans own that shared type.

### P0: floating-window and host-page damage scan and deep-clone model rows

Floating document/header paths call `row_data` while searching windows, then the header path scans all
windows again to union frames. Host-page tab and template-node paths also call `row_data` for every row
despite reading only fixed-size frame fields. `TemplatePaneNodeData` is a 163-field DTO, making these
avoidable deep clones on the input path.

M0 now replaces these accesses with borrowed iterators and borrowed rows. Complexity remains O(W/T/N)
where the current semantics require a scan, but heap/reference clones drop to zero. M1 replaces the
remaining scans with retained owner/frame projections and stable ids.

### P0: control-id strings guess invalidation scope

`viewport_toolbar_click_affects_viewport_or_status` matches four string spellings and an `align.`
prefix. Pane buttons assume the broadest plausible effect. Plugin or renamed actions can silently
under-damage; conservative fallback then encourages full/broad damage.

M1 attaches a typed `UiInvalidationEffect` to registered actions/callback results. Plugin registration
validates effects once; event dispatch consumes the descriptor without prefix classification.

### P1: z-order damage lacks overlap-aware ownership

Floating-header activation repaints the bounding union of every floating window even when most do not
overlap the moved-to-front window. Correct z-order change requires the target and affected overlap
chain, not all windows and not just the header. M1 stores spatial overlap/order metadata in the
floating-layer generation and returns exact affected owner ranges/regions.

### P2: four local union implementations duplicate one shared primitive

Chrome, close prompt, pane button and viewport toolbar carry near-identical visibility/union wrapper
modules. Runtime cost is tiny and fixed-size `FrameRect` clone is not the bottleneck. M2 deletes these
wrappers when the common damage-set API lands; a standalone refactor is not a performance milestone.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`

Slate separates pre-update invalidation reasons (`Layout`, attribute registration, visibility, child
order) from post-update reasons (`Layout`, paint, volatility, render transform, prepass) and queues the
appropriate proxy work (`SlateInvalidationRoot.cpp:179-196, 299-327`). `ProcessInvalidation` processes
ordered pre/attribute/prepass/post lists and clears cached elements only on the explicit slow path
(`1281-1404`). `FWidgetProxy::Update` repaints only proxies carrying repaint flags; layout invalidation
recomputes desired size and propagates to the parent only when necessary (`WidgetProxy.cpp:52-119,
122-197`).

The transferable rule is typed invalidation reason plus retained owner/range propagation. Zircon does
not need Slate proxy objects, but action semantics must identify what changed before spatial damage is
projected.

## Target architecture

1. Registered UI/editor actions return typed invalidation effects and stable target owner ids.
2. `HostPresentationGeneration` maps owner ids to frames, scene ranges and z-order overlap metadata.
3. Damage projection produces a bounded `DamageRegionSet` plus layout/paint/order/status reason bits.
4. External redraw, event-loop coalescing, retry, paint extraction and presenter preserve those regions
   and reasons without bounding union until backend policy explicitly promotes them.
5. Full/broad promotion is reason-coded and reports useful versus submitted area.
6. Pointer routes never scan/clone unrelated model rows after stable owner projection lands.

## Instrumentation and acceptance

| Evidence | Acceptance |
| --- | --- |
| damage requests/reasons/owner ids | every request attributable to typed effect |
| model rows visited/cloned bytes | M0 zero row clones; M1 O(1) owner lookup for stable routes |
| input/output regions/useful/union area | disjoint regions retained; amplification reported |
| floating overlaps/z-order affected owners | visits only target overlap chain |
| full/broad promotions | explicit bounded reason, never string-guess fallback |
| pointer-to-present CPU p50/p95/p99 | slope independent of unrelated windows/nodes |
| correctness | exact changed pixels, z-order, status, pane/tab/toolbar and plugin parity |

Matrix: windows/tabs/template nodes `1/10/100/1K/10K`; action `local/selection/status/layout/z-order/
plugin`; regions `1/2/8/64`; placement `overlap/disjoint/opposite`; input repeat `1/125/500/1K Hz`;
backend `GPU/softbuffer/snapshot`; scale `1x/1.5x/2x/4K`.

WPR owns CPU, allocation, context-switch and power evidence. RenderDoc is used after a current-source
GPU presenter is launchable for scissor/draw/resource/pixel parity. All artifacts remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Borrow floating/tab/template rows instead of deep-cloning them. | applied; static contract GREEN, managed Rust/dynamic pending |
| M1 | Add typed invalidation effects and retained owner/overlap projections with telemetry. | no control-id scope guesses; bounded owner lookup |
| M2 | Emit and propagate shared multi-region damage; delete local union wrappers/single-rect APIs. | no default bounding-space repaint |
| M3 | Run scale/event-storm/WPR/power and RenderDoc scissor/pixel matrices. | quantified acceptance and milestone closeout |

## Validation state

- Owner source review: passed, 31/31 current Rust files.
- Route, action caller, presentation, redraw, paint and presenter boundaries: read and mapped.
- Unreal invalidation reason/root/proxy sources: read and mapped.
- M0 static performance contract moved RED 0/2 to GREEN 2/2. Together with asset-pointer, dock/pane
  damage, scene damage-state and GPU submitted-damage contracts, the focused set passes 9/9.
- The changed Rust files pass independent `rustfmt --check`; scoped `git diff --check` passes with
  line-ending warnings only.
- Managed Rust tests, current-source launch, WPR and RenderDoc remain pending because the managed
  Cargo Session is terminal `archived` with `cargo_session_not_executable`. No raw Cargo bypass is
  allowed.
- M0 dynamic acceptance and M1-M3 remain pending; this owner stays out of `review.md` until dynamic
  acceptance.
