---
related_code:
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs
  - zircon_runtime_interface/src/ui/template/document.rs
  - zircon_runtime_interface/src/ui/widget.rs
  - zircon_runtime/src/ui/accessibility/mod.rs
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/accessibility/name.rs
  - zircon_runtime/src/ui/accessibility/diagnostics.rs
  - zircon_runtime/src/ui/accessibility/action.rs
  - zircon_runtime/src/ui/accessibility/action/activate.rs
  - zircon_runtime/src/ui/accessibility/action/activate/fallback.rs
  - zircon_runtime/src/ui/accessibility/action/expanded.rs
  - zircon_runtime/src/ui/accessibility/action/expanded/target.rs
  - zircon_runtime/src/ui/accessibility/action/focus.rs
  - zircon_runtime/src/ui/accessibility/action/popup.rs
  - zircon_runtime/src/ui/accessibility/action/popup/tooltip.rs
  - zircon_runtime/src/ui/accessibility/action/range.rs
  - zircon_runtime/src/ui/accessibility/action/range/adjustment.rs
  - zircon_runtime/src/ui/accessibility/action/result.rs
  - zircon_runtime/src/ui/accessibility/action/result/binding.rs
  - zircon_runtime/src/ui/accessibility/action/scroll.rs
  - zircon_runtime/src/ui/accessibility/action/scroll/binding.rs
  - zircon_runtime/src/ui/accessibility/action/scroll/payload.rs
  - zircon_runtime/src/ui/accessibility/action/scroll/result.rs
  - zircon_runtime/src/ui/accessibility/action/target.rs
  - zircon_runtime/src/ui/accessibility/action/text.rs
  - zircon_runtime/src/ui/accessibility/action/text/replace.rs
  - zircon_runtime/src/ui/accessibility/action/text/replace/replacement.rs
  - zircon_runtime/src/ui/accessibility/action/text/replace/result.rs
  - zircon_runtime/src/ui/accessibility/action/text/selection.rs
  - zircon_runtime/src/ui/accessibility/action/text/selection/payload.rs
  - zircon_runtime/src/ui/accessibility/action/text_state.rs
  - zircon_runtime/src/ui/accessibility/action/text_state/metadata.rs
  - zircon_runtime/src/ui/accessibility/action/value.rs
  - zircon_runtime/src/ui/accessibility/action/value/payload.rs
  - zircon_runtime/src/ui/accessibility/action/value/result.rs
  - zircon_runtime/src/ui/accessibility/action/value/text.rs
  - zircon_runtime/src/ui/accessibility/action/value_target.rs
  - zircon_runtime/src/ui/accessibility/accesskit.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/popup.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs
  - zircon_runtime/src/ui/surface/component_state.rs
  - zircon_runtime/src/ui/surface/interaction_gate.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/text_constraints.rs
  - zircon_runtime/src/ui/tests/accessibility_disabled_gate.rs
  - zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs
  - zircon_runtime/src/ui/tests/accessibility_widget_actions.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
implementation_files:
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs
  - zircon_runtime/src/ui/accessibility/mod.rs
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/accessibility/name.rs
  - zircon_runtime/src/ui/accessibility/diagnostics.rs
  - zircon_runtime/src/ui/accessibility/action.rs
  - zircon_runtime/src/ui/accessibility/action/activate.rs
  - zircon_runtime/src/ui/accessibility/action/activate/fallback.rs
  - zircon_runtime/src/ui/accessibility/action/expanded.rs
  - zircon_runtime/src/ui/accessibility/action/expanded/target.rs
  - zircon_runtime/src/ui/accessibility/action/focus.rs
  - zircon_runtime/src/ui/accessibility/action/popup.rs
  - zircon_runtime/src/ui/accessibility/action/popup/tooltip.rs
  - zircon_runtime/src/ui/accessibility/action/range.rs
  - zircon_runtime/src/ui/accessibility/action/range/adjustment.rs
  - zircon_runtime/src/ui/accessibility/action/result.rs
  - zircon_runtime/src/ui/accessibility/action/result/binding.rs
  - zircon_runtime/src/ui/accessibility/action/scroll.rs
  - zircon_runtime/src/ui/accessibility/action/scroll/binding.rs
  - zircon_runtime/src/ui/accessibility/action/scroll/payload.rs
  - zircon_runtime/src/ui/accessibility/action/scroll/result.rs
  - zircon_runtime/src/ui/accessibility/action/target.rs
  - zircon_runtime/src/ui/accessibility/action/text.rs
  - zircon_runtime/src/ui/accessibility/action/text/replace.rs
  - zircon_runtime/src/ui/accessibility/action/text/replace/replacement.rs
  - zircon_runtime/src/ui/accessibility/action/text/replace/result.rs
  - zircon_runtime/src/ui/accessibility/action/text/selection.rs
  - zircon_runtime/src/ui/accessibility/action/text/selection/payload.rs
  - zircon_runtime/src/ui/accessibility/action/text_state.rs
  - zircon_runtime/src/ui/accessibility/action/text_state/metadata.rs
  - zircon_runtime/src/ui/accessibility/action/value.rs
  - zircon_runtime/src/ui/accessibility/action/value/payload.rs
  - zircon_runtime/src/ui/accessibility/action/value/result.rs
  - zircon_runtime/src/ui/accessibility/action/value/text.rs
  - zircon_runtime/src/ui/accessibility/action/value_target.rs
  - zircon_runtime/src/ui/accessibility/accesskit.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/popup.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs
  - zircon_runtime/src/ui/surface/component_state.rs
  - zircon_runtime/src/ui/surface/interaction_gate.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/text_constraints.rs
  - zircon_runtime/src/ui/tests/accessibility_disabled_gate.rs
  - zircon_runtime_interface/src/ui/widget.rs
plan_sources:
  - docs/superpowers/plans/2026-05-09-accesskit-bridge.md
  - docs/superpowers/specs/2026-05-08-accesskit-bridge-design.md
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
  - user: 2026-05-09 Milestone 2 Accessibility Action Dispatch Through Shared UI Behavior
  - user: 2026-05-09 Milestone 3 Runtime ABI Snapshot Capture And Serialized Action Roundtrip
  - user: 2026-05-16 Bevy-level UI/Text/Widgets/Focus/A11y completion plan continuation
tests:
  - zircon_runtime/src/dynamic_api/tests.rs
  - zircon_runtime/src/ui/tests/accessibility.rs
  - zircon_runtime/src/ui/tests/accessibility_disabled_gate.rs
  - zircon_runtime/src/ui/tests/accessibility_state_values.rs
  - zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs
  - zircon_runtime/src/ui/tests/accessibility_widget_actions.rs
  - zircon_runtime/src/ui/tests/widget_radio_behavior.rs
  - zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs
  - zircon_runtime/src/ui/tests/accesskit.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
  - cargo test -p zircon_runtime --lib dynamic_api --locked --jobs 1 --target-dir "E:\\cargo-targets\\zircon-accesskit-bridge" --message-format short --color never
  - cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\\cargo-targets\\zircon-accesskit-bridge" --message-format short --color never
doc_type: module-detail
---

# Runtime UI Accessibility

`zircon_runtime::ui::accessibility` extracts a neutral `UiAccessibilityTreeSnapshot` from an existing `UiSurface` and maps neutral accessibility action requests into existing runtime UI behavior. The dynamic runtime API serializes the neutral snapshot/action DTOs across the `zircon_app` ABI boundary. With the `accessibility-accesskit` feature enabled, the module also converts neutral snapshots into AccessKit `TreeUpdate` values and converts supported AccessKit action requests back into neutral `UiAccessibilityActionRequest` values. Winit/app-host adapter ownership remains outside this module.

## Extraction Source

`UiTemplateNodeMetadata` retains `UiAccessibilityContract` and `UiWidgetContract` with serde defaults. Template tree building copies `UiTemplateNode.a11y` and `UiTemplateNode.widget` into retained tree metadata, so snapshot extraction reads from the same `UiTree` used by layout, hit testing, render extraction, and focus.

`UiSurface::accessibility_snapshot()` delegates to `crate::ui::accessibility::accessibility_snapshot(self)` and does not mutate layout or focus state. `surface.rs` remains an oversized retained UI owner file; Milestone 1 only added this narrow delegating method and did not refactor its existing responsibilities.

Widget behavior is also an accessibility source. When `UiWidgetContract::behavior` is explicit, extraction uses it before component-name fallback: `Button` and `MenuItem` expose activation, `Toggle` exposes a checkbox role, `RadioGroup` exposes a radio-group role, `Radio` exposes a radio role with activate action, `Range` exposes a slider role with increment/decrement/set-value actions, `TextInput` exposes a text-input role with set-value and text-editing actions, and `Disclosure` plus generic `Popup` controls expose activation plus a state-sensitive expand or collapse action. A `Popup` whose explicit role is `Dialog` is treated as a popup owner instead of a button-like opener: while it is expanded/open it exposes default `Dismiss` and does not synthesize `Activate`, `Expand`, or `Collapse`. A `Popup` whose explicit role is `Menu` exposes only state-sensitive `Expand` or `Collapse`, so menu containers do not emit unsupported button-style activation. Tooltip roles also expose `Dismiss` so AccessKit `HideTooltip` can route through the runtime effect boundary instead of host-local state. `UiWidgetBehavior::Auto` preserves the old component-name inference path for legacy templates; `Passive` suppresses default role/action inference for structural components.

This mirrors Bevy's separation between headless behavior and accessibility metadata. Local Bevy `dev/bevy/crates/bevy_ui_widgets/src/{button.rs,checkbox.rs,slider.rs}` attach AccessKit roles to behavior components, while `dev/bevy/crates/bevy_a11y/src/lib.rs` keeps AccessKit node representation reusable. Zircon keeps a neutral tree first so runtime/editor hosts share extraction semantics and only the optional `accessibility-accesskit` bridge performs AccessKit conversion.

Radio extraction follows local Bevy `dev/bevy/crates/bevy_ui_widgets/src/radio.rs`, which gives groups and radio buttons distinct AccessKit roles and lets activation route through the same headless widget path. The neutral tree now has `UiA11yRole::RadioGroup` and maps it to `accesskit::Role::RadioGroup` when the optional bridge is enabled; `Radio` keeps checked-state extraction through authored `checked_property`, retained attributes, and runtime component-state values.

Scrollbar extraction follows local Bevy `dev/bevy/crates/bevy_ui_widgets/src/scrollbar.rs`: a scrollbar widget is headless control chrome for a scrollable container, so `UiWidgetBehavior::Scrollbar` and `ScrollbarThumb` do not infer a role, name, or default action and are excluded from the neutral tree unless the author supplies explicit a11y metadata. The scrollable container itself can expose `ScrollTo`, which keeps assistive scroll behavior on the content owner rather than on decorative track/thumb nodes.

## Inclusion Rules

The extractor includes visible roots and nodes when they have explicit accessibility metadata, a non-default `UiWidgetContract`, focus/pointer/widget interactivity, text metadata, image/icon alt metadata, or are relation targets from participating owners. Relation targets are collected only from owners that are themselves included without relation-target promotion, so references owned only by excluded hidden subtrees do not retain otherwise anonymous targets. Normal hidden nodes and ordinary descendants of hidden ancestors are excluded. Hidden nodes referenced as labels or descriptions are retained as label-only nodes with no children or actions only when they provide usable source text. Hidden textless description targets are pruned after resolution when the owner description is cleared. Hidden `label_for` and widget `label_for` targets are not retained as label/description source nodes unless they are independently includable. Hidden relation-only nodes are not exposed through normal parent `children` traversal.

Disabled nodes remain discoverable, and disabled state is inherited from ancestors. Extraction delegates this test to the shared runtime interaction-gate helper, so it uses the same sources as input owner validation and widget behavior: retained enabled flags, component-state `disabled`/inverse `enabled` values, canonical runtime disabled flags, retained `disabled` attributes, and `UiWidgetContract.disabled` on the node or any ancestor. Invalid actions are filtered during extraction; currently only `Focus` is retained for disabled focusable nodes. The extractor records `DisabledAction` diagnostics for the filtered actions so the output tree is safe while preserving validation evidence.

Children are filtered through the included-node map and preserve retained UI tree child order. Anonymous or otherwise excluded visible intermediate containers do not break reachability; their included descendants are promoted to the nearest included ancestor. Hidden excluded containers block descendant promotion, and ordinary descendants under hidden ancestors are excluded from the snapshot unless they are explicit label/description relation targets. Node output is built through a `BTreeMap<UiNodeId, UiAccessibilityNode>` before becoming the snapshot vector, so output order is deterministic by node id.

## Name Priority

Accessible names resolve in this order:

1. Explicit `UiAccessibilityContract.name`.
2. `labelled_by` target name or text.
3. Own string attributes: `text`, `label`, then `value`.
4. Alt metadata: `accessibility_label`, `alt`, `alt_text`, then `icon_alt`.
5. Tooltip fallback from the accessibility contract, widget contract, or `tooltip` attribute.

The string resolver accepts string TOML values and scalar numeric/bool values. Empty strings do not produce names.

Tooltip-only nodes are included because tooltip text is the final accessible-name fallback for nodes without explicit names, labels, own text, or alt text.

Description references that use `#<node-id>` resolve during extraction using the same direct target text fallback as `labelled_by`: explicit accessibility name, own text, alt text, accessibility-contract tooltip, widget tooltip, then tooltip attribute text. Description references are parsed exactly once after the first `#`, so values such as `##2` are malformed instead of being double-stripped. Direct target lookup only accepts nodes already retained in the snapshot and does not depend on target `name` resolution order. If an internal reference is malformed, points outside the retained snapshot, or points at a retained node with no usable accessible text, extraction clears the raw `#...` description and records `DanglingDescription` so unresolved ids do not leak to accessibility hosts.

## State And Bounds

`hidden` follows effective retained node visibility. `disabled` walks the node and ancestors through `ui_surface_effective_disabled(...)`, combining runtime enabled state, typed component-state `disabled`/`enabled` values, canonical runtime disabled flags, retained `disabled` attributes, and `UiWidgetContract.disabled`; this keeps a11y extraction aligned with runtime input-owner validation and the shared widget interaction gate. `focused` is true only when the surface focus points at a visible, enabled, included node. `selected` reads the retained `selected` attribute before typed component-state `selected` values and canonical selected flags, so menu/list/tab-like authored controls can expose selection without a component-name special case and unrelated component-state records cannot mask authored selection. `expanded` reads the authored disclosure/popup `open_property` attribute first, then the same property in runtime component state; legacy extraction falls back to retained/component-state `expanded`, `popup_open`, or `open` values and then true runtime expanded/popup flags. This keeps custom disclosure aliases from being masked by unrelated component-state entries. `checked` reads the authored `checked_property` attribute first, then component-state values for that alias, runtime component checked flags, and widget/static state, so custom toggles expose the same checked state that pointer, keyboard, accessibility activation, or code-side component state mutate. `pressed` reads typed component-state `pressed`/`active` values, canonical pressed flags, retained `pressed`/`active` attributes, and legacy node state flags, matching the headless button press model used by runtime pointer routing. `value` reads the authored `value_property` attribute first, then the same property in runtime component state; legacy extraction reads retained `value` and text-input `text` attributes before component-state `value`/`text`, then falls back to `UiWidgetContract.value` projected with `UiValue::display_text()` so assistive hosts receive user-facing text instead of Rust debug formatting.

TextInput nodes also expose optional `state.text_selection` through the neutral `UiA11yTextSelection` contract. Extraction reads retained `caret_offset`, `selection_anchor`, and `selection_focus` attributes first, then the same typed values from `UiSurfaceComponentStateStore`; missing selection endpoints collapse to the caret, and missing caret defaults to the exposed text length. Offsets are byte offsets in the same `state.value` string and are clamped to the text length and a valid UTF-8 boundary before the snapshot is emitted, so keyboard, pointer, IME, and a11y hosts do not need separate caret geometry estimates.

Bounds come from the arranged tree first and fall back to the retained node layout cache when the arranged tree has not been rebuilt yet. Bounds must be finite with positive width and height. Named or interactive visible nodes without valid arranged or layout-cache bounds stay in the snapshot but receive `MissingBounds` diagnostics. Hidden relation-only nodes retained only as label/description sources do not emit `MissingBounds` noise.

## Diagnostics

The extraction and validation passes check malformed label references, duplicate ids, dangling label/description references, missing names, missing bounds, hidden focusable nodes, disabled invalid actions, unsupported role/action pairs, invalid focus, excluded focus, and simple two-node `labelled_by` cycles. Malformed `labelled_by`/`label_for` strings record `InvalidLabelReference`; malformed `#` description references record `DanglingDescription`. Interactive, focusable, or actionable nodes without resolved accessible names record `MissingName`. Hidden focusable nodes that are excluded from normal traversal record `HiddenFocusable` even when they are also focused; hidden focused nodes also produce `ExcludedFocusedNode` through focus validation. The role/action whitelist treats `Dialog + Dismiss` and `Menu + Expand/Collapse` as supported, so dialog and menu popup containers can expose their default close/open actions without producing `UnsupportedRoleAction` diagnostics.

If focus points at a missing, hidden, disabled, or excluded node, the snapshot records an error diagnostic and falls back to the first visible enabled snapshot root, or clears focus if no valid fallback exists. The validation pass also synchronizes `state.focused` so exactly the snapshot focus target is marked focused, without mutating `UiSurface.focus`.

## Action Dispatch

`UiInputEvent::Accessibility` is routed through `UiSurface::dispatch_input_event` into `dispatch_accessibility_action`. The dispatcher first captures a fresh accessibility snapshot and validates the requested target against the snapshot node list, hidden state, disabled state, and the requested behavior. Disabled snapshot targets reject every non-`Focus` action before role-specific dispatch, so `Activate`, `SetValue`, text-selection edits, range adjustments, popup state changes, and scroll mutations cannot bypass the shared disabled gate or property-alias pipeline. It returns ordinary `UiInputDispatchResult` values, so action status is encoded in `diagnostics.notes` with strings such as `status=accepted`, `status=rejected`, `status=unsupported`, `status=stale_target`, and role/error codes.

The top-level `action.rs` module remains the route validator and snapshot capture boundary. Snapshot-target diagnostics and target availability gates live in `action/target.rs`, including included-node hidden/disabled rejections, stale runtime nodes, hidden-but-excluded nodes, and snapshot-excluded targets. Focus dispatch lives in `action/focus.rs`, where programmatic focus requests route through `UiSurface::focus_node_with_reason`; Activate dispatch lives in `action/activate.rs`, where typed widget activation is attempted first; `action/activate/fallback.rs` owns the generic `Commit` fallback event construction used when no typed widget behavior handles activation. Shared action result helpers live in `action/result.rs`: handled/unhandled status notes, unsupported-role replies, and action status labels are centralized so all action submodules emit the same `UiInputDispatchResult` vocabulary. `action/result/binding.rs` owns the property-mutation binding report diagnostics that append applied/unchanged/rejected counts and the first binding source note. Expandable state branches live in `action/expanded.rs`: `Expand`/`Collapse` action exposure checks, expanded-state availability checks, target lookup, and property mutation dispatch are grouped there so disclosure state is not coupled to popup dismissal. `action/expanded/result.rs` owns accepted/unchanged/rejected mutation result shaping, binding-report diagnostics, and component-event emission. `action/expanded/target.rs` owns Disclosure/Popup widget matching, authored `open_property` resolution, and popup/disclosure component-event mapping. Popup dismissal branches live in `action/popup.rs`: popup owner `Dismiss` lookup, popup close mutation dispatch, tooltip fallback routing, and unsupported fallback are isolated from expandable-state mutation. `action/popup/result.rs` owns accepted/unchanged/rejected popup close mutation result shaping, binding-report diagnostics, and `ClosePopup` component-event emission. Tooltip hide routing lives in `action/popup/tooltip.rs`: Tooltip role matching, runtime tooltip-state lookup, and `UiDispatchEffect::Tooltip { kind: Hide }` dispatch remain separate from popup owner mutation. Range-adjust action branches live in `action/range.rs`: Slider `Increment`/`Decrement` action exposure checks, role validation, and range-step mutation dispatch are isolated from route validation. `action/range/adjustment.rs` owns increment/decrement direction parsing and `ValueChanged` component-event construction. `action/range/result.rs` owns accepted/unchanged/rejected mutation result shaping and binding-report diagnostics for range-step mutations. Scroll action branches live in `action/scroll.rs`: `ScrollTo` action exposure checks, offset payload reuse, previous-offset capture, and runtime scroll mutation dispatch are grouped together. `action/scroll/binding.rs` owns `AccessibilityAction -> RuntimeState(scroll_offset)` binding report construction and scroll-state offset reads. `action/scroll/payload.rs` owns numeric/string offset payload parsing plus neutral point-to-axis projection for `ScrollableBox`. `action/scroll/result.rs` owns accepted/unchanged/rejected `ScrollTo` mutation result shaping, unchanged diagnostic notes, and scroll binding report dispatch. Whole-field value branches live in `action/value.rs`: `SetValue` action exposure checks, role/property/payload gates, TextInput preparation delegation, and runtime property mutation are grouped there. `action/value/result.rs` owns accepted/unchanged/rejected mutation result shaping, binding-report diagnostics, TextInput edit metadata sync, and value-change component-event construction. `action/value/text.rs` owns TextInput read-only rejection, whole-field retained constraint sanitization, and `accessibility_text_value_sanitized` note selection for `SetValue`. `action/value/payload.rs` owns role-specific `SetValue` payload parsing for TextInput strings/numeric text fallback and Slider finite float conversion. Shared mutable value target resolution lives in `action/value_target.rs`: authored `value_property`, component-state backed values, retained widget defaults, and fallback `value`/`text` attributes are resolved once for both whole-field and selected-text mutations. Text-edit action routing lives in structural `action/text.rs`; selected-range replacement dispatch lives in `action/text/replace.rs`, while `action/text/replace/replacement.rs` owns selected-range next-text construction, constraint-note calculation, and caret collapse offsets. `action/text/replace/result.rs` owns accepted/unchanged/rejected `ReplaceSelectedText` mutation result shaping, binding-report diagnostics, edit metadata sync, and value-change component-event construction. Neutral `SetTextSelection` lives in `action/text/selection.rs`; `action/text/selection/payload.rs` owns missing-payload rejection metadata plus caret/anchor/focus UTF-8 boundary clamping for that action. TextInput retained-state helpers live in `action/text_state.rs`: read-only metadata checks, selected-range calculation, UTF-8 boundary helpers, and caret/selection synchronization entry points are shared by `SetValue`, `ReplaceSelectedText`, and `SetTextSelection`. `action/text_state/metadata.rs` owns retained metadata mutation, binding-report diagnostics, and composition cleanup for those synchronization paths. These splits are structural; they preserve the same binding report, component event, and diagnostic vocabulary.

Accepted `Focus` actions call `UiSurface::focus_node_with_reason` with `UiFocusChangeReason::Programmatic` and visible focus reason `UiFocusVisibleReason::Programmatic`. Successful focus dispatch sets `diagnostics.routed = true`, `route_target = Some(target)`, and `handled_phase = "accessibility.focus"`.

Accepted `Activate` actions use the existing component event and widget behavior vocabulary instead of a host-only branch. When the current snapshot target exposes `Activate` and is not disabled or hidden, dispatch first tries the same typed default widget behavior used by focused keyboard activation: toggles mutate their checked property using retained attributes or component-state alias values as the current state, disclosure and popup controls mutate their authored open property using retained/component-state alias values as the current state, and button/menu bindings can receive commit events. MenuItem activation also reuses the keyboard/pointer popup-close path, so a menu item inside an open popup closes its nearest popup ancestor even when the item itself has no activation binding; the popup close mutation still returns a `WidgetBehavior` binding report. If no typed behavior handles the target, dispatch preserves the existing generic button-compatible event by emitting `UiComponentEvent::Commit { property: "activated", value: UiValue::Bool(true) }`. Handled activation records phase `accessibility.activate`.

Accepted `Expand` and `Collapse` actions are limited to disclosure and popup widget behavior targets that expose expandable state in the fresh snapshot. Extraction emits only one of the two actions at a time: collapsed controls expose `Expand`, while expanded controls expose `Collapse`. Generic popup openers expose activation plus expand/collapse, Menu-role popups expose expand/collapse without activation, and Dialog-role popups are intentionally excluded from this opener contract and instead expose `Dismiss` while open. Dispatch writes the widget `open_property` alias, falling back to `expanded` for disclosures and `popup_open` for popups, through `UiSurface::mutate_property` with the `AccessibilityAction` binding source. Accepted disclosure actions emit `UiComponentEvent::ToggleExpanded { expanded }`; accepted popup actions emit `OpenPopup` or `ClosePopup`. This keeps assistive open/close requests on the same property alias, dirty-domain, component-state mirror, disabled gate, and event vocabulary used by pointer, keyboard, and default widget activation.

Accepted `Dismiss` actions first close the nearest open default popup owner for the requested target. An explicit Dialog-role `Popup` reaches this path through default extraction when it is open, so dialog close requests use the same owner lookup as Escape/menu dismissal rather than a dialog-special host branch. The runtime popup helper resolves the open popup ancestor, but it only treats owners that pass the ancestor-aware shared widget interaction gate as closeable; a disabled popup owner or a popup under a disabled ancestor therefore cannot be closed by an enabled descendant's accessibility action. When a closeable owner exists, action dispatch writes that popup's `open_property` alias to `false` through `UiSurface::mutate_property` with the `AccessibilityAction` binding source. Accepted popup dismissals route to the popup owner, emit `UiComponentEvent::ClosePopup`, and preserve retained-attribute plus component-state mirroring. If no open popup owner is found and the snapshot target is a tooltip role while runtime has an active tooltip state, dispatch applies a shared `UiDispatchEffect::Tooltip { kind: Hide }`, clears `UiSurfaceInputState.tooltip`, and emits the matching host tooltip-hide request. If neither owner exists, dispatch keeps the existing unsupported note `accessibility dismiss requires popup id`, so dialog/tooltip contracts that are not backed by an enabled runtime popup owner or tooltip state fail explicitly instead of inventing host-local close behavior.

Accepted `SetValue` actions are limited to `TextInput` and `Slider` roles that already expose a mutable value property. If `UiWidgetContract::value_property` is authored, dispatch uses that alias when it exists as a retained attribute, runtime component-state value, or static widget value; this keeps custom range and text controls on the same property alias used by pointer, keyboard, and accessibility extraction. Without an authored alias, dispatch preserves legacy compatibility by mutating existing `value` first, otherwise existing `text`. It does not create a new fallback property solely because the role is editable. Slider values must be finite floats. TextInput values reject `read_only`/`input_read_only` fields before mutation and sanitize whole-field replacement text through the same retained `text_constraints.rs` rules used by keyboard, text, and IME routes: `input_filter`/`text_filter`, `max_graphemes`/`max_chars`/`max_length`, and `multiline = false`. The mutation goes through `UiSurface::mutate_property` with `UiReflectedPropertySource::RuntimeState`; accepted mutations now also mirror the typed value into `UiSurfaceComponentStateStore` before boolean pseudo-state flags are synchronized. Accepted TextInput string replacements also mutate retained `caret_offset`, `selection_anchor`, and `selection_focus` to the new string length and clear `composition_start`, `composition_end`, `composition_text`, and `composition_restore_text` through the same accessibility-action binding source, so render extraction, input state, IME metadata, and the neutral a11y snapshot agree that the field contains only the replacement text and a collapsed selection at the replacement end. Accepted mutations emit `UiComponentEvent::ValueChanged` for the mutated value property and use phase `accessibility.set_value`; rejected mutations return structured rejection notes instead of direct metadata writes.

Accepted `ReplaceSelectedText` actions are limited to `TextInput` roles. They use the current neutral `state.text_selection` range from the snapshot, sanitize the replacement payload with the same retained TextInput constraints against that selected range, mutate only the selected span of the value property, collapse caret/selection to the end of the inserted replacement, and clear composition metadata through the same accessibility-action binding reports. This keeps AccessKit `ReplaceSelectedText` distinct from whole-field `SetValue` while still sharing the same value-property alias, read-only gate, constraint parser, dirty domains, and component event path.

Accepted `SetTextSelection` actions are also limited to `TextInput` roles. They consume the neutral `request.text_selection` payload through `action/text/selection/payload.rs`, clamp caret/anchor/focus independently to valid byte offsets in the current exposed value, write `caret_offset`, `selection_anchor`, and `selection_focus` through `UiSurface::mutate_property` with the `AccessibilityAction` binding source, and clear active composition metadata at the new caret offset. This action is allowed for read-only TextInput nodes because it does not mutate the value; the existing hidden and disabled gates still reject non-focus actions before the TextInput-specific branch.

Unsupported or rejected behavior remains explicit. Stale targets return `status=stale_target` without routing to a runtime node. Hidden or snapshot-excluded targets return `status=rejected` with `hidden_target` or `excluded_target`. Disabled non-focus requests return `status=rejected code=disabled_action`. `Increment` and `Decrement` are accepted only for slider-like targets that expose the range value contract used by `mutate_default_range_step_value`; `value_property`, `min_property`, `max_property`, and `step_property` resolve retained attributes first, then runtime component-state values. `ScrollTo` is accepted only for nodes whose retained container is scrollable and whose snapshot exposes the action; it consumes `numeric_value`, a parseable string `value`, or a neutral `scroll_offset` point. The current runtime scroll model is single-axis, so `scroll_offset.y` is used for vertical `ScrollableBox` containers and `scroll_offset.x` is used for horizontal containers. Dispatch then delegates to `UiRuntimeTreeScrollExt::set_scroll_offset` so clamping, dirty flags, and virtual-window invalidation remain owned by the scrollable container. Other roles return `status=unsupported code=unsupported_role_action`. `Dismiss` keeps the exact unsupported note `accessibility dismiss requires popup id` when the target is not inside an open runtime popup and no active runtime tooltip can be hidden.

## AccessKit Bridge

`zircon_runtime/src/ui/accessibility/accesskit.rs` is compiled only behind `accessibility-accesskit`, which depends on `accesskit 0.22` without default features. The bridge deliberately consumes the neutral snapshot instead of walking `UiSurface` directly, so AccessKit platform adapters cannot bypass Zircon's existing inclusion, name, bounds, diagnostics, focus, and disabled/hidden filtering rules.

`snapshot_to_accesskit_tree_update(...)` maps `UiAccessibilityTreeSnapshot` into an AccessKit full-tree `TreeUpdate`. Single-root snapshots use the Zircon root id directly. Multi-root snapshots gain a synthetic AccessKit `Window` root with `NodeId(u64::MAX)` and the Zircon roots as children, keeping AccessKit's single-root tree requirement separate from the neutral snapshot format. Focus falls back to the AccessKit root when the neutral focused id is absent from the emitted node list.

Role mapping follows Bevy's AccessKit precedent where practical: buttons/images/labels become AccessKit button/image/label-style nodes, editor panels become `Pane`, text inputs become `TextInput`, and sliders/checkboxes/radio buttons/menu items/tabs map to their native AccessKit roles. Zircon `Text` nodes expose their name as AccessKit `value`, matching Bevy's label behavior; other controls expose names as AccessKit `label`.

State and relation mapping preserves the neutral contract: hidden and disabled become AccessKit flags, selected and expanded become AccessKit boolean properties, checked maps to `Toggled::{False, True, Mixed}`, bounds become AccessKit `Rect`, child ids become `children`, `labelled_by` becomes `labelled_by`, and `label_for` becomes `controls`. String widget values become AccessKit `value`; finite numeric strings additionally populate `numeric_value` for controls such as sliders.

Neutral TextInput `state.text_selection` is intentionally not mapped into AccessKit node state yet. AccessKit text positions require a text-run shaped subtree, while Zircon currently emits a compact widget-level snapshot; the bridge should add snapshot-side selection mapping only after runtime text shaping exposes stable TextRun or shaped-line node ids.

Supported AccessKit actions map back into neutral requests before runtime dispatch. `Click`, `Focus`, `Increment`, `Decrement`, `SetValue`, `ReplaceSelectedText`, `SetTextSelection`, `Expand`, `Collapse`, `ScrollIntoView`, `SetScrollOffset`, `Blur`, and `HideTooltip` are accepted. `SetValue` remains whole-field replacement, `ReplaceSelectedText` maps to the neutral selected-range edit action, and `SetTextSelection` converts AccessKit anchor/focus character indexes through the current neutral snapshot text value before filling Zircon's byte-offset `UiA11yTextSelection` payload. If the referenced text node is missing, lacks exposed text, or spans multiple AccessKit text nodes, the bridge returns `None` instead of guessing byte offsets. `Expand`/`Collapse` stay distinct from dismiss semantics, `Blur`/`HideTooltip` map to neutral `Dismiss`, and `SetScrollOffset` copies the AccessKit point into the neutral `scroll_offset` payload. Neutral `Dismiss` export is role-aware: Dialog nodes expose AccessKit `Blur`, Tooltip nodes expose AccessKit `HideTooltip`, and unsupported role/action pairs do not synthesize a platform dismiss action. Neutral `Dismiss` dispatch then routes either to popup property mutation or to the shared tooltip hide effect when a tooltip role and active runtime tooltip state exist. `ScrollToPoint` remains unsupported because AccessKit defines it in platform/tree-container coordinates, not the retained scroll-content offset owned by Zircon's `ScrollableBox`. Value and numeric payloads are copied into the neutral request. Unsupported AccessKit-only actions return `None` so the eventual app-host adapter can decline them without inventing hidden runtime behavior.

## Dynamic Runtime ABI

`zircon_runtime/src/dynamic_api/exports.rs` now exposes `ZrRuntimeApiV1.capture_accessibility_tree` for hosts that understand the appended optional ABI field. The function validates `ZrRuntimeAccessibilityTreeRequestV1.abi_version`, rejects non-default viewport handles with `NotFound`, captures the current dynamic preview accessibility snapshot, serializes it as JSON `UiAccessibilityTreeSnapshot`, and writes the bytes into `ZrOwnedByteBuffer`.

Accessibility tree byte ownership mirrors frame byte ownership but uses a distinct owner token, `0x5a52_4131_3159_0001`, and a dedicated `free_runtime_accessibility_bytes` callback. Null output pointers return `InvalidArgument` with the diagnostic `missing accessibility tree output`. Invalid ownership, null data, or impossible length/capacity pairs are rejected by the accessibility free callback as invalid runtime accessibility buffers. Existing frame capture and frame byte reclamation remain unchanged.

The dynamic preview session currently owns the 3D runtime preview state and render bridge, not a retained `UiSurface`. For that reason `RuntimeDynamicSession::capture_accessibility_tree` returns a minimal neutral preview snapshot instead of fake widget data: one `Panel` root named `Zircon Runtime Preview`, no focused widget, no children, and an info diagnostic with the exact message `runtime UI surface accessibility extraction unavailable in dynamic preview`.

`RuntimeDynamicSession::handle_event` handles `ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1` by deserializing `UiAccessibilityActionRequest` from `ZrRuntimeEventV1.payload`. Invalid JSON returns `InvalidArgument` with `invalid accessibility action payload`. Valid requests deserialize successfully, then return `NotFound` with `runtime UI surface accessibility action dispatch unavailable in dynamic preview` because there is no stored `UiSurface` to dispatch through in this runtime preview path. The runtime UI action dispatcher remains the owner of stale-target, hidden-target, disabled-action, and unsupported-action semantics when a retained surface is available.

## Focused Tests

`zircon_runtime/src/ui/tests/accessibility.rs` covers:

- extraction of interactive, text, alt, and explicit accessibility nodes;
- widget-only inclusion from non-default `UiWidgetContract`;
- role and action inference from explicit `UiWidgetBehavior` without relying on a known component name;
- value snapshot state from runtime component state and authored aliases;
- checked snapshot state from runtime component state and authored aliases;
- disabled snapshot state and invalid-action filtering from retained `disabled` attributes;
- selected snapshot state from retained `selected` attributes;
- pressed snapshot state from retained `active`/`pressed` attributes and runtime component state;
- name priority across explicit, `labelled_by`, own text, alt, and tooltip sources;
- order-independent `labelled_by` resolution from higher-id tooltip-only targets;
- hidden label references;
- hidden `label_for` targets staying excluded unless independently includable;
- description reference resolution from retained hidden text targets;
- textless description references clearing raw ids, reporting `DanglingDescription`, and pruning hidden textless targets;
- relation targets ignored when only referenced by excluded hidden owners;
- two-node `labelled_by` cycle diagnostics;
- malformed, double-hash, and dangling description reference diagnostics with raw `#...` descriptions cleared;
- malformed label reference diagnostics;
- disabled node discovery with invalid action filtering and `DisabledAction` diagnostics;
- hidden focusable diagnostics without normal hidden-node inclusion;
- hidden ancestor subtree exclusion and focus fallback for focused descendants;
- invalid focus fallback diagnostics, including the combined `HiddenFocusable` plus `ExcludedFocusedNode` case;
- layout-cache bounds fallback when arranged bounds are unavailable;
- missing bounds diagnostics;
- missing name diagnostics for nameless interactive nodes;
- focus fallback state synchronization and disabled-root focus clearing;
- unsupported role/action diagnostics;
- child promotion through excluded containers;
- hidden excluded containers blocking descendant promotion;
- accessibility action dispatch for accepted focus, accepted activation component commits, typed widget `Activate` mutation through authored toggle aliases backed by retained attributes or runtime component state, MenuItem activation closing the nearest open popup through the shared widget close path even without an item activation binding, disclosure aliases, disabled and selected state from retained state/attributes, expanded/checked/value snapshot state from authored widget property aliases, stale target rejection, hidden and visible-excluded target rejection, disabled activation rejection, unsupported increment, accessibility increment/decrement through retained or component-state backed range value/min/max/step aliases, accepted popup dismiss through `AccessibilityAction` binding reports, accepted tooltip dismiss through shared tooltip hide effects, unsupported dismiss when no open popup or active tooltip owner exists, editable text `SetValue` property mutation, `SetValue` through authored `UiWidgetContract::value_property` aliases backed by retained attributes or runtime component state, and unsupported `SetValue` when no existing mutable value metadata property is available.
- TextInput `SetValue` edit metadata sync, including value mutation plus retained/component-state `caret_offset`, `selection_anchor`, `selection_focus`, `composition_start`, `composition_end`, `composition_text`, and `composition_restore_text` mutation through accessibility-action binding reports, with snapshot `state.text_selection` collapsed to the new value end and stale IME composition metadata cleared.
- TextInput `ReplaceSelectedText`, including AccessKit request mapping, selected-range replacement through the same value-property alias path, retained constraint sanitization, caret/selection collapse, composition cleanup, and accessibility-action binding reports.
- TextInput `SetTextSelection`, including AccessKit request mapping from character indexes into byte offsets, read-only selection movement, distinct caret/anchor/focus preservation, UTF-8 offset clamping, composition cleanup at the new caret offset, and accessibility-action binding reports without value-change component events.
- Scrollable `ScrollTo`, including numeric/string offset payloads, neutral point `scroll_offset` payloads, AccessKit `SetScrollOffset` request mapping, axis-aware offset selection, and accessibility-action binding reports.

`zircon_runtime/src/ui/tests/accessibility_state_values.rs` keeps focused state-extraction coverage out of the oversized all-in-one accessibility test file. It covers runtime component-state disabled/selected/pressed values, disabled inheritance from retained parent attributes, and TextInput `state.text_selection` from retained caret/selection attributes and runtime component-state offsets, including clamping to UTF-8 text boundaries.

`zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs` keeps TextInput accessibility action edge cases out of the oversized all-in-one accessibility test file. It covers read-only `SetValue` rejection, constraint sanitization before accessibility-action binding reports mutate value, caret, selection anchor, and selection focus, active composition metadata clearing after full-field replacement, `ReplaceSelectedText` selected-range replacement plus constraint sanitization, and `SetTextSelection` selection-only metadata updates on read-only fields plus distinct clamped caret preservation.

`zircon_runtime/src/ui/tests/accessibility_widget_actions.rs` keeps widget open-state accessibility action cases out of the oversized all-in-one accessibility test file. It covers disclosure and popup `open_property` aliases, state-sensitive snapshot exposure of `Expand` versus `Collapse`, runtime `Expand`/`Collapse` dispatch through the same accessibility-action binding reports that mirror retained attributes into component state and emit `ToggleExpanded`, `OpenPopup`, or `ClosePopup`, popup `Dismiss` dispatch through the same alias path used by Escape/menu dismissal, Dialog-role popup default `Dismiss` without unsupported diagnostics, Menu-role popup expand/collapse without synthetic activation, tooltip `Dismiss` dispatch through the shared tooltip hide effect plus host request path, and MenuItem `Activate` dispatch closing the popup through a widget-origin binding report even when the item itself has no activation binding.

`zircon_runtime/src/ui/tests/accessibility_disabled_gate.rs` owns disabled accessibility gate coverage that would otherwise bloat the widget-action test module. It covers the disabled-owner guard that prevents a descendant accessibility `Dismiss` from closing a disabled popup, disabled-owner `Focus` rejection through the runtime focus/input owner gate, inherited-disabled descendant `Focus` rejection when the child is enabled/focusable but an ancestor is disabled, inherited-disabled toggle `Activate` rejection before checked-property mutation, and inherited-disabled TextInput `SetValue` rejection before retained/component-state value mutation.

`zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs` covers the Bevy-aligned scrollbar a11y boundary: default Scrollbar/ScrollbarThumb widgets are excluded from the neutral tree unless explicit a11y metadata is authored, explicit `UiA11yRole::Scrollbar` still maps through the optional AccessKit bridge, and `ScrollTo` mutates the scrollable container rather than the headless scrollbar chrome. It also covers neutral point `scroll_offset` payloads for vertical and horizontal `ScrollableBox` axes.

`zircon_runtime/src/ui/tests/accesskit.rs` covers optional AccessKit bridge conversion for neutral snapshots and action requests. It now locks role-aware `Dismiss` export so Dialog nodes expose AccessKit `Blur`, Tooltip nodes expose `HideTooltip`, and incoming `Blur` or `HideTooltip` requests both return neutral `UiAccessibilityAction::Dismiss` for runtime dispatch.

`zircon_runtime/src/dynamic_api/tests.rs` covers runtime API table accessibility capture presence, null output rejection, wrong ABI rejection before session lookup, unknown viewport rejection, serialized preview snapshot capture/free, invalid accessibility free ownership rejection, invalid accessibility action JSON rejection, and valid action-payload rejection when the dynamic preview has no retained UI surface.

`zircon_runtime/src/ui/tests/widget_radio_behavior.rs` covers RadioGroup/Radio accessibility role projection and default Radio activate action alongside runtime pointer selection, disabled group rejection, and keyboard selection behavior, so a11y role inference stays tied to the same widget contract that mutates checked state.

## Validation Evidence

TextInput accessibility text-selection snapshot evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs"`: PASS.
- `git diff --check -- "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime_interface/ui/accessibility.md" "docs/zircon_runtime_interface/ui/contract-spine.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260522-0050-ui-a11y-text-selection.md"`: PASS with LF/CRLF warnings only.
- New focused test code covers retained TextInput caret/selection attributes, component-state offset fallback, UTF-8 boundary clamping, and interface serde/default compatibility; runtime Cargo validation remains deferred during this implementation slice because 8 unrelated `cargo` processes and 4 unrelated `rustc` processes are active in the shared checkout.

TextInput accessibility SetValue selection-sync evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260522-0110-ui-a11y-setvalue-selection-sync.md"`: PASS with LF/CRLF warnings only.
- Focused coverage now asserts TextInput accessibility `SetValue` mutates retained text, caret, selection anchor, and selection focus through accessibility-action binding reports, then re-extracts `state.text_selection` collapsed at the new value end. Runtime Cargo validation remains deferred because 10 unrelated `cargo` processes and 5 unrelated `rustc` processes are active in the shared checkout.

TextInput accessibility SetValue constraint evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/input/text_constraints.rs" "zircon_runtime/src/ui/surface/input/mod.rs" "zircon_runtime/src/ui/surface/mod.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs" "zircon_runtime/src/ui/tests/mod.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/surface/input/text_constraints.rs" "zircon_runtime/src/ui/surface/input/mod.rs" "zircon_runtime/src/ui/surface/mod.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs" "zircon_runtime/src/ui/tests/mod.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime/ui/surface/input.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260522-0120-ui-a11y-setvalue-constraints.md"`: PASS with LF/CRLF warnings only.
- Focused coverage asserts read-only TextInput `SetValue` rejects before mutation, and constrained TextInput `SetValue` applies retained filter/max/multiline rules before value/caret/selection binding reports. Runtime Cargo validation remains deferred because 6 unrelated `cargo` processes and 3 unrelated `rustc` processes are active in the shared checkout.

TextInput accessibility SetValue composition-clear evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime/ui/surface/input.md"`: PASS with LF/CRLF warnings only.
- `Select-String` trailing-whitespace scan passed for the touched Rust files, docs, plan, and active session note.
- Focused coverage now asserts TextInput accessibility `SetValue` clears `composition_start`, `composition_end`, `composition_text`, and `composition_restore_text` through accessibility-action binding reports after whole-field replacement. Runtime Cargo validation remains deferred because 4 unrelated `cargo` processes and 1 unrelated `rustc` process are active in the shared checkout.

TextInput accessibility ReplaceSelectedText evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime/ui/surface/input.md" "docs/zircon_runtime_interface/ui/accessibility.md" "docs/zircon_runtime_interface/ui/contract-spine.md"`: PASS with LF/CRLF warnings only.
- Focused coverage asserts AccessKit `ReplaceSelectedText` maps to neutral `UiAccessibilityAction::ReplaceSelectedText`, TextInput snapshots expose that default action, selected-range replacement mutates only the selected span, and retained constraints sanitize selected-range replacements before value/caret/selection/composition binding reports. Runtime Cargo validation remains deferred because unrelated `cargo`/`rustc` processes are active in the shared checkout.

TextInput accessibility SetTextSelection evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs" "zircon_runtime/src/dynamic_api/tests.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs" "zircon_runtime/src/dynamic_api/tests.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime_interface/ui/accessibility.md" "docs/zircon_runtime_interface/ui/contract-spine.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, and plan. Focused coverage asserts AccessKit `SetTextSelection` maps to neutral `request.text_selection`, converts AccessKit character indexes to UTF-8 byte offsets through the snapshot text value instead of copying indexes directly, returns no neutral request when the bridge lacks text context, TextInput snapshots expose the default action, read-only TextInput selection movement updates caret/anchor/focus plus composition metadata without value-change component events, and invalid UTF-8 offsets are clamped before metadata writes. Runtime/interface Cargo validation remains deferred because 6 unrelated `cargo` processes and 3 unrelated `rustc` processes are active in the shared checkout.

Scrollable accessibility SetScrollOffset evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs" "zircon_runtime/src/dynamic_api/tests.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs"`: PASS.
- `git diff --check -- "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs" "zircon_runtime/src/dynamic_api/tests.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime_interface/ui/accessibility.md" "docs/zircon_runtime_interface/ui/contract-spine.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260522-0145-ui-a11y-scroll-offset.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Focused coverage asserts AccessKit `SetScrollOffset` maps to neutral `ScrollTo` with `scroll_offset`, neutral `ScrollTo` exposes AccessKit `SetScrollOffset`, AccessKit `ScrollToPoint` remains unsupported, and runtime `ScrollTo` chooses vertical or horizontal point axes before producing accessibility-action scroll binding reports. Runtime/interface Cargo validation remains deferred because 14 unrelated `cargo` processes and 5 unrelated `rustc` processes are active in the shared checkout.

AccessKit bridge continuation evidence from the 2026-05-16 Bevy-level UI/a11y plan:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/mod.rs"`: PASS after applying targeted `rustfmt`.
- `git diff --check -- "zircon_runtime/Cargo.toml" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/mod.rs" "docs/zircon_runtime/ui/accessibility.md"`: PASS with LF/CRLF warnings only.
- `cargo test -p zircon_runtime --lib ui::tests::accesskit --features accessibility-accesskit --locked --jobs 1 --message-format short --color never`: BLOCKED before the focused AccessKit tests executed by unrelated active asset/shader compile errors: unresolved shader asset imports in `zircon_runtime/src/asset/{mod.rs,importer/ingest/import_shader_package.rs}` and missing fields in `zircon_runtime/src/asset/tests/project/zmeta.rs`.
- `cargo check -p zircon_runtime --lib --features accessibility-accesskit --locked --jobs 1 --message-format short --color never`: BLOCKED by unrelated input compile errors where `InputButton` does not implement `Default` in `zircon_runtime/src/core/framework/input/input_frame_snapshot.rs`, `zircon_runtime/src/core/framework/input/mod.rs`, and `zircon_runtime/src/input/runtime/input_state.rs`.

Widget behavior contract continuation evidence from the same plan:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/widget.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime/src/ui/surface/surface/default_interactions.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/pointer_click_semantics.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS.
- `cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-widget-behavior" --message-format short --color never`: PASS with 5 passed, 0 failed, 91 filtered.

Radio widget continuation evidence:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/widget.rs" "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime/src/ui/surface/surface/default_interactions.rs" "zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/mod.rs" "zircon_runtime/src/ui/tests/widget_radio_behavior.rs" "zircon_runtime/src/ui/tests/accesskit.rs"`: PASS after applying targeted `rustfmt`.
- `git diff --check -- "zircon_runtime_interface/src/ui/widget.rs" "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime/src/ui/surface/surface/default_interactions.rs" "zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/mod.rs" "zircon_runtime/src/ui/tests/widget_radio_behavior.rs" "zircon_runtime/src/ui/tests/accesskit.rs"`: PASS with LF/CRLF warnings only.
- `cargo test -p zircon_runtime_interface --lib ui_contract_spine --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-radio-contract" --message-format short --color never`: PASS with 5 passed, 0 failed.
- `cargo test -p zircon_runtime --lib widget_radio_behavior --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-radio-runtime" --message-format short --color never`: INCONCLUSIVE; timed out after 360 seconds without usable Rust diagnostics while the shared checkout still had many active `cargo` and `rustc` processes.

Accessibility SetValue alias continuation evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md" ".codex/sessions/20260516-1316-ui-focus-a11y-contract.md"`: PASS with LF/CRLF warnings only.
- Focused runtime Cargo testing was deferred for this narrow alias slice because many unrelated `cargo` and `rustc` jobs were already active in the shared checkout; earlier focused runtime test attempts for this milestone were inconclusive for the same workspace-lock pressure.

Accessibility typed activation continuation evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md" ".codex/sessions/20260516-1316-ui-focus-a11y-contract.md"`: PASS with LF/CRLF warnings only.

Accessibility alias state extraction evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS after applying targeted `rustfmt`.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md" ".codex/sessions/20260516-1316-ui-focus-a11y-contract.md"`: PASS with LF/CRLF warnings only.

Accessibility runtime value state extraction evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/surface/component_state.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/surface/surface/default_interactions.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS after applying targeted `rustfmt`.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/surface/component_state.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/surface/surface/default_interactions.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md" ".codex/sessions/20260516-1316-ui-focus-a11y-contract.md"`: PASS with LF/CRLF warnings only.
- Focused runtime Cargo validation remains deferred for this narrow slice because 14 unrelated `cargo` and 7 unrelated `rustc` jobs were active in the shared checkout.

Accessibility runtime checked state extraction evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS after applying targeted `rustfmt`.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md" ".codex/sessions/20260516-1316-ui-focus-a11y-contract.md"`: PASS with LF/CRLF warnings only.
- Focused runtime Cargo validation remains deferred for this narrow slice because many unrelated `cargo` and `rustc` jobs were active in the shared checkout.

Accessibility expanded alias extraction evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md" ".codex/sessions/20260516-1316-ui-focus-a11y-contract.md"`: PASS with LF/CRLF warnings only.

Accessibility selected state extraction evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md" ".codex/sessions/20260516-1316-ui-focus-a11y-contract.md"`: PASS with LF/CRLF warnings only.

Accessibility disabled attribute extraction evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS after applying targeted `rustfmt`.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md" ".codex/sessions/20260516-1316-ui-focus-a11y-contract.md"`: PASS with LF/CRLF warnings only.

Accessibility pressed state extraction evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md" ".codex/sessions/20260516-1316-ui-focus-a11y-contract.md"`: PASS with LF/CRLF warnings only.
- Focused runtime Cargo validation remains deferred for this narrow slice because many unrelated `cargo` and `rustc` jobs were active in the shared checkout.

Milestone 3 testing stage evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/dynamic_api/frame.rs" "zircon_runtime/src/dynamic_api/session.rs" "zircon_runtime/src/dynamic_api/exports.rs" "zircon_runtime/src/dynamic_api/tests.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs"`: PASS.
- Initial parallel Cargo tests hit shared lock timeouts; sequential reruns were used for final evidence.
- `cargo test -p zircon_runtime --lib dynamic_api --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: PASS with 11 passed, 0 failed, 1185 filtered out; emitted two unrelated dead-code warnings.
- `cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: PASS with 36 passed, 0 failed, 1159 filtered out; emitted two unrelated dead-code warnings.
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: PASS.
- `git diff --check -- "zircon_runtime/src/dynamic_api/frame.rs" "zircon_runtime/src/dynamic_api/session.rs" "zircon_runtime/src/dynamic_api/exports.rs" "zircon_runtime/src/dynamic_api/tests.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/sessions/20260509-0435-accesskit-bridge-implementation.md"`: PASS with LF/CRLF warnings for dynamic API Rust files only.

Milestone 2 testing stage evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS.
- `cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: PASS with 36 passed, 0 failed, 0 ignored.
- `cargo test -p zircon_runtime --lib ui::tests::runtime_input_ownership --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: PASS with 12 passed, 0 failed, 0 ignored.
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/sessions/20260509-0435-accesskit-bridge-implementation.md"`: PASS with an LF/CRLF warning for `zircon_runtime/src/ui/surface/input/dispatch.rs` only.

Milestone 1 testing stage evidence:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs" "zircon_runtime/src/ui/mod.rs" "zircon_runtime/src/ui/template/build/tree_builder.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/accessibility/mod.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/name.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/tests/mod.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS after adding the M1 `accesskit.rs` stub and formatting.
- `cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: BLOCKED before accessibility tests execute by unrelated test compile errors in `zircon_runtime/src/scene/tests/ecs_systems.rs:128-129`; `ResParam<MissingScore>` and `ResMutParam<MissingScore>` do not implement `Debug` for `unwrap_err()`.
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: PASS with unrelated resource streamer warnings.
- `git diff --check -- "zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs" "zircon_runtime/src/ui/mod.rs" "zircon_runtime/src/ui/template/build/tree_builder.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/accessibility/mod.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/name.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/tests/mod.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md"`: PASS with LF/CRLF warnings only.

The known earlier render material syntax blocker at `zircon_runtime/src/core/framework/render/material/readiness_report.rs:43` was not encountered by the Milestone 1 `cargo check` rerun.

Spec-compliance follow-up evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS.
- `cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: BLOCKED before accessibility tests execute by unrelated test compile errors in `zircon_runtime/src/scene/tests/ecs_systems.rs:128-129`; `ResParam<MissingScore>` and `ResMutParam<MissingScore>` do not implement `Debug` for `unwrap_err()`.
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: BLOCKED by unrelated render/resource-streamer import drift in `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs:9`; `super::super::MaterialDependencyReadinessKey` is unresolved.

Neither blocker is part of runtime accessibility extraction. The earlier known render syntax blocker at `zircon_runtime/src/core/framework/render/material/readiness_report.rs:43` was not reached in this follow-up run.

Hidden-focused-focusable follow-up evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS.
- `cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: BLOCKED before accessibility tests execute by unrelated test compile errors in `zircon_runtime/src/scene/tests/ecs_systems.rs:128-129`; `ResParam<MissingScore>` and `ResMutParam<MissingScore>` do not implement `Debug` for `unwrap_err()`.

Code-quality follow-up evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/name.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs"`: PASS after local formatting corrections.
- `cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: first run timed out during compilation; longer rerun was BLOCKED before accessibility tests executed by unrelated active reflection work at `zircon_runtime/src/scene/reflect/mod.rs:4`, missing module file `fixed`.
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: BLOCKED by unrelated active reflection fixed-adapter module declarations in `zircon_runtime/src/scene/reflect/fixed/mod.rs:1-5`, missing files for `active_self`, `local_transform`, `render_layer_mask`, and `rigid_body_component`.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/name.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/sessions/20260509-0435-accesskit-bridge-implementation.md"`: PASS with an LF/CRLF warning for `zircon_runtime/src/ui/surface/input/dispatch.rs` only.

Hidden-subtree follow-up evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS.
- `cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: first run timed out after 300 seconds during dependency compilation; rerun with a longer timeout was BLOCKED before accessibility tests executed by unrelated active render/graphics/reflection work: unresolved `core_pipeline::MeshPhaseInput`, stale `RenderPassStage::GBuffer` references in `zircon_runtime/src/graphics/tests/pipeline_compile.rs`, and immutable `world` borrow in `zircon_runtime/src/scene/tests/ecs_reflect.rs:806`.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/sessions/20260509-0435-accesskit-bridge-implementation.md"`: PASS before this evidence update.

Final code-quality follow-up evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS.
- `cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: BLOCKED before accessibility tests executed by unrelated active render work at `zircon_runtime/src/core/framework/render/mod.rs:32`, unresolved import `core_pipeline::MeshPhaseInput`.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/sessions/20260509-0435-accesskit-bridge-implementation.md"`: PASS before this evidence update.

Relation/focus follow-up evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS after formatting corrections.
- `cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: first run compiled and ran the focused tests with 19 passed, 1 failed in `name_priority_uses_explicit_labelled_by_text_alt_then_tooltip`; the failure exposed tooltip-only fallback nodes not being included. After adding tooltip fallback inclusion, rerun PASS with 20 passed, 0 failed, 0 ignored, 1132 filtered.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/sessions/20260509-0435-accesskit-bridge-implementation.md"`: PASS before this evidence update.

Ultimate relation-source follow-up evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: first run reported formatting diffs in `extract.rs` and `ui/tests/accessibility.rs`; `rustfmt --edition 2021` was applied to the same files.
- `cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: PASS with 24 passed, 0 failed, 0 ignored, 1139 filtered; emitted two unrelated dead-code warnings in graphics/plugin test helpers.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS after formatting.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/sessions/20260509-0435-accesskit-bridge-implementation.md"`: PASS before final evidence update.

Final-pass leak/order follow-up evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: first run reported formatting diffs in `extract.rs` and `ui/tests/accessibility.rs`; `rustfmt --edition 2021` was applied to the same files, then the same check passed.
- `cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: BLOCKED before accessibility tests executed by unrelated active graphics/render compile errors in `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs` and `zircon_runtime/src/graphics/tests/{pipeline_compile.rs,render_product_submit.rs}`.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/sessions/20260509-0435-accesskit-bridge-implementation.md"`: PASS before final evidence update.

Double-hash/pruned-hidden-source follow-up evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs"`: PASS.
- `cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never`: first run timed out during dependency/runtime compilation before tests executed; longer rerun PASS with 27 passed, 0 failed, 0 ignored, 1143 filtered; emitted two unrelated dead-code warnings in graphics/plugin test helpers.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/sessions/20260509-0435-accesskit-bridge-implementation.md"`: PASS before final evidence update.

Widget-behavior alias convergence evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/surface/component_state.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/surface/surface/default_interactions.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "zircon_runtime/src/ui/tests/mod.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/surface/component_state.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/surface/surface/default_interactions.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "zircon_runtime/src/ui/tests/mod.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md" ".codex/sessions/20260516-1316-ui-focus-a11y-contract.md"`: PASS with LF/CRLF warnings only.
- Focused runtime Cargo validation for the new open-property alias tests is deferred during this implementation slice because the shared checkout still has 16 active `cargo` processes and 9 active `rustc` processes.

Component-state value convergence evidence:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/mod.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/mod.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md" ".codex/sessions/20260516-1316-ui-focus-a11y-contract.md"`: PASS with LF/CRLF warnings only.
- `cargo test -p zircon_runtime --lib ui::tests::accessibility_state_values --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-state-values" --message-format short --color never`: first attempt timed out after 300 seconds before a usable test result. A later attempt exited non-zero after dependency compilation lines without a Rust diagnostic. A final plain-output rerun timed out after 420 seconds. Focused runtime Cargo validation remains unresolved; further retries were stopped while 10 `cargo` and 5 `rustc` processes were active in the shared checkout.

Popup open-alias coverage evidence:

- `zircon_runtime/src/ui/tests/accessibility_widget_actions.rs` now covers both disclosure and popup runtime `open_property` aliases for snapshot `expanded` state and accessibility `Activate` mutation.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- Focused runtime Cargo validation for this test module was deferred because 16 `cargo` and 6 `rustc` processes were active in the shared checkout.

Accessibility Expand/Collapse action follow-up evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/tests/ui_contract_spine.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime_interface/ui/accessibility.md" "docs/zircon_runtime_interface/ui/contract-spine.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260522-0155-ui-a11y-expand-collapse.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Runtime/interface Cargo validation remains deferred because unrelated `cargo`/`rustc` processes remain active in the shared checkout.

Accessibility popup Dismiss action follow-up evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/surface/default_interactions/popup.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/surface/surface/default_interactions/popup.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime_interface/ui/accessibility.md" "docs/zircon_runtime_interface/ui/contract-spine.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260522-0205-ui-a11y-popup-dismiss.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Runtime Cargo validation remains deferred because unrelated `cargo`/`rustc` processes remain active in the shared checkout.

Accessibility tooltip Dismiss action follow-up evidence from 2026-05-22:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime_interface/ui/accessibility.md" "docs/zircon_runtime_interface/ui/contract-spine.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260522-0220-ui-a11y-tooltip-dismiss.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Runtime Cargo validation remains deferred because unrelated `cargo`/`rustc` processes remain active in the shared checkout.

MenuItem activation close follow-up evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/surface/default_interactions.rs" "zircon_runtime/src/ui/tests/widget_menu_behavior.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/surface/surface/default_interactions.rs" "zircon_runtime/src/ui/tests/widget_menu_behavior.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/surface/input.md" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime/ui/binding/update_report.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260523-0005-ui-menuitem-activation-close.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Runtime Cargo validation remains deferred because 4 unrelated `cargo` and 4 `rustc` processes remain active in the shared checkout.

Dialog Popup default Dismiss follow-up evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260523-0015-ui-dialog-popup-default-dismiss.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Focused coverage now asserts the open Dialog Popup default action set contains `Dismiss`, omits `Activate`/`Expand`/`Collapse`, and emits no `UnsupportedRoleAction` diagnostic for `Dialog + Dismiss`. Runtime Cargo validation remains deferred because 6 unrelated `cargo` and 2 `rustc` processes remain active in the shared checkout.

AccessKit Dismiss role-action export evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/tests/accesskit.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/tests/accesskit.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260523-0030-ui-accesskit-dismiss-role-actions.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Focused coverage asserts Dialog `Dismiss` exports AccessKit `Blur`, Tooltip `Dismiss` exports `HideTooltip`, and incoming `Blur`/`HideTooltip` requests both return neutral `Dismiss`. Runtime Cargo validation remains deferred because 8 unrelated `cargo` and 4 `rustc` processes remain active in the shared checkout.

Menu Popup default Expand/Collapse follow-up evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260523-0040-ui-menu-popup-default-expand-collapse.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Focused coverage asserts open Menu Popup default actions contain `Collapse`, omit `Activate`/`Expand`/`Dismiss`, emit no `UnsupportedRoleAction`, and dispatch `Collapse` through the existing popup open-property mutation. Runtime Cargo validation remains deferred because 12 unrelated `cargo` and 11 `rustc` processes remain active in the shared checkout.

Disabled Popup owner Dismiss gate evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: first run reported a formatting diff in the new assertion chain; after applying `rustfmt`, rerun PASS.
- `git diff --check -- "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust file, docs, and plan. Focused coverage asserts a descendant Dialog node with explicit `Dismiss` cannot close a disabled Popup owner: dispatch returns unsupported, produces no binding report or `ClosePopup`, and leaves retained/component-state `popup_open=true`. Runtime Cargo validation remains deferred because 10 unrelated `cargo` and 5 `rustc` processes remain active in the shared checkout.

Ancestor disabled gate evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/surface/interaction_state.rs" "zircon_runtime/src/ui/surface/input/validation.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/widget_menu_behavior.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/surface/surface/interaction_state.rs" "zircon_runtime/src/ui/surface/input/validation.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/widget_menu_behavior.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime/ui/surface/input.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260523-0055-ui-ancestor-disabled-gate.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Focused coverage asserts retained parent `disabled = true` marks a nested button disabled in the a11y snapshot, filters actions to `Focus`, and records `DisabledAction`; menu behavior coverage asserts disabled popup owners reject descendant focus, stale Escape dismissal, and outside-click dismissal. Runtime Cargo validation remains deferred because 10 unrelated `cargo` and 4 `rustc` processes remain active in the shared checkout.

Shared disabled gate helper evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/surface/interaction_gate.rs" "zircon_runtime/src/ui/surface/mod.rs" "zircon_runtime/src/ui/surface/surface/interaction_state.rs" "zircon_runtime/src/ui/surface/input/validation.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/widget_menu_behavior.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/surface/interaction_gate.rs" "zircon_runtime/src/ui/surface/mod.rs" "zircon_runtime/src/ui/surface/surface/interaction_state.rs" "zircon_runtime/src/ui/surface/input/validation.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/tests/accessibility_state_values.rs" "zircon_runtime/src/ui/tests/widget_menu_behavior.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_runtime/ui/surface/input.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260523-0060-ui-shared-disabled-gate-helper.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. This is a refactor-only convergence slice over the same focused coverage. A lightweight `cargo check -p zircon_runtime --lib --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-disabled-gate-helper" --message-format short --color never` attempt was blocked before compilation because Cargo needed to update `Cargo.lock` and `--locked` forbids that; locked Cargo validation needs a separate lockfile-state pass before it can produce Rust diagnostics.

Disabled accessibility Focus gate evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260523-0065-ui-disabled-a11y-focus-gate.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust file, docs, plan, and session note. Focused coverage asserts disabled popup owners still expose snapshot `Focus` as the only retained action, but dispatch routes through runtime focus owner validation, returns `focus_rejected`, leaves runtime focus unchanged, and emits no binding report or component event.

Disabled accessibility gate test split evidence from 2026-05-23:

- `zircon_runtime/src/ui/tests/accessibility_disabled_gate.rs` now owns disabled popup Dismiss and Focus gate coverage, and `zircon_runtime/src/ui/tests/mod.rs` registers it as a focused test module.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/tests/accessibility_disabled_gate.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "zircon_runtime/src/ui/tests/mod.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/tests/accessibility_disabled_gate.rs" "zircon_runtime/src/ui/tests/accessibility_widget_actions.rs" "zircon_runtime/src/ui/tests/mod.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260523-0070-ui-disabled-a11y-gate-test-split.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust file, docs, plan, and session note. The split keeps `accessibility_widget_actions.rs` below the large-file warning path while preserving the same focused behavior coverage.

Inherited disabled accessibility Focus gate evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/tests/accessibility_disabled_gate.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/tests/accessibility_disabled_gate.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260523-0075-ui-inherited-disabled-a11y-focus-gate.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust file, docs, plan, and session note. Focused coverage asserts a child under retained parent `disabled = true` remains discoverable with only `Focus`, records `DisabledAction`, and rejects accessibility `Focus` through runtime focus owner validation without changing `UiSurface.focus`.

Inherited disabled accessibility non-Focus gate evidence from 2026-05-23:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/tests/accessibility_disabled_gate.rs"`: first run reported a formatting diff in the new TextInput fixture; after applying `rustfmt`, rerun PASS.
- Focused coverage now asserts a toggle under retained parent `disabled = true` rejects accessibility `Activate` with `code=disabled_action`, emits no binding report/component event, and leaves the authored `checked` property plus component-state mirror untouched. It also asserts a TextInput under the same inherited disabled state rejects accessibility `SetValue` before the value-property alias mutates retained `text` or runtime component state.
- Runtime Cargo validation remains deferred in this implementation slice because the shared checkout still has unrelated Cargo/Rust compiler activity and the earlier locked check was blocked before compilation by `Cargo.lock` needing an update.

M5 disabled descendant Dialog gate evidence from 2026-05-27:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/tests/accessibility_disabled_gate.rs"`: PASS.
- Focused coverage now asserts a disabled descendant `Dialog` remains discoverable with only `Focus`, records `DisabledAction`, and rejects a forced `Dismiss` dispatch with `code=disabled_action` before popup-id-specific handling.
- `cargo test -p zircon_runtime --lib --locked --target-dir F:\cargo-targets\zircon-platform-m5-workspace --message-format short --color never -- --format terse`: PASS with 2102 passed, 0 failed.

TextInput SetTextSelection caret follow-up evidence from 2026-05-27:

- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs"`: PASS.
- `git diff --check -- "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs" "docs/zircon_runtime/ui/accessibility.md" ".codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md" ".codex/sessions/archive/20260527-0358-editor-ui-a11y-text-selection-caret.md"`: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. Focused coverage now asserts `SetTextSelection` preserves a distinct neutral caret after UTF-8 boundary clamping and clears composition metadata at that caret offset. Focused runtime Cargo validation was attempted with `cargo test -p zircon_runtime --lib accessibility_set_text_selection_preserves_distinct_clamped_caret_offset --locked --jobs 1 --target-dir "F:\cargo-targets\zircon-a11y-selection-caret-20260527" --message-format short --color never`; it timed out after 10 minutes without Rust diagnostics or test assertion failures, and post-timeout process inspection showed remaining Cargo/Rust compiler activity belonged to other hub/sound sessions.

Accessibility value/text action module split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action.rs` now owns routing and shared helpers, while `zircon_runtime/src/ui/accessibility/action/value.rs` owns `SetValue`, `ReplaceSelectedText`, and `SetTextSelection` value/text edit handling without changing the existing public dispatch contract.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/value.rs"`: PASS.
- Scoped `git diff --check` for tracked touched Rust/docs files plus no-index whitespace checks for the new module, plan, and archived session note: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. A lightweight compile check was attempted with `cargo check -p zircon_runtime --lib --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir "F:\cargo-targets\zircon-a11y-value-action-split-20260527" --message-format short --color never`; it timed out after 10 minutes without Rust diagnostics, and follow-up process inspection showed active Cargo/Rust compiler lanes from other sound, shader/material, Hub, ZUI, and texture sessions.

Accessibility popup action module split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action.rs` now owns routing plus scroll, range, focus, activate, and shared diagnostics, while `zircon_runtime/src/ui/accessibility/action/popup.rs` owns `Expand`, `Collapse`, popup `Dismiss`, tooltip hide dispatch, widget open-property resolution, and popup/disclosure component-event mapping without changing the public accessibility dispatch contract.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/value.rs"`: PASS.
- Scoped `git diff --check` for runtime action files, this document, the UI/Text/Input/A11y plan, and the session note: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. No new Cargo check was started for this structural split because process inspection showed unrelated editor ZUI, Hub UI, texture KTX2, and sound Cargo/Rust compiler jobs already active in the shared checkout.
Accessibility scroll/range action module split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action.rs` now stays as the route validator plus Focus/Activate/shared diagnostic helper boundary, while `zircon_runtime/src/ui/accessibility/action/scroll.rs` owns `ScrollTo`, offset payload parsing, axis projection, scroll mutation, unchanged handling, and scroll binding reports.
- `zircon_runtime/src/ui/accessibility/action/range.rs` owns Slider `Increment`/`Decrement`, role/action validation, range-step mutation, binding-report diagnostics, and `ValueChanged` component events.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/range.rs" "zircon_runtime/src/ui/accessibility/action/scroll.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/value.rs"`: PASS.
- Scoped `git diff --check` for runtime action files, this document, the UI/Text/Input/A11y plan, and the session note: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. No new Cargo check was started because process inspection showed the sound runtime `--no-run` Cargo lane compiling `zircon_runtime` in the shared checkout.
Accessibility action result helper module split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/result.rs` now owns `finish_handled`, `finish_unhandled`, `finish_unhandled_with_note`, `unsupported_role_action`, and `action_note` for all accessibility action submodules.
- `zircon_runtime/src/ui/accessibility/action/result/binding.rs` now owns binding report diagnostic emission for property mutation reports while `result.rs` re-exports the helper for existing action submodule call sites.
- `zircon_runtime/src/ui/accessibility/action.rs` now keeps route validation, missing-target handling, Focus dispatch, and Activate dispatch, while popup, range, scroll, and value/text submodules depend on the same result helper vocabulary.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/result.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/range.rs" "zircon_runtime/src/ui/accessibility/action/scroll.rs" "zircon_runtime/src/ui/accessibility/action/value.rs"`: PASS.
- Scoped `git diff --check` for runtime action files, this document, the UI/Text/Input/A11y plan, and the session note: PASS with LF/CRLF warnings only.
- Trailing-whitespace scan passed for touched Rust files, docs, plan, and session note. No new Cargo check was started because process inspection showed unrelated sound runtime and editor ZUI Cargo/Rust compiler jobs active in the shared checkout.
Accessibility text edit action module split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/text.rs` owns `ReplaceSelectedText` and `SetTextSelection` role/action dispatch, value mutation, selected-range constraint sanitization, and component-event mapping.
- `zircon_runtime/src/ui/accessibility/action/value.rs` owns whole-field `SetValue`, value-property alias resolution, and Slider finite-value payload parsing; TextInput selection/composition synchronization is shared through `action/text_state.rs`.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/text.rs" "zircon_runtime/src/ui/accessibility/action/value.rs" "zircon_runtime/src/ui/accessibility/action/result.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/range.rs" "zircon_runtime/src/ui/accessibility/action/scroll.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for the new text module, routing module, value module, docs, plan, and session note.
- No new Cargo check was started because process inspection still shows unrelated Cargo/Rust compiler jobs active in the shared checkout.

Accessibility text state helper module split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/text_state.rs` now owns TextInput read-only metadata checks, selected-range calculation, UTF-8 boundary helpers, and caret/selection synchronization entry points for `SetValue`, `ReplaceSelectedText`, and `SetTextSelection`.
- `zircon_runtime/src/ui/accessibility/action/text_state/metadata.rs` now owns retained metadata mutation, binding-report diagnostics, and composition cleanup for TextInput accessibility edit metadata synchronization.
- `zircon_runtime/src/ui/accessibility/action/text.rs` now stays focused on TextInput edit action dispatch and delegates retained metadata writes to `text_state.rs`.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/text.rs" "zircon_runtime/src/ui/accessibility/action/text_state.rs" "zircon_runtime/src/ui/accessibility/action/value.rs" "zircon_runtime/src/ui/accessibility/action/result.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/range.rs" "zircon_runtime/src/ui/accessibility/action/scroll.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for the routing, text, text-state, value, docs, plan, and session files.
- No new Cargo check was started because process inspection still shows unrelated Cargo/Rust compiler jobs active in the shared checkout.
Accessibility focus/activate action module split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/focus.rs` now owns `Focus` role/action validation and programmatic focus dispatch through `UiSurface::focus_node_with_reason`.
- `zircon_runtime/src/ui/accessibility/action/activate.rs` now owns `Activate` role/action validation, typed default widget activation reuse, binding-report propagation, and generic commit fallback.
- `zircon_runtime/src/ui/accessibility/action.rs` now stays focused on snapshot capture, target diagnostics, hidden/disabled action gates, missing-target handling, and route dispatch.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/focus.rs" "zircon_runtime/src/ui/accessibility/action/activate.rs" "zircon_runtime/src/ui/accessibility/action/text.rs" "zircon_runtime/src/ui/accessibility/action/text_state.rs" "zircon_runtime/src/ui/accessibility/action/value.rs" "zircon_runtime/src/ui/accessibility/action/result.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/range.rs" "zircon_runtime/src/ui/accessibility/action/scroll.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for the routing, focus, activate, text, text-state, value, docs, plan, and session files.
- No new Cargo check was started because process inspection still shows unrelated Cargo/Rust compiler jobs active in the shared checkout.
Accessibility target helper module split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/target.rs` now owns target diagnostic propagation, stale-target rejection, hidden excluded target rejection, snapshot-excluded target rejection, and the ancestor render-visibility helper used by missing-target handling.
- `zircon_runtime/src/ui/accessibility/action.rs` now stays focused on snapshot capture, hidden/disabled gates for included snapshot targets, and route dispatch.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/target.rs" "zircon_runtime/src/ui/accessibility/action/focus.rs" "zircon_runtime/src/ui/accessibility/action/activate.rs" "zircon_runtime/src/ui/accessibility/action/text.rs" "zircon_runtime/src/ui/accessibility/action/text_state.rs" "zircon_runtime/src/ui/accessibility/action/value.rs" "zircon_runtime/src/ui/accessibility/action/result.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/range.rs" "zircon_runtime/src/ui/accessibility/action/scroll.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for the routing, target, focus, activate, text, text-state, value, docs, plan, and session files.
- No new Cargo check was started because process inspection still shows unrelated Cargo/Rust compiler jobs active in the shared checkout.
Accessibility included target gate split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/target.rs` now owns included snapshot target diagnostics plus hidden and disabled action rejection, so the same target module handles both included-target and missing-target availability decisions.
- `zircon_runtime/src/ui/accessibility/action.rs` now stays focused on snapshot capture and route dispatch after receiving an accepted target validation result.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/target.rs" "zircon_runtime/src/ui/accessibility/action/focus.rs" "zircon_runtime/src/ui/accessibility/action/activate.rs" "zircon_runtime/src/ui/accessibility/action/text.rs" "zircon_runtime/src/ui/accessibility/action/text_state.rs" "zircon_runtime/src/ui/accessibility/action/value.rs" "zircon_runtime/src/ui/accessibility/action/result.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/range.rs" "zircon_runtime/src/ui/accessibility/action/scroll.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for the routing, target, focus, activate, text, text-state, value, docs, plan, and session files.
- No new Cargo check was started because process inspection still shows unrelated Cargo/Rust compiler jobs active in the shared checkout.
Accessibility expanded action module split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/expanded.rs` now owns `Expand` and `Collapse`, widget `open_property` resolution, expandable-state mutation, and popup/disclosure component-event mapping.
- `zircon_runtime/src/ui/accessibility/action/popup.rs` now stays focused on popup `Dismiss` and tooltip hide dispatch, so expandable Disclosure/Popup state changes are not coupled to dismissal routing.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/expanded.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/target.rs" "zircon_runtime/src/ui/accessibility/action/focus.rs" "zircon_runtime/src/ui/accessibility/action/activate.rs" "zircon_runtime/src/ui/accessibility/action/text.rs" "zircon_runtime/src/ui/accessibility/action/text_state.rs" "zircon_runtime/src/ui/accessibility/action/value.rs" "zircon_runtime/src/ui/accessibility/action/result.rs" "zircon_runtime/src/ui/accessibility/action/range.rs" "zircon_runtime/src/ui/accessibility/action/scroll.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for the routing, expanded, popup, docs, plan, and session files.
- No new Cargo check was started because process inspection still shows unrelated Cargo/Rust compiler jobs active in the shared checkout.

Accessibility value target helper module split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/value_target.rs` now owns mutable value/text property resolution shared by `SetValue` and selected-text replacement.
- `zircon_runtime/src/ui/accessibility/action/text.rs` no longer depends on the whole-field `action/value.rs` dispatch module for property lookup; both modules depend on the narrower `value_target.rs` helper.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/value.rs" "zircon_runtime/src/ui/accessibility/action/value_target.rs" "zircon_runtime/src/ui/accessibility/action/text.rs" "zircon_runtime/src/ui/accessibility/action/text_state.rs" "zircon_runtime/src/ui/accessibility/action/result.rs" "zircon_runtime/src/ui/accessibility/action/target.rs" "zircon_runtime/src/ui/accessibility/action/focus.rs" "zircon_runtime/src/ui/accessibility/action/activate.rs" "zircon_runtime/src/ui/accessibility/action/expanded.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/range.rs" "zircon_runtime/src/ui/accessibility/action/scroll.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for the routing, value, value-target, text, docs, plan, and session files.
- No new Cargo check was started because process inspection still shows unrelated Cargo/Rust compiler jobs active in the shared checkout.

Accessibility text edit submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/text.rs` now stays structural and re-exports the two text-edit dispatch branches.
- `zircon_runtime/src/ui/accessibility/action/text/replace.rs` owns `ReplaceSelectedText` role/action validation, value mutation, binding-report diagnostics, edit metadata sync, and value-change component events.
- `zircon_runtime/src/ui/accessibility/action/text/replace/replacement.rs` owns selected-range next-text construction, retained TextInput constraint sanitization, sanitized note selection, and caret collapse offset calculation.
- `zircon_runtime/src/ui/accessibility/action/text/selection.rs` owns `SetTextSelection` role/action validation and caret/anchor/focus metadata writes through `action/text_state.rs`.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/text.rs" "zircon_runtime/src/ui/accessibility/action/text/replace.rs" "zircon_runtime/src/ui/accessibility/action/text/selection.rs" "zircon_runtime/src/ui/accessibility/action/text_state.rs" "zircon_runtime/src/ui/accessibility/action/value_target.rs" "zircon_runtime/src/ui/accessibility/action/value.rs" "zircon_runtime/src/ui/accessibility/action/result.rs" "zircon_runtime/src/ui/accessibility/action/target.rs" "zircon_runtime/src/ui/accessibility/action/focus.rs" "zircon_runtime/src/ui/accessibility/action/activate.rs" "zircon_runtime/src/ui/accessibility/action/expanded.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/range.rs" "zircon_runtime/src/ui/accessibility/action/scroll.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for routing, text root, text submodules, docs, plan, and session files.
- No new Cargo check was started because process inspection still shows unrelated Cargo/Rust compiler jobs active in the shared checkout.

Accessibility popup tooltip submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/popup.rs` now owns popup owner `Dismiss`, popup close mutation, and unsupported fallback.
- `zircon_runtime/src/ui/accessibility/action/popup/tooltip.rs` owns Tooltip role matching, runtime tooltip-state lookup, and `UiDispatchEffect::Tooltip { kind: Hide }` routing.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/popup/tooltip.rs" "zircon_runtime/src/ui/accessibility/action/expanded.rs" "zircon_runtime/src/ui/accessibility/action/target.rs" "zircon_runtime/src/ui/accessibility/action/focus.rs" "zircon_runtime/src/ui/accessibility/action/activate.rs" "zircon_runtime/src/ui/accessibility/action/text.rs" "zircon_runtime/src/ui/accessibility/action/text/replace.rs" "zircon_runtime/src/ui/accessibility/action/text/selection.rs" "zircon_runtime/src/ui/accessibility/action/text_state.rs" "zircon_runtime/src/ui/accessibility/action/value_target.rs" "zircon_runtime/src/ui/accessibility/action/value.rs" "zircon_runtime/src/ui/accessibility/action/result.rs" "zircon_runtime/src/ui/accessibility/action/range.rs" "zircon_runtime/src/ui/accessibility/action/scroll.rs"`: PASS.
- `cargo check -p zircon_runtime --lib --no-default-features --features core-min,plugin-ui --locked --jobs 1 --target-dir E:\cargo-targets\zircon-a11y-popup-tooltip-split-20260527 --message-format short --color never`: timed out after 10 minutes with no captured Rust diagnostic; the command process exited naturally after the timeout, but the detached output cannot be treated as a pass.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for popup routing, tooltip submodule, docs, plan, and session archive.

Accessibility popup result submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/popup/result.rs` now owns accepted/unchanged/rejected popup `Dismiss` close mutation result shaping, binding-report diagnostics, and `ClosePopup` component-event emission.
- `zircon_runtime/src/ui/accessibility/action/popup.rs` now stays focused on popup owner lookup, tooltip fallback routing, popup close mutation dispatch, and the existing unsupported dismiss note.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/popup/result.rs" "zircon_runtime/src/ui/accessibility/action/popup/tooltip.rs" "zircon_runtime/src/ui/accessibility/action/result.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for popup dispatch, popup result helper, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated Cargo/Rust compiler jobs active in the shared checkout.

Accessibility SetValue payload submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/value/payload.rs` now owns TextInput string/numeric text fallback and Slider finite-float `SetValue` payload parsing.
- `zircon_runtime/src/ui/accessibility/action/value.rs` now stays focused on `SetValue` role/action validation, TextInput read-only and constraint gates, property mutation, binding-report diagnostics, edit metadata sync, and value-change component events.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/value.rs" "zircon_runtime/src/ui/accessibility/action/value/payload.rs" "zircon_runtime/src/ui/accessibility/action/value_target.rs" "zircon_runtime/src/ui/accessibility/action/text.rs" "zircon_runtime/src/ui/accessibility/action/text/replace.rs" "zircon_runtime/src/ui/accessibility/action/text/selection.rs" "zircon_runtime/src/ui/accessibility/action/text_state.rs" "zircon_runtime/src/ui/accessibility/action/result.rs"`: PASS.
- No new Cargo check was started because process inspection showed unrelated platform policy Cargo/Rust compiler jobs active in the shared checkout.

Accessibility ScrollTo payload submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/scroll/payload.rs` now owns numeric/string offset payload parsing and neutral `scroll_offset` point axis projection for `ScrollableBox`.
- `zircon_runtime/src/ui/accessibility/action/scroll.rs` now stays focused on `ScrollTo` action validation, runtime scroll mutation, unchanged handling, and scroll binding report dispatch.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/scroll.rs" "zircon_runtime/src/ui/accessibility/action/scroll/payload.rs" "zircon_runtime/src/ui/accessibility/action/range.rs" "zircon_runtime/src/ui/accessibility/action/result.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for scroll routing, scroll payload parsing, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated editor ZUI, sound, Hub, texture, and other Cargo/Rust compiler jobs active during process inspections.

Accessibility expanded target submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/expanded/target.rs` now owns Disclosure/Popup widget matching, authored `open_property` resolution, expandable target typing, and popup/disclosure component-event mapping.
- `zircon_runtime/src/ui/accessibility/action/expanded.rs` now stays focused on `Expand`/`Collapse` action exposure checks, expanded-state availability checks, mutation dispatch, binding-report diagnostics, and accepted/rejected action results.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/expanded.rs" "zircon_runtime/src/ui/accessibility/action/expanded/target.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/result.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for expanded dispatch, expanded target resolution, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated platform, Hub, ZUI, sound, material, texture, and other Cargo/Rust compiler jobs active in the shared checkout.

Accessibility expanded result submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/expanded/result.rs` now owns accepted/unchanged/rejected `Expand`/`Collapse` mutation result shaping, binding-report diagnostics, and accepted popup/disclosure component-event emission.
- `zircon_runtime/src/ui/accessibility/action/expanded.rs` now stays focused on `Expand`/`Collapse` action exposure checks, expanded-state availability checks, expandable target lookup, runtime property mutation dispatch, and delegation to result shaping.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/expanded.rs" "zircon_runtime/src/ui/accessibility/action/expanded/result.rs" "zircon_runtime/src/ui/accessibility/action/expanded/target.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/result.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for expanded dispatch, expanded result helper, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated Cargo/Rust compiler jobs active in the shared checkout.

Accessibility range adjustment submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/range/adjustment.rs` now owns `Increment`/`Decrement` direction parsing and `ValueChanged` component-event construction for accepted Slider range mutations.
- `zircon_runtime/src/ui/accessibility/action/range.rs` now stays focused on Slider role/action validation, default range step mutation, binding-report diagnostics, mutation rejection handling, and accepted action result shaping.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/range.rs" "zircon_runtime/src/ui/accessibility/action/range/adjustment.rs" "zircon_runtime/src/ui/accessibility/action/result.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for range dispatch, range adjustment helper, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated platform/runtime, Hub, ZUI, sound, material, texture, and other Cargo/Rust compiler jobs active in the shared checkout.

Accessibility range result submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/range/result.rs` now owns accepted/unchanged/rejected Slider `Increment`/`Decrement` range-step mutation result shaping and binding-report diagnostics.
- `zircon_runtime/src/ui/accessibility/action/range.rs` now stays focused on Slider action exposure checks, role validation, direction selection, range-step mutation dispatch, and delegation to result shaping.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/range.rs" "zircon_runtime/src/ui/accessibility/action/range/adjustment.rs" "zircon_runtime/src/ui/accessibility/action/range/result.rs" "zircon_runtime/src/ui/accessibility/action/result.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for range dispatch, range result helper, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated Cargo/Rust compiler jobs active in the shared checkout.

Accessibility activate fallback submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/activate/fallback.rs` now owns the generic `Commit { property: "activated", value: true }` fallback component event used when typed widget activation does not handle `Activate`.
- `zircon_runtime/src/ui/accessibility/action/activate.rs` now stays focused on `Activate` action exposure checks, typed widget activation reuse, binding/component report propagation, mutation-error handling, and accepted fallback result shaping.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/activate.rs" "zircon_runtime/src/ui/accessibility/action/activate/fallback.rs" "zircon_runtime/src/ui/accessibility/action/result.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for activate dispatch, activate fallback helper, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated platform/runtime, Hub, ZUI, sound, material, texture, and other Cargo/Rust compiler jobs active in the shared checkout.

Accessibility text metadata submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/text_state/metadata.rs` now owns TextInput retained metadata mutation, binding-report diagnostic emission, selection/composition note construction, and composition cleanup writes.
- `zircon_runtime/src/ui/accessibility/action/text_state.rs` now stays focused on TextInput read-only checks, selected-range calculation, UTF-8 boundary clamping, and caret/selection synchronization entry points.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/text_state.rs" "zircon_runtime/src/ui/accessibility/action/text_state/metadata.rs" "zircon_runtime/src/ui/accessibility/action/text/replace.rs" "zircon_runtime/src/ui/accessibility/action/text/selection.rs" "zircon_runtime/src/ui/accessibility/action/value.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for text-state helper, text metadata helper, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated Hub, platform, material, and other Cargo/Rust compiler jobs active in the shared checkout.



Accessibility ReplaceSelectedText replacement submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/text/replace/replacement.rs` now owns selected-range next-text construction, retained TextInput constraint sanitization, `accessibility_replace_selected_text_sanitized` note selection, and caret collapse offset calculation.
- `zircon_runtime/src/ui/accessibility/action/text/replace.rs` now stays focused on `ReplaceSelectedText` action exposure checks, TextInput/read-only/value-target gates, property mutation, binding-report diagnostics, edit metadata sync, value-change events, and accepted/rejected result shaping.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/text.rs" "zircon_runtime/src/ui/accessibility/action/text/replace.rs" "zircon_runtime/src/ui/accessibility/action/text/replace/replacement.rs" "zircon_runtime/src/ui/accessibility/action/text_state.rs" "zircon_runtime/src/ui/accessibility/action/value_target.rs" "zircon_runtime/src/ui/accessibility/action/result.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for replace dispatch, replacement helper, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated ZUI, texture, Hub, platform, material, and other Cargo/Rust compiler jobs active in the shared checkout.

Accessibility ReplaceSelectedText result submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/text/replace/result.rs` now owns accepted/unchanged/rejected `ReplaceSelectedText` mutation result shaping, binding-report diagnostics, edit metadata sync, and `ValueChanged` component-event construction.
- `zircon_runtime/src/ui/accessibility/action/text/replace.rs` now stays focused on `ReplaceSelectedText` action exposure checks, TextInput/read-only/value-target gates, selected replacement construction, runtime property mutation request construction, and dispatch to result shaping.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/text.rs" "zircon_runtime/src/ui/accessibility/action/text/replace.rs" "zircon_runtime/src/ui/accessibility/action/text/replace/replacement.rs" "zircon_runtime/src/ui/accessibility/action/text/replace/result.rs" "zircon_runtime/src/ui/accessibility/action/text_state.rs" "zircon_runtime/src/ui/accessibility/action/value_target.rs" "zircon_runtime/src/ui/accessibility/action/result.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for replace dispatch, replacement helper, result helper, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated Cargo/Rust compiler jobs active in the shared checkout.

Accessibility SetTextSelection payload submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/text/selection/payload.rs` now owns missing `text_selection` payload rejection metadata and caret/anchor/focus UTF-8 boundary clamping for neutral `SetTextSelection` requests.
- `zircon_runtime/src/ui/accessibility/action/text/selection.rs` now stays focused on `SetTextSelection` action exposure checks, TextInput role validation, accepted result shaping, and retained selection metadata synchronization.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/text.rs" "zircon_runtime/src/ui/accessibility/action/text/selection.rs" "zircon_runtime/src/ui/accessibility/action/text/selection/payload.rs" "zircon_runtime/src/ui/accessibility/action/text_state.rs" "zircon_runtime/src/ui/accessibility/action/result.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for selection dispatch, selection payload helper, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated Cargo/Rust compiler jobs active in the shared checkout.

Accessibility SetValue TextInput preparation submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/value/text.rs` now owns TextInput `SetValue` read-only rejection, whole-field retained TextInput constraint sanitization, and `accessibility_text_value_sanitized` note selection.
- `zircon_runtime/src/ui/accessibility/action/value.rs` now stays focused on `SetValue` action exposure checks, role/property/payload gates, runtime property mutation, binding-report diagnostics, edit metadata sync, value-change events, and accepted/rejected result shaping.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/value.rs" "zircon_runtime/src/ui/accessibility/action/value/text.rs" "zircon_runtime/src/ui/accessibility/action/value/payload.rs" "zircon_runtime/src/ui/accessibility/action/value_target.rs" "zircon_runtime/src/ui/accessibility/action/result.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for value dispatch, TextInput SetValue helper, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated Cargo/Rust compiler jobs active in the shared checkout.

Accessibility SetValue mutation result submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/value/result.rs` now owns accepted/unchanged/rejected `SetValue` mutation result shaping, binding-report diagnostics, TextInput edit metadata sync, and `ValueChanged` component-event construction.
- `zircon_runtime/src/ui/accessibility/action/value.rs` now stays focused on `SetValue` action exposure checks, role/property/payload gates, TextInput preparation delegation, runtime property mutation request construction, and dispatch to result shaping.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/value.rs" "zircon_runtime/src/ui/accessibility/action/value/result.rs" "zircon_runtime/src/ui/accessibility/action/value/text.rs" "zircon_runtime/src/ui/accessibility/action/value/payload.rs" "zircon_runtime/src/ui/accessibility/action/value_target.rs" "zircon_runtime/src/ui/accessibility/action/result.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for value dispatch, SetValue result helper, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated Cargo/Rust compiler jobs active in the shared checkout.

Accessibility ScrollTo binding submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/scroll/binding.rs` now owns `AccessibilityAction -> RuntimeState(scroll_offset)` binding report construction, dirty fallback selection, scroll-state offset reads, and scroll binding diagnostic notes.
- `zircon_runtime/src/ui/accessibility/action/scroll.rs` now stays focused on `ScrollTo` action exposure checks, payload reuse, runtime scroll mutation, unchanged handling, and accepted/rejected action result shaping.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/scroll.rs" "zircon_runtime/src/ui/accessibility/action/scroll/binding.rs" "zircon_runtime/src/ui/accessibility/action/scroll/payload.rs" "zircon_runtime/src/ui/accessibility/action/result.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for scroll dispatch, scroll binding helper, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated platform, Hub, sound, material, and other Cargo/Rust compiler jobs active in the shared checkout.
Accessibility ScrollTo result submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/scroll/result.rs` now owns accepted/unchanged/rejected `ScrollTo` mutation result shaping, unchanged diagnostic notes, and scroll binding report dispatch after mutation.
- `zircon_runtime/src/ui/accessibility/action/scroll.rs` now stays focused on `ScrollTo` action exposure checks, offset payload reuse, previous-offset capture, runtime scroll mutation dispatch, and result shaping delegation.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/scroll.rs" "zircon_runtime/src/ui/accessibility/action/scroll/binding.rs" "zircon_runtime/src/ui/accessibility/action/scroll/payload.rs" "zircon_runtime/src/ui/accessibility/action/scroll/result.rs" "zircon_runtime/src/ui/accessibility/action/result.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for scroll dispatch, scroll result helper, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated Hub, editor, sound, app profiling, material, and other Cargo/Rust compiler jobs active in the shared checkout.

Accessibility result binding diagnostic submodule split evidence from 2026-05-27:

- `zircon_runtime/src/ui/accessibility/action/result/binding.rs` now owns property mutation binding-report diagnostic note emission, including applied/unchanged/rejected counts and the first binding source note.
- `zircon_runtime/src/ui/accessibility/action/result.rs` now stays focused on handled/unhandled result shaping, status note construction, unsupported-role replies, and a structural re-export of the binding diagnostic helper used by action submodules.
- `rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/action/result.rs" "zircon_runtime/src/ui/accessibility/action/result/binding.rs" "zircon_runtime/src/ui/accessibility/action/popup.rs" "zircon_runtime/src/ui/accessibility/action/range.rs" "zircon_runtime/src/ui/accessibility/action/value.rs"`: PASS.
- Scoped `git diff --check` for tracked touched files passed with LF/CRLF warnings only. Touched-file trailing-whitespace scan and conflict-marker scan passed for result helper, result binding helper, docs, plan, and session archive files.
- No new Cargo check was started because process inspection showed unrelated layout/platform, Hub, ZUI, sound, material, texture, and other Cargo/Rust compiler jobs active in the shared checkout.
