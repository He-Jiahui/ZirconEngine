---
related_code:
  - zircon_editor/src/ui/binding/animation/mod.rs
  - zircon_editor/src/ui/binding/animation/command.rs
  - zircon_editor/src/ui/binding/animation/codec.rs
  - zircon_editor/src/ui/binding/core/payload.rs
  - zircon_editor/src/ui/binding/core/payload_codec.rs
  - zircon_editor/src/ui/binding/core/payload_constructors.rs
  - zircon_editor/src/ui/binding_dispatch/animation/animation_host_event.rs
  - zircon_editor/src/ui/binding_dispatch/animation/dispatch.rs
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/core/editor_event/runtime/binding_normalization.rs
  - zircon_editor/src/core/editor_event/runtime/execution/animation_event.rs
  - zircon_editor/src/core/editor_event/runtime/execution/asset_event.rs
  - zircon_editor/src/ui/animation_editor/mod.rs
  - zircon_editor/src/ui/animation_editor/presentation.rs
  - zircon_editor/src/ui/animation_editor/session.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/mod.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/lifecycle.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/sync.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/editing.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/animation_sequence_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/animation_graph_view_descriptor.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/floating_windows.rs
  - zircon_editor/ui/workbench/animation_editor_pane.slint
  - zircon_editor/ui/workbench/pane_surface.slint
  - zircon_editor/src/tests/ui/binding/mod.rs
  - zircon_editor/src/tests/ui/binding/animation.rs
  - zircon_editor/src/tests/host/binding_dispatch.rs
  - zircon_editor/src/tests/editor_event/runtime.rs
  - zircon_editor/src/tests/editor_event/animation_runtime.rs
  - zircon_editor/src/tests/host/animation_editor.rs
  - zircon_editor/src/tests/host/manager/mod.rs
  - zircon_editor/tests/workbench_animation_editor_shell.rs
implementation_files:
  - zircon_editor/src/ui/binding/animation/mod.rs
  - zircon_editor/src/ui/binding/animation/command.rs
  - zircon_editor/src/ui/binding/animation/codec.rs
  - zircon_editor/src/ui/binding/core/payload.rs
  - zircon_editor/src/ui/binding/core/payload_codec.rs
  - zircon_editor/src/ui/binding/core/payload_constructors.rs
  - zircon_editor/src/ui/binding_dispatch/animation/animation_host_event.rs
  - zircon_editor/src/ui/binding_dispatch/animation/dispatch.rs
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/core/editor_event/runtime/binding_normalization.rs
  - zircon_editor/src/core/editor_event/runtime/execution/animation_event.rs
  - zircon_editor/src/core/editor_event/runtime/execution/asset_event.rs
  - zircon_editor/src/ui/animation_editor/mod.rs
  - zircon_editor/src/ui/animation_editor/presentation.rs
  - zircon_editor/src/ui/animation_editor/session.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/mod.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/lifecycle.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/sync.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/editing.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/animation_sequence_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/animation_graph_view_descriptor.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/floating_windows.rs
  - zircon_editor/ui/workbench/animation_editor_pane.slint
  - zircon_editor/ui/workbench/pane_surface.slint
plan_sources:
  - user: 2026-04-20 继续正在runtime/editor/framework实现完整的物理和动画系统
  - user: 2026-04-20 PLEASE IMPLEMENT THIS PLAN
  - .codex/plans/Physics  Full Animation Support Plan.md
  - .codex/plans/Physics + Full Animation Support 新计划.md
tests:
  - zircon_editor/src/tests/ui/binding/animation.rs
  - zircon_editor/src/tests/host/binding_dispatch.rs
  - zircon_editor/src/tests/editor_event/runtime.rs
  - zircon_editor/src/tests/editor_event/animation_runtime.rs
  - zircon_editor/src/ui/animation_editor/session/tests.rs
  - zircon_editor/src/tests/host/animation_editor.rs
  - zircon_editor/src/tests/host/manager/mod.rs
  - zircon_editor/tests/workbench_animation_editor_shell.rs
  - cargo test -p zircon_editor animation_command_bindings_roundtrip_for_timeline_graph_and_state_machine_authoring --lib
  - cargo test -p zircon_editor animation_timeline_graph_and_state_machine_bindings_dispatch_into_host_events --lib
  - cargo test -p zircon_editor animation_graph_and_state_machine_bindings_normalize_into_editor_events --lib
  - cargo test -p zircon_editor animation_editor --locked
  - cargo test -p zircon_editor --locked tests::editor_event::animation_runtime:: -- --nocapture --test-threads=1
doc_type: module-detail
---

# Animation Binding Command Surface

## Purpose

`zircon_editor` 的动画 authoring binding 不再停留在少量 timeline 特例 payload。当前已经收束成独立的 `AnimationCommand` 域，并把 sequence、graph、state-machine authoring 一起放进同一条 typed command surface，保证 editor UI、binding dispatch、editor event runtime 使用同一组命令和路径模型。

## Ownership

当前命令链分成三层：

- `ui::binding::animation`
  - `AnimationCommand`
  - binding codec
- `ui::binding::core`
  - `EditorUiBindingPayload::AnimationCommand(...)`
  - native binding roundtrip
- `ui::binding_dispatch::animation`
  - `AnimationHostEvent`
  - shared `AnimationTrackPath` 解析与 host dispatch

`EditorEventRuntime` 现在已经把 `AnimationCommand` 归一化成正式 `EditorAnimationEvent`，并通过统一的 runtime journal / undo-policy / status-line 链执行。也就是说，这一层已经不再停留在 host dispatch，而是正式进入 editor 事件主链。

## Command Set

当前 `AnimationCommand` 已按 authoring 领域覆盖三类操作：

- timeline / track
  - `AddKey { track_path, frame }`
  - `RemoveKey { track_path, frame }`
  - `CreateTrack { track_path }`
  - `RemoveTrack { track_path }`
  - `RebindTrack { from_track_path, to_track_path }`
  - `ScrubTimeline { frame }`
  - `SetTimelineRange { start_frame, end_frame }`
  - `SelectTimelineSpan { track_path, start_frame, end_frame }`
  - `SetPlayback { playing, looping, speed }`
- graph
  - `AddGraphNode { graph_path, node_id, node_kind }`
  - `RemoveGraphNode { graph_path, node_id }`
  - `ConnectGraphNodes { graph_path, from_node_id, to_node_id }`
  - `DisconnectGraphNodes { graph_path, from_node_id, to_node_id }`
  - `SetGraphParameter { graph_path, parameter_name, value_literal }`
- state machine
  - `CreateState { state_machine_path, state_name, graph_path }`
  - `RemoveState { state_machine_path, state_name }`
  - `SetEntryState { state_machine_path, state_name }`
  - `CreateTransition { state_machine_path, from_state, to_state, duration_frames }`
  - `RemoveTransition { state_machine_path, from_state, to_state }`
  - `SetTransitionCondition { state_machine_path, from_state, to_state, parameter_name, operator, value_literal }`

这里所有 timeline track targeting 都强制走 canonical `track_path` 字符串，后续统一解析成 `zircon_runtime::core::framework::animation::AnimationTrackPath`。graph 和 state-machine 资产则继续使用字符串 `graph_path` / `state_machine_path` 指向 editor 当前资产域中的目标对象。

## Native Binding Surface

当前稳定的 native binding symbol 前缀是：

- `AnimationCommand.AddKey`
- `AnimationCommand.RemoveKey`
- `AnimationCommand.CreateTrack`
- `AnimationCommand.RemoveTrack`
- `AnimationCommand.RebindTrack`
- `AnimationCommand.ScrubTimeline`
- `AnimationCommand.SetTimelineRange`
- `AnimationCommand.SelectTimelineSpan`
- `AnimationCommand.SetPlayback`
- `AnimationCommand.AddGraphNode`
- `AnimationCommand.RemoveGraphNode`
- `AnimationCommand.ConnectGraphNodes`
- `AnimationCommand.DisconnectGraphNodes`
- `AnimationCommand.SetGraphParameter`
- `AnimationCommand.CreateState`
- `AnimationCommand.RemoveState`
- `AnimationCommand.SetEntryState`
- `AnimationCommand.CreateTransition`
- `AnimationCommand.RemoveTransition`
- `AnimationCommand.SetTransitionCondition`

这意味着 editor UI 层现在可以用和 `AssetCommand / DockCommand / ViewportCommand` 相同的命令化结构表达动画操作，而不是继续引入 animation-only 的单独字符串格式。

## Dispatch Contract

`dispatch_animation_binding(...)` 当前做两类规范化：

- 把 `AnimationCommand` 规范化成 `AnimationHostEvent`
- 对 timeline / track 命令统一调用 `AnimationTrackPath::parse(...)`

因此 host 侧收到的是 typed 事件：

- `AddKey`
- `RemoveKey`
- `CreateTrack`
- `RemoveTrack`
- `RebindTrack`
- `ScrubTimeline`
- `SetTimelineRange`
- `SelectTimelineSpan`
- `SetPlayback`
- `AddGraphNode`
- `RemoveGraphNode`
- `ConnectGraphNodes`
- `DisconnectGraphNodes`
- `SetGraphParameter`
- `CreateState`
- `RemoveState`
- `SetEntryState`
- `CreateTransition`
- `RemoveTransition`
- `SetTransitionCondition`

track path 无效时，dispatch 统一返回 `EditorBindingDispatchError::InvalidAnimationTrackPath(...)`。graph / state-machine 命令当前不做额外 schema 验证，而是把路径和值字面量保留到 editor event runtime。

## Runtime Event Chain

当前完整链路已经固定为：

- `AnimationCommand`
- `AnimationHostEvent`
- `EditorAnimationEvent`
- `EditorEventRuntime` execution

`binding_normalization` 负责把 typed animation binding 转成 `EditorAnimationEvent`，`execution::animation_event` 负责把 timeline / track / graph / state-machine 操作投影到 editor runtime 状态和 event journal。这样 animation binding 不再作为 unsupported payload 被丢弃。

## Current Execution Boundary

当前执行层已经不只是“命令进主链”，而是有了真实的 animation editor session owner：

- `ui::animation_editor::AnimationEditorSession` 现在直接从 `.sequence.zranim` / `.graph.zranim` / `.state_machine.zranim` 读资产，并维护 sequence timeline、graph parameter/node 摘要、state-machine state/transition 摘要
- `ui::host::animation_editor_sessions` 负责 workspace 恢复、按 `serializable_payload["path"]` 惰性回补会话、host dirty/title/payload 同步，以及 command 到 session 的真实变更调用
- `execution::animation_event` 不再只是写状态栏；当存在兼容的 animation editor target 时，它会真正修改当前 session，并继续标记 `PresentationChanged` / `ReflectionChanged`
- 这让 sequence editor 与 graph editor 不只是拿到稳定命令 ABI，也拿到了可以驱动 pane 内容变化的真实 runtime session

同时保留一个刻意的降级边界：

- 如果动画 binding 在没有兼容活动 animation editor 页签的情况下被触发，runtime execution 会把它视为受控 no-op，同时仍产出统一的 `EditorAnimationEvent`、status line 和 presentation/reflection effects
- 如果命令成功进入 animation session，但底层 session 判断这次 authoring 没有造成真实变更，`execution::animation_event` 现在同样会把状态栏写成 ignored，而不是伪装成成功写入
- 这样旧的 headless binding 归一化测试和 stray UI dispatch 不会重新退回 hard error

当前仍然没有完整交付的部分是：

- 真实资产写回与增量引用分析
- graph/state-machine 更深的 schema 验证
- 复杂 undo inverse 与跨资产 authoring 事务

## Session Integrity Rules

命令面虽然保持统一，但当前 session 层已经补了几条必要的一致性规则，避免 UI 和资产摘要脱钩：

- `RemoveGraphNode { node_id: "output" }` 现在会真实删除 graph output 节点，而不是因为 output 节点没有常规 `id` 字段而落成 silent no-op
- sequence editor 删除当前已选中的 track 时，会同步清掉 timeline `selected_span`，不再留下悬空的选区摘要
- sequence editor 成功 rebind 当前已选中的 track 时，会把 `selected_span` 迁移到目标 track path，而不是继续指向已经失效的旧路径
- sequence editor 对不存在的 track 执行 `SelectTimelineSpan` 时现在返回 `Ok(false)`，不会制造 phantom selection
- sequence editor 对 `SelectTimelineSpan` 的越界端点现在会 clamp 到当前 timeline range，而不是把不可见帧区间直接写入 `selected_span`
- sequence editor 在 `SetTimelineRange` 收缩可见区间时，也会同步把已有 `selected_span` clamp 回新范围内，避免 pane 摘要继续显示越界帧
- sequence editor 对 `SetPlayback` 的非有限速度输入现在也返回 `Ok(false)`，不会把播放标签静默污染成 `speed=NaN` 或 `speed=inf`
- graph editor 对 `SetGraphParameter` 的 `NaN` / `Inf` 标量和带非有限分量的 vector literal 现在也统一返回 `Ok(false)`，不会把 typed default 悄悄改写成无效值
- state-machine editor 对 `SetTransitionCondition` 的 `NaN` / `Inf` 标量和带非有限分量的 vector literal 现在也统一返回 `Ok(false)`，不会把现有条件值偷偷污染成无效数据
- graph/state-machine 的若干 no-op authoring（重复 output、缺失 source、自环 graph connection、未知 node kind、缺失 state/transition、未知 transition operator、无效 parameter literal / condition literal 等）现在统一返回 `Ok(false)`，再由事件执行层投影成 ignored status line

这些规则保证 animation command surface 不只是“能发命令”，而是能把 editor pane 保持在与当前文档一致的状态。

## Asset Routing

动画 authoring 现在还补齐了 workbench 侧的资产打开路由：

- `*.sequence.zranim` 打开到 `editor.animation_sequence`
- `*.graph.zranim` 与 `*.state_machine.zranim` 打开到 `editor.animation_graph`

因此 sequence editor 与 graph/state-machine editor 已经有正式 descriptor，不再借用 `editor.ui_asset` 或退回状态栏提示。

## Pane Surface

workbench 壳现在也不再把 animation 资产页签渲染成 fallback pane：

- `pane_surface.slint` 增加了 animation branch
- `animation_editor_pane.slint` 定义了 sequence / graph / state-machine 三种 pane 数据视图
- `pane_projection`、`shell_presentation`、`floating_windows`、`apply_presentation` 与 `host_lifecycle` 把 `AnimationEditorPanePresentation` 一路投影到 Slint host

因此 `.sequence.zranim` 和 `.graph/.state_machine.zranim` 不只是 descriptor 路由正确，workbench 中心页签和浮动窗口也都能展示真实 animation pane 内容，而不是通用 empty/fallback 提示。

## Why This Matters

这一层现在已经把 sequence/graph/state-machine authoring 所需要的核心边界补齐了：

- inspector 选中属性后可以直接产生 canonical track path
- UI roundtrip、headless binding、host dispatch 都共享一组 command symbol
- animation binding 已经进入正式 runtime event journal
- 动画资产可以打开到稳定的 sequence / graph workbench view
- editor 不需要为 timeline、graph、state-machine authoring 再发明平行 payload 或 ad hoc callback ABI

这正是后续继续把 timeline、graph、state-machine authoring 往同一条 editor/runtime 事件面收拢的前提。
