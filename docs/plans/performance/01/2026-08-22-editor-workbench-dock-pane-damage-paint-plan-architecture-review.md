---
title: Editor workbench dock pane damage and retained paint-plan performance review
date: 2026-08-22
module: zircon_editor retained-host paint_workbench_renderer docks shell
priority: MVP-P0 basic editor dock and pane paint path
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate invalidation widget proxy and retained docking tab content
---

# Goal

Make dock/pane damage and retained paint generations authoritative before dispatching viewport,
native, template, debug and fallback content. A damage region outside a pane must enter none of those
backends. Stable dock chrome, viewport toolbar layout and active content must become retained paint
ranges rather than being rediscovered and rebuilt through string-based fan-out on every repaint.

## Reviewed source

- Rust files: 18/18
- lines: 1,307
- bytes: 42,006
- joined path-and-raw-source-bytes SHA256:
  `54f5cdb4a39b4c96b29deeb7e811d4d1e2461a0862c99ae9cb4ac29c8bf8955b`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

Scope: `paint_workbench_renderer/docks.rs` and dock shell files under
`paint_workbench_renderer/docks/**`. Six already separately reviewed asset-content, console-output and
selection projector children under `docks/pane/template_nodes/**` are excluded; their shared dispatch
root `docks/pane/template_nodes.rs` is included.

## Correct foundations to retain

1. Side, bottom and document docks reject non-visible regions before shell paint.
2. Floating windows reject damage outside their window-plus-shadow paint bounds before body paint.
3. Primitive raster and recording resolve the frame paint clip before pixel fill or command record.
4. Pane content order is explicit: Welcome native foundation precedes template controls while other
   native overlays follow them.
5. Viewport toolbar slots use runtime text metrics, compact inside narrow extents and preserve text
   clipping/ellipsis.
6. Empty-state geometry rejects non-finite, collapsed and undersized pane bodies.

## Structural findings

### P0: pane content fans out before pane-level damage rejection

`draw_pane` checks only whether `content` has a visible extent. It always paints the background and
then calls viewport-image, native-content, template-node and debug-overlay backends; fallback is
decided afterward. Primitive pixel work eventually sees the frame paint clip, but image lookup,
native dispatch, template projection/command construction, debug preparation and text measurement can
run before their final primitive is rejected.

M1 now rejects `content` that does not intersect `frame.paint_clip()` before background or content
fan-out. For one off-damage pane this reduces pane-shell dispatch `1 -> 0` and the four unconditional
content backend probes `4 -> 0`. All valid pane drawing is clipped to `content`/`body`; dock header,
rail, border and floating shadow are separate callers and remain independently damage-tested.

### P0: no retained pane paint plan owns active content and draw ranges

Every pane repaint rediscovers its content route. It probes the viewport backend, classifies Welcome
ordering, calls native content, selects a template model/projector from string pane kinds, calls native
debug overlay and possibly fallback. A stable active pane therefore repeats cross-module routing and
rebuilds downstream commands even when only another dock is damaged.

M2 must compile a `PanePaintPlan` when source/layout/style/interaction generations change. The plan
owns active backend kind, layer order and retained command ranges; damage selects ranges without
probing inactive backends. This is not accepted as another cloned pane DTO.

### P0: viewport toolbar remeasures four stable labels on every eligible repaint

Scene/Game pane paint maps four labels, measures each through runtime text metrics, recomputes slots
and emits four text commands every time the toolbar is visited. Mode, transform space, display mode,
grid mode, width, font and theme are already sufficient cache inputs.

M3 retains toolbar label/layout/text artifacts by those generations. Unchanged toolbar text must have
zero measure/layout/string-copy work, while width or label changes rebuild one toolbar generation.

### P1: active rail lookup scans every button and performs suffix matching

The active marker loops through all rail frames and compares exact ids plus `ends_with` for each row.
This is O(buttons * id bytes) every side-dock repaint and makes string naming part of paint routing.
M4 projects an exact active row/index from chrome generation; paint performs one indexed lookup.

### P1: floating damage scans every window

The floating layer intersects the current damage with every window-plus-shadow bounds before drawing.
This is reasonable for a few windows but scales linearly with floating count. M4 adds a generation-
qualified z-ordered bounds index or exact dirty-window ids and retains correct overlap order.

### P1: stable chrome is still rebuilt into immediate primitives

Dock shell, border, panel header, rail and floating window chrome are re-issued for each intersecting
damage. Global clip bounds the pixel writes, so raw `FrameRect::clone` calls are not a meaningful
target. M2 should retain typed chrome command ranges and patch only source/style/layout changes.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabStack.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Docking/SDockTab.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/DrawElements.cpp`

Slate repaints from explicit widget update flags. Its dock stack retains a widget hierarchy containing
the tab well, inline areas and selected content; `SDockTab::SetContent` refreshes parent content when
the owner changes instead of reselecting every possible content backend during paint. The window
element list retains widget cache handles and typed draw elements.

The transferable constraints are explicit invalidation, retained active-content ownership, bounded
child paint and retained typed draw ranges. Zircon should not copy Unreal pointer lifetimes or assume
Unreal timing numbers; current-source Zircon captures remain the acceptance source.

## Target architecture

1. Workbench projection publishes stable dock/pane ids plus source, layout, style, interaction and
   viewport-resource generations.
2. Each active pane owns one compiled paint plan with exact layer/backend kinds and retained command
   ranges; inactive backends are not probed.
3. Damage maps to exact dock, pane and retained range ids before content preparation.
4. Toolbar text/layout and dock chrome are retained artifacts invalidated only by relevant generations.
5. Rail active state is an exact row/index; floating windows use z-ordered retained bounds and dirty ids.
6. Runtime/editor prepared render-list ownership consumes the same ranges without a second command
   reconstruction or sorter.

## Instrumentation and acceptance

Matrix: panes `1/4/16/64`, floating windows `0/1/16/256`, rail buttons `0/8/64/1k`, damage
`outside/one pane/1%/100%`, backend `viewport/native/template/debug/fallback`, toolbar
`stable/label change/width change`, one/eight surfaces.

| Evidence | Acceptance |
| --- | --- |
| pane damage reject and backend probe counts | off-damage pane probes zero |
| pane-plan/range rebuild and reuse counts/bytes | rebuild proportional to dirty panes/ranges |
| toolbar text measure/layout/command counts | zero for unchanged generation |
| rail comparisons and floating bounds tests | indexed/sublinear or proportional to dirty candidates |
| CPU/allocation/RSS/latency/context switches/power | same executable/workload before and after |
| RenderDoc draw/batch/GPU and pixel/text parity | accepted current backend build |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add pane reject/probe, plan/range, toolbar measure, rail/floating candidate counters; capture. | attributable baseline |
| M1 | Reject off-damage pane content before shell and backend fan-out. | off-damage probes `4 -> 0` |
| M2 | Retain generation-qualified pane plans and chrome/content command ranges. | inactive probes zero; dirty-range rebuild |
| M3 | Retain viewport toolbar text/layout artifacts. | unchanged measure/layout zero |
| M4 | Compile active rail index and retained z-ordered floating bounds/dirty ids. | no full stable scans |
| M5 | Converge pane ranges with shared prepared render list and remove immediate duplicate routes. | one presentation authority |
| M6 | Run managed scale/input/WPR/power and RenderDoc/pixel parity matrix. | quantified accepted milestone |

## M1 implementation result

`draw_pane` now evaluates one pure `pane_intersects_damage` gate before shell/body paint. The gate
requires a visible content extent and, when a frame paint clip exists, a non-empty intersection. No
damage clip preserves full-paint behavior. Dock header, rail, border and floating shadow remain outside
the pane function and are unaffected.

For one pane fully outside current damage:

| Static dispatch work | Before | After | Change |
| --- | ---: | ---: | ---: |
| pane shell/body calls | 1 | 0 | -100% |
| viewport-image backend probes | 1 | 0 | -100% |
| native-content backend probes | 1 | 0 | -100% |
| template-node backend probes | 1 | 0 | -100% |
| debug-overlay backend probes | 1 | 0 | -100% |
| optional toolbar measure/layout | up to 4 labels | 0 | -100% for rejected pane |

These are source-path operation counts, not elapsed-time claims. M0/M2 must measure backend work and
retained-range reuse on the damage matrix.

Post-M1 direct owner scope:

- Rust files: 18/18
- lines: 1,346
- bytes: 43,114
- joined path-and-raw-source-bytes SHA256:
  `c85967cec19d25d1d51486b460786b84c0c7a1e8cd4f48c5b7def1b717690a2d`
- unchanged direct owner files: 17 retain their pre-M1 fingerprints inside the joined hash above

| Changed direct owner file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `docks/pane.rs` | 75 | 2,098 | `50096d9e1d4ece621e45f529ca44027757f35ecaf50272e63758be681b1cf425` |

Focused static contract:
`tools/tests/test_editor_workbench_dock_pane_damage_performance_contract.py`, 38 lines, 1,360
bytes, SHA256
`5025a15bc70edd4f8e6772ca4b2317d0828d71ff70c268d9933d78bfa58b3165`.

## Validation state

- Full owner review: passed, 18/18 Rust files.
- Primitive clip, pane callers and relevant Unreal sources above: traced/read.
- M1 focused contract: RED 2/2 before the change, GREEN 2/2 after the change.
- Current owned editor performance-contract set: GREEN 61/61.
- `rustfmt --check` for the changed Rust file and scoped `git diff --check`: passed.
- Two Rust geometry behavior tests cover no-clip, intersecting, disjoint and collapsed cases; they are
  present but not claimed passing until managed Cargo is executable.
- M0 and M2-M6 remain pending; no elapsed-time, GPU or power claim is made from static counts.
- Managed Cargo remains unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived` and rejects Cargo
  launch with `cargo_session_not_executable`.
- WPR and RenderDoc remain pending a launchable current-source editor. RenderDoc cannot validate
  CPU backend probing, text measurement or retained-plan reuse.

The module remains in `pending.md` until M0-M6 pass on one source/executable/workload fingerprint.
