# Zircon 运行时/编辑器共享 UI 布局与事件系统架构计划

## Summary
- 以你的 [布局系统.md](/E:/Git/ZirconEngine/.codex/plans/布局系统.md) 为功能基线，采用“共享 UI 内核优先 + 混合 retained UI tree + ECS bridge”的路线，而不是做一套编辑器壳专用布局后再事后抽象。
- 架构上借鉴 Unreal Slate/UMG、Godot Control、Fyrox UI 的 retained tree + desired size + invalidation 体系，借鉴 Bevy 的输入归一化、焦点/导航与调度分层；编辑器 docking/workbench 像 Unreal/Fyrox/Unity EditorWindow 一样建立在通用 UI core 之上，而不是混入运行时基础控件层。
- 所有真实布局、命中、焦点、冒泡、捕获、滚动、虚拟化、渲染抽取都下沉到 `zircon_ui`；`zircon_editor_ui` 和 `zircon_editor` 只保留编辑器特有的 workbench/docking/windowing 语义；`Slint` 仅作为宿主渲染与事件适配层，不再做布局真源。

## Public Interfaces
- `zircon_ui` 成为共享 UI 权威层，新增或定型 `UiTree`、`UiNodeId`、`UiSurface`、`UiDirtyFlags`、`UiLayoutCache`、`UiHitTestIndex`、`UiFocusState`、`UiNavigationState`、`UiRenderList/UiRenderExtract` 等核心对象。
- 布局公共类型固定为 `StretchMode`、`AxisConstraint`、`ResolvedAxisConstraint`、`Pivot`、`Anchor`、`Position`、`BoxConstraints`、`DesiredSize`、`LayoutBoundary`；它们既服务 runtime widget，也服务 editor workbench shell。
- 布局算法固定为“两段式”：反向 measure 求 `desired_size`，正向 arrange 分配最终 frame；失效传播遵循“子变更向上冒泡，遇到固定边界或外部尺寸边界停止”，这直接对应你文档里的设脏和反/正向求解要求。
- 通用容器层首批覆盖 `Container`、`Overlay`、`HorizontalBox`、`VerticalBox`、`FlowBox`、`GridBox`、`ScrollableBox`、`Space`；其中 `ScrollableBox`/`GridBox` 默认支持虚拟化，避免把编辑器资源面板和运行时长列表都绑死在全量节点渲染上。
- 事件体系拆成三层：`zircon_input` 只提供规范化原始输入事实；`zircon_ui` 负责 `GlobalInputEvent`、`RootInputEvent`、`BubbledInputEvent`、`FocusedInputEvent`、`NavigationInputEvent` 的推导与分发；`zircon_editor_ui` 只承接 editor-only payload 和反射协议。
- 命中与路由以“反向绘制顺序 + clip 链检查 + 可选空间索引优化”为准，空间索引只是优化层，不是语义真源；这样既保留你文档里四叉树式加速的方向，也避免 overlay、裁剪、穿透和堆叠 UI 被纯空间结构误判。
- 事件结果语义固定支持“处理、阻止、穿透、捕获”四类效果，并且显式建模 `UiInputPolicy { Inherit, Receive, Ignore }` 与可见性继承；键盘/手柄导航从 focus 节点开始冒泡，未处理时回退根响应。
- `WorkbenchLayout` 继续保留为编辑器持久化拓扑模型，但它不再直接承担底层 UI 布局引擎角色；执行时应投影到通用 `UiTree`/layout graph，由共享 solver 求解，再把 frame 下发给宿主。
- `zircon_editor_ui` 保持 editor-only 边界，负责 `ActivityView`、`ActivityWindow`、docking/window payload、反射适配与 headless 控制；编辑器 docking、drawer、document tabs、floating window 不下沉到 runtime widget core。
- `zircon_graphics` 只消费 `UiRenderExtract`/draw list，不感知 editor shell；`Slint` 继续只消费 frame、状态、route id 和外部纹理，不再拥有布局、焦点或事件语义。

## Milestones
- 里程碑 1：冻结共享边界与权威模型，先把 `zircon_ui`、`zircon_input`、`zircon_editor_ui`、`zircon_editor` 的职责重新钉死，再把现有 workbench autolayout、`event_ui`、binding/reflection 原型映射到统一 UI core 术语。
- 里程碑 2：落地通用 retained UI tree、dirty/invalidation、measure/arrange、anchor/pivot、scroll/clip、虚拟化容器；把当前 workbench solver 里可复用的 `AxisConstraint` 体系提升为通用布局契约，而不是继续只服务 shell。
- 里程碑 3：落地命中、冒泡、焦点、导航、捕获和全局事件体系；统一鼠标、触摸、键盘、滚轮、拖拽和双击推导规则；让 headless 控制、真实宿主点击和未来网络驱动进入同一 dispatcher。
- 里程碑 4：补齐共享渲染抽取与测量服务，包括文本、图片、裁剪、滚动条和主题 token；先把 screen-space runtime UI 与 editor shell 跑通，再向世界空间 UI 或更复杂皮肤系统扩展。
- 里程碑 5：把编辑器 workbench/docking/window host 迁移到共享 core，上层保留 `WorkbenchLayout`、`LayoutCommand`、`EditorEventDispatcher`、`EditorUiReflectionAdapter`，但底层布局、hit-test、focus、resize、drag/drop 统一走 `zircon_ui`。
- 里程碑 6：增加 ECS bridge，让运行时 UI 能以“UI tree 权威、ECS 为宿主桥接”的方式接入 scene/world；运行时和编辑器最终共用同一布局/事件/渲染抽取核心，只保留各自不同的节点库和宿主语义。

## Test Plan
- 布局验证覆盖 `min/max/preferred/priority/weight/stretch`、anchor/pivot/position、dirty 传播停止条件、滚动与裁剪、容器嵌套、虚拟化窗口范围更新，以及编辑器 workbench 与 runtime 普通控件混合树的稳定性。
- 事件验证覆盖重叠节点命中、裁剪后命中、处理/阻止/穿透/捕获语义、pointer enter/leave、click/double-click/drag、focus 切换、导航键冒泡、根响应回退，以及可见性和 `Ignore` 策略继承。
- 性能回归必须覆盖“大量静态节点只局部重排”、“长列表滚动不全量布局”、“编辑器 dock/resize 不触发整树重算”、“headless 调用与真实宿主调用走同一路由且无字符串热路径解析”。
- 编辑器集成测试覆盖现有 preset/layout roundtrip、drawer/document/floating window 迁移、splitter/drag target 行为、反射与 replay 一致性，以及当前 Slint workbench 交互在迁移后不发生行为回退。
- 运行时集成测试覆盖基础控件、滚动容器、文本测量、输入桥接、render extract 输出和 ECS bridge；要求运行时 UI 不依赖 editor-only 类型，也不反向污染 `zircon_input`。
- 验收标准固定为：同一套 shared core 同时支撑编辑器 workbench 与运行时 UI；Slint 不再是布局/事件真源；`WorkbenchLayout` 只做编辑器拓扑持久化；`zircon_ui` 成为真正可扩展的通用 UI 子系统。

## Assumptions And Defaults
- 已锁定默认路线为“共享内核优先”和“混合 retained tree + ECS bridge”；不采用纯 ECS UI，也不再走“先把编辑器壳写死，之后再抽 runtime core”的路线。
- V1 先覆盖 screen-space UI、编辑器 workbench、常用容器/基础控件、输入/焦点/滚动/虚拟化；world-space UI、复杂动画系统、可视化样式编辑器可以建立在同一 core 上，但不作为本计划首批闭环。
- 主题与样式先采用轻量 token/descriptor 模型，保证文字测量、间距、颜色、滚动条与状态样式能驱动布局和渲染；不在第一阶段引入完整 CSS 风格系统。
- 编辑器 docking 继续属于 editor-only layer；运行时共享的是容器、约束、事件、命中、滚动、绘制与反射基础，不共享 editor page/drawer/window 语义。
- 现有 `zircon_ui` binding/reflection 原型、`zircon_editor_ui` 协议层、`zircon_editor` 的 workbench autolayout 和 editor event/replay 计划都作为迁移起点；后续新增设计不能与 [全系统重构方案](/E:/Git/ZirconEngine/.codex/plans/全系统重构方案.md) 的分层边界冲突。
