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
  - zircon_runtime/src/ui/accessibility/accesskit.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs
  - zircon_runtime/src/ui/surface/component_state.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
implementation_files:
  - zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs
  - zircon_runtime/src/ui/accessibility/mod.rs
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/accessibility/name.rs
  - zircon_runtime/src/ui/accessibility/diagnostics.rs
  - zircon_runtime/src/ui/accessibility/action.rs
  - zircon_runtime/src/ui/accessibility/accesskit.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs
  - zircon_runtime/src/ui/surface/component_state.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime_interface/src/ui/widget.rs
plan_sources:
  - docs/superpowers/plans/2026-05-09-accesskit-bridge.md
  - docs/superpowers/specs/2026-05-08-accesskit-bridge-design.md
  - user: 2026-05-09 Milestone 2 Accessibility Action Dispatch Through Shared UI Behavior
  - user: 2026-05-09 Milestone 3 Runtime ABI Snapshot Capture And Serialized Action Roundtrip
  - user: 2026-05-16 Bevy-level UI/Text/Widgets/Focus/A11y completion plan continuation
tests:
  - zircon_runtime/src/dynamic_api/tests.rs
  - zircon_runtime/src/ui/tests/accessibility.rs
  - zircon_runtime/src/ui/tests/accessibility_state_values.rs
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

Widget behavior is also an accessibility source. When `UiWidgetContract::behavior` is explicit, extraction uses it before component-name fallback: `Button` and `MenuItem` expose activation, `Toggle` exposes a checkbox role, `RadioGroup` exposes a radio-group role, `Radio` exposes a radio role with activate action, `Range` exposes a slider role with increment/decrement/set-value actions, and `TextInput` exposes a text-input role with set-value action. `UiWidgetBehavior::Auto` preserves the old component-name inference path for legacy templates; `Passive` suppresses default role/action inference for structural components.

This mirrors Bevy's separation between headless behavior and accessibility metadata. Local Bevy `dev/bevy/crates/bevy_ui_widgets/src/{button.rs,checkbox.rs,slider.rs}` attach AccessKit roles to behavior components, while `dev/bevy/crates/bevy_a11y/src/lib.rs` keeps AccessKit node representation reusable. Zircon keeps a neutral tree first so runtime/editor hosts share extraction semantics and only the optional `accessibility-accesskit` bridge performs AccessKit conversion.

Radio extraction follows local Bevy `dev/bevy/crates/bevy_ui_widgets/src/radio.rs`, which gives groups and radio buttons distinct AccessKit roles and lets activation route through the same headless widget path. The neutral tree now has `UiA11yRole::RadioGroup` and maps it to `accesskit::Role::RadioGroup` when the optional bridge is enabled; `Radio` keeps checked-state extraction through authored `checked_property`, retained attributes, and runtime component-state values.

Scrollbar extraction follows local Bevy `dev/bevy/crates/bevy_ui_widgets/src/scrollbar.rs`: a scrollbar widget is headless control chrome for a scrollable container, so `UiWidgetBehavior::Scrollbar` and `ScrollbarThumb` do not infer a role, name, or default action and are excluded from the neutral tree unless the author supplies explicit a11y metadata. The scrollable container itself can expose `ScrollTo`, which keeps assistive scroll behavior on the content owner rather than on decorative track/thumb nodes.

## Inclusion Rules

The extractor includes visible roots and nodes when they have explicit accessibility metadata, a non-default `UiWidgetContract`, focus/pointer/widget interactivity, text metadata, image/icon alt metadata, or are relation targets from participating owners. Relation targets are collected only from owners that are themselves included without relation-target promotion, so references owned only by excluded hidden subtrees do not retain otherwise anonymous targets. Normal hidden nodes and ordinary descendants of hidden ancestors are excluded. Hidden nodes referenced as labels or descriptions are retained as label-only nodes with no children or actions only when they provide usable source text. Hidden textless description targets are pruned after resolution when the owner description is cleared. Hidden `label_for` and widget `label_for` targets are not retained as label/description source nodes unless they are independently includable. Hidden relation-only nodes are not exposed through normal parent `children` traversal.

Disabled nodes remain discoverable. Their invalid actions are filtered during extraction; currently only `Focus` is retained for disabled focusable nodes. The extractor records `DisabledAction` diagnostics for the filtered actions so the output tree is safe while preserving validation evidence.

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

`hidden` follows effective retained node visibility. `disabled` combines runtime enabled state, typed component-state `disabled`/`enabled` values, canonical runtime disabled flags, retained `disabled` attributes, and `UiWidgetContract.disabled`. `focused` is true only when the surface focus points at a visible, enabled, included node. `selected` reads the retained `selected` attribute before typed component-state `selected` values and canonical selected flags, so menu/list/tab-like authored controls can expose selection without a component-name special case and unrelated component-state records cannot mask authored selection. `expanded` reads the authored disclosure/popup `open_property` attribute first, then the same property in runtime component state; legacy extraction falls back to retained/component-state `expanded`, `popup_open`, or `open` values and then true runtime expanded/popup flags. This keeps custom disclosure aliases from being masked by unrelated component-state entries. `checked` reads the authored `checked_property` attribute first, then component-state values for that alias, runtime component checked flags, and widget/static state, so custom toggles expose the same checked state that pointer, keyboard, accessibility activation, or code-side component state mutate. `pressed` reads typed component-state `pressed`/`active` values, canonical pressed flags, retained `pressed`/`active` attributes, and legacy node state flags, matching the headless button press model used by runtime pointer routing. `value` reads the authored `value_property` attribute first, then the same property in runtime component state; legacy extraction reads retained `value` and text-input `text` attributes before component-state `value`/`text`, then falls back to `UiWidgetContract.value` projected with `UiValue::display_text()` so assistive hosts receive user-facing text instead of Rust debug formatting.

Bounds come from the arranged tree first and fall back to the retained node layout cache when the arranged tree has not been rebuilt yet. Bounds must be finite with positive width and height. Named or interactive visible nodes without valid arranged or layout-cache bounds stay in the snapshot but receive `MissingBounds` diagnostics. Hidden relation-only nodes retained only as label/description sources do not emit `MissingBounds` noise.

## Diagnostics

The extraction and validation passes check malformed label references, duplicate ids, dangling label/description references, missing names, missing bounds, hidden focusable nodes, disabled invalid actions, unsupported role/action pairs, invalid focus, excluded focus, and simple two-node `labelled_by` cycles. Malformed `labelled_by`/`label_for` strings record `InvalidLabelReference`; malformed `#` description references record `DanglingDescription`. Interactive, focusable, or actionable nodes without resolved accessible names record `MissingName`. Hidden focusable nodes that are excluded from normal traversal record `HiddenFocusable` even when they are also focused; hidden focused nodes also produce `ExcludedFocusedNode` through focus validation.

If focus points at a missing, hidden, disabled, or excluded node, the snapshot records an error diagnostic and falls back to the first visible enabled snapshot root, or clears focus if no valid fallback exists. The validation pass also synchronizes `state.focused` so exactly the snapshot focus target is marked focused, without mutating `UiSurface.focus`.

## Action Dispatch

`UiInputEvent::Accessibility` is routed through `UiSurface::dispatch_input_event` into `dispatch_accessibility_action`. The dispatcher first captures a fresh accessibility snapshot and validates the requested target against the snapshot node list, hidden state, disabled state, and the requested behavior. It returns ordinary `UiInputDispatchResult` values, so action status is encoded in `diagnostics.notes` with strings such as `status=accepted`, `status=rejected`, `status=unsupported`, `status=stale_target`, and role/error codes.

Accepted `Focus` actions call `UiSurface::focus_node_with_reason` with `UiFocusChangeReason::Programmatic` and visible focus reason `UiFocusVisibleReason::Programmatic`. Successful focus dispatch sets `diagnostics.routed = true`, `route_target = Some(target)`, and `handled_phase = "accessibility.focus"`.

Accepted `Activate` actions use the existing component event and widget behavior vocabulary instead of a host-only branch. When the current snapshot target exposes `Activate` and is not disabled or hidden, dispatch first tries the same typed default widget behavior used by focused keyboard activation: toggles mutate their checked property using retained attributes or component-state alias values as the current state, disclosure and popup controls mutate their authored open property using retained/component-state alias values as the current state, and button/menu bindings can receive commit events. If no typed behavior handles the target, dispatch preserves the existing generic button-compatible event by emitting `UiComponentEvent::Commit { property: "activated", value: UiValue::Bool(true) }`. Handled activation records phase `accessibility.activate`.

Accepted `SetValue` actions are limited to `TextInput` and `Slider` roles that already expose a mutable value property. If `UiWidgetContract::value_property` is authored, dispatch uses that alias when it exists as a retained attribute, runtime component-state value, or static widget value; this keeps custom range and text controls on the same property alias used by pointer, keyboard, and accessibility extraction. Without an authored alias, dispatch preserves legacy compatibility by mutating existing `value` first, otherwise existing `text`. It does not create a new fallback property solely because the role is editable. Slider values must be finite floats. The mutation goes through `UiSurface::mutate_property` with `UiReflectedPropertySource::RuntimeState`; accepted mutations now also mirror the typed value into `UiSurfaceComponentStateStore` before boolean pseudo-state flags are synchronized. Accepted mutations emit `UiComponentEvent::ValueChanged` for the mutated property and use phase `accessibility.set_value`; rejected mutations return structured rejection notes instead of direct metadata writes.

Unsupported or rejected behavior remains explicit. Stale targets return `status=stale_target` without routing to a runtime node. Hidden or snapshot-excluded targets return `status=rejected` with `hidden_target` or `excluded_target`. Disabled non-focus requests return `status=rejected code=disabled_action`. `Increment` and `Decrement` are accepted only for slider-like targets that expose the range value contract used by `mutate_default_range_step_value`; `value_property`, `min_property`, `max_property`, and `step_property` resolve retained attributes first, then runtime component-state values. `ScrollTo` is accepted only for nodes whose retained container is scrollable and whose snapshot exposes the action; it consumes `numeric_value` or a parseable string `value`, then delegates to `UiRuntimeTreeScrollExt::set_scroll_offset` so clamping, dirty flags, and virtual-window invalidation remain owned by the scrollable container. Other roles return `status=unsupported code=unsupported_role_action`. `Dismiss` returns `status=unsupported code=unsupported_role_action` and the exact note `accessibility dismiss requires popup id` until the neutral request includes a popup id source.

## AccessKit Bridge

`zircon_runtime/src/ui/accessibility/accesskit.rs` is compiled only behind `accessibility-accesskit`, which depends on `accesskit 0.22` without default features. The bridge deliberately consumes the neutral snapshot instead of walking `UiSurface` directly, so AccessKit platform adapters cannot bypass Zircon's existing inclusion, name, bounds, diagnostics, focus, and disabled/hidden filtering rules.

`snapshot_to_accesskit_tree_update(...)` maps `UiAccessibilityTreeSnapshot` into an AccessKit full-tree `TreeUpdate`. Single-root snapshots use the Zircon root id directly. Multi-root snapshots gain a synthetic AccessKit `Window` root with `NodeId(u64::MAX)` and the Zircon roots as children, keeping AccessKit's single-root tree requirement separate from the neutral snapshot format. Focus falls back to the AccessKit root when the neutral focused id is absent from the emitted node list.

Role mapping follows Bevy's AccessKit precedent where practical: buttons/images/labels become AccessKit button/image/label-style nodes, editor panels become `Pane`, text inputs become `TextInput`, and sliders/checkboxes/radio buttons/menu items/tabs map to their native AccessKit roles. Zircon `Text` nodes expose their name as AccessKit `value`, matching Bevy's label behavior; other controls expose names as AccessKit `label`.

State and relation mapping preserves the neutral contract: hidden and disabled become AccessKit flags, selected and expanded become AccessKit boolean properties, checked maps to `Toggled::{False, True, Mixed}`, bounds become AccessKit `Rect`, child ids become `children`, `labelled_by` becomes `labelled_by`, and `label_for` becomes `controls`. String widget values become AccessKit `value`; finite numeric strings additionally populate `numeric_value` for controls such as sliders.

Supported AccessKit actions map back into neutral requests before runtime dispatch. `Click`, `Focus`, `Increment`, `Decrement`, `SetValue`, `ReplaceSelectedText`, `ScrollIntoView`, `ScrollToPoint`, `Blur`, `Collapse`, and `HideTooltip` are accepted. Value and numeric payloads are copied into the neutral request. Unsupported AccessKit-only actions return `None` so the eventual app-host adapter can decline them without inventing hidden runtime behavior.

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
- accessibility action dispatch for accepted focus, accepted activation component commits, typed widget `Activate` mutation through authored toggle aliases backed by retained attributes or runtime component state, disclosure aliases, disabled and selected state from retained state/attributes, expanded/checked/value snapshot state from authored widget property aliases, stale target rejection, hidden and visible-excluded target rejection, disabled activation rejection, unsupported increment, accessibility increment/decrement through retained or component-state backed range value/min/max/step aliases, unsupported dismiss, editable text `SetValue` property mutation, `SetValue` through authored `UiWidgetContract::value_property` aliases backed by retained attributes or runtime component state, and unsupported `SetValue` when no existing mutable value metadata property is available.

`zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs` covers the Bevy-aligned scrollbar a11y boundary: default Scrollbar/ScrollbarThumb widgets are excluded from the neutral tree unless explicit a11y metadata is authored, explicit `UiA11yRole::Scrollbar` still maps through the optional AccessKit bridge, and `ScrollTo` mutates the scrollable container rather than the headless scrollbar chrome.

`zircon_runtime/src/dynamic_api/tests.rs` covers runtime API table accessibility capture presence, null output rejection, wrong ABI rejection before session lookup, unknown viewport rejection, serialized preview snapshot capture/free, invalid accessibility free ownership rejection, invalid accessibility action JSON rejection, and valid action-payload rejection when the dynamic preview has no retained UI surface.

`zircon_runtime/src/ui/tests/widget_radio_behavior.rs` covers RadioGroup/Radio accessibility role projection and default Radio activate action alongside runtime pointer selection, disabled group rejection, and keyboard selection behavior, so a11y role inference stays tied to the same widget contract that mutates checked state.

## Validation Evidence

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
