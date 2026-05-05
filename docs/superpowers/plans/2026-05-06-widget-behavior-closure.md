# Widget Behavior Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the shared Runtime UI widget/component event loop for pointer hover, press, focus, capture, release-inside click, scroll fallback, diagnostics, and descriptor-driven interaction metadata.

**Architecture:** Keep widget behavior in `zircon_runtime_interface::ui` DTOs and `zircon_runtime::ui` shared routing code, with `UiSurfaceFrame.arranged_tree + hit_grid` as the spatial source of truth. Align with Unreal Slate's `WidgetPath + FReply` semantics by treating dispatch effects as transient router replies consumed once by `UiSurface::dispatch_pointer_event(...)`, while `.ui.toml`/Material/editor hosts consume typed component envelopes instead of adding control-specific coordinate branches.

**Tech Stack:** Rust, Cargo, `zircon_runtime_interface::ui` DTOs, `zircon_runtime::ui` surface/router, `.ui.toml` template metadata, Unreal Slate references under `dev/UnrealEngine`, Slint Material references under `dev/slint`, Markdown docs and acceptance notes.

---

## Execution Policy

- Work directly on `main` in the existing checkout.
- Do not create worktrees or feature branches.
- Do not commit unless the user explicitly asks for a commit.
- Preserve unrelated dirty work. Do not revert active UI, editor, layout, painter, media, or docs changes from other sessions.
- Use the `zirconEngine` milestone-first cadence: implementation slices may add production code, unit-test code, comments, and docs; compile/build/unit-test execution belongs to each milestone testing stage unless a blocker requires earlier evidence.
- Before implementation and before widening scope, refresh `.codex/plans` and `.codex/sessions` with the cross-session coordination script and update `.codex/sessions/20260506-0414-widget-behavior-closure.md` when touched modules, tests, or blockers change.
- Avoid active sibling-session paths unless this plan proves a lower shared behavior dependency:
  - Avoid broad Material metric/layout edits owned by `.codex/sessions/20260506-0112-material-layout-foundation.md`.
  - Avoid painter/media/SVG/FPS/Asset Browser visual paths owned by `.codex/sessions/20260505-2334-asset-browser-material-svg-fps.md`.
  - Avoid native text/input/painter behavior owned by `.codex/sessions/20260505-1106-editor-native-text-input-regression.md`.
  - Avoid editor host hit-test rewrites owned by `.codex/sessions/20260505-1502-editor-ui-layout-regression.md` unless the shared event closure exposes a direct integration blocker.

## Design Source

- Approved spec: `docs/superpowers/specs/2026-05-06-widget-behavior-closure-design.md`.
- User request: `2026-05-06 集中完善 widget 组件行为闭环，参照 dev 下虚化源码`.
- Plans:
  - `.codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md`
  - `.codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md`
  - `.codex/plans/Shared Slate-Style UI Layout, Render, And Hit Framework.md`
- Unreal reference files:
  - `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SWidget.h`
  - `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp`
  - `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/Reply.h`
  - `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/Reply.cpp`
  - `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/SlateApplication.h`
  - `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp`
  - `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/HittestGrid.h`
  - `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp`
  - `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/WidgetPath.h`
  - `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Layout/WidgetPath.cpp`
- Slint Material reference files:
  - `dev/slint/ui-libraries/material/src/ui/components/base_button.slint`
  - `dev/slint/ui-libraries/material/src/ui/components/state_layer.slint`
  - `dev/slint/ui-libraries/material/src/ui/components/text_field.slint`
  - `dev/slint/ui-libraries/material/src/ui/components/menu.slint`
  - `dev/slint/ui-libraries/material/src/ui/components/dialog.slint`
  - `dev/slint/ui-libraries/material/src/ui/components/base_navigation.slint`

## Current Repository Baseline

- `UiSurfaceFrame` already carries arranged tree, render extract, hit grid, and focus snapshot.
- `UiPointerRoute` already carries pointer kind, button, activation phase, point, scroll delta, hit path, bubble route, stacked hits, hover entered/left, captured, pressed, click target, release-inside flag, focused node, and root fallback targets.
- `UiPointerDispatchEffect` currently has `Unhandled`, `Handled`, `Blocked`, `Passthrough`, and `Captured`.
- `UiPointerDispatchResult` currently records `handled_by`, `blocked_by`, `passthrough`, `captured_by`, diagnostics, and `component_events`.
- `UiSurface::dispatch_pointer_event(...)` currently routes pointer events, applies capture for `Captured`, releases capture on `Up`, scrolls the nearest scrollable fallback, and emits hover/press/release/click component envelopes for matching bindings.
- `UiSurface::route_pointer_event_with_details(...)` already sets focus on `Down`, stores pressed target, resolves release-inside click target from physical hit stack, clears pressed/capture on `Up`, and diffs hover stacks.
- `UiComponentEvent` already includes `Focus`, `Hover`, `Press`, `Commit`, and richer follow-up event variants.
- `zircon_runtime/src/ui/template/build/interaction.rs` currently infers interactive state from non-empty bindings or component names `Button`, `IconButton`, and `TextField`; this is the main descriptor/capability gap this plan closes without adding editor host special cases.

## File Structure And Responsibilities

### Interface Contracts

- Modify `zircon_runtime_interface/src/ui/dispatch/pointer/effect.rs` for transient Slate-like pointer reply effects. Keep this enum focused on dispatch effects only.
- Modify `zircon_runtime_interface/src/ui/dispatch/pointer/invocation.rs` so invocation records can preserve effect payloads if effects gain node IDs or dirty/damage metadata.
- Modify `zircon_runtime_interface/src/ui/dispatch/pointer/result.rs` for diagnostics, release-capture, focus-change, dirty, and damage result fields.
- Modify `zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs` for new semantic reasons such as `FocusGained`, `FocusLost`, `ReleaseCapture`, `DefaultClickRejected`, or use a minimal subset if tests prove fewer are needed.
- Modify `zircon_runtime_interface/src/ui/dispatch/pointer/mod.rs` only for structural re-exports if new files are added.
- Modify `zircon_runtime_interface/src/ui/component/descriptor/component_descriptor.rs` only if component descriptors need a small, serializable interaction contract in this slice.
- Modify `zircon_runtime_interface/src/tests/contracts.rs` when DTO shape changes need interface contract coverage.

### Runtime Shared Behavior

- Modify `zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs` to consume the richer `UiPointerDispatchEffect` and populate `UiPointerDispatchResult` without durable side effects.
- Modify `zircon_runtime/src/ui/surface/surface.rs` for focus/blur envelope generation, release-capture effect application, dirty/damage diagnostics, descriptor-driven default behavior hooks, and scroll fallback diagnostics.
- Modify `zircon_runtime/src/ui/template/build/interaction.rs` to stop growing component-name special cases and use binding/component interaction metadata instead.
- Modify `zircon_runtime/src/ui/template/build/tree_builder.rs` only if descriptor/capability metadata must be carried into `UiTemplateNodeMetadata` or `UiTreeNode` during build.
- Modify `zircon_runtime/src/ui/tests/event_routing.rs` for focused event-loop tests.
- Modify `zircon_runtime/src/ui/tests/shared_core.rs` only for shared helper or descriptor-driven tree-building tests that do not belong in `event_routing.rs`.
- Modify `zircon_runtime/src/ui/tests/hit_grid.rs` only when hit-grid diagnostics or disabled/hidden boundary assertions need event-loop coverage.

### Docs And Acceptance

- Update `docs/ui-and-layout/shared-ui-core-foundation.md` with the widget behavior closure contract and validation evidence.
- Update or create `tests/acceptance/widget-behavior-closure.md` with scoped acceptance inventory, commands, results, and remaining gaps.
- Update `.codex/sessions/20260506-0414-widget-behavior-closure.md` during execution and delete it on clean completion if no handoff is needed.

## Milestone 0: Coordination And Baseline Freeze

- Goal: Freeze the active-session boundary and exact current behavior before source edits begin.
- In-scope behaviors: coordination scan, dirty-worktree awareness, current type inventory, current focused test inventory.
- Dependencies: existing `.codex/plans`, `.codex/sessions`, approved design spec, and current dirty worktree.

### Implementation Slices

- [ ] Run the recent coordination context script from the repository root:

```powershell
.\.opencode\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot "E:\Git\ZirconEngine" -LookbackHours 4
```

- [ ] Read active session notes that touch `zircon_runtime/src/ui`, `zircon_runtime_interface/src/ui`, `zircon_editor/src/ui/slint_host`, `zircon_editor/assets/ui`, or `docs/ui-and-layout`.
- [ ] Update `.codex/sessions/20260506-0414-widget-behavior-closure.md` so `Current Step`, `Touched Modules`, `Related Tests`, and `Coordination Notes` match the first implementation milestone.
- [ ] Read these files immediately before editing so the plan is applied to live dirty baseline, not stale memory:
  - `zircon_runtime_interface/src/ui/dispatch/pointer/effect.rs`
  - `zircon_runtime_interface/src/ui/dispatch/pointer/result.rs`
  - `zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs`
  - `zircon_runtime_interface/src/ui/surface/pointer/route.rs`
  - `zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs`
  - `zircon_runtime/src/ui/surface/surface.rs`
  - `zircon_runtime/src/ui/template/build/interaction.rs`
  - `zircon_runtime/src/ui/tests/event_routing.rs`

### Testing Stage: Baseline Gate

- [ ] No Cargo build is required for this milestone.
- [ ] Record coordination timestamp and active overlapping session names in `.codex/sessions/20260506-0414-widget-behavior-closure.md`.
- [ ] Record the current focused test inventory in `tests/acceptance/widget-behavior-closure.md` if the file already exists; otherwise create it in Milestone 4 after behavior lands.

### Exit Evidence

- The active session note states the current milestone, overlap boundaries, and planned test commands.
- No source behavior is changed before the overlap boundary is recorded.

## Milestone 1: Pointer Reply Contract Closure

- Goal: Extend current pointer dispatch contracts just enough to express Slate-like transient reply effects, focus/capture side effects, dirty flags, damage frames, and richer diagnostics without creating durable widget state.
- In-scope behaviors: release capture, set/clear focus, request dirty flags, request damage, focus/capture diagnostics, default-click rejection diagnostics, and backward-compatible handling of existing `Handled`, `Blocked`, `Passthrough`, and capture behavior.
- Dependencies: current `UiPointerDispatchEffect`, `UiPointerDispatchInvocation`, `UiPointerDispatchResult`, `UiDirtyFlags`, `UiFrame`, `UiNodeId`, and existing interface contract tests.

### Implementation Slices

- [ ] In `zircon_runtime_interface/src/ui/dispatch/pointer/effect.rs`, replace the payload-free effect variants with payload-carrying variants that still have ergonomic constructors:

```rust
use serde::{Deserialize, Serialize};

use crate::ui::{event_ui::UiNodeId, layout::UiFrame, tree::UiDirtyFlags};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum UiPointerDispatchEffect {
    #[default]
    Unhandled,
    Handled,
    Blocked,
    Passthrough,
    CapturePointer,
    ReleasePointerCapture,
    SetFocus { focus_visible: bool },
    ClearFocus,
    RequestDirty(UiDirtyFlags),
    RequestDamage(UiFrame),
}

impl UiPointerDispatchEffect {
    pub const fn handled() -> Self {
        Self::Handled
    }

    pub const fn blocked() -> Self {
        Self::Blocked
    }

    pub const fn passthrough() -> Self {
        Self::Passthrough
    }

    pub const fn capture() -> Self {
        Self::CapturePointer
    }

    pub const fn release_capture() -> Self {
        Self::ReleasePointerCapture
    }

    pub const fn set_focus(focus_visible: bool) -> Self {
        Self::SetFocus { focus_visible }
    }

    pub const fn clear_focus() -> Self {
        Self::ClearFocus
    }

    pub const fn request_dirty(flags: UiDirtyFlags) -> Self {
        Self::RequestDirty(flags)
    }

    pub const fn request_damage(frame: UiFrame) -> Self {
        Self::RequestDamage(frame)
    }

    pub const fn target_override(self, node_id: UiNodeId) -> Option<UiNodeId> {
        match self {
            Self::Unhandled
            | Self::Handled
            | Self::Blocked
            | Self::Passthrough
            | Self::CapturePointer
            | Self::ReleasePointerCapture
            | Self::SetFocus { .. }
            | Self::ClearFocus
            | Self::RequestDirty(_)
            | Self::RequestDamage(_) => Some(node_id),
        }
    }
}
```

- [ ] Keep `UiPointerDispatchEffect::capture()` available so existing tests and call sites using the current constructor continue to compile while the variant name becomes `CapturePointer`.
- [ ] In `zircon_runtime_interface/src/ui/dispatch/pointer/invocation.rs`, remove `Eq` from the derive list because `UiPointerDispatchEffect::RequestDamage(UiFrame)` contains `f32` fields and cannot implement `Eq`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerDispatchInvocation {
    pub node_id: UiNodeId,
    pub effect: UiPointerDispatchEffect,
}
```

- [ ] If Rust rejects `Copy` for any added field, keep `Clone` and remove `Copy` from both `UiPointerDispatchEffect` and `UiPointerDispatchInvocation`. Update call sites by cloning effects where necessary.
- [ ] In `zircon_runtime_interface/src/ui/dispatch/pointer/result.rs`, extend diagnostics and result fields:

```rust
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPointerDispatchDiagnostics {
    pub pointer_routed: bool,
    pub ignored_same_target_hover: bool,
    pub hover_entered: usize,
    pub hover_left: usize,
    pub focus_changed: bool,
    pub capture_started: bool,
    pub capture_released: bool,
    pub click_target_resolved: bool,
    pub default_click_rejected: bool,
    pub component_event_count: usize,
    pub scroll_defaulted: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerDispatchResult {
    pub route: UiPointerRoute,
    pub invocations: Vec<UiPointerDispatchInvocation>,
    pub handled_by: Option<UiNodeId>,
    pub blocked_by: Option<UiNodeId>,
    pub passthrough: Vec<UiNodeId>,
    pub captured_by: Option<UiNodeId>,
    #[serde(default)]
    pub released_capture: Option<UiNodeId>,
    #[serde(default)]
    pub focus_changed_to: Option<UiNodeId>,
    #[serde(default)]
    pub focus_cleared: bool,
    #[serde(default)]
    pub requested_dirty: UiDirtyFlags,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requested_damage: Vec<UiFrame>,
    #[serde(default)]
    pub diagnostics: UiPointerDispatchDiagnostics,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_events: Vec<UiPointerComponentEvent>,
}
```

- [ ] Import `UiFrame` and `UiDirtyFlags` in `result.rs` and initialize the new fields in `UiPointerDispatchResult::new(...)`.
- [ ] In `component_event.rs`, add reasons that describe the new semantic envelopes and diagnostic outcomes:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiPointerComponentEventReason {
    #[default]
    DirectBinding,
    DefaultClick,
    DefaultClickRejected,
    HoverEnter,
    HoverLeave,
    PressBegin,
    PressEnd,
    FocusGained,
    FocusLost,
    ScrollFallback,
}
```

- [ ] In `zircon_runtime_interface/src/tests/contracts.rs`, add or update one focused test that constructs `UiPointerDispatchResult::new(route)` and asserts new fields default to no side effects and diagnostics preserve `click_target_resolved`.
- [ ] Keep all modified root modules structural. Only update `mod.rs` re-exports if a new declaration file becomes necessary.

### Testing Stage: Contract Gate

- [ ] Run scoped formatting for interface files touched in this milestone:

```powershell
rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/dispatch/pointer/effect.rs" "zircon_runtime_interface/src/ui/dispatch/pointer/invocation.rs" "zircon_runtime_interface/src/ui/dispatch/pointer/result.rs" "zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs" "zircon_runtime_interface/src/ui/dispatch/pointer/mod.rs" "zircon_runtime_interface/src/tests/contracts.rs"
```

- [ ] Run interface-focused tests:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime_interface --lib ui_pointer --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture
```

- [ ] If the filter does not match because contract tests use broader names, run:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture
```

- [ ] Run interface type check:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never
```

- [ ] Debug/correction loop: if failures show derived `Copy` conflicts, remove `Copy` and update call sites; if failures show missing serde defaults, add `#[serde(default)]`; if failures show old `Captured` variant references, update runtime code to `CapturePointer` or constructor `capture()`.

### Exit Evidence

- Interface pointer reply/effect DTOs can represent capture, release capture, focus, dirty, and damage without durable widget state.
- Existing handled/blocked/passthrough/capture call sites remain expressible.
- Focused interface tests and `cargo check -p zircon_runtime_interface --lib --locked` pass or blockers are recorded with exact diagnostics.

## Milestone 2: Runtime Router Reply Consumption And Semantic Envelopes

- Goal: Make `UiSurface::dispatch_pointer_event(...)` consume transient pointer effects exactly once, update focus/capture state, emit focus/blur and click/press/hover envelopes, and return diagnostics/damage suggestions.
- In-scope behaviors: release capture, set focus, clear focus, focus/blur events, capture start/end diagnostics, same-target hover idle, release-inside click, release-outside rejection diagnostic, scroll fallback diagnostic, and component event count.
- Dependencies: Milestone 1 pointer DTOs, current `UiSurface::route_pointer_event_with_details(...)`, current `pointer_component_events(...)`, and current runtime event-routing tests.

### Implementation Slices

- [ ] In `zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs`, update the effect match to handle new variants without mutating `UiSurface` directly:

```rust
match effect {
    UiPointerDispatchEffect::Unhandled => {}
    UiPointerDispatchEffect::Handled => {
        result.handled_by = Some(node_id);
        return Ok(result);
    }
    UiPointerDispatchEffect::Blocked => {
        result.blocked_by = Some(node_id);
        continue 'candidate;
    }
    UiPointerDispatchEffect::Passthrough => {
        result.passthrough.push(node_id);
    }
    UiPointerDispatchEffect::CapturePointer => {
        result.captured_by = Some(node_id);
        result.handled_by = Some(node_id);
        return Ok(result);
    }
    UiPointerDispatchEffect::ReleasePointerCapture => {
        result.released_capture = Some(node_id);
        result.handled_by = Some(node_id);
        return Ok(result);
    }
    UiPointerDispatchEffect::SetFocus { .. } => {
        result.focus_changed_to = Some(node_id);
        result.handled_by = Some(node_id);
        return Ok(result);
    }
    UiPointerDispatchEffect::ClearFocus => {
        result.focus_cleared = true;
        result.handled_by = Some(node_id);
        return Ok(result);
    }
    UiPointerDispatchEffect::RequestDirty(flags) => {
        result.requested_dirty.layout |= flags.layout;
        result.requested_dirty.hit_test |= flags.hit_test;
        result.requested_dirty.render |= flags.render;
        result.requested_dirty.style |= flags.style;
        result.requested_dirty.text |= flags.text;
        result.requested_dirty.input |= flags.input;
        result.requested_dirty.visible_range |= flags.visible_range;
    }
    UiPointerDispatchEffect::RequestDamage(frame) => {
        result.requested_damage.push(frame);
    }
}
```

- [ ] In `UiSurface::dispatch_pointer_event(...)`, record `focus_before_dispatch`, `capture_before_dispatch`, and `hover_before_dispatch` before routing/dispatch if those are not already available after `route_pointer_event_with_details(...)`.
- [ ] Apply `result.captured_by` by setting `self.focus.captured` and set `result.diagnostics.capture_started = true` when capture changes from a different previous value.
- [ ] Apply `result.released_capture` only if `self.focus.captured == Some(node_id)` or route captured was `Some(node_id)`. Clear `self.focus.captured`, set `result.diagnostics.capture_released = true`, and do not clear another node's capture.
- [ ] Apply `result.focus_changed_to` by calling `self.focus_node(node_id)` and preserve `navigation.focus_visible` according to `UiPointerDispatchEffect::SetFocus { focus_visible }` if the effect payload is retained in `invocations`. If retrieving the payload from invocations is too brittle, set pointer reply focus to `focus_visible = false` and leave keyboard focus-visible policy to navigation.
- [ ] Apply `result.focus_cleared` by calling `self.clear_focus()`.
- [ ] Set diagnostics after all reply effects:

```rust
result.diagnostics.focus_changed = focus_before_dispatch != self.focus.focused;
result.diagnostics.capture_released = result.diagnostics.capture_released
    || (matches!(event.kind, UiPointerEventKind::Up)
        && capture_before_dispatch.is_some()
        && self.focus.captured.is_none());
result.diagnostics.component_event_count = result.component_events.len();
result.diagnostics.default_click_rejected = route.activation_phase == UiPointerActivationPhase::PrimaryRelease
    && route.pressed.is_some()
    && route.click_target.is_none();
```

- [ ] Add a helper in `surface.rs` to emit focus/blur component envelopes by comparing old and new focus:

```rust
fn push_focus_component_events(
    &self,
    events: &mut Vec<UiPointerComponentEvent>,
    old_focus: Option<UiNodeId>,
    new_focus: Option<UiNodeId>,
) -> Result<(), UiTreeError> {
    if old_focus == new_focus {
        return Ok(());
    }
    if let Some(node_id) = old_focus {
        self.push_pointer_component_events(
            events,
            node_id,
            UiEventKind::Blur,
            UiComponentEvent::Focus { focused: false },
            UiPointerComponentEventReason::FocusLost,
        )?;
    }
    if let Some(node_id) = new_focus {
        self.push_pointer_component_events(
            events,
            node_id,
            UiEventKind::Focus,
            UiComponentEvent::Focus { focused: true },
            UiPointerComponentEventReason::FocusGained,
        )?;
    }
    Ok(())
}
```

- [ ] Change component event collection so focus/blur envelopes are appended after pointer hover/press/click envelopes for the same dispatch result.
- [ ] Keep `pointer_component_events(...)` filtering by actual bindings. A focusable node without `onFocus`/`onBlur` binding should update focus state but emit no focus/blur envelope.
- [ ] Add `requested_damage` for hover/press/focus changes by pushing old/new arranged frames when available from `self.arranged_tree.get(node_id)`. If damage frame support causes too much churn, keep the field empty in this milestone and record damage frame wiring as follow-up in Milestone 4 docs; do not invent editor repaint code in runtime.
- [ ] Add tests to `zircon_runtime/src/ui/tests/event_routing.rs` using the existing `button_surface_with_metadata(...)` helper style:
  - `focus_component_events_emit_focus_and_blur_for_matching_bindings`
  - `release_capture_effect_clears_only_current_captor`
  - `release_outside_pressed_target_reports_default_click_rejected`
  - `scroll_fallback_reports_scroll_defaulted_when_unhandled`
  - `pointer_dispatch_result_counts_component_events`
- [ ] For the release-capture test, register a pointer handler on the pressed/captured node that returns `UiPointerDispatchEffect::release_capture()` and assert `surface.focus.captured == None`, `result.released_capture == Some(node)`, and `result.diagnostics.capture_released`.
- [ ] For the focus/blur test, bind the first button to `UiEventKind::Focus` and `UiEventKind::Blur`, click it, then click a second focusable button and assert blur/focus event order.
- [ ] For the scroll fallback test, create a scrollable child with `UiInputPolicy::Receive`, dispatch unhandled `UiPointerEventKind::Scroll` with a nonzero delta, and assert `result.handled_by == Some(scroll_node)` plus `result.diagnostics.scroll_defaulted`.

### Testing Stage: Runtime Router Gate

- [ ] Run scoped formatting for changed runtime/interface test files:

```powershell
rustfmt --edition 2021 --check "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/tests/event_routing.rs" "zircon_runtime/src/ui/tests/shared_core.rs" "zircon_runtime/src/ui/tests/hit_grid.rs"
```

- [ ] Run focused event routing tests:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture
```

- [ ] Run existing shared pointer/hit coverage that this milestone can regress:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture
```

- [ ] Run runtime type check:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never
```

- [ ] Debug/correction loop: if a new runtime test fails because focus is already changed during route construction, adjust the focus comparison point so focus/blur emits from pre-route focus to post-dispatch focus; if scroll tests fail because no scroll state exists, use an existing scrollable-node helper from `shared_core.rs` instead of adding a special production branch.

### Exit Evidence

- Runtime dispatch consumes capture, release capture, set focus, clear focus, dirty, and damage effects without storing reply objects.
- Focus/blur, hover, press, release, and default-click envelopes are emitted only for matching bindings.
- Default-click rejection and scroll fallback diagnostics are observable in focused runtime tests.
- Focused runtime event tests, hit-grid tests, and `cargo check -p zircon_runtime --lib --locked` pass or blockers are recorded with exact diagnostics.

## Milestone 3: Descriptor-Driven Interaction Inference

- Goal: Stop expanding hard-coded component-name interaction special cases and make template-built nodes interactive through binding metadata plus descriptor/capability-derived metadata.
- In-scope behaviors: bound controls remain interactive, unbound components with declared interactive capabilities become focusable/clickable/hoverable, and runtime tests prove a new component name can become interactive without an editor host branch.
- Dependencies: current component descriptor catalog, `UiComponentDescriptor::events`, `UiTemplateNodeMetadata`, template build path, and Milestone 2 semantic routing.

### Implementation Slices

- [ ] Inspect `zircon_runtime/src/ui/component/catalog/editor_showcase.rs` and the descriptor registry type to find the smallest way to query component events during template tree building. Prefer passing an optional registry or capability lookup into the builder only if it already exists in nearby build sessions.
- [ ] If no registry is available in `UiTemplateTreeBuilder`, avoid a broad dependency injection refactor in this slice. Instead, add a metadata-driven helper in `interaction.rs` that treats a node as interactive when any binding event is one of the event kinds that requires pointer/focus input:

```rust
fn binding_requires_interaction(event: zircon_runtime_interface::ui::binding::UiEventKind) -> bool {
    matches!(
        event,
        UiEventKind::Click
            | UiEventKind::DoubleClick
            | UiEventKind::Hover
            | UiEventKind::Press
            | UiEventKind::Release
            | UiEventKind::Change
            | UiEventKind::Submit
            | UiEventKind::Toggle
            | UiEventKind::Focus
            | UiEventKind::Blur
            | UiEventKind::Scroll
            | UiEventKind::DragBegin
            | UiEventKind::DragUpdate
            | UiEventKind::DragEnd
            | UiEventKind::Drop
    )
}
```

- [ ] Replace the current component-name match in `infer_interaction(...)` with binding-event inference plus a narrow metadata attribute fallback:

```rust
let is_interactive = node
    .bindings
    .iter()
    .any(|binding| binding_requires_interaction(binding.event))
    || node
        .attributes
        .get("input_interactive")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
```

- [ ] Add support for optional explicit metadata attributes only if existing `.ui.toml` parser already carries bool values in `node.attributes`. Use these exact keys when needed:
  - `input_interactive = true`
  - `input_focusable = true`
  - `input_clickable = true`
  - `input_hoverable = true`
- [ ] Set `UiStateFlags` fields independently when explicit attributes exist:

```rust
let clickable = bool_attr(node, "input_clickable").unwrap_or(is_interactive);
let hoverable = bool_attr(node, "input_hoverable").unwrap_or(is_interactive);
let focusable = bool_attr(node, "input_focusable").unwrap_or(is_interactive);
let receives_input = clickable || hoverable || focusable || is_interactive;
```

- [ ] Keep existing `Button`/`IconButton`/`TextField` fallback only if removing it breaks existing authored assets with no bindings and no explicit metadata. If it must remain, isolate it behind a function named `legacy_component_interaction_fallback(...)`, add a comment that it is a temporary authored-asset fallback, and add a doc note that future `.ui.toml` components should use binding or explicit input metadata.
- [ ] Add tests proving:
  - a custom component name with `onClick` binding becomes `UiInputPolicy::Receive`, clickable, hoverable, and focusable,
  - a custom component name with `input_focusable = true` but no click binding becomes focusable and receives input,
  - an unbound visual component without input metadata remains non-interactive,
  - a bound custom component emits click envelope through `UiSurface::dispatch_pointer_event(...)` after template tree build.
- [ ] Put these tests in `zircon_runtime/src/ui/tests/event_routing.rs` if they exercise dispatch, or `zircon_runtime/src/ui/tests/template.rs` if they only inspect built tree flags. Prefer `event_routing.rs` for the end-to-end closure assertion.

### Testing Stage: Interaction Metadata Gate

- [ ] Run scoped formatting:

```powershell
rustfmt --edition 2021 --check "zircon_runtime/src/ui/template/build/interaction.rs" "zircon_runtime/src/ui/template/build/tree_builder.rs" "zircon_runtime/src/ui/tests/event_routing.rs" "zircon_runtime/src/ui/tests/template.rs"
```

- [ ] Run focused runtime tests:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture
```

- [ ] If tests were added to template tests, run:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib template --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture
```

- [ ] Run runtime type check:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never
```

- [ ] Debug/correction loop: if `.ui.toml` bool values are not represented as TOML booleans in `node.attributes`, use the actual existing `toml::Value` shape from parser output and update the helper accordingly; do not add a new parser branch unless the parser drops bool values entirely.

### Exit Evidence

- Bound custom component names become interactive without adding editor host or component-name dispatch branches.
- Explicit input metadata can make an unbound control focusable/clickable/hoverable where authored assets require it.
- Existing Button/IconButton/TextField authored behavior is preserved or any retained legacy fallback is isolated and documented.
- Focused runtime tests and `cargo check -p zircon_runtime --lib --locked` pass or blockers are recorded with exact diagnostics.

## Milestone 4: Docs, Acceptance, And Scoped Validation

- Goal: Record the shared widget behavior closure, reference evidence, implementation files, focused validation, and remaining gaps before claiming the slice complete.
- In-scope behaviors: docs header maintenance, acceptance file, session note update, full scoped command inventory, and no workspace-wide green claim unless workspace validation is actually run.
- Dependencies: Milestones 1-3 complete, focused tests passing or blockers understood.

### Implementation Slices

- [ ] Update `docs/ui-and-layout/shared-ui-core-foundation.md` frontmatter:
  - add `zircon_runtime_interface/src/ui/dispatch/pointer/effect.rs`, `result.rs`, `component_event.rs`, `zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs`, `zircon_runtime/src/ui/surface/surface.rs`, and `zircon_runtime/src/ui/template/build/interaction.rs` to `related_code` and `implementation_files` if not already listed,
  - add `docs/superpowers/specs/2026-05-06-widget-behavior-closure-design.md` and this plan to `plan_sources`,
  - add focused test files and validation commands to `tests` or body evidence.
- [ ] In the body of `shared-ui-core-foundation.md`, add a concise section named `Widget Behavior Closure` covering:
  - pointer replies are transient,
  - capture/release/focus are router state mutations,
  - component envelopes are typed and binding-filtered,
  - same-target hover is idle,
  - release-inside click is the default activation rule,
  - descriptor/binding metadata owns interaction, not editor host coordinate branches.
- [ ] Create `tests/acceptance/widget-behavior-closure.md` with this shape:

```markdown
# Widget Behavior Closure

## Scope
- Shared Runtime UI widget/component pointer behavior closure for hover, press, focus, capture, release-inside click, scroll fallback, diagnostics, and metadata-driven interaction inference.

## Reference Evidence
- Unreal Slate: `SWidget`, `FReply`, `FSlateApplication`, `FHittestGrid`, and `FWidgetPath` files listed in `docs/superpowers/specs/2026-05-06-widget-behavior-closure-design.md`.
- Slint Material: state-layer, button, text-field, menu, dialog, and navigation files listed in the spec.

## Test Inventory
- `zircon_runtime_interface` pointer reply/effect contract tests.
- `zircon_runtime::ui::tests::event_routing` focus/blur, capture release, release-inside click, release-outside rejection, same-target hover idle, scroll fallback, and component envelope count tests.
- `zircon_runtime::ui::tests::hit_grid` existing visibility/clip/disabled hit-grid regressions.
- `zircon_runtime::ui::tests::template` or `event_routing` metadata-driven interaction tests if added in Milestone 3.

## Results
- Record exact command lines and pass/fail output summaries after running the testing stage.

## Acceptance Decision
- Accepted only if all focused contract/runtime checks pass. If failures remain, mark this file partial and list blockers.
```

- [ ] Update `.codex/sessions/20260506-0414-widget-behavior-closure.md` with final touched files, exact validation commands, results, blockers, and next steps.
- [ ] If no handoff is needed after final response, delete `.codex/sessions/20260506-0414-widget-behavior-closure.md`. If another active session needs the result, move it to `.codex/sessions/archive/` with `status: completed` and a short completion summary.

### Testing Stage: Final Scoped Gate

- [ ] Run scoped formatting for every touched Rust file. Use an explicit file list; do not run broad formatting on active sibling-session files unless they are touched by this plan.
- [ ] Run interface validation if interface files changed:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never
```

- [ ] Run runtime focused validation:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never
```

- [ ] If this slice changes public DTOs consumed by editor, run editor type check as a boundary check:

```powershell
$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never
```

- [ ] Run `git diff --check` and record whether it reports only existing line-ending warnings or new whitespace errors.
- [ ] Debug/correction loop: if upper-layer editor type check fails from DTO changes, fix the shared DTO consumer imports/fields rather than adding editor-only compatibility shims; if runtime tests fail because a lower hit-grid/visibility assumption changed, apply support-first diagnosis to `UiSurfaceFrame`/hit-grid before changing event tests.

### Exit Evidence

- `tests/acceptance/widget-behavior-closure.md` records exact commands and outcomes.
- `docs/ui-and-layout/shared-ui-core-foundation.md` documents the behavior contract and points to implementation files.
- Final report states whether validation was scoped or workspace-wide, whether `--locked` was used, and whether old compatibility paths were removed or none existed.
- No claim of workspace-wide green is made unless workspace-wide commands are run and pass.

## Out Of Scope Follow-Up Backlog

- Full keyboard/text/IME route and editable text state machine.
- Drag threshold, drag/drop operation lifetime, accepted/rejected payload diagnostics.
- Popup/menu/dialog surface registry and outside-click close semantics.
- Multi-pointer, multi-user, multi-window capture/focus.
- Editor native host full cutover for every surface after shared runtime behavior is proven.
- Widget Reflector-style live/snapshot event-path and invalidation debugging.
