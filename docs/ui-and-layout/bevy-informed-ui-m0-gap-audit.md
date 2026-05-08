---
related_code:
  - dev/bevy/crates/bevy_window/src/event.rs
  - dev/bevy/crates/bevy_ui/src/lib.rs
  - dev/bevy/crates/bevy_ui/src/layout/convert.rs
  - dev/bevy/crates/bevy_ui/src/focus.rs
  - dev/bevy/crates/bevy_ui_widgets/src/lib.rs
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime_interface/src/ui/pipeline/mod.rs
  - zircon_runtime_interface/src/ui/pipeline/stage.rs
  - zircon_runtime_interface/src/ui/pipeline/stage_report.rs
  - zircon_runtime_interface/src/ui/pipeline/frame_report.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/render/batch.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/surface/render/cache.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - zircon_editor/src/ui/slint_host/app/invalidation.rs
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
implementation_files:
  - docs/ui-and-layout/bevy-informed-ui-m0-gap-audit.md
  - tests/acceptance/bevy-informed-ui-m0-baseline.md
plan_sources:
  - .codex/plans/Bevy-Informed Zircon UI 架构优化里程碑计划.md
  - .codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md
  - .codex/plans/Zircon UI 增量布局、增量重绘与控件池优化计划.md
  - .codex/plans/Editor 绘制与鼠标事件优化计划.md
  - docs/ui-and-layout/shared-ui-input-events.md
  - docs/ui-and-layout/slate-style-ui-surface-frame.md
  - docs/assets-and-rendering/runtime-ui-graphics-integration.md
tests:
  - tests/acceptance/bevy-informed-ui-m0-baseline.md
  - zircon_runtime_interface/src/tests/layout_engine_contracts.rs
  - zircon_runtime_interface/src/tests/pipeline_contracts.rs
  - .github/workflows/ci.yml
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
doc_type: milestone-detail
---

# Bevy-Informed UI M0 Gap Audit

## Scope

This M0 audit re-measures the current Zircon UI state against the Bevy reference files named by `.codex/plans/Bevy-Informed Zircon UI 架构优化里程碑计划.md`. It is intentionally documentation and acceptance evidence only. Runtime/editor implementation files remain owned by concurrent UI sessions.

M0 does not accept previous UI documents as completion proof. Existing documents are treated as historical evidence unless their current source/test anchors still support the claim.

## Reference Evidence

The reference scan was refreshed with:

```powershell
python .opencode\skills\zircon-project-skills\zr-language-feature-design\scripts\search_feature_evidence.py "UiSystems|RenderUiSystems|CursorMoved|WindowScaleFactorChanged|FocusPolicy|UiWidgetsPlugins|from_node|taffy" --languages bevy --code-only --max-count 40
```

Primary Bevy anchors:

- `dev/bevy/crates/bevy_window/src/event.rs:27` defines typed window events such as `WindowResized`, `RequestRedraw`, close lifecycle, cursor enter/move/leave, IME, focus, scale factor, file drag/drop, and theme changes.
- `dev/bevy/crates/bevy_winit/src/state.rs` is the winit translation seam found by the evidence script for cursor movement and scale-factor events before they enter Bevy's neutral window-event stream.
- `dev/bevy/crates/bevy_ui/src/lib.rs:90` defines `UiSystems::{Focus, Prepare, Propagate, Content, Layout, PostLayout, Stack}` and wires them into `PreUpdate`/`PostUpdate` in `UiPlugin`.
- `dev/bevy/crates/bevy_ui/src/layout/convert.rs:64` maps Bevy `Node` layout fields into `taffy::style::Style`, including display, overflow, flex, grid, size, margin, padding, border, gap, and placement.
- `dev/bevy/crates/bevy_ui/src/focus.rs:26` defines `Interaction`, `RelativeCursorPosition`, and `FocusPolicy`; `ui_focus_system` clears hidden-node interaction, clips recursively, walks `UiStack` top-down, and stops on blocking focus policy.
- `dev/bevy/crates/bevy_ui_widgets/src/lib.rs:1` states that standard widgets are headless and unstyled, use external state management, and emit typed entity events such as `Activate` and `ValueChange<T>`.
- `dev/bevy/crates/bevy_ui_render/src/lib.rs:118` defines render extract sets for camera views, box shadows, backgrounds, images, texture slices, borders, text backgrounds, text shadows, text, cursor, debug, and gradient.
- `dev/bevy/crates/bevy_ui_render/src/lib.rs:216` chains UI extract stages and then queues, sorts, prepares bind groups, and submits UI passes in `Core2d`/`Core3d`.

## Current Zircon Baseline

Current Zircon anchors:

- `zircon_runtime_interface/src/ui/dispatch/input/event.rs:10` has shared `UiInputEvent` families for pointer, keyboard, text, IME, navigation, analog, drag/drop, popup, and tooltip timers.
- `zircon_runtime/src/ui/surface/surface.rs:45` records `UiSurfaceRebuildReport` counts and phase timings for dirty, layout, arranged, hit-grid, and render work.
- `zircon_runtime/src/ui/surface/surface.rs:210` implements `UiSurface::rebuild_dirty(...)`, including incremental layout, arranged-tree rebuild, hit-grid rebuild, render-cache update, and no-dirty cached counts.
- `zircon_runtime/src/ui/layout/pass/measure.rs:19` and `zircon_runtime/src/ui/layout/pass/arrange.rs:17` implement recursive measure/arrange passes over current Zircon container kinds rather than a taffy-backed layout engine.
- `zircon_runtime_interface/src/ui/surface/render/batch.rs:9` defines `UiBatchPlan`, `UiBatchKey`, split reasons, and CPU-estimated draw-call counts from paint elements.
- `zircon_runtime_interface/src/ui/surface/diagnostics.rs:61` exposes debug snapshots, rebuild stats, render debug stats, hit-grid stats, invalidation, damage, and optional backend render stats.
- `zircon_runtime/src/ui/surface/render/cache.rs:10` retains render commands by node and reports reused, rebuilt, and damage counts.
- `zircon_editor/src/ui/slint_host/app/invalidation.rs:3` has an editor-native invalidation mask that separates layout, tree, presentation, paint-only, pointer-hover, viewport-image, hit-test, window-metrics, and render reasons.
- `zircon_runtime/src/ui/tests/event_routing.rs:179` covers repeated same-target mouse movement staying idle without dirtying or rebuilding a surface.
- `zircon_runtime/src/ui/tests/surface_dirty_domains.rs:13` covers dirty-domain phase separation and incremental layout/render-cache expectations.
- `zircon_editor/src/tests/host/slint_window/native_host_contract.rs:1117` and nearby tests cover native host repeated hover fast paths and shared pane/template routing.

## Gap Matrix

| Area | Bevy reference behavior | Zircon current evidence | M0 gap and target milestone |
| --- | --- | --- | --- |
| Window and input pump | Bevy translates winit events into neutral typed window messages for cursor, focus, scale factor, redraw, close, IME, and file drag/drop. | Zircon has shared `UiInputEvent` DTOs for UI input and editor-native translation helpers, but no single shared window-event DTO/pump that both runtime winit and editor Slint/native hosts consume. | M1 must add a neutral window/input pump for cursor enter/move/leave, focus, scale factor, redraw, close, IME, and file drag/drop, then route runtime and editor hosts through it. |
| UI schedule | Bevy names and orders `Focus`, `Prepare`, `Propagate`, `Content`, `Layout`, `PostLayout`, and `Stack`; UI render has separate extract, queue, prepare, and pass stages. | Zircon `UiSurface::rebuild_dirty(...)` and editor invalidation masks expose useful phase stats, but the runtime UI path is still method-driven rather than a named pipeline schedule with stable stage DTOs. | M2 must define explicit UI stages such as `InputCollect`, `FocusInteraction`, `ContentMeasure`, `Layout`, `PostLayoutStack`, `HitGrid`, `RenderExtract`, `BatchPrepare`, `PaintSubmit`, and `Diagnostics`. |
| Layout engine | Bevy converts node fields into `taffy::style::Style` for flex/grid/block/content-size semantics. | Zircon uses recursive `measure_node(...)` and `arrange_node(...)` over custom container kinds. Searches found no production `UiLayoutEngine`, `TaffyLayoutEngine`, or UI taffy adapter. | M3 must introduce a `UiLayoutEngine` abstraction, retain Zircon-specific `Free`, `Overlay`, `Scrollable`, and virtualized list semantics, and map flex/grid/wrap/block-compatible fields to taffy. |
| Focus, hit, and pointer interaction | Bevy focus walks the stack top-down, resets hidden nodes, applies recursive clip checks, tracks relative cursor, and stops on blocking focus policy. | Zircon has hit-grid routing, focus/capture state, same-target hover suppression, and visibility-aware tests, but focus, pointer capture, popup, tooltip, and editor hover policies are not yet unified under one headless behavior layer. | M4/M8 must converge focus, capture, tooltip, popup, menu, disabled/hidden, keyboard navigation, and accessibility policy around one runtime-owned behavior/focus contract. |
| Headless widgets | Bevy widgets are unstyled behavior plugins with external state and typed events. | Zircon has component metadata, bindings, Material/style documents, and tests for button click and text input, but widget behavior still appears dispersed across component names, event routing, editor projections, popup/tooltip state, and native host callbacks. | M4 must introduce a headless widget behavior registry that emits typed events and leaves style/theme projection to `.ui.toml` and Material/editor layers. |
| Render extract and batches | Bevy separates background/image/border/text/cursor/debug/gradient/box-shadow extraction, queueing, preparation, and pass submission. | Zircon has `UiRenderExtract`, paint elements, `UiBatchPlan`, CPU split reasons, render-command cache reuse, and optional backend stats fields, but M0 found no mandatory backend-confirmed UI draw-call/batch counters. | M5 must harden extract families, batch keys, clip/scissor/resource/text/material split reasons, and backend-submitted render stats before render acceptance. |
| Diagnostics and performance gates | Bevy has named UI/render stages and debug overlay hooks. | Zircon has rebuild reports, debug snapshots, hit-grid stats, invalidation masks, and focused tests for repeated same-target mouse movement and native hover fast paths. | M2/M6/M7/M9 must turn those diagnostics into required acceptance gates for mouse move latency, hover/taskbar behavior, tooltip/popup timing, layout rebuild counts, render command reuse, batch counts, and template reload avoidance. |
| Editor/runtime convergence | Bevy keeps UI behavior in shared systems while render/extract consumes shared components. | Zircon editor consumes `UiSurfaceFrame` in several native host paths, but also has editor-local invalidation and native dispatch seams. | M7 must route workbench shell, menus, drawers, floating panels, viewport toolbar, asset preview, and hover taskbar through shared hit grid, input state, widget behavior, and paint contracts. |

## Failure Samples To Preserve As Baseline

M0 found these current or recently accepted baseline cases that should become milestone gates instead of informal checks:

- Repeated same-target runtime mouse moves: `repeated_same_target_mouse_moves_do_not_dirty_or_rebuild_surface` in `zircon_runtime/src/ui/tests/event_routing.rs` asserts 100 same-target moves do not dirty flags, emit events, request damage, or mutate the last rebuild report.
- Runtime dirty-domain separation: `surface_dirty_rebuild_separates_hit_input_render_and_legacy_state_flags`, `surface_dirty_layout_skips_siblings_under_non_auto_parent`, and `surface_dirty_render_reuses_unchanged_commands_without_damage` in `zircon_runtime/src/ui/tests/surface_dirty_domains.rs` define the current incremental rebuild baseline.
- Editor native hover fast path: `native_host_repeated_hierarchy_hover_moves_do_not_rebuild_presentation` and neighboring repeated-hover tests in `zircon_editor/src/tests/host/slint_window/native_host_contract.rs` define the current no-slow-path baseline for host hover motion.
- Click cancellation: `primary_release_outside_pressed_target_does_not_mark_click_target` and `captured_release_uses_hit_path_not_capture_target_for_click_target` in `zircon_runtime/src/ui/tests/event_routing.rs` define release-outside behavior that headless widgets must preserve.
- Text/native input routing: `native_host_welcome_material_text_field_accepts_keyboard_input` in `zircon_editor/src/tests/host/slint_window/native_host_contract.rs` records the editor-native text-input route that later shared input pump work must not regress.

## M0 Decisions

- Keep `.ui.toml`, `UiSurfaceFrame`, runtime/editor layering, and Zircon's Slate-style surface authority. Bevy informs stage boundaries and behavior ownership; it is not a direct framework dependency.
- The dominant reference for this M0 lane is Bevy because the active plan asks for Bevy UI/window/widget/render convergence. Existing Zircon Slate docs remain secondary because they define current repository truth and editor/runtime boundary constraints.
- Treat `zircon_runtime_interface` as the target home for neutral DTOs, `zircon_runtime` as the behavior/layout/render-extract owner, and `zircon_editor` as a consumer of shared contracts and editor-host projection.
- Do not introduce non-network `server` naming for UI architecture. Future stage and manager names should follow `zircon_runtime::core::framework`, `zircon_runtime::core::manager`, and runtime/editor vocabulary.
- Do not add compatibility shims for old UI paths during later milestones unless persisted external data or explicit user requirements force coexistence.

## M1-M9 Acceptance Checklist

- M1 Window/Input Pump: neutral window events for cursor enter/move/leave, focus, scale factor, redraw, close, IME, and file drag/drop; runtime and editor hosts translate into one stream; cursor leave clears hover; scale factor marks metrics/layout only; duplicate redraw coalesces.
- M2 UI Pipeline/Schedule: named stage report records input, focus, content, layout, post-layout/stack, hit, render extract, batch prepare, paint submit, and diagnostics timings/counters; 100 mouse moves do not reload templates or perform full layout.
- M3 Taffy Layout Engine: flex/grid/block/wrap mappings have golden tests; Zircon `Free`, `Overlay`, `Scrollable`, and virtualized list behavior remain intact; existing `.ui.toml` runtime/editor fixtures do not regress.
- M4 Headless Widget Behavior: button release-outside rejects activation; slider capture is stable; tooltip arm/show/cancel is timed and cancelable; menu popup outside-click/ESC/focus return is deterministic; disabled/focus/hover states are shared across runtime and editor.
- M5 Render Extract/Batch/Pass: extract families cover background, image, border, text, cursor, tooltip, debug, shadow, gradient, and material; batch keys include shader/resource/clip/text backend/material/z layer; backend stats confirm submitted draw calls and split reasons.
- M6 Incremental Dirty, Cache, Pool: dirty domains are retained; resize/mouse/state changes do not reload `.ui.toml`; unchanged nodes reuse generation/render commands; pooled nodes clear hover, focus, capture, tooltip, and popup state.
- M7 Editor Host UX Cutover: workbench, menu, drawer, floating panel, hover taskbar, viewport toolbar, and asset preview consume shared `UiSurfaceFrame`, hit grid, input state, widget behavior, and paint contracts.
- M8 Accessibility, Focus, Navigation: role/name/state/focus policy/navigation groups are in `.ui.toml`; Tab, Shift-Tab, directional navigation, popup focus trap, disabled/hidden filtering, and tooltip/a11y snapshots are tested.
- M9 Docs And CI Gates: module docs and acceptance records include Bevy references, Zircon implementation files, test commands, failure notes, screenshots/perf data where applicable, and explicit workspace-green status.

## Open Risks

- Current active sessions are editing runtime UI layout/surface and editor UI internals. M0 therefore records source and test anchors but does not claim code-level acceptance for the dirty checkout.
- The current source imports `compute_incremental_layout_tree` from `zircon_runtime::ui::layout` in `UiSurface`; sibling session notes report this area is still under active validation. Later milestones must re-read the settled implementation before building on this audit.
- CPU-side batch statistics exist now, but backend-confirmed draw-call and batch counters are optional in the DTO. Render milestones must not accept estimated-only counters as backend proof.
- Existing tests cover important slices, but M0 found no single scenario fixture that records mouse move, hover, tooltip, popup, taskbar, layout rebuild, render batch, and template reload metrics together. M2/M6/M7 should add that integrated scenario.

## M2 Interface Slice Evidence

The interface-only M2 slice adds `zircon_runtime_interface::ui::pipeline` as the neutral DTO owner for the named pipeline schedule and report counters. `zircon_runtime_interface/src/tests/pipeline_contracts.rs` covers the exact stage order, dirty-reason and timing aggregation, layout/hit/render/batch counters, and the 100 pointer-move fast-path report shape with zero template reloads and zero full-layout work.

Scoped validation on 2026-05-08 passed `cargo test -p zircon_runtime_interface --lib pipeline_contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-pipeline-m2 --message-format short --color never` with 3 tests, and `cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-pipeline-m2 --message-format short --color never` for the interface crate.

This evidence is contract-only. Runtime `UiSurface`, editor host, and backend render code still need later M2 implementation slices to populate `UiPipelineFrameReport` from real frame execution after the active layout/render/input lanes settle.

## M3 Interface Slice Evidence

The interface-only M3 preflight adds `UiLayoutEngineCapability`, `UiLayoutEngineRequest`, `UiLayoutEngineSelection`, and `UiLayoutEngineSelectionReport` under `zircon_runtime_interface::ui::layout`. The DTOs establish the neutral boundary for future legacy-vs-taffy routing: flex/grid/block-compatible families can report Taffy selection, while Free, Overlay, Scrollable, and virtualized-list semantics remain Zircon-owned and fall back to the current layout path.

Scoped validation on 2026-05-08 passed `cargo test -p zircon_runtime_interface --lib layout_engine_contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-layout-engine-m3 --message-format short --color never` with 3 tests, and `cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-layout-engine-m3 --message-format short --color never` for the interface crate.

This evidence is contract-only. Runtime `zircon_runtime::ui::layout` still needs the actual `UiLayoutEngine` abstraction, Taffy style conversion, golden layout fixtures, and `UiSurfaceFrame` preservation checks in later M3 slices.
