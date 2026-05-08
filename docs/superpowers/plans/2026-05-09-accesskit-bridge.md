# AccessKit Bridge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the end-to-end AccessKit bridge for Zircon UI accessibility: neutral action contracts, runtime `UiSurface` tree generation, validation rules, optional AccessKit conversion, winit host adapter wiring, and action request roundtrip.

**Architecture:** `zircon_runtime_interface` stays AccessKit-free and owns neutral snapshot/action DTOs plus ABI/event payloads. `zircon_runtime::ui::accessibility` extracts and validates `UiAccessibilityTreeSnapshot` from `UiSurface`, maps actions into shared UI dispatch/focus/widget behavior, and optionally converts neutral snapshots to AccessKit. `zircon_app::entry::runtime_accessibility` owns `accesskit_winit::Adapter` lifecycle because the app host owns the winit window and event loop.

**Tech Stack:** Rust workspace on `main`, serde JSON payloads over existing runtime ABI buffers, optional `accesskit`/`accesskit_winit` feature gates, existing `UiSurface`/`UiTree`/`UiInputEvent` contracts, milestone-stage Cargo validation with `--locked --jobs 1` and external target dir `E:\cargo-targets\zircon-accesskit-bridge`.

---

## Repository Policy

- Work in the existing `main` checkout; do not create a worktree or feature branch unless the user later changes branch policy.
- Do not commit unless the user explicitly requests a commit.
- Refresh `.codex/plans` and `.codex/sessions` coordination before implementation begins because this plan touches shared runtime UI and app host files.
- Preserve unrelated dirty work. If implementation finds changes already present in a listed file, read and integrate without reverting.
- Add tests during implementation slices, but run Cargo/rustfmt in each milestone's testing stage unless a compile blocker requires earlier evidence.
- If an upper-layer app-host or AccessKit test fails, diagnose lower layers first: interface DTOs, snapshot extraction, conversion, then host wiring.

## Architecture Note

- Owner crate for stable contracts: `zircon_runtime_interface::ui::accessibility`, `zircon_runtime_interface::ui::dispatch`, and `zircon_runtime_interface::runtime_api`.
- Owner crate for retained UI accessibility semantics: `zircon_runtime::ui::accessibility`.
- Owner crate for platform bridge: `zircon_app::entry::runtime_accessibility`.
- AccessKit is an optional implementation detail. Public Zircon UI and runtime ABI contracts must stay neutral and serializable.
- `zircon_editor` is out of scope for this plan. The editor can later consume neutral snapshots or add its own Slint/AccessKit bridge.

## Reference Evidence

- `dev/bevy/crates/bevy_a11y/src/lib.rs`: `AccessibilityRequested`, action request event wrapper, and separation between reusable a11y primitives and platform integration.
- `dev/bevy/crates/bevy_ui/src/accessibility.rs`: UI-side label and bounds calculation after layout.
- `dev/bevy/crates/bevy_winit/src/accessibility.rs`: winit `Adapter` lifecycle, initial tree request, active-only updates, and action queue forwarding.
- `dev/slint/internal/backends/winit/accesskit.rs`: window-owned adapter, dirty-tree update policy, focus-only updates, and action request translation.
- `dev/godot/servers/display/accessibility_server.h`: platform accessibility boundary, focus/bounds/name/state/action update surface. Translate this precedent into Zircon's `app`/`runtime` vocabulary; do not introduce non-network `server` naming.

## File Map

### Create

- `zircon_runtime_interface/src/tests/accessibility_contracts.rs`
  - Focused contract tests for new action request/result DTOs, diagnostics, input event payloads, runtime ABI request shape, and snapshot serde.
- `zircon_runtime/src/ui/accessibility/mod.rs`
  - Runtime accessibility module root and public exports used by `UiSurface` and dynamic API.
- `zircon_runtime/src/ui/accessibility/extract.rs`
  - `UiSurface` to `UiAccessibilityTreeSnapshot` extraction.
- `zircon_runtime/src/ui/accessibility/name.rs`
  - Deterministic accessible name resolution: explicit name, `labelled_by`, own text, icon/image alt, tooltip.
- `zircon_runtime/src/ui/accessibility/diagnostics.rs`
  - Snapshot validation helpers and diagnostics for dangling labels, duplicate node ids, missing bounds, invalid focus, hidden focusable nodes, disabled actions, and relation cycles.
- `zircon_runtime/src/ui/accessibility/action.rs`
  - Neutral `UiAccessibilityActionRequest` handling and mapping to shared focus/widget/input behavior or structured rejection.
- `zircon_runtime/src/ui/accessibility/accesskit.rs`
  - Optional `accesskit` conversion module behind `accessibility-accesskit`.
- `zircon_runtime/src/ui/tests/accessibility.rs`
  - Runtime snapshot, name priority, state propagation, diagnostics, and action dispatch tests.
- `zircon_app/src/entry/runtime_accessibility/mod.rs`
  - App-host accessibility module facade, feature-gated no-op/real host exports.
- `zircon_app/src/entry/runtime_accessibility/accesskit_host.rs`
  - Optional `accesskit_winit` adapter owner, snapshot update, initial tree, and action queue bridge.
- `docs/zircon_runtime_interface/ui/accessibility.md`
  - Interface-level contracts, action payloads, diagnostics, and ABI notes.
- `docs/zircon_runtime/ui/accessibility.md`
  - Runtime snapshot extraction, name/state/action rules, and validation behavior.
- `docs/zircon_app/entry/runtime_accessibility.md`
  - App-host AccessKit adapter lifecycle and feature-gated platform bridge.
- `tests/acceptance/accesskit-bridge.md`
  - Acceptance record for scoped validation and manual screen-reader smoke status.

### Modify

- `Cargo.toml`
  - Add optional workspace dependency entries for `accesskit` and `accesskit_winit` only after version compatibility is verified against `winit = 0.31.0-beta.2`.
- `zircon_runtime_interface/src/ui/accessibility.rs`
  - Add action request/result/source/status DTOs and precise diagnostics.
- `zircon_runtime_interface/src/ui/dispatch/input/event.rs`
  - Add `UiInputEvent::Accessibility(UiAccessibilityInputEvent)` and its payload struct.
- `zircon_runtime_interface/src/ui/dispatch/mod.rs`
  - Re-export the new `UiAccessibilityInputEvent` next to the existing input event payload types.
- `zircon_runtime_interface/src/runtime_api.rs`
  - Add `ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1`, `ZrRuntimeAccessibilityTreeRequestV1`, `ZrRuntimeCaptureAccessibilityTreeFnV1`, and append `capture_accessibility_tree` to `ZrRuntimeApiV1`.
- `zircon_runtime_interface/src/tests/mod.rs`
  - Register `accessibility_contracts`.
- `zircon_runtime/Cargo.toml`
  - Add `accessibility-accesskit` feature and optional `accesskit` dependency.
- `zircon_runtime/src/ui/mod.rs`
  - Add `pub mod accessibility`.
- `zircon_runtime/src/ui/surface/surface.rs`
  - Add `UiSurface::accessibility_snapshot()` that delegates to `zircon_runtime::ui::accessibility::accessibility_snapshot` without mutating layout.
- `zircon_runtime/src/ui/surface/input/dispatch.rs`
  - Route `UiInputEvent::Accessibility` to `accessibility::dispatch_accessibility_action`.
- `zircon_runtime/src/ui/template/build/tree_builder.rs`
  - Preserve `UiTemplateNode.a11y` and `UiTemplateNode.widget` into `UiTemplateNodeMetadata`; the retained `UiTreeNode` metadata is the extraction source after template expansion.
- `zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs`
  - Add retained `a11y: UiAccessibilityContract` and `widget: UiWidgetContract` fields with serde defaults.
- `zircon_runtime/src/dynamic_api/frame.rs`
  - Add an accessibility JSON byte-buffer encoder/free function using the same ownership validation pattern as frame bytes and a distinct owner token.
- `zircon_runtime/src/dynamic_api/session.rs`
  - Implement `capture_accessibility_tree`; handle accessibility action runtime events by deserializing `UiAccessibilityActionRequest` and dispatching it through runtime UI accessibility handling.
- `zircon_runtime/src/dynamic_api/exports.rs`
  - Populate the appended optional API function.
- `zircon_runtime/src/ui/tests/mod.rs`
  - Register `accessibility`.
- `zircon_app/Cargo.toml`
  - Add `accessibility-accesskit-winit` feature and optional `accesskit`/`accesskit_winit` dependencies.
- `zircon_app/src/entry/runtime_entry_app/mod.rs`
  - Add an optional accessibility host field.
- `zircon_app/src/entry/runtime_entry_app/construct.rs`
  - Initialize the accessibility host field in disabled/no-op state.
- `zircon_app/src/entry/runtime_entry_app/application_handler.rs`
  - Create the accessibility host after window creation, process relevant window events, update active snapshots, and drain action requests into `RuntimeSession::handle_event`.
- `zircon_app/src/entry/runtime_library/runtime_session.rs`
  - Add `capture_accessibility_tree` and helper for sending accessibility action payload events.
- `zircon_app/src/entry/runtime_library/loaded_runtime.rs`
  - Require `capture_accessibility_tree` only under `accessibility-accesskit-winit`; keep existing runtime loading valid without the feature.
- `zircon_app/src/entry/runtime_library/mod.rs`
  - Re-export `RuntimeAccessibilityTree` from `runtime_session.rs` for app host code.
- `zircon_app/src/entry/tests/mod.rs`
  - Add static/source tests proving runtime accessibility uses neutral runtime API boundaries and no direct `zircon_runtime::ui` imports.
- `docs/superpowers/specs/2026-05-08-accesskit-bridge-design.md`
  - Add plan source and validation evidence when implementation completes.
- `docs/ui-and-layout/bevy-ui-text-widgets-focus-a11y-m0-gap-audit.md`
  - Update M9 evidence when the bridge lands.

## Milestone 0: Coordination, Dependency Compatibility, And Contract Baseline

**Goal:** Confirm the implementation lane is safe, verify AccessKit/winit dependency compatibility, and land neutral interface contracts without runtime behavior.

**In-scope behaviors:**

- Fresh coordination scan and one active session note for implementation.
- Optional `accesskit`/`accesskit_winit` dependency version decision recorded in docs or the session note.
- Neutral action request/result DTOs.
- New diagnostic codes for invalid trees.
- `UiInputEvent::Accessibility` payload.
- Runtime ABI additions for optional accessibility snapshot capture with a generation hint and accessibility action event kind.

**Dependencies:**

- Existing `UiAccessibilityNode`, `UiAccessibilityTreeSnapshot`, `UiAccessibilityAction`, `UiInputEvent`, `ZrRuntimeEventV1`, and `ZrRuntimeApiV1`.

**Implementation slices:**

- [ ] Run coordination scan before source edits:

```powershell
.\.codex\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot E:\Git\ZirconEngine -LookbackHours 4
```

- [ ] Create or update one `.codex/sessions/*.md` implementation note with current goal, touched modules, related tests, blockers, and overlap warnings.
- [ ] Verify AccessKit compatibility by inspecting current crate metadata and attempting dependency resolution only after choosing versions. Start from Bevy's reference versions `accesskit = "0.24"` and `accesskit_winit = "0.32"`, but adapt if `winit = "0.31.0-beta.2"` requires a newer compatible `accesskit_winit`.
- [ ] In `zircon_runtime_interface/src/ui/accessibility.rs`, add:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiAccessibilityActionSource {
    #[default]
    AssistiveTechnology,
    Keyboard,
    Pointer,
    Programmatic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiAccessibilityActionRequest {
    pub target: UiNodeId,
    pub action: UiAccessibilityAction,
    pub source: UiAccessibilityActionSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_value: Option<f64>,
}

impl Default for UiAccessibilityActionRequest {
    fn default() -> Self {
        Self {
            target: UiNodeId::default(),
            action: UiAccessibilityAction::Activate,
            source: UiAccessibilityActionSource::AssistiveTechnology,
            value: None,
            numeric_value: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiAccessibilityActionStatus {
    #[default]
    Accepted,
    Rejected,
    Unsupported,
    StaleTarget,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiAccessibilityActionResult {
    pub target: UiNodeId,
    pub action: UiAccessibilityAction,
    pub status: UiAccessibilityActionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
```

- [ ] Extend `UiAccessibilityDiagnosticCode` with `DuplicateNodeId`, `MissingBounds`, `InvalidFocus`, `DanglingLabel`, `DanglingDescription`, `RelationCycle`, `UnsupportedRoleAction`, and `ExcludedFocusedNode`.
- [ ] In `zircon_runtime_interface/src/ui/dispatch/input/event.rs`, import `UiAccessibilityActionRequest` and add:

```rust
UiInputEvent::Accessibility(UiAccessibilityInputEvent)
```

plus:

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAccessibilityInputEvent {
    pub metadata: UiInputEventMetadata,
    pub request: UiAccessibilityActionRequest,
}
```

- [ ] In `zircon_runtime_interface/src/runtime_api.rs`, add an event kind, accessibility tree request, and accessibility tree capture function type:

```rust
pub const ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1: u32 = 8;

pub type ZrRuntimeCaptureAccessibilityTreeFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeAccessibilityTreeRequestV1,
    *mut ZrOwnedByteBuffer,
) -> ZrStatus;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeAccessibilityTreeRequestV1 {
    pub abi_version: u32,
    pub viewport: ZrRuntimeViewportHandle,
    pub size: ZrRuntimeViewportSizeV1,
    pub generation_hint: u64,
}

impl ZrRuntimeAccessibilityTreeRequestV1 {
    pub const fn new(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
        generation_hint: u64,
    ) -> Self {
        Self {
            abi_version,
            viewport,
            size,
            generation_hint,
        }
    }
}
```

- [ ] Append `capture_accessibility_tree: Option<ZrRuntimeCaptureAccessibilityTreeFnV1>` to `ZrRuntimeApiV1`, set it to `None` in `empty`, and preserve existing fields/order before the append.
- [ ] Add `ZrRuntimeEventV1::accessibility_action(abi_version, viewport, payload: ZrByteSlice)` constructor that sets `kind` to `ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1` and stores serialized request bytes in `payload`.
- [ ] Add `zircon_runtime_interface/src/tests/accessibility_contracts.rs` with tests for DTO roundtrip, snake_case diagnostics, input event payload roundtrip, runtime ABI request constructor, capture function type, and API table default optional field.
- [ ] Register `accessibility_contracts` in `zircon_runtime_interface/src/tests/mod.rs`.
- [ ] Update `docs/zircon_runtime_interface/ui/accessibility.md` with `related_code` first in YAML frontmatter, plan source, new DTOs, diagnostics, and tests.

**Testing stage:**

- Run formatting for touched interface files:

```powershell
rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/ui/dispatch/input/event.rs" "zircon_runtime_interface/src/runtime_api.rs" "zircon_runtime_interface/src/tests/mod.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs"
```

- Run focused interface contracts:

```powershell
cargo test -p zircon_runtime_interface --lib accessibility_contracts --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run interface check:

```powershell
cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run whitespace check:

```powershell
git diff --check -- "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/ui/dispatch/input/event.rs" "zircon_runtime_interface/src/runtime_api.rs" "zircon_runtime_interface/src/tests/mod.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs" "docs/zircon_runtime_interface/ui/accessibility.md"
```

**Lightweight checks:**

- If dependency resolution is uncertain, use a scoped non-build metadata/check command after editing Cargo manifests:

```powershell
cargo metadata --locked --format-version 1
```

**Exit evidence:**

- Focused interface test passes.
- `cargo check -p zircon_runtime_interface` passes.
- AccessKit dependency version decision is recorded.
- No runtime/app behavior is claimed in this milestone.

## Milestone 1: Runtime Accessibility Snapshot Extraction And Validation

**Goal:** Generate a validated `UiAccessibilityTreeSnapshot` from an actual `UiSurface` using deterministic focus/name/hidden/disabled/bounds rules.

**In-scope behaviors:**

- Retain a11y/widget contracts in runtime tree metadata or an equivalent source accessible to extraction.
- Include explicit a11y nodes, interactive focus/picking/widget nodes, text nodes, image/icon alt nodes, and relation targets.
- Name priority: explicit name, `labelled_by`, own text, icon/image alt metadata, tooltip fallback.
- Hidden nodes excluded except label/description-only references.
- Disabled nodes discoverable with invalid actions filtered/diagnosed.
- Focus must be visible/enabled/included or produce invalid-focus diagnostics and safe fallback.
- Bounds come from arranged tree or layout cache.

**Dependencies:**

- Milestone 0 interface contracts.
- Existing `UiSurface`, `UiTree`, `UiTreeNode`, `UiTemplateNodeMetadata`, layout cache, render text resolver behavior, and focus state.

**Implementation slices:**

- [ ] Add `a11y: UiAccessibilityContract` and `widget: UiWidgetContract` with serde defaults to `UiTemplateNodeMetadata`.
- [ ] Update all `UiTemplateNodeMetadata` constructors in runtime code/tests to provide `..UiTemplateNodeMetadata::default()` after the explicitly relevant fields instead of manually listing every field. Known touched constructor sites include `zircon_runtime/src/ui/template/build/tree_builder.rs`, `zircon_runtime/src/ui/tests/focus_navigation.rs`, `zircon_runtime/src/ui/tests/shared_core.rs`, `zircon_runtime/src/ui/tests/surface_node_pool.rs`, `zircon_runtime/src/ui/tests/timeline.rs`, `zircon_runtime/src/ui/tests/hit_grid.rs`, `zircon_runtime/src/ui/tests/material_layout.rs`, `zircon_runtime/src/ui/tests/event_routing.rs`, `zircon_runtime/src/ui/tests/text_layout.rs`, `zircon_runtime/src/ui/tests/pointer_click_semantics.rs`, `zircon_runtime/src/ui/tests/layout_slots.rs`, `zircon_runtime/src/ui/tests/surface_frame_authority.rs`, and `zircon_runtime/src/ui/tests/diagnostics.rs`.
- [ ] Update `zircon_runtime/src/ui/template/build/tree_builder.rs` so retained metadata copies `node.a11y.clone()` and `node.widget.clone()`.
- [ ] Create `zircon_runtime/src/ui/accessibility/mod.rs` exporting:

```rust
pub(crate) use action::dispatch_accessibility_action;
pub(crate) use extract::accessibility_snapshot;

mod action;
mod diagnostics;
mod extract;
mod name;

#[cfg(feature = "accessibility-accesskit")]
pub(crate) mod accesskit;
```

- [ ] Add `pub mod accessibility;` in `zircon_runtime/src/ui/mod.rs`.
- [ ] Implement `name.rs` helpers that resolve string attributes from `UiTemplateNodeMetadata.attributes`, including keys `text`, `label`, `value`, `accessibility_label`, `alt`, `alt_text`, `icon_alt`, and `tooltip`.
- [ ] Implement `extract.rs` with `pub(crate) fn accessibility_snapshot(surface: &UiSurface) -> UiAccessibilityTreeSnapshot`.
- [ ] Implement inclusion and child filtering in extraction using a `BTreeMap<UiNodeId, UiAccessibilityNode>` for deterministic output.
- [ ] Implement `diagnostics.rs` with a validation pass over the snapshot that checks duplicate ids, dangling labels/descriptions, missing bounds for interactive/named nodes, invalid focus, hidden focusable nodes, disabled actions, and simple relation cycles.
- [ ] Add this method in `surface.rs` delegating to the module:

```rust
pub fn accessibility_snapshot(&self) -> UiAccessibilityTreeSnapshot {
    crate::ui::accessibility::accessibility_snapshot(self)
}
```
- [ ] Add `zircon_runtime/src/ui/tests/accessibility.rs` with fixtures and tests for extraction, name priority, hidden label references, disabled action filtering, focus fallback diagnostics, and missing bounds diagnostics.
- [ ] Register the test module in `zircon_runtime/src/ui/tests/mod.rs`.
- [ ] Update `docs/zircon_runtime/ui/accessibility.md` with extraction rules, name priority, state propagation, diagnostics, and focused tests.

**Testing stage:**

- Run formatting for touched runtime accessibility files:

```powershell
rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs" "zircon_runtime/src/ui/mod.rs" "zircon_runtime/src/ui/template/build/tree_builder.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/accessibility/mod.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/name.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/tests/mod.rs" "zircon_runtime/src/ui/tests/accessibility.rs"
```

- Run focused runtime accessibility tests:

```powershell
cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run runtime lib check if focused tests pass or fail in unrelated upper layers:

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run whitespace check on runtime snapshot files and docs:

```powershell
git diff --check -- "zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs" "zircon_runtime/src/ui/mod.rs" "zircon_runtime/src/ui/template/build/tree_builder.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/accessibility/mod.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/name.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/tests/mod.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "docs/zircon_runtime/ui/accessibility.md"
```

**Lightweight checks:**

- During slices, run formatting on the new module files before the full milestone testing stage when syntax uncertainty blocks progress:

```powershell
rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/mod.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/name.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/tests/accessibility.rs"
```

**Exit evidence:**

- Runtime accessibility tests pass.
- Runtime lib check reaches success or any blocker is classified as external with file/line evidence.
- Docs describe all focus/name/hidden/disabled rules implemented in this milestone.

## Milestone 2: Accessibility Action Dispatch Through Shared UI Behavior

**Goal:** Convert neutral accessibility action requests into the same runtime UI behavior paths used by focus, input, widgets, text, popup, and scroll handling.

**In-scope behaviors:**

- `UiInputEvent::Accessibility` dispatches through `UiSurface::dispatch_input_event`.
- `Focus` changes runtime focus with accessibility-visible reason.
- `Activate` emits or routes through existing activation/commit/component behavior where available.
- `SetValue` mutates value/text only through existing property/input mutation helpers.
- `Increment`, `Decrement`, `ScrollTo`, and `Dismiss` either route to existing supported behavior or return structured unsupported/rejected results.
- Stale or hidden targets produce structured rejection and diagnostics.

**Dependencies:**

- Milestone 0 action DTOs.
- Milestone 1 snapshot validation and target inclusion checks.
- Existing focus APIs and input dispatch result types.

**Implementation slices:**

- [ ] Implement `zircon_runtime/src/ui/accessibility/action.rs` with:

```rust
pub(crate) fn dispatch_accessibility_action(
    surface: &mut UiSurface,
    event: UiAccessibilityInputEvent,
) -> UiInputDispatchResult
```

- [ ] In `zircon_runtime/src/ui/surface/input/dispatch.rs`, add a `UiInputEvent::Accessibility(accessibility)` match arm that calls `dispatch_accessibility_action`.
- [ ] For `Focus`, call existing focus APIs with `UiFocusChangeReason::Programmatic` and `UiFocusVisible::visible(UiFocusVisibleReason::Programmatic)`, then return handled `UiInputDispatchResult` with route target and phase `accessibility.focus`.
- [ ] For `Activate`, use existing component event/report vocabulary rather than inventing host-only activation. If current headless widget behavior is incomplete, emit a structured `Unsupported` result and diagnostic note rather than adding per-widget special cases.
- [ ] For `SetValue`, use existing property mutation/text edit paths only when the target supports a value/text property; otherwise reject with `UnsupportedRoleAction`.
- [ ] For `Dismiss`, reject with `UiAccessibilityActionStatus::Unsupported` and a diagnostic note `accessibility dismiss requires popup id` until a popup id source is part of the neutral request.
- [ ] Add runtime tests in `ui/tests/accessibility.rs` for accepted focus action, stale target rejection, disabled activation rejection, unsupported increment rejection, unsupported dismiss rejection, and set-value behavior for an editable text fixture using existing `text`/`value` metadata.
- [ ] Update `docs/zircon_runtime/ui/accessibility.md` with action mapping and unsupported-action behavior.

**Testing stage:**

- Run formatting for action/dispatch/test files:

```powershell
rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs" "zircon_runtime/src/ui/tests/accessibility.rs"
```
- Run focused accessibility tests:

```powershell
cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run a focused existing dispatch suite to catch input integration regressions:

```powershell
cargo test -p zircon_runtime --lib ui::tests::runtime_input_ownership --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run runtime lib check:

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

**Lightweight checks:**

- Before the testing stage, limit ad hoc checks to formatting the three files most likely to receive iterative edits:

```powershell
rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs" "zircon_runtime/src/ui/tests/accessibility.rs"
```

**Exit evidence:**

- Accessibility action dispatch tests pass.
- Existing runtime input ownership focused suite still passes or any failure is diagnosed as unrelated.
- Unsupported actions return structured results, not panics or host-only branches.

## Milestone 3: Runtime ABI Snapshot Capture And Serialized Action Roundtrip

**Goal:** Expose accessibility snapshots and action requests across the dynamic runtime API used by `zircon_app`.

**In-scope behaviors:**

- `capture_accessibility_tree` ABI function returns serialized `UiAccessibilityTreeSnapshot` in `ZrOwnedByteBuffer`.
- Runtime action events deserialize `UiAccessibilityActionRequest` from `ZrRuntimeEventV1.payload`.
- Runtime rejects wrong ABI versions, unknown viewport handles, invalid JSON payloads, stale targets, and missing output pointers with `ZrStatus`.
- Existing frame capture/free behavior remains unchanged.

**Dependencies:**

- Milestones 0-2.
- Existing dynamic API session registry and owned byte buffer pattern in `dynamic_api/frame.rs`.

**Implementation slices:**

- [ ] In `zircon_runtime/src/dynamic_api/frame.rs`, add `encode_accessibility_tree(snapshot: &UiAccessibilityTreeSnapshot) -> Result<ZrOwnedByteBuffer, serde_json::Error>` that serializes JSON bytes, exports them with owner token `0x5a52_4131_3159_0001`, and installs `free_runtime_accessibility_bytes` for reclamation.
- [ ] In `zircon_runtime/src/dynamic_api/frame.rs`, add `write_accessibility_tree(destination: *mut ZrOwnedByteBuffer, buffer: ZrOwnedByteBuffer) -> ZrStatus` that mirrors `write_frame` null-output handling with diagnostic `missing accessibility tree output`.
- [ ] In `zircon_runtime/src/dynamic_api/session.rs`, add `capture_accessibility_tree` extern function mirroring `capture_frame` validation style.
- [ ] Add `RuntimeDynamicSession::capture_accessibility_tree(request: ZrRuntimeAccessibilityTreeRequestV1) -> Result<UiAccessibilityTreeSnapshot, String>`. The current dynamic runtime preview path does not store a `UiSurface`, so this method must return a minimal valid window/root snapshot with a diagnostic `runtime UI surface accessibility extraction unavailable in dynamic preview`; do not fake widget data.
- [ ] In `RuntimeDynamicSession::handle_event`, add `ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1` handling that deserializes `UiAccessibilityActionRequest` from payload and dispatches to the current runtime UI accessibility action path or rejects with a structured `ZrStatus` if no UI surface is present.
- [ ] In `zircon_runtime/src/dynamic_api/exports.rs`, wire `capture_accessibility_tree: Some(capture_accessibility_tree)`.
- [ ] Add dynamic API tests in existing `zircon_runtime/src/dynamic_api/tests.rs` for API table field presence, null output rejection, wrong ABI rejection, serialized snapshot success, and invalid action payload rejection.
- [ ] Update `docs/zircon_runtime/ui/accessibility.md` with ABI capture/action behavior.

**Testing stage:**

- Run formatting for dynamic API/session/frame/accessibility files:

```powershell
rustfmt --edition 2021 --check "zircon_runtime/src/dynamic_api/frame.rs" "zircon_runtime/src/dynamic_api/session.rs" "zircon_runtime/src/dynamic_api/exports.rs" "zircon_runtime/src/dynamic_api/tests.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs"
```
- Run focused dynamic API/runtime accessibility tests:

```powershell
cargo test -p zircon_runtime --lib dynamic_api --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run runtime accessibility tests again:

```powershell
cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run runtime lib check:

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

**Lightweight checks:**

- Use `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never` before the testing stage only if ABI type changes create broad compile uncertainty.

**Exit evidence:**

- Runtime API exposes the optional accessibility snapshot capture function.
- Serialized snapshot capture works through ABI buffers.
- Serialized action event path rejects malformed data safely.

## Milestone 4: Optional AccessKit Conversion

**Goal:** Convert neutral `UiAccessibilityTreeSnapshot` values into AccessKit tree updates without making AccessKit part of Zircon's public UI contract.

**In-scope behaviors:**

- Optional `accessibility-accesskit` feature in `zircon_runtime`.
- Role mapping for current `UiA11yRole` values.
- Bounds/name/description/children/focus mapping.
- Action mapping for `Activate`, `Focus`, `Increment`, `Decrement`, `SetValue`, `ScrollTo`, and `Dismiss` where supported by the selected AccessKit version.
- Disabled/hidden/checked/selected/expanded/value state mapping where AccessKit exposes equivalent properties.
- Stable `accesskit::NodeId` derivation from `UiNodeId` plus a collision-free synthetic window root ID.

**Dependencies:**

- Milestone 1 snapshots and diagnostics.
- AccessKit dependency compatibility decision from Milestone 0.

**Implementation slices:**

- [ ] Add optional `accesskit` workspace dependency and `zircon_runtime/accessibility-accesskit` feature in this milestone when Milestone 0 only recorded the compatibility decision and did not edit Cargo manifests.
- [ ] Implement `zircon_runtime/src/ui/accessibility/accesskit.rs` with this converter API:

```rust
pub(crate) struct UiAccessKitTreeUpdate {
    pub update: accesskit::TreeUpdate,
    pub diagnostics: Vec<UiAccessibilityDiagnostic>,
}

#[cfg(feature = "accessibility-accesskit")]
pub(crate) fn snapshot_to_accesskit_update(
    snapshot: &UiAccessibilityTreeSnapshot,
    window_label: String,
) -> UiAccessKitTreeUpdate
```

- [ ] Map each `UiA11yRole` to the closest available `accesskit::Role`; use generic role only when no closer role exists and record a conversion diagnostic for required interactive roles without equivalent actions.
- [ ] Map state and actions using helper functions so unsupported AccessKit version differences are isolated to this module.
- [ ] Add feature-gated runtime tests for conversion root/children, stable IDs, label/bounds, focus fallback, and unsupported mapping diagnostics.
- [ ] Update `docs/zircon_runtime/ui/accessibility.md` with AccessKit conversion scope and feature flag.

**Testing stage:**

- Run formatting for AccessKit module/tests:

```powershell
rustfmt --edition 2021 --check "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/ui/tests/accessibility.rs"
```
- Run feature-gated conversion tests:

```powershell
cargo test -p zircon_runtime --lib ui::tests::accessibility --features accessibility-accesskit --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run no-feature runtime check to prove optional dependency isolation:

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run feature-gated runtime check:

```powershell
cargo check -p zircon_runtime --lib --features accessibility-accesskit --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

**Lightweight checks:**

- After dependency edits, use the same feature-gated runtime check declared in this milestone's testing stage before continuing with converter code:

```powershell
cargo check -p zircon_runtime --lib --features accessibility-accesskit --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

**Exit evidence:**

- Neutral snapshots convert to AccessKit tree updates under the feature.
- No-feature runtime builds/checks do not require AccessKit.
- Conversion limitations are documented as diagnostics, not hidden behavior.

## Milestone 5: Winit Host Adapter And App Runtime Integration

**Goal:** Wire the runtime winit host to AccessKit behind an app feature flag and route AccessKit action requests back through the dynamic runtime session.

**In-scope behaviors:**

- Optional `zircon_app/accessibility-accesskit-winit` feature.
- Runtime entry app creates an AccessKit host after creating the winit window.
- Initial tree request calls `RuntimeSession::capture_accessibility_tree`.
- Active-only updates avoid repeated snapshot submission when accessibility is inactive or unchanged.
- AccessKit action requests serialize `UiAccessibilityActionRequest` and call `RuntimeSession::handle_event`.
- Existing non-accessibility app tests pass with feature disabled.

**Dependencies:**

- Milestones 0-4.
- `accesskit_winit` compatibility with current `winit`.

**Implementation slices:**

- [ ] Add optional `accesskit` and `accesskit_winit` dependencies to `zircon_app/Cargo.toml` plus `accessibility-accesskit-winit` feature.
- [ ] Create `zircon_app/src/entry/runtime_accessibility/mod.rs` with a no-op host for feature-disabled builds and re-export a real `RuntimeAccessibilityHost` when the feature is enabled.
- [ ] Create `zircon_app/src/entry/runtime_accessibility/accesskit_host.rs` under `#[cfg(feature = "accessibility-accesskit-winit")]`.
- [ ] Implement host state: window label, last snapshot hash/generation, active flag, action queue, and adapter handle.
- [ ] Add `RuntimeSession::capture_accessibility_tree(viewport, size, generation_hint)` in `runtime_session.rs`, deserializing `UiAccessibilityTreeSnapshot` from `ZrOwnedByteBuffer` and freeing it with the returned buffer's `free` callback.
- [ ] Add `RuntimeAccessibilityTree` in `runtime_session.rs` wrapping the snapshot and owned buffer lifecycle, matching `RuntimeFrame`'s drop pattern but exposing `snapshot(&self) -> &UiAccessibilityTreeSnapshot`.
- [ ] Add `RuntimeSession::send_accessibility_action(viewport, request)` that serializes the neutral request into owned `Vec<u8>`, builds `ZrRuntimeEventV1::accessibility_action(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport, ZrByteSlice { data, len })`, calls `handle_event` while the bytes are still alive, and drops the vector after the call returns.
- [ ] In `runtime_entry_app/mod.rs`, add `accessibility: RuntimeAccessibilityHost` field.
- [ ] In `construct.rs`, initialize `RuntimeAccessibilityHost::disabled()`.
- [ ] In `application_handler.rs`, after window creation, create/attach the host with the active event loop, window, session, viewport, and current size.
- [ ] In `application_handler.rs`, forward relevant `WindowEvent`s to the accessibility host before regular runtime handling when the feature is enabled.
- [ ] On redraw or after runtime events that can mutate UI state, call a host update method that captures and submits a changed snapshot only if active.
- [ ] Drain action requests and send them through `RuntimeSession::send_accessibility_action`.
- [ ] Add app static/source tests proving the app host uses `RuntimeSession` and `zircon_runtime_interface` for a11y, not direct `zircon_runtime::ui` imports.
- [ ] Add feature-gated host smoke tests for disabled no-op behavior, action queue serialization, and snapshot update gating. Keep pure host-state logic in a testable struct so CI does not need to construct a real OS event loop; adapter creation is covered by feature-gated compile checks plus manual smoke evidence.
- [ ] Update `docs/zircon_app/entry/runtime_accessibility.md`.

**Testing stage:**

- Run formatting for app accessibility/runtime entry files:

```powershell
rustfmt --edition 2021 --check "zircon_app/src/entry/runtime_accessibility/mod.rs" "zircon_app/src/entry/runtime_accessibility/accesskit_host.rs" "zircon_app/src/entry/runtime_entry_app/mod.rs" "zircon_app/src/entry/runtime_entry_app/construct.rs" "zircon_app/src/entry/runtime_entry_app/application_handler.rs" "zircon_app/src/entry/runtime_library/runtime_session.rs" "zircon_app/src/entry/runtime_library/loaded_runtime.rs" "zircon_app/src/entry/runtime_library/mod.rs" "zircon_app/src/entry/tests/mod.rs"
```
- Run no-feature app tests:

```powershell
cargo test -p zircon_app --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run feature-gated app check:

```powershell
cargo check -p zircon_app --lib --features accessibility-accesskit-winit --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run focused app host tests with the planned module filter:

```powershell
cargo test -p zircon_app --lib runtime_accessibility --features accessibility-accesskit-winit --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

**Lightweight checks:**

- After app dependency or adapter API edits, use the same feature-gated app check declared in this milestone's testing stage before continuing with event-loop wiring:

```powershell
cargo check -p zircon_app --lib --features accessibility-accesskit-winit --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

**Exit evidence:**

- App compiles without AccessKit by default.
- App compiles with AccessKit host feature.
- Host state tests or compile checks cover initial tree, update gating, and action queue path.
- Runtime entry retains dynamic API boundary and avoids direct runtime UI imports.

## Milestone 6: Docs, Acceptance, And Full Scoped Validation

**Goal:** Consolidate evidence for the full AccessKit bridge and document remaining manual screen-reader acceptance work.

**In-scope behaviors:**

- Update module docs for interface, runtime, and app host.
- Update the approved design spec with implementation evidence.
- Update UI/Text/Widgets/Focus/A11y M0 gap audit with M9 bridge evidence.
- Create acceptance record with commands, pass/fail status, and manual screen-reader caveat.
- Archive/retire the active session note.

**Dependencies:**

- Milestones 0-5.

**Implementation slices:**

- [ ] Update `docs/zircon_runtime_interface/ui/accessibility.md` frontmatter `implementation_files`, `tests`, and evidence.
- [ ] Update `docs/zircon_runtime/ui/accessibility.md` with final runtime behavior and test evidence.
- [ ] Update `docs/zircon_app/entry/runtime_accessibility.md` with feature flags, host lifecycle, and action routing evidence.
- [ ] Update `docs/superpowers/specs/2026-05-08-accesskit-bridge-design.md` with implementation plan source and validation results.
- [ ] Update `docs/ui-and-layout/bevy-ui-text-widgets-focus-a11y-m0-gap-audit.md` M9/a11y section with bridge evidence.
- [ ] Create `tests/acceptance/accesskit-bridge.md` with implemented files, reference files, exact commands, pass/fail output summaries, external/manual screen-reader status, and known limitations.
- [ ] Move the implementation session note to `.codex/sessions/archive/` with `status: completed` and a short completion summary.

**Testing stage:**

- Run formatting across all touched Rust files:

```powershell
rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/ui/dispatch/input/event.rs" "zircon_runtime_interface/src/ui/dispatch/mod.rs" "zircon_runtime_interface/src/runtime_api.rs" "zircon_runtime_interface/src/tests/mod.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs" "zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs" "zircon_runtime/src/ui/mod.rs" "zircon_runtime/src/ui/template/build/tree_builder.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs" "zircon_runtime/src/ui/accessibility/mod.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/name.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/dynamic_api/frame.rs" "zircon_runtime/src/dynamic_api/session.rs" "zircon_runtime/src/dynamic_api/exports.rs" "zircon_runtime/src/ui/tests/mod.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "zircon_app/src/entry/runtime_accessibility/mod.rs" "zircon_app/src/entry/runtime_accessibility/accesskit_host.rs" "zircon_app/src/entry/runtime_entry_app/mod.rs" "zircon_app/src/entry/runtime_entry_app/construct.rs" "zircon_app/src/entry/runtime_entry_app/application_handler.rs" "zircon_app/src/entry/runtime_library/runtime_session.rs" "zircon_app/src/entry/runtime_library/loaded_runtime.rs" "zircon_app/src/entry/runtime_library/mod.rs" "zircon_app/src/entry/tests/mod.rs"
```

- Run interface tests:

```powershell
cargo test -p zircon_runtime_interface --lib accessibility_contracts --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run runtime accessibility tests without AccessKit:

```powershell
cargo test -p zircon_runtime --lib ui::tests::accessibility --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run runtime accessibility tests with AccessKit:

```powershell
cargo test -p zircon_runtime --lib ui::tests::accessibility --features accessibility-accesskit --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run runtime lib check:

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run app no-feature tests/check:

```powershell
cargo test -p zircon_app --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run app feature check:

```powershell
cargo check -p zircon_app --lib --features accessibility-accesskit-winit --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-accesskit-bridge" --message-format short --color never
```

- Run whitespace checks on all touched source/docs/acceptance/session files:

```powershell
git diff --check -- "Cargo.toml" "zircon_runtime/Cargo.toml" "zircon_app/Cargo.toml" "zircon_runtime_interface/src/ui/accessibility.rs" "zircon_runtime_interface/src/ui/dispatch/input/event.rs" "zircon_runtime_interface/src/ui/dispatch/mod.rs" "zircon_runtime_interface/src/runtime_api.rs" "zircon_runtime_interface/src/tests/mod.rs" "zircon_runtime_interface/src/tests/accessibility_contracts.rs" "zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs" "zircon_runtime/src/ui/mod.rs" "zircon_runtime/src/ui/template/build/tree_builder.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/surface/input/dispatch.rs" "zircon_runtime/src/ui/accessibility/mod.rs" "zircon_runtime/src/ui/accessibility/extract.rs" "zircon_runtime/src/ui/accessibility/name.rs" "zircon_runtime/src/ui/accessibility/diagnostics.rs" "zircon_runtime/src/ui/accessibility/action.rs" "zircon_runtime/src/ui/accessibility/accesskit.rs" "zircon_runtime/src/dynamic_api/frame.rs" "zircon_runtime/src/dynamic_api/session.rs" "zircon_runtime/src/dynamic_api/exports.rs" "zircon_runtime/src/ui/tests/mod.rs" "zircon_runtime/src/ui/tests/accessibility.rs" "zircon_app/src/entry/runtime_accessibility/mod.rs" "zircon_app/src/entry/runtime_accessibility/accesskit_host.rs" "zircon_app/src/entry/runtime_entry_app/mod.rs" "zircon_app/src/entry/runtime_entry_app/construct.rs" "zircon_app/src/entry/runtime_entry_app/application_handler.rs" "zircon_app/src/entry/runtime_library/runtime_session.rs" "zircon_app/src/entry/runtime_library/loaded_runtime.rs" "zircon_app/src/entry/runtime_library/mod.rs" "zircon_app/src/entry/tests/mod.rs" "docs/zircon_runtime_interface/ui/accessibility.md" "docs/zircon_runtime/ui/accessibility.md" "docs/zircon_app/entry/runtime_accessibility.md" "docs/superpowers/specs/2026-05-08-accesskit-bridge-design.md" "docs/ui-and-layout/bevy-ui-text-widgets-focus-a11y-m0-gap-audit.md" "tests/acceptance/accesskit-bridge.md"
```

**Lightweight checks:**

- Use focused `cargo check` for the affected crate while fixing validation failures. Do not run workspace-wide validation until scoped checks are clean.

**Exit evidence:**

- Interface, runtime, AccessKit-feature runtime, app no-feature, and app feature checks pass or any external blocker is documented with owner/scope.
- Docs and acceptance records list exact command results.
- Active session note is retired from `.codex/sessions/` root.

## Debug And Correction Loop

- If interface tests fail, fix `zircon_runtime_interface` DTO serde/defaults before touching runtime extraction.
- If runtime snapshot tests fail, inspect retained metadata and arranged bounds before changing action dispatch or AccessKit conversion.
- If action dispatch tests fail, verify target inclusion and focus/widget support before adding new action behavior.
- If ABI tests fail, validate request ABI version, viewport handle, output pointer, buffer owner token, and JSON serialization separately.
- If AccessKit conversion fails, isolate version-specific API differences in `accesskit.rs`; do not leak AccessKit types into interface DTOs.
- If app host tests/checks fail, verify optional feature wiring and runtime API availability before editing winit event flow.
- If a failure is in a sibling-owned unrelated module, record it in the session note and acceptance doc rather than patching outside this plan.

## Implementation Order Summary

- [ ] M0: Interface contracts and ABI additions.
- [ ] M1: Runtime snapshot extraction and diagnostics.
- [ ] M2: Runtime action dispatch.
- [ ] M3: Dynamic runtime API snapshot/action roundtrip.
- [ ] M4: Optional AccessKit conversion.
- [ ] M5: Winit app host bridge.
- [ ] M6: Docs, acceptance, and scoped validation.

## Manual Screen Reader Smoke

After scoped CI-friendly validation passes, run one manual platform smoke on a machine with screen reader support:

- Start the runtime preview with `zircon_app/accessibility-accesskit-winit` enabled.
- Enable a platform screen reader.
- Confirm the window exposes a named root and at least one named interactive node.
- Move UI focus and confirm screen reader focus follows the runtime snapshot.
- Trigger an accessibility activation and confirm it reaches the runtime action path.
- Record platform, screen reader, command, observed behavior, and limitations in `tests/acceptance/accesskit-bridge.md`.

This manual smoke is acceptance evidence, not a default CI requirement.
