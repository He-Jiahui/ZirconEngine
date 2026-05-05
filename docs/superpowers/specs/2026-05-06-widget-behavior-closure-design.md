---
related_code:
  - zircon_runtime_interface/src/ui/component/descriptor/component_descriptor.rs
  - zircon_runtime_interface/src/ui/component/event.rs
  - zircon_runtime_interface/src/ui/component/state.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/result.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs
  - zircon_runtime_interface/src/ui/surface/pointer/route.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs
  - zircon_runtime/src/ui/component/state_reducer.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
implementation_files:
  - zircon_runtime_interface/src/ui/component/descriptor/component_descriptor.rs
  - zircon_runtime_interface/src/ui/component/event.rs
  - zircon_runtime_interface/src/ui/component/state.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/result.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs
  - zircon_runtime_interface/src/ui/surface/pointer/route.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs
  - zircon_runtime/src/ui/component/state_reducer.rs
plan_sources:
  - user: 2026-05-06 集中完善 widget 组件行为闭环，参照 dev 下虚化源码
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - .codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md
  - .codex/plans/Shared Slate-Style UI Layout, Render, And Hit Framework.md
tests:
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - .github/workflows/ci.yml
doc_type: design-spec
---

# Widget Behavior Closure Design

## Summary

本设计把本轮 UI 工作限定为 **共享 widget/component 事件闭环**。目标不是一次性完成全部 Material 视觉、文本编辑、popup、drag/drop 或 editor host cutover，而是先把 Slate 风格的 widget path、reply/effect、focus/capture、默认语义事件和诊断闭环落到 `zircon_runtime_interface::ui` 与 `zircon_runtime::ui` 共享层。

本轮以 `UiSurfaceFrame` 为唯一空间事实：layout 产出的 arranged tree 和 hit grid 决定 pointer path；router 根据 path、focus、capture、pressed、hover 和 descriptor capability 产出一次性 dispatch reply/effect、component event envelope、dirty/damage 建议和诊断。Editor/Material 控件后续只消费这个共享语义，不新增控件名专用坐标表或 host 点击分支。

## Goals

- 建立 Slate `WidgetPath + FReply` 对等语义，但使用 Zircon 自己的 DTO 命名和 retained `.ui.toml` 数据流。
- 让 pointer down/move/up/scroll 在共享层闭环：route、dispatch、state transition、component event、diagnostic、dirty/damage 建议。
- 让 widget 默认行为由 descriptor/capability/binding event kind 驱动，禁止靠 `control_id` 或 component 名字符串特判。
- 保持 `UiSurfaceFrame.arranged_tree + hit_grid` 是 hit、render、input 的共同事实。
- 用 runtime tests 先证明行为闭环，再让 editor native host 和 Material `.ui.toml` 控件逐步接入。

## Non-Goals

- 不在本轮恢复生成式 Slint UI，也不把 `.slint` 文件重新作为行为真源。
- 不做大面积 painter、SVG/image、FPS、Asset Browser 响应式布局或 Material 视觉 token 改造；这些属于当前活跃兄弟会话。
- 不实现完整 IME、富文本编辑、shape-aware hit test、multi-window/multi-user pointer capture 或 world-space UI 射线拾取。
- 不在 editor host 增加 `Button`、`Slider`、`ComboBox`、`Toolbar` 等控件专用 Rust dispatch 分支。
- 不用兼容 shim 保留旧 host 坐标路径；若本轮触及旧路径，应收束到共享 surface/hit adapter。

## Current Evidence

Zircon 当前工作区已经具备本轮需要的基础，但行为链条仍不完整：

- `UiSurfaceFrame` 已包含 arranged tree、render extract、hit grid 和 focus snapshot。
- `UiPointerRoute` 已记录 target、hit path、stacked route、hover enter/leave、capture、pressed、click target、root fallback 和 activation phase。
- `UiPointerDispatchResult` 已支持 handled、blocked、passthrough、captured 和 component event envelopes。
- `UiComponentEvent` 与 `UiComponentState` 已覆盖 hover、press、focus、commit、value change、popup、selection、drag/drop、virtual list 和 world surface 等 retained state 语义。
- `UiSurface::dispatch_pointer_event(...)` 已能路由 pointer、处理 capture、scroll fallback、focus/capture diagnostics，并生成部分 component event envelope。
- 现有缺口是：reply/effect 语义还不够接近 Slate 的一次性命令缓冲；focus/blur、release capture、dirty/damage 建议、descriptor-driven default action、toggle/select/open-popup 等默认行为还没有统一闭环。

工作区当前有大量 UI dirty/untracked 文件。本设计把这些视为活跃 baseline，不要求回滚，也不把 Git 提交态当作唯一事实。

## Reference Alignment

Unreal Slate 提供行为主线：

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

Transferable semantics:

- Widget 是 retained object，但事件上下文是 transient path、geometry、clip 和 hit order。
- Event route 通过 root-to-leaf path tunnel 或 leaf-to-root bubble，handled reply 终止传播。
- Reply 是一次性命令缓冲，包含 handled 状态和副作用，例如 capture、release capture、focus、drag detection、cursor/lock 等。
- Focus/capture 是 application/router 状态，不应该由 widget 永久持有 reply 对象。
- Hit grid 来源于 arranged/painted geometry，并输出 widget path，而不是每个控件维护自己的坐标表。

Slint Material 提供控件状态语义参考：

- `dev/slint/ui-libraries/material/src/ui/components/base_button.slint`
- `dev/slint/ui-libraries/material/src/ui/components/state_layer.slint`
- `dev/slint/ui-libraries/material/src/ui/components/text_field.slint`
- `dev/slint/ui-libraries/material/src/ui/components/menu.slint`
- `dev/slint/ui-libraries/material/src/ui/components/dialog.slint`
- `dev/slint/ui-libraries/material/src/ui/components/base_navigation.slint`

Transferable semantics:

- Material 状态以 `enabled/disabled`、`hovered`、`pressed`、`focused`、`selected`、`checked`、`popup_open` 等显式状态驱动。
- Button family 共享行为，视觉 variant 不应该改变事件语义。
- Menu/list/navigation 通过 selected/current index 和 row activation 表达状态，不应在 host 坐标层编码。

## Ownership

- `zircon_runtime_interface::ui` owns neutral DTO contracts: component descriptor/capability, component events, pointer dispatch reply/effects, pointer route, surface frame, diagnostics, binding event kind.
- `zircon_runtime::ui` owns shared behavior: hit path routing, hover diff, focus/capture/pressed transitions, default activation, reducer application, scroll fallback, dirty/damage suggestions, focused runtime tests.
- `zircon_editor::ui` remains host/authoring adapter: normalize native input, submit events to shared router, dispatch typed envelopes to editor command/adapters, and request repaint from returned damage.
- `.ui.toml` component assets describe public controls, bindings, slots, variants, and capability metadata. They do not own the shared pointer/focus/capture semantics.

## Design

### 1. Dispatch Reply And Effects

Introduce or consolidate a Slate-like `UiDispatchReply` shape around the existing pointer dispatch result model. It should remain a transient return value consumed once by `UiSurface::dispatch_pointer_event(...)`.

Required effects for this slice:

- `Handled(node_id)` records the first semantic handler.
- `Blocked(node_id)` blocks current candidate and allows stacked fallback where current behavior already supports it.
- `Passthrough(node_id)` records observation while continuing route.
- `CapturePointer(node_id)` captures future move/up to a node.
- `ReleasePointerCapture(node_id)` releases capture if the current node owns it.
- `SetFocus(node_id, focus_visible)` updates focus and navigation root.
- `ClearFocus(reason)` clears focus on surface blur, hidden/disabled target, or explicit reply.
- `RequestDirty { layout, render, hit_test, input }` reports semantic state changes that require surface rebuild layers.
- `RequestDamage(rect_or_node_id)` reports old/new frames that need repaint.

The reply must not become durable widget state. Durable state remains in `UiFocusState`, `UiNavigationState`, `UiTreeNode`, and `UiComponentState`.

### 2. Pointer Route To Semantic Event

Shared router behavior should follow this sequence:

1. Normalize pointer input into `UiPointerEvent`.
2. Resolve route using capture first, then `UiSurfaceFrame.hit_grid`/arranged hit path.
3. Compare previous and current hover path; emit hover enter/leave only when the path changes.
4. On primary down, record pressed target, optionally focus deepest focusable target, and emit `Press { pressed: true }` for matching bindings.
5. On move, avoid dispatching same-target hover churn; captured widgets may still receive move deltas later when drag/value controls are implemented.
6. On primary up, emit `Press { pressed: false }`, release capture, and emit default click/commit only when pressed target equals release-inside target and no drag consumed the gesture.
7. On scroll, give dispatch handlers first chance; if unhandled, bubble to nearest scrollable candidate.
8. Apply reply effects once, then return component envelopes, diagnostics, and dirty/damage hints.

### 3. Descriptor-Driven Defaults

Default behavior must read metadata, not names. The contract should make room for these fields, whether implemented as new descriptor fields or existing capability data:

- focusability: none, pointer focus, keyboard focus, both.
- activation policy: primary release inside, primary press, keyboard activate, manual only.
- capture policy: none, press-to-capture, drag-threshold capture, explicit reply only.
- value property: `checked`, `selected`, `value`, `current_index`, or absent.
- commit policy: on click, on release, on blur, on enter, on drag end.
- supported binding event kinds.

This slice should implement only the minimum default action set needed for closed behavior:

- Button-like command: release-inside click emits `Commit { property: "activated", value: true }` for `Click` bindings.
- Pressable controls: down/up emit `Press` when matching bindings exist.
- Hoverable controls: enter/leave emit `Hover` when matching bindings exist.
- Focusable controls: focus changes emit `Focus { focused }` when matching bindings exist.

Toggle/select/menu/slider/text edit can be represented in the descriptor surface but may remain follow-up behavior unless needed to make runtime tests coherent.

### 4. Component State Application

The router should expose two layers of output:

- envelope output for editor/runtime adapters and binding routes,
- optional retained-state reducer application for surfaces that own component state directly.

For this slice, the minimum requirement is that every semantic event can be represented as a `UiComponentEventEnvelope` with document id, control id, node id, binding id, event kind, reason, and typed event. If existing code already applies events through `state_reducer.rs`, the plan may wire focused cases into that reducer, but editor adapters must not depend on reducer mutation to receive envelopes.

### 5. Diagnostics And Damage

Each dispatch result should explain what happened without ad hoc logs:

- routed or ignored,
- hit leaf and bubble route,
- handled/blocked/passthrough/captured node,
- focus changed,
- capture began or ended,
- hover entered/left counts,
- same-target hover ignored,
- component envelope count,
- default click emitted or rejected,
- rejection reason such as disabled, hidden, clipped, unsupported event, missing binding, release outside, lost capture.

Dirty/damage suggestions should distinguish layout, render, hit-test, input-only, and pointer-only repaint. Hover/press/focus changes should prefer old/new node-frame damage, not full presentation rebuild.

## Milestones

### M1 Contract Closure

- Audit current `UiPointerDispatchResult`, `UiPointerEffect`, `UiPointerComponentEvent`, `UiPointerRoute`, `UiComponentEvent`, and descriptor capability fields.
- Add missing reply/effect fields only where current types cannot express required semantics.
- Keep root modules structural and avoid broad module reorganization unless a touched file becomes mixed-responsibility.
- Add focused contract tests in `zircon_runtime_interface` if DTO shape changes require it.

### M2 Router Behavior Closure

- Update `UiSurface::dispatch_pointer_event(...)` and pointer dispatcher integration to consume reply/effects once.
- Implement focus/blur envelopes, release capture reply, same-target hover idle diagnostics, release-inside default click, and scroll fallback diagnostics.
- Keep behavior in `zircon_runtime::ui`, not editor host.
- Add runtime tests in `zircon_runtime/src/ui/tests/event_routing.rs` and adjacent hit-grid/shared-core tests.

### M3 Descriptor Defaults And Binding Coverage

- Ensure `.ui.toml` expanded root controls carry component, control id, bindings, and supported event metadata.
- Expand default interactive recognition from the current narrow button/text-field set only through descriptor/capability data.
- Add tests proving a new bound component does not need host coordinate or component-name dispatch branches.

### M4 Docs And Acceptance

- Update `docs/ui-and-layout/shared-ui-core-foundation.md` or create a focused module doc if the implementation changes code behavior materially.
- Record reference files, implementation files, tests, and validation commands in docs headers.
- Add or update acceptance notes under `tests/acceptance` if this slice changes milestone evidence.

## Test Plan

Implementation should write tests during the milestone and run them during the testing stage, following the repository milestone-first policy.

Focused runtime tests:

- primary release inside pressed target emits one default click/commit and clears press state,
- primary release outside pressed target emits no click and reports release-outside diagnostic,
- capture routes move/up to captured target but click still requires physical release-inside,
- release capture reply clears capture and reports capture release,
- focusable primary down sets focus and emits focus envelope; blur/clear focus emits blur envelope,
- same-target move after hover enter emits no redundant hover envelope and reports idle diagnostic,
- multiple matching bindings on a target produce ordered envelopes,
- scroll falls back to nearest scrollable candidate when unhandled,
- disabled/hidden/clipped/input-ignored nodes do not receive default focus or click.

Suggested commands for the testing stage:

- `cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\widget-behavior-closure --message-format short --color never -- --nocapture`
- `cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir E:\zircon-build\targets\widget-behavior-closure --message-format short --color never -- --nocapture`
- `cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\widget-behavior-closure --message-format short --color never -- --nocapture`
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\widget-behavior-closure --message-format short --color never`

If shared interface DTOs change, add:

- `cargo test -p zircon_runtime_interface --lib ui --locked --jobs 1 --target-dir E:\zircon-build\targets\widget-behavior-closure --message-format short --color never -- --nocapture`
- `cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\widget-behavior-closure --message-format short --color never`

## Acceptance Criteria

- Widget behavior for pointer hover, press, focus, capture, release-inside click, and scroll fallback is closed in shared runtime code.
- Dispatch replies are transient and consumed once by the surface/router.
- Component envelopes carry enough typed data for editor/runtime adapters without host coordinate guesses.
- Default behavior comes from descriptor/capability/binding metadata, not component-name or control-id string special cases.
- Runtime tests cover positive, negative, boundary, and diagnostic cases.
- Docs record reference sources, implementation files, tests, and remaining gaps.

## Remaining Follow-Up After This Slice

- Full keyboard/text/IME route and editable text state machine.
- Drag threshold, drag/drop operation lifetime, accepted/rejected payload diagnostics.
- Popup/menu/dialog surface registry and outside-click close semantics.
- Multi-pointer, multi-user, multi-window capture/focus.
- Editor native host full cutover for all surfaces once shared behavior is proven.
- Widget Reflector-style live/snapshot event-path and invalidation debugging.
