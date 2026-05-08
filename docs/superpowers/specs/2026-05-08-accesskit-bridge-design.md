---
related_code:
  - dev/bevy/crates/bevy_a11y/src/lib.rs
  - dev/bevy/crates/bevy_ui/src/accessibility.rs
  - dev/bevy/crates/bevy_winit/src/accessibility.rs
  - dev/slint/internal/backends/winit/accesskit.rs
  - dev/godot/servers/display/accessibility_server.h
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/tree/node/tree_node.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
implementation_files:
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime_interface/src/tests/accessibility_contracts.rs
  - zircon_runtime/src/ui/accessibility/mod.rs
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/accessibility/name.rs
  - zircon_runtime/src/ui/accessibility/diagnostics.rs
  - zircon_runtime/src/ui/accessibility/action.rs
  - zircon_runtime/src/ui/accessibility/accesskit.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tests/accessibility.rs
  - zircon_app/src/entry/runtime_accessibility/mod.rs
  - zircon_app/src/entry/runtime_accessibility/accesskit_host.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
plan_sources:
  - user: 2026-05-08 AccessKit Bridge missing gap list
  - .codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md
  - .codex/plans/Bevy-Informed Zircon UI 架构优化里程碑计划.md
tests:
  - zircon_runtime_interface/src/tests/accessibility_contracts.rs
  - zircon_runtime/src/ui/tests/accessibility.rs
  - zircon_app/src/entry/tests/mod.rs
  - cargo test -p zircon_runtime_interface --lib accessibility_contracts --locked
  - cargo test -p zircon_runtime --lib ui::tests::accessibility --locked
  - cargo test -p zircon_app --lib runtime_accessibility --locked
doc_type: design-spec
---

# AccessKit Bridge Design

## Goal

Close the current accessibility gap by turning Zircon's neutral `UiAccessibilityNode`, `UiAccessibilityTreeSnapshot`, and `UiAccessibilityAction` DTOs into an end-to-end accessibility bridge. The bridge must generate a runtime-owned accessibility tree from `UiSurface`, convert it to AccessKit when the winit host is active, route AccessKit action requests back into the UI event/command path, and validate invalid tree states before they reach a screen reader.

The approved first scope is end-to-end host coverage. It includes runtime snapshot generation, diagnostics, AccessKit conversion, winit adapter lifecycle, action roundtrip, and smoke tests. It does not require a platform screen reader to run in CI.

## Current Context

`zircon_runtime_interface/src/ui/accessibility.rs` already defines neutral roles, states, actions, nodes, snapshots, diagnostics, and authoring contracts. Those contracts are currently DTO-only. `zircon_runtime::ui::surface::UiSurface` owns the retained `UiTree`, arranged tree, hit grid, focus state, render extract, and input dispatch path, but it does not build an accessibility snapshot. `zircon_app` owns the winit `ApplicationHandler`, the created `Window`, and the dynamic runtime session API, but it has no AccessKit adapter or accessibility action path.

The existing runtime API has `handle_event` and `capture_frame`; it has no accessibility snapshot capture function. `ZrRuntimeEventV1` has a payload field and can carry a new accessibility action event kind without changing its struct layout, but the API table needs an appended optional accessibility snapshot capture function for host-driven initial tree and update pulls.

## Reference Evidence

Bevy splits accessibility across three layers. `bevy_a11y` owns reusable AccessKit-oriented primitives and an `AccessibilityRequested` flag. `bevy_ui/src/accessibility.rs` computes labels and bounds from UI nodes and text. `bevy_winit/src/accessibility.rs` owns `accesskit_winit::Adapter`, creates initial trees, sends updates only when active/requested, and forwards AccessKit action requests through an event channel.

Slint's winit backend uses an `AccessKitAdapter` owned by the window adapter. It handles initial tree requests, defers full tree reloads, sends dirty-node updates, tracks focus separately, and translates AccessKit actions such as click, focus, increment, decrement, set value, and replace selected text into toolkit actions.

Godot exposes accessibility through a display/platform server boundary. Its `AccessibilityServer` owns platform window lifecycle, active updates, focus, bounds, roles, names, values, relations, and actions. The useful precedent for Zircon is the explicit platform boundary and the broad validation surface, not the server naming.

Zircon should combine these precedents by keeping neutral contracts in `zircon_runtime_interface`, runtime semantics in `zircon_runtime`, and platform adapter lifecycle in `zircon_app`.

## Approved Scope

In scope:

- Add neutral action request/result DTOs and missing diagnostics under `zircon_runtime_interface::ui::accessibility`.
- Add `UiInputEvent::Accessibility(UiAccessibilityInputEvent)` to the neutral UI input event path so accessibility actions use the same dispatch boundary as pointer, keyboard, text, popup, tooltip, and navigation input.
- Generate `UiAccessibilityTreeSnapshot` from `UiSurface` after layout has produced arranged bounds.
- Resolve accessible names from explicit names, labels, text, icon/image alt metadata, and tooltips.
- Propagate hidden, disabled, focused, selected, checked, expanded, pressed, and value state into snapshot nodes.
- Validate dangling labels, duplicate node ids, missing bounds, illegal focus, hidden focusable nodes, disabled actions, missing names, and relation cycles where applicable.
- Add optional `accesskit` conversion under runtime UI accessibility code, feature-gated away from headless builds.
- Add optional `accesskit_winit` host wiring under `zircon_app`'s winit runtime entry path.
- Add runtime API support for host-side accessibility snapshot capture and action roundtrip.
- Add unit and host smoke tests that do not require an external screen reader process.
- Update module docs and acceptance evidence for the new bridge.

Out of scope for this first end-to-end slice:

- AccessKit bridge support for the Slint editor host. The editor can consume the same neutral contracts later, but this slice targets `zircon_app` runtime winit host wiring first.
- Full text-edit accessibility such as selection offsets, line navigation, replace-selected-text parity, and text range queries beyond basic `SetValue`/text action forwarding.
- Platform-specific screen reader automation in CI. The smoke test should verify adapter/action behavior at the host seam; manual screen reader evidence can be added as a later acceptance artifact.
- Replacing the existing focus, widget, or picking systems wholesale. This design routes through their public seams and only adds the accessibility-specific bridge needed for M9.

## Architecture

### Owner Boundaries

`zircon_runtime_interface::ui::accessibility` remains AccessKit-independent. It owns serializable DTOs for snapshots, diagnostics, action requests, action results, authoring contracts, and input-event payloads. This prevents AccessKit version churn from becoming a public Zircon contract.

`zircon_runtime::ui::accessibility` owns runtime semantics. It reads `UiSurface`, `UiTree`, arranged bounds, focus state, render/text metadata, widget contracts, and template metadata to produce `UiAccessibilityTreeSnapshot`. It owns name resolution, state propagation, validation, and action-to-UI dispatch mapping.

`zircon_runtime::ui::accessibility::accesskit` owns optional conversion from neutral snapshots to `accesskit::TreeUpdate` and from neutral actions to AccessKit actions where tests need direct conversion. This module is feature-gated and does not become the primary public API.

`zircon_app::entry::runtime_accessibility` owns `accesskit_winit::Adapter` lifecycle, activation/deactivation, initial tree requests, update submission, and action request intake. It is the only layer that should know about the winit event loop and window object.

`zircon_editor` is not an owner in this slice. It remains a future consumer of the neutral snapshot and action contracts.

### Cargo Features And Dependencies

The workspace should add `accesskit` and `accesskit_winit` as optional dependencies after verifying version compatibility with the current workspace `winit = 0.31.0-beta.2`. The preferred version family should match the AccessKit versions used by Bevy's current winit integration unless Cargo resolution or winit compatibility proves otherwise.

Feature shape:

- `zircon_runtime/accessibility-accesskit` enables the optional `accesskit` converter only.
- `zircon_app/accessibility-accesskit-winit` enables `platform-winit`, `zircon_runtime/accessibility-accesskit`, `dep:accesskit`, and `dep:accesskit_winit`.
- Headless and non-winit builds compile without AccessKit dependencies.

### Runtime API

Append an optional function to `ZrRuntimeApiV1`:

- `capture_accessibility_tree(session, request, out_buffer) -> ZrStatus` returns a serialized `UiAccessibilityTreeSnapshot` for a viewport.

The new function should be optional for ABI compatibility in generic runtime loading, but `zircon_app` should require it when `accessibility-accesskit-winit` is enabled. The request should include ABI version, viewport handle, viewport metrics or size, and an optional generation hint. The output should use `ZrOwnedByteBuffer` so the host can deserialize the neutral snapshot without adding a C ABI struct for every accessibility field.

Add a new `ZrRuntimeEventV1` kind for accessibility action requests. The event payload should serialize a neutral `UiAccessibilityActionRequest` containing target `UiNodeId`, requested action, optional string value, optional numeric value, source, and viewport. Runtime validates the target against the latest or freshly generated accessibility snapshot before dispatching the action.

### Snapshot Generation

Snapshot extraction starts from `UiSurface` after layout or dirty rebuild. It should use arranged bounds from `UiArrangedTree` or node layout cache and stable `UiNodeId` values. The snapshot roots mirror visible surface roots after filtering hidden nodes.

Node inclusion rules:

- Include nodes with explicit `UiAccessibilityContract`, interactive widget/focus/picking behavior, text content, image/icon alt metadata, or relation targets.
- Exclude fully hidden nodes from normal traversal.
- Keep hidden nodes only when they are referenced as labels or descriptions and their text/name is needed for another node.
- Disabled nodes remain discoverable but expose only actions that remain valid for disabled content, usually none except focus if explicitly supported later.
- Children preserve UI tree order unless z/layer-aware accessibility ordering is added in a later milestone.

Bounds rules:

- Use arranged logical bounds in surface coordinates as the neutral snapshot bounds.
- Mark interactive or named nodes without bounds as diagnostics.
- Defer physical-coordinate conversion to the AccessKit host/converter, using host metrics when needed.

### Name Resolution

Name resolution is deterministic and records the winning source in diagnostics or debug metadata when useful.

Priority:

1. Explicit `UiAccessibilityContract.name`.
2. `labelled_by` target resolved to that node's accessible name or text content.
3. Own text content or resolved text render payload.
4. Image/icon alt metadata, including existing `accessibility_label` forwarded by Material/editor assets.
5. Tooltip text as a fallback.

If an interactive node still has no name after this sequence, emit `MissingName`. If `labelled_by` or `label_for` references cannot resolve, emit `InvalidLabelReference` or the more precise dangling-label diagnostic added by this slice.

### State Propagation

State resolution combines authored accessibility state with runtime state:

- `hidden` follows effective visibility.
- `disabled` is the inverse of enabled state or explicit widget disabled state.
- `focused` matches `UiSurface.focus.focused` only when the node is visible and enabled.
- `selected`, `checked`, `expanded`, `pressed`, and `value` come from widget contract/state flags and component state where available.
- `actions` are filtered by role, enabled state, input policy, and widget capability.

If focus points to a missing, hidden, disabled, or excluded node, emit an invalid-focus diagnostic and fall back snapshot focus to the surface/window root for AccessKit output.

### AccessKit Conversion

The converter maps neutral roles to AccessKit roles, neutral bounds to AccessKit rectangles, and neutral actions to AccessKit actions. It should preserve stable node IDs by deriving AccessKit `NodeId` from `UiNodeId` with a reserved window root ID that cannot collide with UI node IDs.

Conversion rules:

- Add a synthetic window/root node when AccessKit requires a root that is not already represented by the snapshot.
- Attach snapshot roots as children of the window/root node.
- Map `UiAccessibilityAction::Activate` to click/default activation.
- Map `Focus`, `Increment`, `Decrement`, `SetValue`, `ScrollTo`, and `Dismiss` only when supported by the AccessKit version in use.
- Preserve disabled/hidden/checked/selected/expanded/value state when AccessKit exposes equivalent properties.
- Emit conversion diagnostics for unsupported role/action pairs instead of silently dropping required behavior.

### Winit Host Adapter

`zircon_app::entry::runtime_accessibility` should create the AccessKit adapter when a runtime winit window is created and `accessibility-accesskit-winit` is enabled. The adapter state owns:

- the window title/root label,
- the current neutral snapshot,
- the last submitted generation or hash,
- a queue of AccessKit action requests,
- an active/requested flag from AccessKit activation.

The adapter should request the initial tree by calling `RuntimeSession::capture_accessibility_tree`. During `WindowEvent` processing, the host should let the AccessKit adapter process relevant winit events before regular runtime event handling when the AccessKit crate requires it. On redraw or after runtime events that can change UI state, the host should capture a new snapshot and call `update_if_active` only when accessibility is active and the snapshot changed.

Action requests should be drained into `RuntimeSession::handle_event` as serialized `UiAccessibilityActionRequest` payloads. Runtime action results should surface as diagnostics or status failures where possible. If an action targets a stale node, runtime returns a structured rejected result rather than panicking.

### Action Dispatch

Runtime action handling should translate neutral accessibility actions into the same behavior path used by pointer, keyboard, and widget input:

- `Activate` routes to headless widget activation or existing component activation/commit behavior.
- `Focus` routes through the focus APIs with an accessibility focus-visible reason.
- `Increment` and `Decrement` route to slider, scrollbar, spinbox, or value-changing widget behavior where present.
- `SetValue` routes to text/value mutation with validation.
- `ScrollTo` routes to scroll state mutation when supported.
- `Dismiss` routes to popup/menu/tooltip close behavior where supported.

The first implementation can reject unsupported actions with a diagnostic, but it must use one shared action path rather than adding per-widget host special cases.

### Diagnostics

Extend `UiAccessibilityDiagnosticCode` with precise invalid-tree cases:

- `DuplicateNodeId`.
- `MissingBounds`.
- `InvalidFocus`.
- `DanglingLabel`.
- `DanglingDescription`.
- `RelationCycle`.
- `UnsupportedRoleAction`.
- `ExcludedFocusedNode`.

Diagnostics should be included in snapshots and should also be available through module docs/debug snapshots. Errors that make the tree unusable should prevent AccessKit update submission; warnings should allow submission with the best valid tree.

## Testing Strategy

Interface tests:

- `UiAccessibilityActionRequest` and result DTO serde roundtrips.
- New diagnostic codes serialize with stable snake_case names.
- Accessibility input event payload roundtrips without breaking existing input event serde.

Runtime tests:

- Snapshot extraction includes interactive, text, image/icon, and explicit a11y nodes.
- Name priority chooses explicit name, then `labelled_by`, then text, then icon alt, then tooltip.
- Hidden nodes are excluded except label-only references.
- Disabled nodes remain discoverable but unsupported actions are filtered or diagnosed.
- Focused, selected, checked, expanded, pressed, and value state propagate from runtime/widget state.
- Invalid trees report dangling labels, duplicate ids, missing bounds, illegal focus, and unsupported action-role pairs.
- Accessibility action requests dispatch through shared focus/widget/input behavior or return structured rejections.

AccessKit conversion tests:

- Neutral snapshots convert to stable `TreeUpdate` root/children/node IDs.
- Role, bounds, labels, state, and actions map to AccessKit nodes.
- Stale focus falls back to the window root with diagnostics.
- Unsupported mappings produce diagnostics and do not panic.

Host smoke tests:

- Runtime winit accessibility adapter can be constructed behind the feature flag.
- Initial tree request calls runtime snapshot capture and submits a valid root.
- Action request queue serializes a neutral accessibility action event and sends it through `RuntimeSession::handle_event`.
- Accessibility inactive mode does not repeatedly capture or submit unchanged snapshots.
- Existing non-accessibility runtime entry tests still pass when the feature is disabled.

Manual acceptance:

- On a platform with screen reader support, a runtime window exposes named controls, focus changes, disabled state, and activation through the OS accessibility layer. This is recorded as acceptance evidence but not required for default CI.

## Risks And Mitigations

- AccessKit version mismatch with `winit = 0.31.0-beta.2`: verify Cargo resolution before coding and keep dependencies optional.
- Runtime/app API churn: append optional runtime API functions and gate host validation by feature instead of replacing existing frame capture behavior.
- Editor overlap: keep this slice in runtime winit host first and leave Slint editor bridge for a later consumer slice.
- Widget behavior gaps: unsupported actions should return structured rejections until M5 headless widget behavior can handle them uniformly.
- Performance: submit AccessKit updates only when active/requested and when snapshot generation/hash changes.

## Acceptance Criteria

- `zircon_runtime_interface` has neutral action request/result DTOs and precise diagnostics with contract tests.
- `zircon_runtime` can generate and validate a `UiAccessibilityTreeSnapshot` from a `UiSurface` using deterministic name, bounds, state, and action rules.
- `zircon_runtime` can map a neutral accessibility action request into shared UI behavior or a structured rejection.
- Optional AccessKit conversion maps a neutral snapshot to an AccessKit tree update with stable node IDs and role/state/action coverage.
- `zircon_app` winit runtime host owns the AccessKit adapter lifecycle behind a feature flag and can handle initial tree and action request smoke tests.
- Headless and non-winit builds compile without AccessKit dependencies.
- Docs record the bridge boundary, reference evidence, tests, and known manual screen-reader acceptance gap.
