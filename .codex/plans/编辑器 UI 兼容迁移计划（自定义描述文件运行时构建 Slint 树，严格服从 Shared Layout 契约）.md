# 编辑器 UI 兼容迁移计划（自定义描述文件运行时构建 Slint 树，严格服从 Shared Layout 契约）

## Summary
- 保留 Slint 作为表现层宿主，但不再让 `.slint` 文件直接定义 editor workbench 的业务树、布局真源或事件真源。
- 新的权威链路固定为：
  自定义 TOML 描述文件 -> 运行时模板解释器 -> shared `UiTree/UiSurface` -> 按 layout 契约完成 measure/arrange、命中、焦点、滚动、虚拟化 -> 投影成通用 Slint 宿主树。
- 迁移保留的兼容面固定为：
  `WorkbenchLayout` 持久化拓扑、`EditorEventRuntime` 和 typed command、reflection/nativeBinding、以及当前编辑器 workbench 的可见外观与交互结果。
- 第一阶段的 TOML 粒度固定为“复合组件装配层”：
  TOML 负责壳体、pane、toolbar、menu、tab、empty state、popup、host 区域组合；
  Button/Label/Input/Icon/ScrollArea 这类 visual primitive 继续由统一 Slint host 组件库承载。

## Key Changes

### 1. 权威边界
- `zircon_ui` 继续是唯一 UI 内核：
  `UiTree`、`UiSurface`、`UiDirtyFlags`、`UiLayoutCache`、`UiHitTestIndex`、`UiFocusState`、`UiNavigationState`、`UiRenderExtract` 保持为真实运行时权威。
- `WorkbenchLayout` 继续只负责 editor docking/preset/restore 拓扑，不承担底层布局引擎。
- Slint 退化为“通用渲染宿主 + 通用交互节点承载层”，不再直接决定：
  pane 树结构、
  dock host 结构、
  splitter/drop target 语义、
  布局公式、
  业务 callback 命名。
- `zircon_editor_ui` 承载 editor-only composite catalog 和模板适配层，不把 editor 壳语义沉到 `zircon_ui` primitive 层。

### 2. 描述文件与运行时解释器
- 新增共享模板层：
  `UiTemplateDocument`、`UiTemplateNode`、`UiComponentTemplate`、`UiSlotTemplate`、`UiBindingRef`、`UiTemplateInstance`、`UiTemplateLoader`、`UiTemplateValidator`。
- TOML 描述文件固定描述：
  组件类型、slot 组合、样式 token、稳定 binding/route 引用、repeat/tree 数据投影、editor composite 参数。
- TOML 不描述：
  `WorkbenchLayout` 拓扑、
  具体像素布局结果、
  editor 行为逻辑、
  宿主专属回调函数名。
- 运行时解释器固定流程：
  `WorkbenchViewModel + EditorChromeSnapshot + WorkbenchLayout`
  -> `EditorTemplateAdapter`
  -> `UiTemplateInstance`
  -> shared `UiTree`
  -> shared layout 求 frame
  -> `SlintUiProjection`
  -> 通用 Slint 宿主树。
- 描述文件默认放在 `zircon_editor/ui/templates/`，作为仓库源文件；build 时嵌入，debug 模式允许文件变更重载，但嵌入版本仍是发布权威。

### 3. Slint 宿主重构
- 现有 `workbench.slint` 从“整套手写业务壳”改为“通用宿主组件库”：
  `UiHostRoot`、`UiHostStack`、`UiHostScroll`、`UiHostTabStrip`、`UiHostToolbar`、`UiHostMenu`、`UiHostEmptyState`、`UiHostViewportChrome`、`UiHostIconButton` 等。
- Slint 宿主只消费运行时投影数据：
  节点类型、frame、clip、z-order、状态、文本、图标 key、列表模型、当前激活态。
- 当前海量 `set_*` 属性投影改为分层收口：
  `SlintUiProjection` 输出统一的 host node model；
  Slint 只绑定通用 node model，不再为每个业务 pane 维护一组专门属性。
- 现有 `slint_host/app.rs` 中的 callback 绑定改为稳定 route 分发：
  Slint 事件 -> 通用 host event -> shared `UiSurface` route -> stable binding id -> `EditorEventRuntime`。
- viewport 继续保留 Slint 窗口与 WGPU 渲染桥，但 Scene/Game 周边 chrome、toolbar、overlay、dock host 都切到描述文件驱动。

### 4. 公共组件与 editor composite
- `zircon_ui` primitive 首批固定继续服务契约中的容器/基础控件：
  `Container`、`Overlay`、`HorizontalBox`、`VerticalBox`、`ScrollableBox`、`Space`，以及 `Label`、`Button`、`Image`、`TextField`、`Canvas`。
- `zircon_editor_ui` 提供 editor composite prefab：
  `WorkbenchShell`、`MenuBar`、`ActivityRail`、`ToolWindowStack`、`DocumentTabs`、`StatusBar`、`WelcomePane`、`SceneViewportToolbar`、`ProjectPane`、`AssetsPane`、`HierarchyPane`、`InspectorPane`、`ConsolePane`。
- 第一阶段迁移顺序固定：
  1. 壳体与 host 区域
  2. menu/rail/tab/splitter/drop overlay/status/welcome
  3. pane surface
  4. pane 内部 editor composite
  5. 通用化 asset/hierarchy/inspector/tree/list 行模板
- 稳定命名空间全部保留：
  `WorkbenchMenuBar/*`、`ActivityRail/*`、`ToolWindow/*`、`DocumentTabs/*`、`InspectorView/*`、`ViewportToolbar/*`、`StatusBar/*`。

### 5. 兼容迁移策略
- 保持现有 `WorkbenchLayout`、project/global layout、preset、view descriptor、view instance 恢复逻辑不变。
- 保持现有 typed command 协议不变：
  `DockCommand`、`SelectionCommand`、`AssetCommand`、`ViewportCommand`。
- 保持现有 reflection/nativeBinding 为稳定控制面，不让 TOML 文件结构泄漏到远控协议。
- 允许迁移期同时存在“旧手写 Slint 壳”和“描述文件驱动 Slint 宿主”的 parity 验证路径，但默认运行主链只允许一个主宿主。
- cutover 之后仍然依赖 Slint，但仅依赖“通用宿主组件库 + runtime projection”，不再依赖手写业务 `.slint` workbench 树。

## Public Interfaces
- `zircon_ui`
  - `UiTemplateDocument`
  - `UiTemplateNode`
  - `UiComponentTemplate`
  - `UiSlotTemplate`
  - `UiBindingRef`
  - `UiTemplateInstance`
  - `UiTemplateLoader`
  - `UiTemplateValidator`
  - 继续复用 `UiTree`、`UiSurface`、`UiRenderExtract`
- `zircon_editor_ui`
  - `EditorComponentCatalog`
  - `EditorTemplateRegistry`
  - `EditorTemplateAdapter`
  - editor composite TOML prefab 集
- `zircon_editor`
  - `SlintUiProjection`
  - `EditorUiHostRuntime`
  - `EditorUiCompatibilityHarness`
  - 继续复用 `WorkbenchLayout`、`LayoutCommand`、`EditorEventRuntime`

## Test Plan
- 模板系统：
  TOML parse/validate、slot 展开、composite 装配、repeat/tree 投影、非法 route/binding 报错、样式 token 继承与覆盖。
- shared layout 契约：
  `min/max/preferred/priority/weight/stretch`、anchor/pivot/position、dirty 传播停止条件、scroll/clip、virtualization、focus/navigation fallback、pointer block/passthrough/capture。
- Slint 宿主投影：
  同一 `UiTemplateInstance + UiLayoutCache` 在 Slint host 下得到稳定 frame、clip、tab/menu/list/tree/popup 结构。
- editor shell parity：
  menu、rail、tabs、splitter、dock target、drag overlay、welcome、status bar、viewport toolbar 的视觉结构和交互结果与当前基线一致。
- 协议兼容：
  `nativeBinding`、reflection `CallAction/InvokeBinding`、journal/replay、typed command dispatch 继续通过，不回退到字符串热路径。
- 布局兼容：
  现有 layout/preset/fixture 在新链路下 roundtrip 稳定，`WorkbenchLayout` 不变。
- 集成验收：
  `cargo test -p zircon_ui --locked`
  `cargo test -p zircon_editor_ui --lib --locked`
  `cargo test -p zircon_editor --locked`
  并新增 parity 测试，要求“旧手写 Slint 壳”和“描述文件驱动 Slint 宿主”在同一 fixture 上产出等价 shell snapshot、route 结果和 event log。

## Assumptions And Defaults
- 已锁定实现路径为“运行时解释树”，不做“编译期由 TOML 生成 `.slint`”作为第一阶段主路线。
- 已锁定 TOML 粒度为“复合组件装配层”，不是第一阶段就把所有 Button/Label 级基础控件都完全模板化。
- Slint 继续保留为窗口与表现层宿主，但不再是布局、事件、业务树结构的真源。
- 第一阶段目标是兼容迁移，不重做 editor 信息架构；先复刻当前 workbench 行为与视觉，再继续抽象公共组件和描述文件能力。
- `WorkbenchLayout` 继续是 editor 拓扑持久化权威；自定义描述文件只负责构建 Slint 宿主树和 shared UI tree，不替代 layout schema。
