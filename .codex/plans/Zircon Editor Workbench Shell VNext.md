# Zircon Editor Workbench Shell VNext

## Summary

- 视觉基线固定为：整体壳体对齐 [VSCode.dev](https://vscode.dev/)，顶部菜单感参考 [JupyterLite Lab](https://jupyterlite.github.io/demo/lab/index.html)，中心文档区 docking 交互参考 [Dockview demo](https://dockview.dev/templates/dockview/demo-dockview/react/index.html)。
- 编辑器宿主不再继续沿 `iced` 演进；V1 改成“两步走”：
  - 第一步先做本地浏览器 workbench 预览壳，固定输出本地 URL 供你快速看样式和交互。
  - 第二步按已确认的预览壳，把桌面编辑器宿主迁移到 `slint`。
- `zircon_graphics` 继续保持独立渲染核心；`zircon_editor` 只做 workbench、菜单、布局、命令和 viewport 宿主，不把 UI 框架下沉到 graphics/scene/asset。
- V1 菜单先做 `slint` 自绘系统菜单风格，不直接做三端原生系统菜单；V2 再补原生菜单映射层。
- Slint 视口桥接固定采用“离屏纹理导入”：
  - `zircon_graphics` 输出离屏 `wgpu::Texture`
  - `slint` 通过 `wgpu_27 + Image::try_from(texture)` 显示
  - 不走 CPU readback，不改“核心渲染独立、编辑器仅宿主”的边界

## Key Changes

### 1. Browser Preview First

- 新增独立的 workbench preview 工程，只用于视觉和交互定稿，不进入 runtime/editor 正式构建链。
- 预览壳默认本地 URL 固定为 `http://127.0.0.1:5173/`，目标是让你先对“壳体像不像”做判断，而不是先被桌面宿主细节拖慢。
- 预览壳直接实现以下目标样式：
  - 顶部一行 app menu：`File / Edit / Selection / View / Window / Help`
  - 第二行 `MainHostTabStrip`：左侧是主工作台页 / 独占页 tabs，右侧是活动文档路径/面包屑混合条
  - 左侧主活动栏 + 可展开 drawer
  - 右侧默认折叠 activity rail
  - 底部面板区 + 底部状态/进度条
  - 中心 document workspace 支持 tab group、split、detach、reattach
- 预览壳中的 docking 规则与最终桌面版保持一致：
  - 6 个固定 `ActivityDrawer`
  - 中心 `DocumentWorkspace` 允许递归 split
  - `ActivityWindow` 可进中心、可独占主壳、可分离为浮窗
- 预览壳使用和 editor 相同的布局 JSON fixture 与 view descriptor fixture，避免“浏览器一套、桌面一套”。

### 2. Editor Refactor For Backend Independence

- `zircon_editor` 先从 `iced` presenter 结构中拆出纯模型层，保留并强化这些 backend-agnostic 部分：
  - `EditorManager`
  - `LayoutManager`
  - `ViewRegistry`
  - `WorkbenchLayout`
  - `ProjectEditorWorkspace`
  - `LayoutCommand`
- 新增 host-neutral 的 workbench view model 层，统一把 editor/session/layout 状态投影成：
  - `MenuBarModel`
  - `MainHostStripModel`
  - `DrawerRingModel`
  - `DocumentWorkspaceModel`
  - `StatusBarModel`
- 现有 `iced` 专属 presenter、message、window routing 不再作为主链路继续扩展；迁移完成后从 editor/runtime 正式依赖中移除 `iced`。
- `ActivityDrawer` 继续是固定槽位容器，不升级成全局 universal docking；只有中心文档区和浮动窗口允许递归 split。

### 3. Slint Desktop Host

- `zircon_editor` 新增 `slint` 宿主实现，负责把上面的 host-neutral 模型映射成真实桌面 workbench。
- `slint` 层固定职责：
  - 渲染顶部菜单条和下拉菜单
  - 渲染顶部主壳页签 + 路径混合条
  - 渲染 6 个 `ActivityDrawer`
  - 渲染中心 `DocumentWorkspace`
  - 渲染原生浮动窗口宿主
  - 把用户交互转换成 `LayoutCommand` / `EditorIntent`
- 顶部菜单 V1 行为固定：
  - 点击展开
  - 打开后 hover 可切换菜单
  - `Esc`/点击外部关闭
  - 显示快捷键提示
  - 实际命令统一映射到 `MenuAction`
- `MainHostTabStrip` 固定为混合结构：
  - 左段显示 `WorkbenchPage` 与 `ExclusiveActivityWindowPage`
  - 右段显示当前活动 view 的 breadcrumb/path chips
  - 切换独占页时隐藏 drawers，但冻结其状态
- 浮动窗口由 `slint` 多窗口宿主统一管理，生命周期仍由 `WindowHostManager` 驱动，不让 view 自己创建窗口。

### 4. WGPU / Slint Viewport Bridge

- Slint 技术栈固定使用 `winit + wgpu_27` 路径，和当前 workspace 的 `wgpu = 27.x` 对齐。
- 初始化路径固定为共享 WGPU 配置：
  - `slint` 使用 `BackendSelector::require_wgpu_27(...)`
  - editor host 与 `zircon_graphics` 共用同一 `Device` / `Queue`
- `Scene` / `Game` view 不直接拥有 surface；它们只消费 `ViewportFrameTextureHandle`。
- 新增 `ViewportTextureBridge`：
  - 输入 `zircon_graphics` 的离屏纹理句柄
  - 输出 `slint::Image`
  - 负责格式和 usage 约束校验
- 首期离屏纹理格式固定为 `Rgba8Unorm` 或 `Rgba8UnormSrgb`，并包含 `TEXTURE_BINDING | RENDER_ATTACHMENT`，以满足 Slint 导入要求。
- 任何 overlay gizmo、grid、selection highlight、picking 仍留在 `zircon_graphics` viewport 子系统，不上浮到 `slint`。

### 5. Runtime UI / Editor UI / Input Boundary

- `zircon_ui` 明确限定为运行时 UI 基础层。
  - 后续承载 `render_pipeline_ui`
  - 后续承载 `event_ui`
  - 放置通用 binding AST、codec、UI 反射树、transport-agnostic 路由
- `zircon_input` 明确限定为运行时输入系统。
  - 只负责输入事实
  - 不直接承载 editor UI 或运行时 UI 的高层事件语义
- `zircon_editor_ui` 明确为编辑器专属 UI 协议层。
  - `ActivityView`
  - `ActivityWindow`
  - editor shell 所需 binding payload、反射适配、headless 事件分发
  - 只允许 editor 语义写入这里，不回灌 `zircon_ui`

### 6. UI Binding And Reflection Model

- 所有 UI 行为都必须能映射到稳定的 `nativeBinding`。
- 但正常 UI 热路径不直接走字符串解析，而是走编译后 route；`nativeBinding` 作为：
  - 单元测试入口
  - REPL 控制入口
  - 网络远控入口
  - 可选的录制/回放与调试日志格式
- `nativeBinding` 文法固定为：

```text
<ViewId>/<ControlId>:<EventKind>(<Call>)
```

- 例如：

```text
AnimationClipEditorView/AddFrameButton:onClick(PositionOfTrackAndFrame("root/child:transform",24))
```

- 结构层固定拆成：
  - `UiEventPath`
  - `UiBindingCall`
  - `UiBindingValue`
  - `UiEventBinding`
- UI 对象必须可反射以下状态：
  - `visible`
  - `enabled`
  - `clickable`
  - `hoverable`
  - `focusable`
- UI 对象必须可反射以下行为目录：
  - `onClick`
  - `onHover`
  - `onPress`
  - 以及 editor/runtime 各自允许暴露的动态函数调用白名单
- 正常 UI 点击和 headless/REPL/网络驱动必须复用同一语义 handler，不允许长期分叉成两套逻辑。

### 7. Editor UI Specialization

- `ActivityView`
  - 可停靠到 drawer
  - 可进入中心 document workspace
  - 可浮出原生窗口
- `ActivityWindow`
  - 可进入中心 document workspace
  - 可独占主壳成为顶层页
  - 可浮出原生窗口
- `zircon_editor_ui` 负责 editor-only payload，例如：
  - `PositionOfTrackAndFrame`
  - `MenuAction`
  - `InspectorFieldBatch`
  - 后续 `DockCommand / SceneViewportCommand / AssetCommand / AnimationCommand`
- `zircon_editor` 只负责把这些 payload 解释为：
  - `LayoutCommand`
  - `EditorIntent`
  - `WorkbenchHostEvent`
  - 以及后续 animation/asset/viewport 专属命令

## Public Interfaces

- `zircon_editor`
  - 保留：`EditorManager`、`LayoutManager`、`ViewRegistry`、`WorkbenchLayout`、`LayoutCommand`
  - 新增：`WorkbenchViewModel`、`MenuBarModel`、`MainHostStripModel`、`DrawerRingModel`、`DocumentWorkspaceModel`
  - 新增：`MenuAction`、`WorkbenchHostEvent`、`ViewportTextureBridge`
  - 新增：`menu_action_binding`、`dispatch_workbench_binding`
  - 新增：`SlintWorkbenchHost`、`SlintWindowHostManager`
- `zircon_ui`
  - 新增：`UiEventPath`、`UiBindingCall`、`UiBindingValue`、`UiEventBinding`、`UiEventRouter`
- `zircon_editor_ui`
  - 新增：`EditorUiBinding`、`EditorUiBindingPayload`、`EditorUiRouter`
- `zircon_graphics`
  - 保留：`RenderService::submit_frame(...)`
  - 新增或明确：`ViewportFrameTextureHandle`
  - 保持只输出离屏纹理，不感知 `slint`
- preview 工程
  - 输入：layout/view fixtures
  - 输出：本地 URL workbench 壳预览
  - 不进入 `zircon_app` 正式依赖图

## Test Plan

- 预览壳验收
  - 本地 URL 打开后，能看到完整 VSCode.dev 基线壳体：顶部菜单、顶部主壳 tabs、左抽屉、右 rail、底部面板、底部状态栏、中心文档区。
  - `Hierarchy / Inspector / Scene / Game / Assets / PrefabEditor / AssetBrowser` 都能按正确宿主规则显示和切换。
  - 拖拽 `ActivityView` 到中心、回 drawer、拆成 split、拖出成浮窗的行为与 Dockview 参考一致。
- 桌面 Slint 宿主验收
  - `cargo check --workspace`
  - `zircon_app` editor 入口改走 Slint 并成功启动
  - `RuntimePreviewRunner` 继续不依赖 editor UI 框架
  - `Scene/Game` 视口通过 GPU 纹理桥接显示，不再出现 CPU readback 路径
- 单元/集成测试
  - `LayoutManager`、`ViewRegistry`、`ProjectEditorWorkspace` roundtrip 继续通过
  - `WorkbenchViewModel` 从 layout/session 生成 chrome 模型的转换测试
  - `ViewportTextureBridge` 对非法 format/usage 的拒绝测试
  - 菜单模型到 `MenuAction` 的 dispatch 测试
  - `nativeBinding` roundtrip、editor payload roundtrip、inspector batch payload roundtrip
  - headless menu/animation/inspector binding dispatch 测试
  - 预览壳与 Slint 宿主共用 layout fixture 的一致性测试
- 人工验收
  - 顶部菜单视觉和交互接近系统菜单感，不再是按钮排布
  - 顶部主壳带“页签 + 路径”混合结构
  - 左开右收、下方面板、状态栏层级清晰，不再杂乱
  - 独占 `ActivityWindowPage` 激活后只保留菜单和顶部 strip
  - 窗口 resize 后各区重排稳定，viewport 不闪烁

## Assumptions

- V1 明确放弃 `iced` 作为 editor 主宿主；迁移完成后它不再是正式 editor/runtime UI 依赖。
- 浏览器预览只是视觉定稿工具，不是 shipping editor 的运行时外壳。
- V1 菜单先自绘，V2 再补三端原生系统菜单。
- 继续采用“混合固定壳 + 受控 docking 树”，不升级为全壳 universal docking。
- `slint` 技术可行性的依据采用官方文档中 `winit`、`require_wgpu_27`、`wgpu_27` 外部纹理导入路径：
  - [Winit Backend](https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backend_winit/)
  - [BackendSelector::require_wgpu_27](https://docs.slint.dev/latest/docs/rust/slint_interpreter/struct.BackendSelector)
  - [slint::wgpu_27](https://docs.slint.dev/latest/docs/rust/slint/wgpu_27/index.html)
