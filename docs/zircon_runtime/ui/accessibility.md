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
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
implementation_files:
  - zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs
  - zircon_runtime/src/ui/accessibility/mod.rs
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/accessibility/name.rs
  - zircon_runtime/src/ui/accessibility/diagnostics.rs
  - zircon_runtime/src/ui/accessibility/action.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
plan_sources:
  - docs/superpowers/plans/2026-05-09-accesskit-bridge.md
  - docs/superpowers/specs/2026-05-08-accesskit-bridge-design.md
  - user: 2026-05-09 Milestone 2 Accessibility Action Dispatch Through Shared UI Behavior
  - user: 2026-05-09 Milestone 3 Runtime ABI Snapshot Capture And Serialized Action Roundtrip
tests:
  - zircon_runtime/src/dynamic_api/tests.rs
  - zircon_runtime/src/ui/tests/accessibility.rs
  - cargo test -p zircon_runtime --lib dynamic_api --locked --jobs 1 --target-dir "E:\\cargo-targets\\zircon-accesskit-bridge" --message-format short --color never
  - cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\\cargo-targets\\zircon-accesskit-bridge" --message-format short --color never
doc_type: module-detail
---

# Runtime UI Accessibility

`zircon_runtime::ui::accessibility` extracts a neutral `UiAccessibilityTreeSnapshot` from an existing `UiSurface` and maps neutral accessibility action requests into existing runtime UI behavior. The dynamic runtime API serializes the neutral snapshot/action DTOs across the `zircon_app` ABI boundary. The module does not perform AccessKit conversion or winit host integration in Milestone 3; those remain later bridge milestones.

## Extraction Source

`UiTemplateNodeMetadata` retains `UiAccessibilityContract` and `UiWidgetContract` with serde defaults. Template tree building copies `UiTemplateNode.a11y` and `UiTemplateNode.widget` into retained tree metadata, so snapshot extraction reads from the same `UiTree` used by layout, hit testing, render extraction, and focus.

`UiSurface::accessibility_snapshot()` delegates to `crate::ui::accessibility::accessibility_snapshot(self)` and does not mutate layout or focus state. `surface.rs` remains an oversized retained UI owner file; Milestone 1 only added this narrow delegating method and did not refactor its existing responsibilities.

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

`hidden` follows effective retained node visibility. `disabled` combines runtime enabled state and `UiWidgetContract.disabled`. `focused` is true only when the surface focus points at a visible, enabled, included node. `checked` and `pressed` are copied from available retained state/widget metadata. Widget `value` is projected with `UiValue::display_text()` so assistive hosts receive user-facing text instead of Rust debug formatting.

Bounds come from the arranged tree first and fall back to the retained node layout cache when the arranged tree has not been rebuilt yet. Bounds must be finite with positive width and height. Named or interactive visible nodes without valid arranged or layout-cache bounds stay in the snapshot but receive `MissingBounds` diagnostics. Hidden relation-only nodes retained only as label/description sources do not emit `MissingBounds` noise.

## Diagnostics

The extraction and validation passes check malformed label references, duplicate ids, dangling label/description references, missing names, missing bounds, hidden focusable nodes, disabled invalid actions, unsupported role/action pairs, invalid focus, excluded focus, and simple two-node `labelled_by` cycles. Malformed `labelled_by`/`label_for` strings record `InvalidLabelReference`; malformed `#` description references record `DanglingDescription`. Interactive, focusable, or actionable nodes without resolved accessible names record `MissingName`. Hidden focusable nodes that are excluded from normal traversal record `HiddenFocusable` even when they are also focused; hidden focused nodes also produce `ExcludedFocusedNode` through focus validation.

If focus points at a missing, hidden, disabled, or excluded node, the snapshot records an error diagnostic and falls back to the first visible enabled snapshot root, or clears focus if no valid fallback exists. The validation pass also synchronizes `state.focused` so exactly the snapshot focus target is marked focused, without mutating `UiSurface.focus`.

## Action Dispatch

`UiInputEvent::Accessibility` is routed through `UiSurface::dispatch_input_event` into `dispatch_accessibility_action`. The dispatcher first captures a fresh accessibility snapshot and validates the requested target against the snapshot node list, hidden state, disabled state, and the requested behavior. It returns ordinary `UiInputDispatchResult` values, so action status is encoded in `diagnostics.notes` with strings such as `status=accepted`, `status=rejected`, `status=unsupported`, `status=stale_target`, and role/error codes.

Accepted `Focus` actions call `UiSurface::focus_node_with_reason` with `UiFocusChangeReason::Programmatic` and visible focus reason `UiFocusVisibleReason::Programmatic`. Successful focus dispatch sets `diagnostics.routed = true`, `route_target = Some(target)`, and `handled_phase = "accessibility.focus"`.

Accepted `Activate` actions use the existing component event vocabulary instead of a host-only branch. When the current snapshot target exposes `Activate` and is not disabled or hidden, dispatch emits a delivered `UiComponentEvent::Commit { property: "activated", value: UiValue::Bool(true) }`, marks the reply handled, and records phase `accessibility.activate`.

Accepted `SetValue` actions are limited to `TextInput` and `Slider` roles that already expose a retained `value` or `text` metadata property. Dispatch mutates existing `value` first, otherwise existing `text`; it does not create a new fallback property solely because the role is editable. Slider values must be finite floats. The mutation goes through `UiSurface::mutate_property` with `UiReflectedPropertySource::RuntimeState`. Accepted mutations emit `UiComponentEvent::ValueChanged` for the mutated property and use phase `accessibility.set_value`; rejected mutations return structured rejection notes instead of direct metadata writes.

Unsupported or rejected behavior remains explicit. Stale targets return `status=stale_target` without routing to a runtime node. Hidden or snapshot-excluded targets return `status=rejected` with `hidden_target` or `excluded_target`. Disabled non-focus requests return `status=rejected code=disabled_action`. `Increment`, `Decrement`, and `ScrollTo` currently return `status=unsupported code=unsupported_role_action` because there is no generic runtime slider/scroll action path in M2. `Dismiss` returns `status=unsupported code=unsupported_role_action` and the exact note `accessibility dismiss requires popup id` until the neutral request includes a popup id source.

## Dynamic Runtime ABI

`zircon_runtime/src/dynamic_api/exports.rs` now exposes `ZrRuntimeApiV1.capture_accessibility_tree` for hosts that understand the appended optional ABI field. The function validates `ZrRuntimeAccessibilityTreeRequestV1.abi_version`, rejects non-default viewport handles with `NotFound`, captures the current dynamic preview accessibility snapshot, serializes it as JSON `UiAccessibilityTreeSnapshot`, and writes the bytes into `ZrOwnedByteBuffer`.

Accessibility tree byte ownership mirrors frame byte ownership but uses a distinct owner token, `0x5a52_4131_3159_0001`, and a dedicated `free_runtime_accessibility_bytes` callback. Null output pointers return `InvalidArgument` with the diagnostic `missing accessibility tree output`. Invalid ownership, null data, or impossible length/capacity pairs are rejected by the accessibility free callback as invalid runtime accessibility buffers. Existing frame capture and frame byte reclamation remain unchanged.

The dynamic preview session currently owns the 3D runtime preview state and render bridge, not a retained `UiSurface`. For that reason `RuntimeDynamicSession::capture_accessibility_tree` returns a minimal neutral preview snapshot instead of fake widget data: one `Panel` root named `Zircon Runtime Preview`, no focused widget, no children, and an info diagnostic with the exact message `runtime UI surface accessibility extraction unavailable in dynamic preview`.

`RuntimeDynamicSession::handle_event` handles `ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1` by deserializing `UiAccessibilityActionRequest` from `ZrRuntimeEventV1.payload`. Invalid JSON returns `InvalidArgument` with `invalid accessibility action payload`. Valid requests deserialize successfully, then return `NotFound` with `runtime UI surface accessibility action dispatch unavailable in dynamic preview` because there is no stored `UiSurface` to dispatch through in this runtime preview path. The runtime UI action dispatcher remains the owner of stale-target, hidden-target, disabled-action, and unsupported-action semantics when a retained surface is available.

## Focused Tests

`zircon_runtime/src/ui/tests/accessibility.rs` covers:

- extraction of interactive, text, alt, and explicit accessibility nodes;
- widget-only inclusion from non-default `UiWidgetContract`;
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
- accessibility action dispatch for accepted focus, accepted activation component commits, stale target rejection, hidden and visible-excluded target rejection, disabled activation rejection, unsupported increment, unsupported dismiss, editable text `SetValue` property mutation, and unsupported `SetValue` when no existing `text` or `value` metadata property is available.

`zircon_runtime/src/dynamic_api/tests.rs` covers runtime API table accessibility capture presence, null output rejection, wrong ABI rejection before session lookup, unknown viewport rejection, serialized preview snapshot capture/free, invalid accessibility free ownership rejection, invalid accessibility action JSON rejection, and valid action-payload rejection when the dynamic preview has no retained UI surface.

## Validation Evidence

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
