# ZirconEngine UI 组件化与 Material 样式器重构计划

## Summary
采用 `Taffy + Zircon 自管布局` 混合方案：Flex/Grid/Wrap/SizeBox 走 Bevy 同源 Taffy 思路，Overlay/Canvas/Popup/VirtualList/Editor docking 由 Zircon 自管。组件模型参考 Fyrox retained widget/message，复合 Widget 参考 Unreal UMG，视觉 token 与控件密度参考 Slint Material。

本轮是硬切：编辑器首屏和核心 pane 迁到新 UI v2 schema，旧 TOML/schema 只保留测试迁移入口，不再作为主运行路径。复合组件 v1 不做内嵌脚本，动态逻辑由 Rust callback/presenter 计算并 patch 组件 props/state。

## Key Changes
- 新增 UI v2 asset/schema：`view`、`component`、`style` 三类；节点改为 flat arena graph，字段固定为 `component/classes/props/state/layout/slots/events/children`，禁止递归 `UiTemplateNode` 作为运行时主结构。
- 扩展 `UiComponentDescriptor`：区分 `Primitive/Layout/Composite/EditorOnly`，声明 props/state/slots/events/default_classes/layout_role/render_capabilities。
- 组件目录固定为三层：
  - 元组件：Button、Text、Image、Svg、Canvas、InputField、ProgressBar、RangeField、Popup、Toggle、ComboBox、RadioField 等。
  - 布局组件：ListView、VirtualList、Overlay、FlexBox、HorizontalGroup、VerticalGroup、GridGroup、CanvasBox、SizeBox、Space。
  - 编辑器组件：AssetField/ObjectField、Foldout、TreeView、EditableTable、MessageBox。
- 样式器改为 CSS-like 编译缓存：支持 `Component`、`.class`、`#control_id`、`:hover/:pressed/:focused/:disabled/:checked/:selected`，按 specificity + order 解析到 `UiResolvedStyle`。
- 复合组件按 “TOML prototype 常驻堆缓存 + arena instancing + props/state patch” 工作，不再每次重新反序列化或重建整棵树。

## Implementation Milestones
- **Milestone 1: UI v2 foundation**
  - 在 `zircon_runtime_interface::ui` 定义 v2 descriptor、node arena、style rule、component graph DTO。
  - 在 `zircon_runtime::ui` 实现 v2 loader/compiler/cache，旧 recursive builder 不再服务主链。
  - 所有 build/layout/style traversal 用显式栈，深度 10k fixture 不溢出。

- **Milestone 2: Layout engine**
  - 引入 Taffy 依赖到 runtime 层，只承接 Flex/Grid/Wrap/SizeBox。
  - Overlay/CanvasBox/Popup/VirtualList 继续由 Zircon 自管，输出统一 `UiFrame`、clip、hit-test、render order。
  - `UiSurface` 持久化 surface，只对 dirty props/state/layout/style/text/render domain 增量刷新。

- **Milestone 3: Component catalog**
  - 将现有 showcase catalog 收敛为正式内置 catalog，补齐用户列出的 runtime/editor-only 组件。
  - RangeField、ComboBox、Toggle、Popup、TreeView、Table、MessageBox 通过统一 component event 上报。
  - Rust callback registry 负责 `event route -> editor action -> state patch`，组件 TOML 只声明 route id，不写脚本表达式。

- **Milestone 4: Material style + editor hard cutover**
  - 以 Slint Material 的 palette、typography、metrics 命名重建 `editor_material` token。
  - 重写 editor host/workbench/project overview/inspector/hierarchy/asset browser/console 这些首批 TOML 到 v2 schema。
  - 删除主路径上的旧 schema fallback 和 pointer event 期间整树 rebuild；旧资产只允许在迁移测试中出现。

## Test Plan
- Runtime tests：v2 TOML parse、component slot validation、style specificity、pseudo-state style invalidation、10k 深度 graph build、VirtualList 只实例化 visible + overscan。
- Layout tests：Flex/Grid/Wrap/SizeBox 对齐 Taffy 结果；Overlay/Canvas/Popup 保持 Zircon 自管语义。
- Editor tests：首屏 assets 全部从 v2 cache 构建；pointer click 不触发全量 ProjectOverview rebuild；MessageBox rich text、TreeView 展开、EditableTable 编辑事件能走 Rust callback。
- Guard tests：主 editor projection 禁止调用旧 recursive `UiTemplateTreeBuilder` 和旧 `UiAssetDocument` fallback。

## Assumptions
- 已锁定用户选择：`Taffy混合`、`硬切首屏`、`Rust回调`。
- 不新增第四个 root crate；DTO 在 `zircon_runtime_interface`，实现和缓存归 `zircon_runtime`，editor-only catalog 和 action registry 归 `zircon_editor`。
- 允许大规模改写现有 editor `.ui.toml`，不为旧 schema 提供长期兼容。
