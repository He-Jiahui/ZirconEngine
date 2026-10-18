# ZirconEngine 宿主编辑器 UI 基础能力计划

## Summary
目标是把宿主编辑器 UI 做成“基本可渲染、可布局、可响应输入”的真实运行层，而不是只还原 `E:/Git/ZirconEngine/docs/ui-and-layout/ai-workbench-style/component-prototype/index.html` 的静态页面。

设计主参考明确采用 `dev/UnrealEngine` 的 Slate 思路：
- `SWidget / SPanel / FChildren` -> Zircon 的组件节点、Slot、组件描述符
- `FGeometry / FArrangedChildren / FWidgetPath` -> Zircon 的布局结果、命中路径、焦点路径
- `FSlateApplication / GenericApplicationMessageHandler` -> Zircon 的统一输入管理层
- `FReply` -> Zircon 的事件响应与副作用模型
- `DrawElements / PaintArgs` -> Zircon 的渲染提取与 Painter 管线

关键差异：布局不照搬 Slate 每个 Panel 手写排列，Flex/Grid/Block/Wrap 统一交给 Taffy；Overlay、Canvas、Scroll、Virtual 等特殊容器保留 Zircon 自己的布局路径。

## Key Changes
- 在 `zircon_runtime` / `zircon_runtime_interface` 收束基础 UI 运行接口：组件树、Slot、布局 Style、Geometry、ArrangedTree、HitPath、RenderExtract、DispatchReply。
- 保留现有 Taffy 依赖与 `zircon_runtime/src/ui/layout` 管线，把 Flex、Grid、Block、Wrap 明确设为 Taffy 权威布局；失败时记录 fallback reason，不能静默退回。
- 建立 Slate 风格输入管理层：平台事件先归一化为 Zircon UI 事件，再按命中路径、焦点路径、捕获目标、弹窗层级进行路由。
- 鼠标、键盘、触摸、文本输入、IME、拖拽、弹窗关闭、Tooltip 计时都走同一个输入管理入口。
- 引入 Painter 样式选择器，按钮不再在 `template_buttons` 里直接硬编码 hovered / pressed / disabled / focused 分支。
- 设计统一状态模型：normal、hovered、pressed、focused、disabled、checked、selected、open、dragging、drop-hover、loading。
- Button、IconButton、Toggle、Checkbox、Radio、Slider、Dropdown、PopupRow、Tooltip 等组件都通过 `UiPainterStyleSelector` 选择最终视觉样式。
- 常用组件先覆盖 Workbench 需要的集合：Container、HorizontalBox、VerticalBox、GridBox、Overlay、ScrollBox、SizeBox、Label、Icon、Button、IconButton、Field、Checkbox、Radio、Toggle、SegmentedControl、Slider、Dropdown、Tab、ListRow、TreeRow、TableRow、PopupMenu、Tooltip、Toast、TopToolbar、ActivityRail、SceneTreePanel、InspectorPanel、ViewportPanel、StatusBar。
- 使用现有 `.zui` 组件资产和 component catalog 作为组件来源，不新建一套平行 UI 系统。

## Milestones
1. **Unreal 参考映射与接口冻结**
   - 产出 Slate 到 Zircon 的映射表，固定 Widget、Panel、Geometry、Reply、InputManager、DrawElement 对应关系。
   - 明确哪些行为模仿 Unreal，哪些由 Zircon + Taffy 替代。

2. **布局与渲染基础层**
   - 完成组件树到 Taffy 布局的稳定路径。
   - `UiSurfaceFrame` 同时驱动布局、命中测试、渲染提取，避免三套状态分裂。
   - Overlay、Canvas、Scroll、Virtual 容器只在明确原因下走自有布局。

3. **输入管理层**
   - 统一平台输入入口，参考 Unreal 的 preview/tunnel、bubble、direct、focus-path、capture routing。
   - 鼠标支持 hover、press、release、double click、wheel、capture、drag。
   - 键盘支持 focus、Tab 导航、Enter/Space 激活、Escape 关闭弹窗、文本输入。
   - 触摸支持 pointer id、start/move/end/cancel，并与鼠标路径共享核心路由。
   - `UiDispatchReply` 负责处理 focus、capture、drag/drop、popup、tooltip、host request 等副作用。

4. **Painter 样式选择器**
   - 新增共享状态选择器，先改 Button，再扩展到 IconButton、Selection Controls、Slider、Dropdown、Popup。
   - Button 的响应视觉必须由 selector 决定，按钮逻辑只产出语义状态。
   - Native painter 与 runtime render extract 使用同一套状态优先级。

5. **Workbench 组件落地**
   - 把 prototype 中的常用区域映射到 `.zui` 组件和 runtime component descriptor。
   - 先保证基本可见、可布局、可点击、可聚焦、可输入，不要求完整像素级还原。
   - 宿主编辑器窗口从组件化 Workbench surface 启动，保留后续视觉细化空间。

## Test Plan
- 布局：Flex、Grid、Wrap、Block、嵌套 Slot、stretch/alignment、collapsed child、非法尺寸 fallback、Overlay/Scroll 特殊路径。
- 渲染：Button、IconButton、Checkbox、Radio、Toggle、Slider、Dropdown、PopupMenu、Tooltip、Toast 的 render extract 与 native painter 输出。
- 输入：鼠标 hover/press/release/capture，键盘 focus/activation/navigation，文本输入与 IME，触摸多 pointer，拖拽开始/结束，弹窗外点关闭。
- 组件契约：`.zui` 模板、component descriptor、slot schema、事件语义、dirty domain 更新。
- 验证命令以 Rust focused tests 为主，再跑 prototype 下现有交互/组件契约脚本；视觉验收只要求非空、结构正确、主要控件可交互。

## Assumptions
- 当前阶段不追求完整复刻 HTML prototype 的页面布局，只要求宿主编辑器具备真实基础 UI 能力。
- Unreal 是架构参考，不复制 C++ 宏体系、UMG 层或 Slate 的手写布局实现。
- Taffy 是 Zircon 常规布局权威；特殊容器保留 Zircon 自有实现。
- 在现有 `main` 工作区上推进，尊重当前未提交文件，不回滚用户已有改动。