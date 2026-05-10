---
related_code:
  - dev/bevy/crates/bevy_ui/src/lib.rs
  - dev/bevy/crates/bevy_ui/src/focus.rs
  - dev/bevy/crates/bevy_ui/src/picking_backend.rs
  - dev/bevy/crates/bevy_input_focus/src/lib.rs
  - dev/bevy/crates/bevy_input_focus/src/gained_and_lost.rs
  - dev/bevy/crates/bevy_input_focus/src/tab_navigation.rs
  - dev/bevy/crates/bevy_input_focus/src/directional_navigation.rs
  - dev/bevy/crates/bevy_text/src/lib.rs
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - dev/bevy/crates/bevy_text/src/text_edit.rs
  - dev/bevy/crates/bevy_text/src/text_editable.rs
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
  - dev/bevy/crates/bevy_ui_widgets/src/lib.rs
  - dev/bevy/crates/bevy_ui_widgets/src/button.rs
  - dev/bevy/crates/bevy_feathers/src/lib.rs
  - dev/bevy/crates/bevy_feathers/src/theme.rs
  - dev/bevy/crates/bevy_a11y/src/lib.rs
  - dev/bevy/crates/bevy_winit/src/accessibility.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/surface/navigation_state.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime_interface/src/ui/surface/pointer/route.rs
  - zircon_runtime_interface/src/ui/tree/node/input_policy.rs
  - zircon_runtime_interface/src/ui/component/event.rs
  - zircon_runtime_interface/src/ui/component/state.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/focus_contract.rs
  - zircon_runtime_interface/src/ui/template/asset/document.rs
  - zircon_runtime_interface/src/ui/surface/render/extract.rs
  - zircon_runtime_interface/src/ui/surface/render/batch.rs
  - zircon_runtime_interface/src/ui/surface/render/editable_text.rs
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime_interface/src/ui/focus.rs
  - zircon_runtime_interface/src/ui/navigation.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/ui/picking.rs
  - zircon_runtime_interface/src/ui/text.rs
  - zircon_runtime_interface/src/ui/widget.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/node_pool.rs
  - zircon_runtime/src/ui/tree/node/focus.rs
  - zircon_runtime/src/ui/tests/focus_navigation.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/template/asset/compiler/node_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs
  - zircon_runtime/src/ui/template/asset/schema/legacy_template.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_editor/assets/ui/editor/material_meta_components.ui.toml
implementation_files:
  - docs/ui-and-layout/bevy-ui-text-widgets-focus-a11y-m0-gap-audit.md
  - docs/zircon_runtime_interface/ui/contract-spine.md
  - docs/zircon_runtime/ui/surface/input.md
plan_sources:
  - user: 2026-05-08 实现 Bevy 对齐的 Zircon UI/Text/Widgets/Focus/A11y 里程碑计划 M0
  - .codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md
  - dev/bevy commit c040d7603 / v0.16.0-rc.4-2455-gc040d7603 reference baseline from plan
tests:
  - docs-only validation: git diff --check docs/ui-and-layout/bevy-ui-text-widgets-focus-a11y-m0-gap-audit.md docs/ui-and-layout/index.md
  - target: cargo test -p zircon_runtime_interface --lib contracts --locked
  - target: cargo test -p zircon_runtime_interface --lib render_contracts --locked
  - target: cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked
  - target: cargo test -p zircon_runtime --lib event_routing --locked
  - target: cargo test -p zircon_runtime --lib hit_grid --locked
  - target: cargo test -p zircon_runtime --lib text_layout --locked
  - target: cargo test -p zircon_runtime --lib surface_dirty_domains --locked
  - target: cargo test -p zircon_runtime --lib focus_navigation --locked
  - target: cargo test -p zircon_editor --lib material_meta_component_contracts --locked
doc_type: milestone-detail
---

# Bevy UI/Text/Widgets/Focus/A11y M0 Gap Audit

## Scope

This document is the M0 Evidence & Gap Audit for `.codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md`.

M0 is documentation and acceptance evidence only. It does not change runtime/editor behavior. Later milestones may add contract DTOs, runtime behavior, editor cutover, AccessKit bridge code, or text shaping dependencies, but those changes must start from the gaps and tests recorded here.

Architecture owner split for the later milestones stays fixed:

| Layer | M0 decision |
|---|---|
| `zircon_runtime_interface::ui` | Owns neutral contracts for focus, navigation, picking, widget events, text/editing DTOs, render extract, batch/debug data, and future a11y snapshots. |
| `zircon_runtime::ui` | Owns retained surface behavior, input dispatch, focus/navigation mutation, hit testing, widget behavior, text layout/editing, dirty rebuilds, a11y extraction, and render extraction. |
| `zircon_editor::ui` | Consumes shared runtime/interface contracts and owns authoring host state, presentation, styled editor widgets, and retained/native host adaptation. It must not become the truth source for runtime UI focus, picking, widget, or a11y semantics. |
| Platform/accessibility bridge | Future platform adapters may consume neutral snapshots and use AccessKit, but the runtime contract must remain AccessKit-independent. |

## Reference Evidence Matrix

| Domain | Bevy evidence | Capability observed | Zircon implication |
|---|---|---|---|
| UI schedule and layout pipeline | `dev/bevy/crates/bevy_ui/src/lib.rs` | Bevy separates UI work into named systems such as focus, prepare, propagation, content, layout, post-layout, and stack. | Zircon needs a named schedule report that can prove `InputCollect -> Focus -> WidgetBehavior -> TextMeasure -> Layout -> PostLayout/Stack -> Picking -> A11yExtract -> RenderExtract -> BatchPrepare`. Current Zircon has method-level rebuild reports, not a formal schedule contract. |
| Pointer interaction and clipping | `dev/bevy/crates/bevy_ui/src/focus.rs` | Bevy tracks `Interaction`, cursor-relative position, focus policy, hidden-node reset, top-down stack hit testing, and clipping ancestors. | Zircon already has hit grid, clip and disabled rejection, hover/press/focus routes; M4 must unify these into a single `UiPickPolicy` rather than leaving input policy, focus policy, and a11y eligibility separate. |
| UI picking backend | `dev/bevy/crates/bevy_ui/src/picking_backend.rs` | Bevy UI picking emits ordered pointer hits and can pick text sections, but it deliberately does not use `FocusPolicy`. | Zircon should intentionally diverge by using one policy for hover, click, focus, and a11y hit eligibility. Tests must cover Bevy's split-policy failure mode as an avoided design. |
| Input focus | `dev/bevy/crates/bevy_input_focus/src/lib.rs`, `dev/bevy/crates/bevy_input_focus/src/gained_and_lost.rs` | Bevy has an `InputFocus` resource, focus visible state, focused input bubbling, initial focus, focus gained/lost events, and despawn cleanup. | Zircon currently has `UiFocusState` and component focus events, but M1/M2 need first-class `UiInputFocus`, `UiFocusChangeEvent`, `UiFocusedInput`, focus visible rules, autofocus, and clear-on-hidden/disabled/despawn semantics. |
| Tab navigation | `dev/bevy/crates/bevy_input_focus/src/tab_navigation.rs` | Bevy has tab index, tab groups, modal traps, next/previous/first/last, and pointer focus-ring hiding. | Zircon currently has tree-order next/previous navigation. M3 must add tab index, groups, modal traps, and Shift-Tab semantics, with surface/layer/z-order as a Zircon-specific extension. |
| Directional navigation | `dev/bevy/crates/bevy_input_focus/src/directional_navigation.rs`, `dev/bevy/crates/bevy_ui/src/auto_directional_navigation.rs` | Bevy supports manual directional neighbors, blocked edges, auto-computed candidates from node bounds, and cleanup. | Zircon has nearest-focusable directional heuristics but no contract for manual override or blocked edges. M3 should add override DTOs and tests for layer-aware spatial order. |
| Text shaping and layout | `dev/bevy/crates/bevy_text/src/lib.rs`, `dev/bevy/crates/bevy_text/src/pipeline.rs` | Bevy uses Parley/Swash-level text shaping, measurement, runs, font atlas glyphs, cursor and selection geometry. | Zircon has a grapheme-aware fixed-advance scaffold and shared shaped DTO placeholders. M6 must introduce a `UiTextShaper` abstraction and backend-gated acceptance for fallback fonts, real glyph metrics, selection/caret geometry, and layout consistency. |
| Text editing | `dev/bevy/crates/bevy_text/src/text_edit.rs`, `dev/bevy/crates/bevy_text/src/text_editable.rs` | Bevy covers insert/copy/cut/paste, grapheme and word deletion, cursor/selection movement, point selection, filters, max chars, newline policy, and `TextEditChange`. | Zircon has byte-boundary caret edits, selection replacement, and IME preedit/commit/cancel scaffold. M6 needs word navigation, clipboard, filters, max chars, multiline policy, cursor blink, point selection, and typed edit-change events. |
| UI render extraction | `dev/bevy/crates/bevy_ui_render/src/lib.rs`, `dev/bevy/crates/bevy_ui_render/src/text.rs` | Bevy extracts backgrounds, images, borders, viewport nodes, text, decorations, selections, cursor, queues/sorts UI nodes, and prepares batches. | Zircon interface already has paint/batch/cache/debug/parity DTOs, while runtime extract still emits legacy command lists. M8 should make all runtime/editor painters consume a single neutral extract kind and batch/debug stats path. |
| Headless widgets | `dev/bevy/crates/bevy_ui_widgets/src/lib.rs`, `dev/bevy/crates/bevy_ui_widgets/src/button.rs` | Bevy widgets are unstyled behavior components; plugin group covers popover, button, checkbox, menu, radio, scrollbar, slider, and editable text. Button activation is keyboard/pointer unified and disabled-aware. | Zircon component events and Material roots exist, but no complete headless widget behavior layer owns button/checkbox/radio/slider/menu/popover/tooltip/text input semantics. M5 must land the behavior layer before editor styling. |
| Styled widgets | `dev/bevy/crates/bevy_feathers/src/lib.rs`, `dev/bevy/crates/bevy_feathers/src/theme.rs` | Feathers adds styled editor controls, tokens, focus outlines, embedded fonts/icons, and tab navigation on top of headless widgets. | Zircon's Material `.ui.toml` assets and native painter state are styling progress, but M10 must sit on the shared headless widget behavior contract instead of forking behavior inside editor host glue. |
| Accessibility | `dev/bevy/crates/bevy_a11y/src/lib.rs`, `dev/bevy/crates/bevy_winit/src/accessibility.rs`, `dev/bevy/crates/bevy_ui/src/accessibility.rs` | Bevy wraps AccessKit nodes/actions/resources and updates UI node bounds, roles, labels, and actions. | Zircon has only adjacent metadata such as `accessibility_label` params and icon-button labels excluded from visual text. M1/M9 must add neutral a11y DTOs, runtime tree snapshots, name-source precedence, hidden/disabled handling, focus updates, action roundtrip, and an AccessKit bridge after the neutral layer exists. |

Useful Bevy test/example sources for later test translation:

| Area | Reference files |
|---|---|
| Focus and focus events | `dev/bevy/crates/bevy_input_focus/src/lib.rs`, `dev/bevy/crates/bevy_input_focus/src/gained_and_lost.rs` |
| Tab navigation | `dev/bevy/crates/bevy_input_focus/src/tab_navigation.rs`, `dev/bevy/examples/ui/widgets/tab_navigation.rs` |
| Directional navigation | `dev/bevy/crates/bevy_input_focus/src/directional_navigation.rs`, `dev/bevy/crates/bevy_input_focus/src/navigator.rs`, `dev/bevy/examples/ui/navigation/directional_navigation.rs`, `dev/bevy/examples/ui/navigation/directional_navigation_overrides.rs` |
| Standard widgets | `dev/bevy/examples/ui/widgets/standard_widgets.rs`, `dev/bevy/examples/ui/widgets/standard_widgets_observers.rs` |
| Feathers | `dev/bevy/examples/ui/widgets/feathers.rs` |
| Editable text | `dev/bevy/examples/ui/text/editable_text.rs`, `dev/bevy/examples/ui/text/multiple_text_inputs.rs`, `dev/bevy/examples/ui/text/multiline_text_input.rs`, `dev/bevy/examples/ui/text/editable_text_filter.rs` |
| Accessibility | `dev/bevy/examples/ui/widgets/button.rs`, `dev/bevy/examples/ui/scroll_and_overflow/scroll.rs` |

## Zircon Baseline And Gaps

| Domain | Zircon evidence | Current baseline | M0 gap conclusion |
|---|---|---|---|
| UI schedule and dirty pipeline | `zircon_runtime/src/ui/surface/surface.rs`, `zircon_runtime_interface/src/ui/surface/timeline.rs` | `UiSurface` owns tree, arranged tree, hit grid, focus, input state, navigation state, render extract, render cache, and rebuild reports. Dirty rebuilds distinguish layout, arranged, hit-grid, render, reuse, damage, and timings. | M7 still needs named stage DTOs and stage-level dirty reports. Current rebuild flow is strong but method-driven: layout/arranged/hit/render are measured; input/focus/widget/text/a11y stages are not first-class schedule outputs. |
| Focus state | `zircon_runtime_interface/src/ui/surface/focus_state.rs`, `zircon_runtime_interface/src/ui/surface/navigation_state.rs`, `zircon_runtime_interface/src/ui/focus.rs`, `zircon_runtime/src/ui/surface/focus.rs`, `zircon_runtime/src/ui/surface/surface.rs`, `zircon_runtime/src/ui/tree/node/focus.rs` | `UiFocusState` tracks focused/captured/pressed/hovered plus M2 previous focus, pending autofocus, focus-visible policy, focus-change events, and focused-input route records. Runtime focus mutation is source-aware, resolves autofocus, records focused keyboard/navigation/text/IME bubbling routes, clears IME/capture/transient ownership on invalid focus targets, and reports hidden/disabled/despawn reasons for accepted tree changes. | M2 runtime focus core is implemented at the shared surface layer. Remaining later work is integration breadth: headless widget focus behavior, accessibility focus snapshots/actions, editor cutover, and product-specific focus restoration. |
| Tab and directional navigation | `zircon_runtime/src/ui/tree/node/focus.rs`, `zircon_runtime_interface/src/ui/surface/navigation_state.rs`, `zircon_runtime_interface/src/ui/navigation.rs` | Surface navigation now uses `UiNavigationContract` on `UiTreeNode`: tab index and group order for Next/Previous, non-modal cross-group traversal, modal traps that also constrain manual directional overrides, manual directional node/group targets, blocked edges, and spatial fallback from node centers. | M3 runtime traversal is implemented for core contracts. Remaining later work is richer first/last commands, layer/z-order popup fixtures, editor overlay cutover, and accessibility/widget action integration. |
| Picking and pointer routes | `zircon_runtime_interface/src/ui/surface/hit.rs`, `zircon_runtime_interface/src/ui/surface/pointer/route.rs`, `zircon_runtime_interface/src/ui/tree/node/input_policy.rs`, `zircon_runtime_interface/src/ui/picking.rs`, `zircon_runtime/src/ui/tree/hit_test.rs`, `zircon_runtime/src/ui/surface/surface.rs` | Hit test queries support surface/window/screen/world spaces, projected virtual pointer, scope ids, cursor radius, ordered stack, bubble route, debug reject reasons, clip/z/paint order, disabled and input-ignore filtering. Pointer routes carry target, hit path, stacked/entered/left/captured/pressed/click/focused data. M1 adds `UiPickPolicy`, `UiPickMode`, `UiPointerCapture`, and `UiPointerCaptureKind`. | M4 still needs behavior: applying the unified policy to hover/click/focus/capture/text hit/overlay/a11y route decisions and cutting over editor host routes. |
| Widget event contract | `zircon_runtime_interface/src/ui/component/event.rs`, `zircon_runtime_interface/src/ui/component/state.rs`, `zircon_runtime_interface/src/ui/widget.rs`, `zircon_runtime/src/ui/surface/surface.rs` | Component state has focused/hovered/pressed/popup/expanded/selected/checked/disabled flags and generic value map. Component events cover `ValueChanged`, `Commit`, focus, hover, press, popup, selection, visible range, page, and world surface events. M1 adds the separate headless `UiWidgetEvent::{Activate, ValueChange, TextEditChange, OpenChanged, SelectionChanged}` vocabulary and `UiWidgetContract`; it does not advertise those widget events as component-envelope kinds until a concrete bridge exists. Pointer release emits default `activated` and double-activated commits. | M5 still needs headless widget behavior so keyboard, pointer, a11y action, and text input converge on these typed events. Current behavior remains scattered across dispatch, component catalog, and Material assets. |
| Template schema and component focus contracts | `zircon_runtime_interface/src/ui/template/asset/component_contract/focus_contract.rs`, `zircon_runtime_interface/src/ui/template/asset/document.rs`, `zircon_runtime/src/ui/template/asset/compiler/node_expander.rs`, `zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs`, `zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs`, `zircon_runtime/src/ui/template/asset/schema/legacy_template.rs`, `zircon_editor/assets/ui/editor/material_meta_components.ui.toml` | Component public contracts can declare `root_focusable`, `initial_focus`, and public targets. Material meta components forward `input_focusable`, `disabled`, `checked`, `focused`, popup anchors, action ids, capability, and `accessibility_label` on many roots. M1 adds optional source `.ui.toml` node sections for `[focus]`, `[navigation]`, `[picking]`, `[a11y]`, and `[widget]`, and the runtime asset compiler preserves those sections into defaulted compiled `UiTemplateNode` contracts. Flat and legacy migration helpers preserve authored sections, but default legacy template values stay absent so migrated component instances do not erase component-root contracts. | Later milestones still need runtime behavior that applies these sections. Existing metadata remains adjacent; M1 only makes the structured schema and compiled contract spine durable. |
| Text layout | `zircon_runtime/src/ui/text/layout_engine.rs`, `docs/zircon_runtime/ui/text/layout_engine.md` | Runtime text layout is grapheme-aware, supports word/glyph/no-wrap, ellipsis, rich runs, low-fidelity bidirectional visual order, shared shaped DTO placeholders, selection/caret/composition decoration geometry, and renderer/editor consumer docs. | M6 still needs a real shaping backend abstraction and final metrics: font fallback, script shaping, glyph positioning, full BiDi, backend glyph ids, atlas/cache ownership, and measure/layout consistency under real fonts. |
| Text editing | `zircon_runtime_interface/src/ui/surface/render/editable_text.rs`, `zircon_runtime/src/ui/text/edit_state.rs`, `docs/ui-and-layout/shared-ui-input-events.md` | Editable state carries text, caret, selection, composition, read-only flag, and edit actions for insert/backspace/delete/move/set selection/composition/commit/cancel. Runtime applies text and IME events through shared input routing. | M6 needs word deletion/navigation, clipboard, max-char/filter/newline policy, cursor blink, mouse/point selection, multiline editing behavior, and typed `TextEditChange` widget event coverage. |
| Render extract and batches | `zircon_runtime_interface/src/ui/surface/render/extract.rs`, `zircon_runtime_interface/src/ui/surface/render/batch.rs`, `zircon_runtime/src/ui/surface/render/extract.rs`, `docs/zircon_runtime_interface/ui/surface/render.md` | Interface has `UiRenderExtract`, `UiRenderList`, paint elements, brushes, text shape, editable text, batch keys, split reasons, cache/debug/parity/visualizer DTOs. Runtime extract currently converts arranged draw order to `UiRenderCommand` list and feeds a render cache. | M8 must harden the runtime boundary so WGPU runtime and retained/native editor painters consume the same neutral extract and batch DTOs. Current interface contracts are ahead of runtime consumer integration. |
| Accessibility | `zircon_runtime_interface/src/ui/accessibility.rs`, `zircon_editor/assets/ui/editor/material_meta_components.ui.toml`, `zircon_runtime/src/ui/surface/render/resolve.rs`, `docs/ui-and-layout/material-ui-token-component-audit.md` | M1 adds neutral a11y DTOs: `UiAccessibilityNode`, `UiA11yRole`, `UiA11yState`, `UiAccessibilityAction`, diagnostics, `UiAccessibilityContract`, and `UiAccessibilityTreeSnapshot`. Adjacent metadata still exists: Material roots forward `accessibility_label`; `IconButton` label is treated as accessibility-only visual text in runtime material layout; docs record accessibility/callback follow-ups. | M9 still needs runtime snapshot generation, name-source precedence, hidden/disabled filtering, bounds/focus updates, action roundtrip, and AccessKit bridge mapping. |
| Styled editor widgets | `zircon_editor/assets/ui/editor/material_meta_components.ui.toml`, `docs/ui-and-layout/material-ui-token-component-audit.md` | Material token roles, state forwarding, root input metadata, SVG/icon sizing, native painter state priority, and component showcase tests exist. | M10 must become a Zircon editor widget kit layered on shared headless behavior. The current styled layer can provide tokens and presentation evidence, but not the behavior truth. |
| Editor cutover | `docs/ui-and-layout/slate-style-ui-surface-frame.md`, `docs/ui-and-layout/shared-ui-template-runtime.md`, `docs/ui-and-layout/index.md` | Existing docs show major editor host routes already consume shared surface frames, hit grid, render commands, template runtime, and Material meta components. | M11 must still delete duplicated local hover/popup/focus/pointer routing in workbench, asset editor, inspector, drawers, floating windows, viewport toolbar, and menus after M1-M10 contracts exist. |

## Target Test Table

| Milestone | Required tests | Existing starting points | Acceptance notes |
|---|---|---|---|
| M1 Contract Spine | Serde/default roundtrips for new focus/navigation/picking/widget/a11y/render DTOs; old `.ui.toml` defaults remain valid; missing a11y name diagnostics are representable. | `zircon_runtime_interface/src/tests/contracts.rs`, `zircon_runtime_interface/src/tests/render_contracts.rs`, `zircon_runtime_interface/src/tests/ui_layout.rs` | Keep this interface-only unless schema validation needs runtime fixtures. Do not change editor behavior in M1. |
| M2 Runtime Input Focus Core | Initial focus, explicit focus change, focus visible by keyboard/pointer source, gained/lost events, focused input bubbling, autofocus, hidden/disabled/despawn clearing focus and IME ownership. | `zircon_runtime/src/ui/tests/event_routing.rs`, `zircon_runtime/src/ui/tests/runtime_input_ownership.rs`, `zircon_runtime/src/ui/tests/shared_core.rs` | Add lower-layer focus tests before editor host tests. Existing `UiFocusState` tests are not enough without bubbling and cleanup evidence. |
| M3 Tab And Directional Navigation | Tab/Shift-Tab, first/last, tab index ordering, negative/non-tabbable skip, modal trap, nested group order, manual directional override, blocked edges, layer-aware spatial routing. | `zircon_runtime/src/ui/tests/event_routing.rs`, `zircon_runtime/src/ui/tests/shared_core.rs`, future `zircon_runtime/src/ui/tests/navigation.rs` | Include editor popup/overlay order as data fixtures, not editor-owned focus behavior. |
| M4 Picking And Pointer Route Convergence | Overlap, clip, scroll, virtual/world hit, text hit, pass/block/transparent/no-event policy, pointer capture, release outside, popover/menu overlay, viewport gizmo vs UI overlay priority. | `zircon_runtime/src/ui/tests/hit_grid.rs`, `zircon_runtime/src/ui/tests/pointer_click_semantics.rs`, `zircon_runtime/src/ui/tests/popup_tooltip_state.rs`, active runtime picking primitive plan tests | Coordinate with `zircon_runtime::core::framework::picking` work so UI route and scene/viewport route share ordering contracts instead of duplicate types. |
| M5 Headless Standard Widgets | Button activate, IconButton activate with accessible name, Checkbox/Radio value change, Slider clamp/step/drag cancel, Scrollbar scroll, TextInput edit/submit, Menu/MenuItem open/close/ESC/outside click, Popover/Tooltip lifecycle, disabled suppression. | `zircon_runtime/src/ui/tests/component_catalog.rs`, `zircon_runtime/src/ui/tests/component_catalog/*`, `zircon_runtime/src/ui/tests/event_routing.rs`, `zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs` | Tests must prove keyboard and pointer trigger the same widget event. Styled editor tests only come after headless behavior passes. |
| M6 Text To Bevy-Level | Shaper abstraction default behavior, font fallback fixture, measure/layout consistency, rich runs, grapheme/word delete, selection geometry, cursor blink state, clipboard action, IME preedit/commit/cancel, multiline, filters, max chars. | `zircon_runtime/src/ui/text/layout_engine/tests.rs`, `zircon_runtime/src/ui/tests/text_layout.rs`, `zircon_runtime_interface/src/tests/render_contracts.rs`, `zircon_editor` native text painter tests | Real shaping dependency entry requires license/platform/CI gate before becoming required. Until then, tests must mark scaffold vs backend expectations explicitly. |
| M7 UI Schedule And Dirty Pipeline | Stable stage order, per-stage dirty/report output, mouse-move hover-only path avoids full tree rebuild, text-measure dirtiness isolated, widget/a11y/render dirtiness isolated, debug timeline records stage stats. | `zircon_runtime/src/ui/tests/surface_dirty_domains.rs`, `zircon_runtime/src/ui/tests/timeline.rs`, `docs/ui-and-layout/slate-style-ui-surface-frame.md` | This promotes current rebuild reports into schedule-stage contracts. Do not regress incremental layout sessions. |
| M8 Render Boundary | Extract kinds for background/border/image/material/gradient/shadow/text/selection/cursor/debug overlay, clip/scissor, batch key and split reason parity, WGPU/editor painter consuming same DTO. | `zircon_runtime_interface/src/tests/render_contracts.rs`, `zircon_runtime` screen-space UI planning tests, `zircon_editor` native painter tests | Runtime command list compatibility can remain only as an internal migration path, not a second public render truth. |
| M9 Accessibility Runtime | Tree snapshot, role/name/state/action/bounds/parent-child, hidden/disabled filtering, focus update, name priority explicit a11y name -> label-for -> text -> tooltip, action roundtrip to widget events. | New `zircon_runtime_interface` a11y contract tests, new `zircon_runtime/src/ui/tests/accessibility.rs`, editor Material metadata fixtures | This is currently a gap, so M9 should start with interface DTO and snapshot tests before AccessKit bridge code. |
| M10 Feathers-like Styled Editor Widgets | Editor widget kit consumes headless behavior events, theme tokens apply without behavior fork, toolbar/inspector/tabs/menus/dock header/property row/color/search/tooltip/popover fixtures. | `zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs`, component showcase tests, native Material painter tests | Any failed styled test should first check M5 headless behavior and M8 render DTO before patching editor host glue. |
| M11 Editor Cutover And Productization | Workbench, asset editor, inspector, drawer, floating window, viewport toolbar, menus route through shared focus/picking/widgets/a11y/render contracts; duplicate local hover/popup/focus routes deleted. | `zircon_editor/src/tests/host/retained_window/*`, `zircon_editor/src/tests/host/template_runtime/*`, `docs/ui-and-layout/shared-ui-template-runtime.md` | This is a hard cutover milestone: no compatibility shim or parallel local route should remain once the shared path covers the workflow. |
| M12 Docs And Acceptance | Module docs updated for every changed source area, plan-source/test headers current, validation failures archived with owner and next action. | This document, `docs/ui-and-layout/index.md`, `docs/zircon_runtime_interface/ui/mod.md`, `docs/zircon_runtime_interface/ui/surface/render.md`, `docs/zircon_runtime/ui/text/layout_engine.md` | M12 must update both overview docs and source-path docs for all implementation milestones. |

## Docs Update List

| Document | Update needed after M0 | Later milestone owner |
|---|---|---|
| `docs/ui-and-layout/index.md` | Link this M0 audit so future UI work starts from the Bevy/Zircon gap matrix. | M0 |
| `docs/zircon_runtime_interface/ui/mod.md` | Add new contract files to `related_code` when M1 creates focus/navigation/picking/widget/a11y DTOs. | M1 |
| `docs/ui-and-layout/shared-ui-input-events.md` | Add focus-visible source policy, focused input bubbling, widget event integration, and text edit event flow once M2/M5/M6 land. | M2, M5, M6 |
| `docs/ui-and-layout/slate-style-ui-surface-frame.md` | Record unified `UiPickPolicy`, pointer capture DTO, text hit, viewport overlay competition, and a11y hit eligibility after M4. | M4 |
| `docs/zircon_runtime/ui/text/layout_engine.md` | Distinguish scaffold text layout from real `UiTextShaper` backend acceptance; record font fallback, shaping, clipboard/IME policy tests. | M6 |
| `docs/zircon_runtime_interface/ui/surface/render.md` | Update extract kind, batch, selection/cursor/debug overlay, and shared consumer status when M8 hardens the render boundary. | M8 |
| `docs/ui-and-layout/material-ui-token-component-audit.md` | Replace accessibility/callback follow-up language with links to headless widget behavior and a11y snapshot contracts as they land. | M5, M9, M10 |
| `docs/zircon_runtime/ui/surface/input.md` | Update focus ownership, pointer capture, tooltip/popover, and input-method cleanup details as M2-M5 converge. | M2-M5 |
| New `docs/zircon_runtime_interface/ui/accessibility.md` or mirrored source docs | Required when M1/M9 create a11y DTOs and runtime snapshot generation. | M1, M9 |
| New `docs/zircon_runtime/ui/widgets/*.md` or `docs/zircon_runtime/ui/widget_behavior.md` | Required when headless standard widget behavior lands. | M5 |

## M0 Acceptance Decision

M0 is accepted when this audit is present, linked from the UI docs index, and validated as docs-only content. The audit establishes these blocker facts for later milestones:

- Accessibility is not implemented beyond adjacent metadata; M1/M9 must start with neutral DTOs and runtime snapshot tests.
- Focus exists as surface state and component events, but Bevy-level input focus, bubbling, visibility, and cleanup are not first-class contracts.
- Picking/hit routing is strong but policy vocabulary is too narrow for the plan's unified hover/click/focus/a11y route.
- Text layout/editing has useful grapheme and IME scaffolds but needs a real shaper abstraction, backend metrics, clipboard, word operations, and input policies.
- Render interface DTOs are ahead of runtime/editor consumer integration; M8 must make the neutral extract and batch path the public truth.
- Styled Material editor widgets have token/state metadata, but M10 must layer them over headless widget behavior rather than treating editor presentation as behavior authority.

No workspace-wide Cargo acceptance is claimed for M0 because no Rust source code changes are part of this slice.
