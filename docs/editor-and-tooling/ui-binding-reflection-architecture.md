---
related_code:
  - zircon_ui/src/lib.rs
  - zircon_editor_ui/src/binding.rs
  - zircon_editor_ui/src/control.rs
  - zircon_editor_ui/src/reflection.rs
  - zircon_editor/src/editor_event/mod.rs
  - zircon_editor/src/editor_event/types.rs
  - zircon_editor/src/editor_event/runtime.rs
  - zircon_editor/src/editor_event/journal.rs
  - zircon_editor/src/editor_event/replay.rs
  - zircon_editor/src/editor_event/transient.rs
  - zircon_editor/src/editor_event/host_adapter.rs
  - zircon_editor/src/host/binding_dispatch.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/host/slint_host/callback_dispatch.rs
  - zircon_editor/src/host/slint_host/drawer_resize.rs
  - zircon_editor/src/host/slint_host/event_bridge.rs
  - zircon_editor/src/host/slint_host/tab_drag.rs
  - zircon_editor/src/host/slint_host/ui.rs
  - zircon_editor/src/host/manager.rs
  - zircon_editor/src/workbench/event.rs
  - zircon_editor/src/workbench/model.rs
  - zircon_editor/src/workbench/project.rs
  - zircon_editor/src/workbench/reflection.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/panes.slint
implementation_files:
  - zircon_editor_ui/src/binding.rs
  - zircon_editor_ui/src/control.rs
  - zircon_editor/src/editor_event/mod.rs
  - zircon_editor/src/editor_event/types.rs
  - zircon_editor/src/editor_event/runtime.rs
  - zircon_editor/src/editor_event/journal.rs
  - zircon_editor/src/editor_event/replay.rs
  - zircon_editor/src/editor_event/transient.rs
  - zircon_editor/src/editor_event/host_adapter.rs
  - zircon_editor/src/host/binding_dispatch.rs
  - zircon_editor/src/host/slint_host/callback_dispatch.rs
  - zircon_editor/src/host/slint_host/event_bridge.rs
  - zircon_editor/src/host/slint_host/drawer_resize.rs
  - zircon_editor/src/host/slint_host/tab_drag.rs
  - zircon_editor/src/host/manager.rs
  - zircon_editor/src/workbench/event.rs
  - zircon_editor/src/workbench/model.rs
  - zircon_editor/src/workbench/project.rs
  - zircon_editor/src/workbench/reflection.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/panes.slint
plan_sources:
  - user: 2026-04-13 收束 JetBrains Hybrid Shell 的 UI 事件、反射和宿主契约
  - user: 2026-04-14 UI事件系统不应该直接和slint耦合，而是独立一套调度和绑定系统
  - .codex/plans/Editor Event Decoupling And Replay Plan.md
  - .codex/plans/Zircon UI Editor UI Binding & Reflection Architecture.md
tests:
  - zircon_editor_ui/src/tests/binding.rs
  - zircon_editor/src/tests/host/binding_dispatch.rs
  - zircon_editor/src/tests/editor_event/runtime.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch.rs
  - zircon_editor/src/tests/host/slint_event_bridge.rs
  - zircon_editor/src/tests/host/manager.rs
  - zircon_editor/src/tests/host/slint_drawer_resize.rs
  - zircon_editor/src/tests/host/slint_tab_drag.rs
  - zircon_editor/src/tests/workbench/host_events.rs
  - zircon_editor/src/tests/workbench/reflection.rs
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_editor_ui --lib --locked
  - cargo test -p zircon_ui --lib --locked
  - cargo test --workspace --locked
doc_type: module-detail
---

# UI Binding And Reflection Architecture

## Purpose

`nativeBinding`、headless 测试、反射树远控、真实宿主点击，现在都必须汇到同一套 editor shell 协议。  
这份文档只定三件事：

- 哪些 payload 是稳定 editor UI 协议
- 哪些 shell chrome 命名空间是稳定的
- 每类 payload 在 `zircon_editor` 中最终映射到什么宿主事件或状态变更

## Layer Boundary

### `zircon_ui`

`zircon_ui` 仍然只负责通用 UI 基础设施：

- binding AST
- binding codec
- 反射树快照与 diff
- route id 注册与调用
- transport-neutral request/subscribe control plane

### `zircon_editor_ui`

`zircon_editor_ui` 负责 editor-only 协议：

- `EditorUiBinding`
- `EditorUiBindingPayload`
- `SelectionCommand`
- `AssetCommand`
- `DockCommand`
- `ViewportCommand`
- `InspectorFieldBatch`
- editor reflection adapter / control service

### `zircon_editor`

`zircon_editor` 现在通过 `editor_event` 子系统独占 editor 行为执行权：

- semantic event normalization
- event dispatch / execution
- transient UI state tracking
- reflection rebuild
- event journal + replay
- menu dispatch
- layout mutation
- selection propagation
- inspector batch commit
- asset open/reveal intent dispatch
- viewport input dispatch

`zircon_editor_ui` / `zircon_ui` 不再拥有 editor 行为闭包。  
它们只保留 typed binding、route metadata、reflection schema、codec 和 query/control primitives。

热路径现在固定为：

1. Slint / headless / MCP 输入先被适配成 `EditorEventEnvelope` 或 `EditorUiBinding`
2. `EditorEventRuntime` 归一化成 `EditorEvent`
3. 同一个 dispatcher 路径执行状态变更
4. runtime 记录 journal record
5. runtime 用 `EditorState + EditorManager + EditorTransientUiState` 重建 reflection snapshot

因此 `nativeBinding`、Slint callback 名称、route id 都只是外层 transport / adapter 输入，不再是语义层日志格式。

## Editor Event Runtime

`zircon_editor/src/editor_event/` 当前包含：

- `types.rs`
  - 定义 canonical `EditorEvent`、`EditorEventRecord`、`EditorEventResult`、`EditorEventUndoPolicy`
- `runtime.rs`
  - `EditorEventRuntime` / `EditorEventDispatcher`
  - 统一拦截 `InvokeBinding`、`InvokeRoute`、`CallAction`
- `transient.rs`
  - hover / focus / pressed / drawer resize / drag projection
- `journal.rs`
  - session-local event record 存储
- `replay.rs`
  - recorded `EditorEvent` 重新走同一 dispatcher path
- `host_adapter.rs`
  - Slint / headless 输入到 normalized event envelope 的薄适配器

当前 canonical log record 固定保存：

- `event_id`
- `sequence`
- `source`
- normalized `EditorEvent`
- `before_revision` / `after_revision`
- emitted effects
- undo policy
- structured success / failure result

## Slint Host Adapter Path

桌面宿主现在也必须遵守和 headless / reflection 相同的 runtime authority：

- `zircon_editor/src/host/slint_host/app.rs`
  - `SlintEditorHost` 持有 `EditorEventRuntime`
  - `ui.on_*` callback 只负责采集桌面输入、必要的瞬态宿主 bookkeeping、提交 dispatch、消费 effect
  - 不再直接持有 editor 行为语义，也不再在 callback 里直接改 `EditorState` / `EditorManager`
- `zircon_editor/src/host/slint_host/callback_dispatch.rs`
  - 把 raw Slint callback 输入收敛成 `EditorEventEnvelope` 或直接语义化 `LayoutCommand`
  - 统一走 `runtime.dispatch_envelope(...)`
- `zircon_editor/src/host/slint_host/event_bridge.rs`
  - 把 `EditorEventRecord.effects` 映射成宿主展示侧需要的 `SlintDispatchEffects`
  - 宿主只消费 `presentation_dirty / layout_dirty / render_dirty / sync_asset_workspace` 之类的结果，不重新解释语义
- `zircon_editor/src/host/slint_host/drawer_resize.rs`
  - 把 splitter group 级别的桌面手势换成 `LayoutCommand::SetDrawerExtent`
- `zircon_editor/src/host/slint_host/tab_drag.rs`
  - 只保留 tab drop target 到 `ViewHost` 的本地翻译，不再在 helper 里直接执行布局变更

保留在宿主侧但不进入当前 canonical event catalog 的内容仍然只有 adapter 级职责：

- raw `InputManager` pointer / keyboard forwarding
- shell geometry 和 viewport texture bridge
- asset/resource polling refresh
- 草稿输入字段和文件路径暂存

## Stable Shell Chrome Namespaces

这一轮固定的 shell chrome 命名空间如下：

- `WorkbenchMenuBar/*`
- `ActivityRail/*`
- `ToolWindow/*`
- `DocumentTabs/*`
- `InspectorView/*`
- `ViewportToolbar/*`
- `StatusBar/*`

这些名字的作用不是替代 activity instance id，而是给壳层 chrome 提供稳定协议面。  
例如：

- `WorkbenchMenuBar/OpenProject`
- `ActivityRail/ProjectToggle`
- `InspectorView/ApplyBatchButton`

## Editor Payload Surface

`zircon_editor_ui::EditorUiBindingPayload` 当前固定包含：

- `PositionOfTrackAndFrame`
- `MenuAction`
- `InspectorFieldBatch`
- `SelectionCommand`
- `AssetCommand`
- `DockCommand`
- `ViewportCommand`
- `Custom`

其中 `MenuAction` 和 `InspectorFieldBatch` 继续保留原形态；workbench shell 相关的新行为全部进入 typed command family，而不是继续堆 stringly-typed custom payload。

## Workbench Shell Event Contract

### Menu

- source: `WorkbenchMenuBar/*`
- payload: `MenuAction`
- dispatch entry: `dispatch_workbench_binding`
- result: `WorkbenchHostEvent::Menu`

menu 继续承载：

- project open/save
- layout save/reset
- undo/redo
- create node
- open view

### Docking And Tool Window Shell

- source: `ActivityRail/*`, `DocumentTabs/*`, `ToolWindow/*`
- payload: `DockCommand`
- dispatch entry: `dispatch_docking_binding`
- result: `LayoutCommand`

`DockCommand` 当前覆盖：

- focus/close view
- attach view to drawer/document
- detach to window
- activate drawer tab
- activate main page
- set drawer mode
- set drawer extent
- save/load preset
- reset to default

rail click、drawer tab 激活、stack 展开/折叠都应继续走这条 typed docking path。

当前 runtime 里，Slint `Window` 菜单上的 preset 条目仍通过宿主菜单回调进入 `LayoutCommand::SavePreset/LoadPreset`，但稳定协议层已经把它们定成 `DockCommand::SavePreset/LoadPreset`。这保证了 headless / reflection / nativeBinding 可以直接走 typed contract，而不是复制一套字符串分派。

当前真实 Slint tab drag/drop 仍是宿主内部回调，不是反射公开协议：

- Slint tab 释放时先落成内部 `drop_tab(tab_id, target_group)` 回调
- `target_group` 只表达 shell 级别的 `left / right / bottom / document`
- 宿主再把它映射到现有 `AttachView` / `SetDrawerMode(Pinned)` 语义

这样做的原因是：pointer drag 本身是本地桌面手势，不适合作为当前这轮的远控协议表面；真正对外稳定的 attach 语义仍然由 `DockCommand::AttachViewToDrawer` 和 `DockCommand::AttachViewToDocument` 承担。

同理，当前 Slint splitters 也先走宿主内部回调：

- `set_drawer_extent(target_group, extent)` 只表达 shell 级别的 `left / right / bottom`
- 宿主把 group fan-out 到对应 drawer slots，再落成 `LayoutCommand::SetDrawerExtent`
- 稳定协议层继续把这类行为定义为 `DockCommand::SetDrawerExtent`，而不是公开桌面手势细节

这样可以把高频 pointer resize 保留在本地宿主里，同时让 headless / reflection / binding roundtrip 仍然对齐同一个 typed docking 语义。

### Selection Sync

- source: hierarchy / scene related controls
- payload: `SelectionCommand`
- dispatch entry: `dispatch_selection_binding`
- apply entry: `apply_selection_binding`
- result: `SelectionHostEvent` or `EditorIntent::SelectNode`

当前已经落地的 typed selection command 是：

- `SelectionCommand::SelectSceneNode`

这条链路的意义是：层级树点击、viewport 点击、headless 绑定调用，最终都收敛到同一条 selection intent，而不是一个地方直接改状态、另一个地方发字符串消息。

### Asset Intent

- source: project/assets related controls
- payload: `AssetCommand`
- dispatch entry: `dispatch_asset_binding`
- result: `AssetHostEvent`

当前已经落地的 typed asset command 是：

- `AssetCommand::OpenAsset`

这里先锁协议，不强行在这一轮补完完整 asset browser 逻辑。关键约束是：asset open/reveal 不能继续走匿名字符串解析。

### Inspector Commit

- source: `InspectorView/ApplyBatchButton`
- payload: `InspectorFieldBatch`
- dispatch entry: `dispatch_inspector_binding`
- apply entry: `apply_inspector_binding`

`InspectorFieldBatch` 仍然是 inspector 唯一允许的持久化属性编辑入口。  
selection 改变可以刷新 inspector subject，但真正提交属性改动仍然只允许这一条 batch path。

### Viewport

- source: viewport surface / viewport toolbar
- payload: `ViewportCommand`
- dispatch entry: `dispatch_viewport_binding`
- apply entry: `apply_viewport_binding`
- result: `ViewportInput` or viewport feedback

当前已经落地的 typed viewport path 覆盖：

- pointer move
- left/right/middle press/release
- scroll
- resize

viewport toolbar 的 typed command 空间已经固定属于 `ViewportCommand`，即使某些具体 toolbar action 还没完全接入 runtime state，也不能再退回 `Custom("TranslateTool")` 这种匿名协议。

当前 runtime 还承担了 viewport gizmo drag 的 editor 语义状态：

- begin / drag / end 仍由桌面宿主采集 pointer 输入
- 是否进入 gizmo drag 由 runtime 内部 `dragging_gizmo` bookkeeping 决定
- scene 变换、render dirtiness、journal record 都继续走统一 dispatcher path

## Reflection Contract

`zircon_editor/src/workbench/reflection.rs` 现在负责把 workbench snapshot 和 view model 投影到 editor UI 树。

反射树中固定暴露：

- menu item node
- page node
- drawer node
- floating window node
- activity node

activity node 上暴露的动作必须来自 typed payload：

- menu item -> `MenuAction`
- focus/detach -> `DockCommand`
- inspector apply -> `InspectorFieldBatch`
- scene/game pointer actions -> `ViewportCommand`

远控现在可以通过 `CallAction`、`InvokeRoute` 或 `InvokeBinding` 进入这些动作，但执行不再发生在 `EditorUiControlService` 闭包里。  
`register_workbench_reflection_routes` 只注册 stub route metadata，真正执行统一回到 `EditorEventRuntime::handle_control_request(...)`。

## Reflection Rebuild Inputs

runtime 每次事件执行后只允许从三类输入重建 reflection：

- `EditorState` 的稳定 editor 数据快照
- `EditorManager` 的 workbench/layout/view 实例快照
- `EditorTransientUiState` 的 hover/focus/pressed/resizing/drag 状态

这意味着：

- 反射树不再依赖 live Slint tree 查询 transient state
- `CallAction` / `InvokeBinding` 返回的结果来自真实 editor 行为，而不是 preview JSON
- replay session 可以重建同一条 reflection surface，而不是只能回放一堆 widget callback 名字

## Hot-Path Rule

实现约束保持不变：

- 正常 UI 热路径不依赖字符串 parse
- route id 和 `nativeBinding` 最终进入同一 handler
- string formatting/parse 只保留给稳定协议、测试和远控

这也是为什么 shell 相关行为需要拆成 `DockCommand / SelectionCommand / AssetCommand / ViewportCommand`：它们既能提供稳定 wire format，又不会逼迫宿主在高频路径上反复做字符串分派。

## Current Coverage

当前已经有自动化覆盖的点：

- `EditorUiBindingPayload` roundtrip
  - `MenuAction`
  - `InspectorFieldBatch`
  - `SelectionCommand`
  - `AssetCommand`
  - `DockCommand`
  - `ViewportCommand`
- representative shell bindings
  - `WorkbenchMenuBar/OpenProject`
  - `ActivityRail/ProjectToggle`
  - `InspectorView/ApplyBatchButton`
- `editor_event` runtime normalization equivalence
  - Slint adapter input
  - `EditorUiBinding`
  - reflection `CallAction`
- session-local journal serialization / replay
- transient reflection projection
  - hovered
  - focused
  - pressed
  - drawer resizing
- `zircon_editor` host dispatch
  - menu dispatch
  - selection dispatch + apply
  - asset dispatch
  - docking dispatch
  - inspector dispatch + apply
  - viewport dispatch + apply
- desktop Slint callback adapters
  - menu action
  - hierarchy selection
  - asset search
- Slint effect bridge
  - `EditorEventRecord.effects` -> `SlintDispatchEffects`
  - layout / render / presentation / asset sync fan-out
- workbench reflection route registration
  - menu
  - docking
  - inspector
  - viewport
  - runtime-backed `CallAction`

## Validation Status

这次文档同步时重新跑过的验证命令：

- `cargo test -p zircon_editor --lib --locked`
- `cargo test -p zircon_editor_ui --lib --locked`
- `cargo test -p zircon_ui --lib --locked`
- `cargo test --workspace --locked`

这些验证同时覆盖了本轮新增或收紧的几个关键点：

- Slint callback adapter 通过 runtime dispatch 执行
- `CallAction` / `InvokeBinding` 继续命中真实 editor behavior
- global default layout 的 empty skeleton 现在会在 bootstrap 时被接受，并补回 builtin shell view placement
- `drawer_resize.rs` 的旧测试 helper 只在 `cfg(test)` 下编译，不再给工作区构建引入死代码告警

## Remaining Work

这一轮已经完成了主线桌面宿主接线：

- `slint_host/app.rs` 的 menu / docking / hierarchy / inspector / asset / viewport callback 已经改成 adapter + runtime dispatch
- `InvokeBinding` / `CallAction` / Slint callback 现在可以对齐到同一个 normalized event path
- reflection rebuild 输入已经收敛到 `EditorState + EditorManager + EditorTransientUiState`

剩下的工作主要是扩展 catalog，而不是继续拆 ownership：

- viewport toolbar / gizmo 的更多 typed action 并入 normalized event catalog
- asset browser / project explorer 的更完整 action catalog
- MCP / network transport adapter 直接消费 runtime journal 和 reflection surface
- 非 scene editor-domain undo 继续在当前 journal / undo metadata 上扩展

也就是说，这一轮已经把“谁拥有执行权”“日志记录什么”“反射动作如何执行”定死了；后续主要是把更多 editor 行为填进同一 dispatcher，而不是再把语义退回 UI 库。
