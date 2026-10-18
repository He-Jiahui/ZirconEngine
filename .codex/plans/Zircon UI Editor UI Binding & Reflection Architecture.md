# Zircon UI / Editor UI Binding & Reflection Architecture

## Summary

- 明确三层边界：
  - `zircon_ui` 只负责运行时 UI 基础设施，后续承接 `render_pipeline_ui` 与 `event_ui`，放置通用 binding AST、反射树、控制面与高性能事件路由。
  - `zircon_input` 只负责运行时输入事实，保留鼠标/键盘/滚轮等原始状态与事件，不直接承载 `onClick/onHover/onPress` 这类 UI 语义。
  - `zircon_editor_ui` 只负责编辑器 UI 协议与 editor-only 语义，封装 `ActivityView`、`ActivityWindow`、workbench/inspector/animation/asset 等绑定与反射适配。
- V1 控制面固定为 `Request + Subscribe`：
  - 支持通过结构化请求或 `nativeBinding` 调用 UI 行为。
  - 支持查询反射树、节点状态、属性值、动作目录。
  - 支持订阅反射 diff 与动作结果，供 REPL、自动化测试、网络远控复用。
- V1 反射深度固定为 `State + Actions`：
  - 暴露 `visible/enabled/clickable/hoverable/focusable/pressed/checked/dirty` 等状态。
  - 暴露属性读写描述与白名单动态调用。
  - 不走任意 Rust 函数反射，只允许通过显式注册的 action schema 暴露可远控能力。
- 正常 UI 热路径不直接走 `nativeBinding` 解析；必须走编译后 route id 或等价快速分发表。同时每个 route 都必须携带稳定的 binding 元数据，保证 headless/REPL/网络驱动与真实 UI 点击进入同一语义 handler。

## Key Changes

### `zircon_ui`

- 将现有 `UiBinding*` 演进为 `event_ui` 子系统，固定包含：
  - `UiEventKind`
  - `UiEventPath`
  - `UiBindingValue`
  - `UiBindingCall`
  - `UiEventBinding`
  - `UiBindingCodec`
  - `UiRouteId`
  - `UiInvocationRequest`
  - `UiInvocationResult`
  - `UiEventManager`
- `nativeBinding` wire format 固定为：
  - `<ViewId>/<ControlId>:<EventKind>(<Call>)`
  - 例：`AnimationClipEditorView/AddFrameButton:onClick(PositionOfTrackAndFrame("root/child:transform",24))`
- 引入 UI 反射树模型，固定包含：
  - `UiTreeId`
  - `UiNodeId`
  - `UiNodePath`
  - `UiNodeDescriptor`
  - `UiStateFlags`
  - `UiPropertyDescriptor`
  - `UiActionDescriptor`
  - `UiReflectionSnapshot`
  - `UiReflectionDiff`
- `UiEventManager` 职责固定为：
  - 注册 UI 节点与动作目录
  - 生成并缓存高性能 route id
  - 用 route id 执行本地热路径事件
  - 用 `UiEventBinding` 执行 headless/REPL/网络事件
  - 提供反射查询
  - 维护订阅并推送 diff
- 节点标识固定采用稳定语义 ID，而不是运行时随机 ID；路径使用逻辑层级路径，允许实例后缀，但不允许纯索引路径作为长期协议主键。
- `render_pipeline_ui` 继续留在 `zircon_ui`，但本轮只为后续 2D UI 渲染预留模块边界，不把 editor 宿主 UI 框架耦合进去。

### `zircon_input`

- 保持输入事实层，不新增 editor 语义类型。
- `InputServer` 继续提供原始输入事件与快照；`UiEventManager` 和具体宿主用这些事实推导 hover/press/click，不反向把 UI 事件塞回 `zircon_input`。
- 只补充与 UI 桥接需要的最小能力：
  - 明确 pointer/button/key 的规范化表示
  - 明确时间戳或序列号字段是否需要用于 click/double-click 推导
  - 不承担反射树、binding、REPL 控制职责

### `zircon_editor_ui`

- 新增 editor-only UI 协议层，固定包含：
  - `ActivityViewDescriptor`
  - `ActivityWindowDescriptor`
  - `EditorUiBinding`
  - `EditorUiBindingPayload`
  - `EditorUiRouter`
  - `EditorUiReflectionAdapter`
  - `EditorUiControlService`
- `ActivityViewDescriptor` 固定表达：
  - 稳定 `view_id`
  - 标题与图标 key
  - 多开能力
  - 可停靠宿主能力
  - 默认 drawer/document/workspace 偏好
  - 对应反射树根节点 ID
- `ActivityWindowDescriptor` 固定表达：
  - 稳定 `window_id`
  - 标题与图标 key
  - 多开能力
  - 可进入中心 tab、独占主壳、浮出 OS window 的能力
  - 对应反射树根节点 ID
- `EditorUiBindingPayload` V1 固定覆盖：
  - `MenuAction`
  - `InspectorFieldBatch`
  - `PositionOfTrackAndFrame`
  - `DockCommand`
  - `SelectionCommand`
  - `AssetCommand`
  - `ViewportCommand`
  - `Custom` 仅作为显式白名单扩展槽位
- `EditorUiReflectionAdapter` 负责把 editor shell、activity、inspector 草稿、selection、animation editor 状态投影成可查询 UI 树，不让 `zircon_editor` 自己暴露杂糅的 UI 反射逻辑。
- `Unity EditorWindow / Editor` 对应关系固定为：
  - `ActivityView`：可停靠、可入文档区、可浮窗，但不是主壳独占页
  - `ActivityWindow`：可入文档区、可独占主壳、可浮窗，适合 prefab/animation/asset editor 一类重型编辑界面

### `zircon_editor`

- 保持 `LayoutManager`、`EditorManager`、`WorkbenchViewModel` 等 backend-neutral 模型，但所有 editor UI 行为入口都逐步改为：
  - 本地热路径：route id 或等价直接处理器
  - 远控/测试路径：`EditorUiBinding` 或 `nativeBinding`
- `WorkbenchHostEvent`、menu dispatch、inspector batch、animation frame add/remove、docking 操作都通过 `zircon_editor_ui` 的 payload 协议统一描述，再映射到 `LayoutCommand`、`EditorIntent`、Undoable command 栈。
- 正常 Slint 或未来其他宿主只负责：
  - 渲染
  - 将宿主事件映射到 route id
  - 同步反射树节点状态
  - 不自己定义另一套 editor 语义协议

### REPL / Network Control

- V1 不锁定具体传输协议实现，但核心服务接口固定为 transport-neutral：
  - `InvokeBinding`
  - `InvokeRoute`
  - `QueryTree`
  - `QueryNode`
  - `QueryProperty`
  - `SetProperty`
  - `CallAction`
  - `SubscribeDiffs`
  - `Unsubscribe`
- REPL 适配器和网络适配器都复用同一个 `UiEventManager` / `EditorUiControlService`，不允许出现“测试入口一套、网络入口一套、真实 UI 一套”的三分叉实现。
- `CallAction` 只允许调用 `UiActionDescriptor` 中显式公开的动作；结果必须走统一的 `UiInvocationResult` 返回，包含成功值或结构化错误，而不是 panic/stringly-typed 失败。

## Public Interfaces

- `zircon_ui`
  - `UiEventKind`
  - `UiEventPath`
  - `UiBindingValue`
  - `UiBindingCall`
  - `UiEventBinding`
  - `UiBindingCodec`
  - `UiRouteId`
  - `UiInvocationRequest`
  - `UiInvocationResult`
  - `UiTreeId`
  - `UiNodeId`
  - `UiNodePath`
  - `UiNodeDescriptor`
  - `UiStateFlags`
  - `UiPropertyDescriptor`
  - `UiActionDescriptor`
  - `UiReflectionSnapshot`
  - `UiReflectionDiff`
  - `UiEventManager`
- `zircon_editor_ui`
  - `ActivityViewDescriptor`
  - `ActivityWindowDescriptor`
  - `EditorUiBinding`
  - `EditorUiBindingPayload`
  - `EditorUiRouter`
  - `EditorUiReflectionAdapter`
  - `EditorUiControlService`
- `zircon_editor`
  - `WorkbenchHostEvent`
  - `dispatch_workbench_binding`
  - 后续扩展：
    - `dispatch_inspector_binding`
    - `dispatch_animation_binding`
    - `dispatch_docking_binding`
    - `dispatch_viewport_binding`

## Test Plan

- binding codec
  - `nativeBinding` parse/format roundtrip
  - string escape、array、bool、number、null roundtrip
  - 非法 grammar、未知 event kind、未知 call symbol 的错误路径
- route and performance boundary
  - 同一动作既可通过 route id 直接调用，也可通过 `UiEventBinding` 调用，并进入同一 handler
  - 热路径不依赖字符串 parse
- reflection
  - UI 树快照生成
  - 节点 flags 更新后 diff 正确
  - 属性读写与白名单 action 反射正确
  - 缺失节点、缺失属性、不可写属性、未公开 action 的失败路径
- editor-specific
  - `ActivityViewDescriptor` / `ActivityWindowDescriptor` 注册与恢复
  - menu、inspector batch、animation frame、dock/view host payload roundtrip
  - `ActivityWindow` 独占主壳与浮窗状态的反射一致性
- integration
  - headless 测试通过 `nativeBinding` 驱动 editor 命令，验证 world/editor state 变化正确
  - REPL/控制服务通过 request/query/subscribe 驱动同一行为
  - `zircon_input` 原始输入能被 UI 层正确解释为 hover/press/click，但 `zircon_input` 自身不引入 UI 语义
  - `slint` 宿主后续接入时，真实 UI 点击与 headless binding 调用结果一致

## Assumptions And Defaults

- 控制面默认采用 `Request + Subscribe`，不先锁死 JSON-RPC、WebSocket 或 REPL 命令语法；传输适配层后续再加，但核心服务接口本轮就要定型。
- 反射深度默认采用 `State + Actions`，包含属性读写和白名单动态调用，不做任意对象/任意函数反射。
- 稳定协议主键默认使用语义化 `view_id / control_id / node_path`，不使用瞬态地址、UI 框架句柄或纯索引路径。
- `nativeBinding` 是稳定控制面与测试面协议，不是正常 UI 热路径主调用方式。
- `zircon_editor_ui` 是 editor-only crate；任何 `ActivityView / ActivityWindow / EditorUiBindingPayload` 都不得回流到 `zircon_ui`。
- `zircon_input` 继续保持运行时输入系统身份，不升级成 UI 事件管理器。
