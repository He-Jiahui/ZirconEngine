---
related_code:
  - zircon_runtime_interface/src/ui/dispatch/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/result.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/surface/pointer/route.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/ui/component/event.rs
  - zircon_runtime/src/ui/dispatch
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/state.rs
  - zircon_runtime/src/ui/surface/input/validation.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/mod.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
implementation_files:
  - zircon_runtime_interface/src/ui/dispatch
  - zircon_runtime_interface/src/ui/surface
  - zircon_runtime/src/ui/dispatch
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/state.rs
  - zircon_runtime/src/ui/surface/input/validation.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/component/state_reducer.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
plan_sources:
  - user: 2026-05-06 完善输入事件内容 参照dev下虚幻源码
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - .codex/plans/Shared Slate-Style UI Layout, Render, And Hit Framework.md
  - .codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/Reply.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/Events.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/NavigationReply.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/DragAndDrop.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SWidget.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateUser.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/NavigationConfig.h
  - dev/slint/internal/core/input.rs
  - dev/slint/internal/core/window.rs
  - dev/slint/internal/backends/winit/event_loop.rs
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
  - cargo test -p zircon_runtime_interface --lib ui --locked
  - cargo test -p zircon_runtime_interface --lib ui_input --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib event_routing --locked
  - cargo test -p zircon_runtime --lib runtime_input_ownership --locked
  - cargo test -p zircon_runtime --lib shared_core --locked
  - cargo test -p zircon_editor --lib native_host_contract --locked
  - cargo check -p zircon_runtime --lib --locked
  - owner-safety-final-validation: rustfmt --edition 2021 --config skip_children=true --check zircon_runtime/src/ui/surface/input/mod.rs zircon_runtime/src/ui/surface/input/state.rs zircon_runtime/src/ui/surface/input/validation.rs zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/surface/input/effect.rs zircon_runtime/src/ui/surface/surface.rs zircon_runtime/src/ui/tests/mod.rs zircon_runtime/src/ui/tests/event_routing.rs zircon_runtime/src/ui/tests/runtime_input_ownership.rs (passed)
  - owner-safety-final-validation: cargo test -p zircon_runtime --lib runtime_input_ownership --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (7 passed, 0 failed, 897 filtered out)
  - owner-safety-final-validation: cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (20 passed, 0 failed, 884 filtered out)
  - owner-safety-final-validation: cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (38 passed, 0 failed, 866 filtered out)
  - owner-safety-final-validation: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never (passed with existing warnings)
  - cargo check -p zircon_editor --lib --locked
doc_type: design-spec
---

# UI Complete Input Events Design

## Summary

This design defines the M5 complete input-event system for Zircon UI. The chosen approach is a shared Slate-style reply/effect foundation: `zircon_runtime_interface::ui` owns neutral event, route, reply, effect, and result DTOs; `zircon_runtime::ui::surface` owns routing and input state transitions; editor and runtime hosts only translate platform events into shared DTOs and consume shared results.

The design uses Unreal Slate as the primary behavior reference, especially `FEventRouter` plus `FReply`, but does not copy Unreal APIs. Zircon keeps its retained `.ui.toml` and `UiSurfaceFrame` model. `UiSurfaceFrame` remains the spatial authority for hit, render, and input; the event system adds complete input semantics above that surface instead of creating editor-local hit paths or control-name branches.

## Goals

- Establish a single `UiDispatchReply` and `UiDispatchEffect` model for pointer, keyboard, text, IME, navigation, analog/gamepad, drag/drop, popup/menu, tooltip, capture, pointer lock, high precision pointer, and double click.
- Route all UI input through `UiSurfaceFrame` state and shared surface dispatch, then apply side effects centrally in `UiSurface`.
- Preserve current pointer and navigation behavior while generalizing it away from pointer-only result types.
- Make editor and runtime hosts use the same input semantics, with host code limited to platform event normalization, focus source selection, and redraw requests.
- Provide complete runtime, interface-contract, editor-host, boundary, and stress validation for the M5 scope.

## Non-Goals

- Do not implement the full M6 text system in this milestone. M5 owns text and IME event delivery, composition ownership, and edit-intent payloads; M6 owns shaping, BiDi, rich text, caret rendering, selection painting, and full editable text layout.
- Do not replace current hit-grid, layout, invalidation, painter, or Widget Reflector work owned by active sibling sessions. This design consumes those seams.
- Do not restore generated `.slint` business behavior as event authority.
- Do not add control-id, component-name, or widget-family special cases in shared routing.
- Do not implement world-space UI raycasting in M5. The event DTOs should be ready for mapped virtual pointer input, but ray/UV/world mapping remains a later host or rendering concern.

## Current Baseline

Zircon currently has a partial Slate-style input foundation:

- `UiPointerEvent` supports down, up, move, scroll, button, point, and scalar scroll delta.
- `UiPointerRoute` records target, hit path, bubble route, stacked hits, entered/left hover lists, capture, pressed target, click target, focus, and root fallback.
- `UiPointerDispatchResult` records invocations, handled/blocked/passthrough/captured nodes, diagnostics, and generated component events.
- `UiNavigationDispatcher` and `UiNavigationState` provide focused navigation fallback for activate, cancel, next, previous, and directional navigation.
- `UiComponentEvent` and `UiEventKind` already name higher-level events such as double click, focus, blur, drag begin/update/end, drop, submit, change, popup, and selection.
- `UiSurface::dispatch_pointer_event(...)` applies capture, scroll fallback, focus diagnostics, capture-release diagnostics, hover/press/release/click component events, and route-derived dispatch.

The missing pieces are architectural rather than just enum variants: no shared keyboard/text/IME DTOs, no unified reply/effect buffer, no generalized tunnel/bubble/direct route phases, no shared drag session, no popup stack, no tooltip state, no double-click recognizer, no pointer-id/device/user/timestamp/modifier metadata, no high-precision or pointer-lock effects, and no editor native `KeyboardInput`/`Ime` bridge into shared dispatch.

## Reference Evidence

### Unreal Slate

Unreal provides the primary semantic model:

- `Reply.h:18-23` defines `FReply` as the event return object used to tell the application how an input event was handled.
- `Reply.h:27-160` covers the side-effect family Zircon must translate: capture mouse, high precision mouse movement, set mouse position, set or clear focus, request navigation, lock or release mouse lock, release capture, detect drag, begin/end drag-drop, and prevent throttling.
- `Reply.h:165-223` exposes effect accessors, and `Reply.h:230-244` separates handled from unhandled replies.
- `SlateApplication.cpp:235-521` contains `FEventRouter` with direct, leafmost, tunnel, and bubble policies. The router stops propagation when a reply is handled and calls `ProcessReply(...)` for side effects.
- `SlateApplication.cpp:4822-4864` routes key preview before key down. `SlateApplication.cpp:5221-5253` routes mouse preview before mouse down. This is the model for Zircon tunnel/preprocess semantics.
- `SlateApplication.cpp:4707-4933` covers key char, key down, and key up focus-path routing.
- `SlateApplication.cpp:5480-5773` covers pointer move, drag detection, drag enter/leave/over, and drag-over routing. `SlateApplication.cpp:5326-5469` covers pointer up and drop.
- `SWidget.h:325-451` names key, preview key, pointer, cursor, and double-click handlers. `SWidget.h:474-535` names drag/drop lifecycle hooks. `SWidget.h:599-631` covers popup method and navigation hooks.
- `NavigationReply.h` and `NavigationConfig.h` show that navigation is a structured reply/request system with explicit, custom, wrap, stop, escape, key, analog, and gamepad mapping rather than a pointer special case.
- `SlateUser.cpp` owns per-user state such as drag/drop content, cursor query, mouse capture, mouse lock, tooltip discovery, and drag/drop cancellation. Zircon should translate this into surface/user-scoped input state, not widget-owned persistent replies.

### Slint

Slint provides a Rust/toolkit-facing host integration cross-check:

- `internal/backends/winit/event_loop.rs:249-365` translates winit keyboard and IME events into internal key events and composition events.
- `internal/backends/winit/event_loop.rs:367-450` translates cursor, wheel, and mouse button input, preserving logical coordinates, wheel deltas, and touch phases.
- `internal/core/window.rs:264-302` models input-method requests with surrounding text, cursor position, anchor position, preedit text, preedit offset, cursor rectangle, input type, and clip rect.
- `internal/core/window.rs:458-480` keeps focused item, active popups, click state, and input state on the window.
- `internal/core/window.rs:594-772` processes mouse input, click repeat, drag/drop conversion, popup close policy, mouse grab, and cursor changes.
- `internal/core/window.rs:803-936` handles keyboard modifiers, capture-key tunnel, focused-item bubble, tab/backtab focus movement, escape popup close, and menubar shortcuts.
- `internal/core/input.rs:24-97` shows a compact Rust input event family with mouse pressed/released/moved/wheel, drag move/drop, gestures, and exit. `internal/core/input.rs:202-219` shows accepted, ignored, grab mouse, and start drag result states.

Slint confirms that the editor host must translate native window events once, then route through runtime UI state. It also confirms that IME requires a bidirectional request surface, not only incoming text events.

## Architecture Decision

Use a unified reply/effect foundation, not parallel event-family dispatchers.

The shared dispatch flow is:

```text
platform event
  -> host normalization into UiInputEvent
  -> UiSurface route resolution from UiSurfaceFrame, focus, capture, popup, and drag state
  -> preprocess / shortcut pass
  -> preview tunnel
  -> direct or target dispatch
  -> bubble dispatch
  -> descriptor-driven default action
  -> UiDispatchReply aggregation
  -> UiSurface applies UiDispatchEffect values once
  -> UiDispatchResult returns component events, diagnostics, state changes, and redraw hints
```

This gives every input family the same side-effect model. Pointer capture, keyboard focus, IME owner, drag session, popup stack, tooltip timer, navigation request, pointer lock, and high precision pointer cannot drift into separate host or dispatcher code paths.

## Ownership

- `zircon_runtime_interface::ui::dispatch` owns serializable input events, dispatch phases, routes, replies, effects, results, invocation records, and diagnostics.
- `zircon_runtime_interface::ui::surface` owns serializable input state snapshots that must cross runtime/editor/plugin boundaries, including focus/capture/hover and future popup/drag/IME summaries.
- `zircon_runtime::ui::dispatch` owns handler registries and family-specific default handler adapters.
- `zircon_runtime::ui::surface` owns route resolution, input state mutation, reply effect application, default actions, component event emission, diagnostics, and dirty/damage hints.
- `zircon_editor::ui` owns native host normalization, source-window focus context, editor command dispatch from component events, and repaint scheduling from shared dispatch results.
- Runtime preview/HUD hosts own platform normalization for runtime windows and use the same shared DTOs as the editor.

## Data Model

### UiInputEvent

`UiInputEvent` is the top-level event enum. Every variant carries common metadata:

- event id
- timestamp or monotonically increasing frame-local sequence
- surface id and optional window id
- user id
- device id
- modifiers
- optional pointer id
- synthetic flag

Required variants:

- `Pointer`: move, enter, leave, down, up, scroll, gesture, cancel; includes position, previous position, delta, button, pressed buttons, click count, touch/stylus metadata when available, precise scroll delta x/y, line/pixel scroll unit, and gesture phase.
- `Keyboard`: key down, key up, repeat; includes logical key, physical key when available, text-without-modifiers when available, modifiers, repeat, and shortcut eligibility.
- `Text`: committed text and edit commands that are not raw key shortcuts.
- `Ime`: enabled, disabled, composition started, composition updated, committed, cancelled; includes preedit text, selection range, cursor/anchor information, input type, and candidate/cursor rectangle metadata where available.
- `Navigation`: activate, cancel, next, previous, directional moves, explicit target, wrap/stop/escape policy, and genesis/source.
- `Analog`: axis, value, device, repeat/threshold metadata, and mapping target for gamepad-style navigation.
- `DragDrop`: detect threshold crossed, begin, enter, leave, over, drop, cancel, end; includes payload, allowed operations, accepted operation, source, target, and position.
- `Popup`: open, close, close top, dismiss outside, restore focus, and menu context events.
- `TooltipTimer`: hover delay elapsed, hide delay elapsed, force show, force hide.

### UiDispatchPhase And Route

`UiDispatchPhase` should include:

- `Preprocess`: global or surface preprocessors such as shortcuts and modal filters.
- `PreviewTunnel`: root-to-leaf preview route.
- `Direct`: capture owner, IME owner, drag owner, popup owner, or explicitly targeted node.
- `Target`: leaf or focused target.
- `Bubble`: leaf-to-root or focused-node-to-root route.
- `DefaultAction`: descriptor-driven fallback after explicit handlers decline.

`UiDispatchRoute` should be a family-neutral route record:

- event family and phase
- target node
- focused node
- captured pointer node
- IME owner node
- drag source and drag current target
- popup stack top and modal root when present
- root-to-leaf route
- bubble route
- front-to-back stacked hit candidates for pointer events
- hit path for pointer events
- fallback roots for unfocused or unhit events
- rejection or fallback reasons

Existing `UiPointerRoute` and `UiNavigationRoute` can remain as specialized views during migration, but the long-term result should be expressible through `UiDispatchRoute`.

### UiDispatchReply

`UiDispatchReply` is a transient return value consumed once by `UiSurface`.

Required fields:

- disposition: unhandled, handled, blocked, or passthrough
- handler node id when a node produced the reply
- event phase that produced the reply
- ordered effects
- optional component events
- optional diagnostics or reason code

The reply must not become durable widget state. Durable state belongs in surface input state, tree node state, component state, or editor/runtime host state.

### UiDispatchEffect

Current implemented M1/M2 effects:

- `SetFocus { target, reason }`
- `ClearFocus { target, reason }`
- `CapturePointer { target, pointer_id, reason }`
- `ReleasePointerCapture { target, pointer_id, reason }`
- `LockPointer { target, policy }`
- `UnlockPointer { target, policy }`
- `UseHighPrecisionPointer { target, enabled }`
- `DragDrop { kind, target, pointer_id, session_id, point, payload }`
- `RequestNavigation { kind, policy }`
- `Popup { kind, popup_id, anchor }`
- `Tooltip { kind, tooltip_id }`
- `RequestInputMethod { request }`
- `DirtyRedraw { target, dirty, reason }`
- `EmitComponentEvent { target, event, policy }`

Longer-lived M5 expansions such as pointer repositioning, detect-drag thresholds, explicit popup close policies, tooltip placement, and separate redraw damage DTOs should extend this shared effect family without reintroducing host-local state machines.

Effects should be small and declarative. Applying an effect validates the node still exists, is enabled/visible when required, and belongs to the current surface or permitted popup subtree.

### UiInputState

Extend surface input state around existing focus/navigation fields rather than letting hosts own separate state machines.

Required state groups:

- pointer state: hovered stack per pointer, pressed target per pointer/button, captured target per pointer, click recognizer state, high precision owner, lock owner, last position, and pressed buttons.
- focus state: focused node, focus-visible flag, focus cause, previous focus, and focus route.
- keyboard state: modifiers, key repeat observations, and shortcut-preprocess diagnostics.
- text/IME state: input owner, requested input-method properties, composition text, composition selection, cursor rect, and active input type.
- navigation state: current navigation root, last navigation source, wrap/stop policy observations, and analog repeat gating.
- drag/drop state: source, payload, button, start position, current target, accepted operation, last enter path, and cancelled/completed status.
- popup state: stack entries, close policy, modal/light-dismiss mode, focus restoration target, and anchor.
- tooltip state: hovered anchor, pending show timer, visible tooltip, and hide reason.

## Event Semantics

### Pointer

Pointer events resolve capture first, then popup/modal roots, then normal `UiSurfaceFrame` hit-test. Hover enter/leave is derived from path differences. Press stores pressed target and can set focus. Release resolves click only when the pressed target remains inside the release hit stack and no drag/drop consumed the gesture.

Double click is generated by shared state from button, target, timestamp, click distance, and platform/user click interval. Hosts may provide a click count, but the shared recognizer must be able to validate or compute the semantic double-click event so editor and runtime match.

Scroll uses handler routing first. If unhandled and not blocked, it falls back to the nearest scrollable candidate in the hit stack or root fallback route. Precise scroll delta x/y and scroll unit must be preserved; editor host should stop collapsing pixel wheel input into a scalar y-only value.

### Keyboard And Shortcuts

Keyboard input routes on the focused path, not the pointer hit path. Preprocess runs first for global shortcuts, menu accelerators, modal filters, and command palettes. Preview tunnel then runs root-to-focused-node. Default bubble then runs focused-node-to-root.

Key events and text input are separate. Printable key events may be shortcut candidates; committed text goes to text/IME ownership. This prevents shortcuts and text editing from double-consuming the same native event.

Tab/backtab and mapped gamepad navigation should become `RequestNavigation` effects instead of hard-coded host focus changes.

### Text And IME

Text input requires an owner. The focused text-capable node can request input method activation with surrounding text, cursor position, anchor position, preedit text, preedit offset, cursor rectangle, input type, and clip rectangle. Incoming IME preedit/commit/cancel events are direct-routed to that owner.

When the owner is hidden, disabled, deleted, blurred, or moved out of the active popup/modal root, `UiSurface` clears the stale local IME owner and rejects stale owner dispatch. Explicit input-method disable requests are represented as `RequestInputMethod { request.kind = Disable, owner }` and are forwarded to the host only when the request owner still matches the active IME owner. Composition cancellation and commit must be explicit result states, not implicit string changes.

### Navigation And Analog/Gamepad

Navigation events are requests with policies. Required policies are explicit target, next/previous, directional, wrap, stop, escape, and custom boundary. Key and analog/gamepad inputs can map to navigation requests, but the route result should record both the original device event and the derived navigation request.

Analog input needs threshold and repeat gating so a held stick or d-pad does not flood focus changes. This state belongs in shared navigation/analog state, not editor host code.

### Drag And Drop

Drag/drop is a shared session:

1. Pointer down or handler reply can install `DetectDrag`.
2. Movement beyond threshold sends `OnDragDetected` semantics to the source.
3. A handled `BeginDragDrop` effect creates the session and captures pointer/drag ownership.
4. Move events calculate enter/leave/over against the current hit path and popup/modal roots.
5. Drop runs on pointer release over an accepting target.
6. Escape, capture loss, owner deletion, surface blur, or explicit reply cancels the session.

Existing component payload DTOs should be reused where possible. The shared route must own target enter/leave diffing so editor lists, asset fields, Material controls, and runtime UI use one drag/drop lifecycle.

### Popup, Menu, And Tooltip

Popup and menu state should live in shared surface input state. `OpenPopup` records anchor, popup surface or node, close policy, focus restoration target, and popup method. Outside pointer press, escape, owner deletion, and modal dismissal emit `ClosePopup` effects. Popup subtrees participate in hit testing ahead of normal roots according to stack order.

Tooltip is a timer-driven input feature. Hover starts a pending tooltip for an anchor. Moving away, pressing, scrolling, popup open, or surface blur cancels it. When the delay elapses, `ShowTooltip` opens a non-focus popup-like overlay with placement metadata. Tooltip content and placement belong in DTOs; timer scheduling can stay host-driven as long as the event is routed through shared dispatch.

### Pointer Capture, Lock, And High Precision

Capture routes future pointer move/up/cancel to the captured node. Lock constrains pointer movement to a node or rect. High precision requests raw/high-resolution movement and requires capture for the same owner and pointer. Releasing or replacing capture disables high precision only for the released or replaced captor, so a stale owner cannot clear another owner's high-precision state.

All three are effects applied by `UiSurface` and then translated by the host into platform APIs when available. The shared result must record unsupported-host warnings instead of silently ignoring lock or high-precision requests.

## Error Handling And Diagnostics

Every `UiDispatchResult` should explain route and effect outcomes. Required diagnostic categories:

- routed, ignored, handled, blocked, passthrough
- route phase that handled the event
- hit leaf, focus target, bubble route, and tunnel route
- focus changed, focus rejected, or focus restored
- capture began, capture kept, capture released, or capture rejected
- click emitted, double click emitted, click rejected, release outside, or drag consumed click
- IME owner set, updated, committed, cancelled, or lost
- drag session detected, began, entered, left, dropped, cancelled, or rejected
- popup opened, closed, dismissed outside, or focus restored
- tooltip pending, shown, hidden, or cancelled
- effect validation failure: missing node, hidden node, disabled node, stale route, wrong surface, missing payload, unsupported host capability
- damage/dirty request summary

Effect application should prefer structured errors in dispatch results over panics. Missing node, stale capture owner, deleted popup owner, or invalid drag payload should cancel the affected state and report a diagnostic.

## Integration Plan

### M5.0 Contract Inventory

- Inventory all current pointer/navigation dispatch DTOs and component event DTOs.
- Define `UiInputEvent`, `UiDispatchPhase`, `UiDispatchRoute`, `UiDispatchReply`, `UiDispatchEffect`, and generalized `UiDispatchResult` in `zircon_runtime_interface::ui::dispatch`.
- Add serde/default compatibility tests for the new DTOs and old pointer cases.
- Keep existing pointer/navigation types as specialized compatibility views only where required during migration; do not create new permanent parallel dispatcher families.

### M5.1 Pointer Migration To Reply/Effect

- Re-express current pointer handled/blocked/passthrough/captured behavior as `UiDispatchReply` and `UiDispatchEffect`.
- Preserve existing hover, press, release, click, capture, and scroll fallback tests.
- Add double-click, precise scroll x/y, pointer id, timestamp, modifiers, pointer lock, and high-precision effect coverage.

### M5.2 Keyboard, Text, And IME

- Add keyboard route resolution on focused path with preprocess, preview tunnel, and bubble phases.
- Add text commit and edit-intent events distinct from keyboard shortcuts.
- Add IME owner, input-method request effects, composition update/commit/cancel, and owner-loss cleanup.
- Add editor native `WindowEvent::KeyboardInput` and `WindowEvent::Ime` translation into shared input events.

### M5.3 Navigation And Analog/Gamepad

- Expand navigation reply/result to explicit, wrap, stop, escape, and target policies.
- Map key and analog/gamepad inputs to navigation requests through shared config data.
- Add analog threshold/repeat state and focused-route diagnostics.

### M5.4 Drag And Drop

- Add detect-drag threshold state and `BeginDragDrop`/`EndDragDrop`/`CancelDragDrop` effects.
- Route drag enter/leave/over/drop through shared hit path and popup/modal priority.
- Emit component events for drag begin, drag update, drag end, and drop.

### M5.5 Popup, Menu, And Tooltip

- Add popup stack state, close policies, outside dismissal, escape close, and focus restoration.
- Route popup subtree input ahead of normal roots.
- Add tooltip pending/visible state and timer event ingestion.

### M5.6 Editor And Runtime Host Cutover

- Translate editor `winit` keyboard, IME, precise wheel, pointer id, modifiers, and focus events into shared DTOs.
- Ensure existing Slint/control-level text callbacks do not double-dispatch when native text input is active.
- Route runtime UI manager input through the same `UiInputEvent` API.
- Retire host-only input semantics as each shared path lands. Manual hit-test adapters may remain only as temporary bridge code that produces `UiSurfaceFrame` queries or shared DTOs.

### M5.7 Documentation And Acceptance

- Update module docs under `docs/ui-and-layout` after implementation changes behavior.
- Add acceptance notes for editor/runtime same-semantics input coverage.
- Record commands, upstream references, and remaining gaps in the affected docs.

## Test Plan

### Interface Contract Tests

- `UiInputEvent` serializes every event family with default-safe optional metadata.
- `UiDispatchEffect` serializes every side effect and preserves stable reason codes.
- Old pointer route/result fixtures deserialize or migrate cleanly if touched.
- Missing optional fields use explicit defaults rather than panics.

### Runtime Routing Tests

- preview tunnel handled event prevents bubble handling.
- bubble handled event stops propagation at first handler.
- blocked target prevents fallback when policy says block.
- passthrough target allows next stacked pointer candidate.
- capture routes move/up outside bounds to captured target.
- capture release disables high precision.
- pointer lock request records unsupported-host diagnostic when no host capability is present.
- primary release inside emits one click; outside emits no click.
- double click emits only when target, button, time, and distance match.
- precise scroll preserves x/y and unit, then falls back to nearest scrollable candidate when unhandled.
- keyboard preprocess shortcut handles before focused bubble.
- keyboard preview tunnel can block focused key down.
- text commit reaches focused text owner and does not trigger shortcut handling.
- IME preedit update, commit, cancel, and owner loss produce explicit results.
- navigation explicit, wrap, stop, escape, next/previous, and directional policies update focus correctly.
- analog repeat gating prevents repeated focus moves before threshold timing.
- drag detect begins only after threshold, enters/leaves targets exactly once, drops on release, and cancels on escape or owner loss.
- popup outside press closes according to policy and restores focus.
- tooltip pending state shows after timer event and cancels on move/press/scroll/blur.

### Editor Host Tests

- native `KeyboardInput` translates into shared keyboard events.
- native `Ime::Preedit` and `Ime::Commit` translate into shared IME/text events.
- pixel wheel deltas preserve x/y precision instead of scalar y-only conversion.
- template text input does not double-dispatch native text and existing control-changed callbacks.
- menu, toolbar, pane template, and popup input use shared dispatch semantics.
- source-window focus context remains correct for floating or child windows.

### Boundary And Stress Tests

- hidden, collapsed, disabled, and hit-test-invisible nodes reject focus, pointer, drag, popup, and IME ownership as appropriate.
- deleting focused/captured/IME/drag/popup owner cancels or restores state deterministically.
- repeated hover move over the same target emits no redundant hover event.
- repeated key down uses repeat metadata and does not duplicate text commit.
- long drag over many targets does not leak enter/leave state.
- popup stack nesting closes in correct order.
- IME composition with empty preedit, long preedit, selection range, and cancellation is stable.
- high-frequency pointer move can be coalesced by host, but shared state remains correct for the final delivered event.

### Suggested Validation Commands

- `cargo test -p zircon_runtime_interface --lib ui --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture`
- `cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture`
- `cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture`
- `cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture`
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never`
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never`

## Acceptance Criteria

- Pointer, keyboard, text, IME, navigation, analog/gamepad, drag/drop, popup/menu, tooltip, double-click, capture, lock, and high-precision semantics are representable through shared `UiInputEvent`, `UiDispatchReply`, and `UiDispatchEffect` DTOs.
- `UiSurface` applies all input side effects centrally and reports structured diagnostics for success and rejection paths.
- Editor and runtime input hosts submit shared events and consume shared dispatch results; they do not own independent focus/capture/drag/popup/tooltip semantics.
- Existing pointer click, hover, capture, scroll, and navigation behavior remains covered after migration.
- New runtime and editor tests cover normal, boundary, failure, and stress cases for every event family in M5 scope.
- No new shared path checks concrete `control_id`, component type name, or event spelling to unlock one feature.
- Docs record Unreal and Slint evidence, implementation files, validation commands, and any deferred M6 text-rendering work.

## Implementation Evidence

Milestone 1 is implemented in `zircon_runtime_interface::ui::dispatch::input` and validated with focused interface contract commands. The final post-review contract gate passed `ui_input` tests with 2 passed, `contracts` tests with 35 passed, and `cargo check -p zircon_runtime_interface --lib` with no errors.

Milestone 2 is implemented in `zircon_runtime::ui::surface::input`. `UiSurfaceInputState` tracks captured pointer id, high-precision owner, pointer-lock owner/policy, input-method owner, and input-method request geometry. `validation.rs` provides the shared valid-owner predicate across effect application and owner-routed dispatch, requiring an existing enabled owner with render-visible self and ancestor chain. `UiSurface::apply_dispatch_reply(...)` consumes ordered shared effects and records applied/rejected effects by `effect_index`; input-method enable/reset/update/disable is encoded through `RequestInputMethod`, returned as a typed host request only when its owner is valid and, for reset/update/disable, still matches current surface IME state. Owner-clearing effects carry explicit targets so stale replies cannot clear another owner: focus clear targets the focused node, pointer capture release targets the current captor plus pointer id, pointer unlock targets the current lock owner, and high-precision disable targets the current high-precision owner. Focus changes clear stale IME ownership only after the focus move succeeds, focus/capture validate hidden ancestors through the same owner predicate, navigation uses the same focus path, direct capture clears stale pointer ids, high-precision enable requires live capture for the same owner, and capture release/transfer clears high precision only for the matching released or replaced captor. `UiSurface::dispatch_input_event(...)` delegates pointer/navigation to existing dispatchers, preserves precise pointer scroll x/y/unit metadata on the shared result event, routes keyboard through focus diagnostics, routes text through a valid IME owner or focus after stale-owner cleanup, rejects invalid IME owners with diagnostics, and clears IME owner on cancel.

The Milestone 2 runtime gate passed on 2026-05-06 with existing warning noise only. After owner-safety final fixes, `cargo test -p zircon_runtime --lib runtime_input_ownership ...` reported 7 passed, 0 failed; `cargo test -p zircon_runtime --lib event_routing ...` reported 20 passed, 0 failed; `cargo test -p zircon_runtime --lib shared_core ...` reported 38 passed, 0 failed; `cargo check -p zircon_runtime --lib ...` passed. Earlier owner-cutover interface gates also passed: `cargo test -p zircon_runtime_interface --lib contracts ...` reported 49 passed, 0 failed, and interface `cargo check --lib` completed successfully. Editor-native translation remains Milestone 3 and was not claimed by this evidence.

## Risks

- The implementation touches active UI areas already owned by sibling sessions. Mitigation: stage work through interface contracts and `UiSurface` first, coordinate before editing hit-test, painter, invalidation, widget behavior, or native text files.
- Adding complete event state at once can bloat `surface.rs`. Mitigation: keep root modules structural and split family state/effect application into folder-backed modules when files approach mixed-responsibility size.
- Native editor text callbacks can double-dispatch with new keyboard/text/IME events. Mitigation: separate committed text/IME ownership from existing `*_control_changed` callbacks and add explicit editor tests.
- Host support for pointer lock and high precision may differ by platform. Mitigation: shared effects report unsupported-host diagnostics and keep state consistent even when the platform cannot fulfill the request.
- Full editable text behavior depends on M6. Mitigation: M5 emits stable text/IME intents and owner state, while M6 consumes them for caret, selection, shaping, and painting.

## Out Of Scope For Implementation Plan

- Full text shaping, BiDi, rich text, caret painting, and selection painting.
- GPU-level overdraw or input visual debugger work from M7.
- World-space UI raycasting and 3D widget hit mapping beyond accepting already-mapped virtual pointer events.
- Long-term removal of every existing editor manual hit-test adapter in one step; M5 should retire paths only after equivalent shared surface-frame routes exist.
