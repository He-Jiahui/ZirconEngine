# Runtime UI 组件库与 Slint Material Showcase Cutover 计划

## Summary

- 以“完整交互可验收”作为 V1：展示窗口里的字段、下拉、折叠、数组、Map、拖拽数值、资产/实例拖入都走真实 Runtime UI 语义与事件，不做纯视觉占位。
- `.ui.toml` 是 UI 结构、组件、样式、示例窗口的唯一业务真源；Slint 只保留为通用 Material Host 渲染层。
- 渲染模型保持 retained/static：`.ui.toml -> UiSurface -> render extract/projection -> Slint host`，用户输入通过事件队列更新状态并触发 dirty/recompute，不引入 IMGUI 即时重绘模式。
- 第一阶段新增一个编辑器 Activity Window：`editor.ui_component_showcase`，用于集中展示组件库、交互状态、事件日志和 Unity/Rider 风格 inspector 控件。

## Key Changes

### Runtime UI 组件契约

- 在 `zircon_runtime::ui` 增加组件描述注册表，统一声明每个组件的：
  - 组件 id、分类、默认 props/layout/style role
  - typed props schema、state schema、slot schema
  - 支持的事件类型和事件 payload
  - 可接收的 drag/drop payload 类型
- 新增共享 typed value/event 契约：
  - `UiValue`: bool、int、float、string、color、vec2/vec3/vec4、asset ref、instance ref、array、map、enum/flags。
  - `UiComponentEvent`: value changed、commit、focus、begin drag、drag delta、end drag、open/close popup、select option、toggle expanded、add/remove/move item、drop reference。
  - `UiValidationState`: normal、warning、error、disabled，带 message。
- 现有 `UiRenderCommandKind` 继续以 `Group/Quad/Text/Image` 为底层绘制原语；组件语义通过 render extract/projection metadata 暴露给 host，避免把每个控件硬编码成独立绘制命令。

### V1 组件库

- 基础展示类：
  - Label、Rich Label、Image、Svg Icon、Separator、Progress Bar、Spinner、Badge/Tag、Help/Info Row。
- 输入类：
  - Button、Icon Button、Toggle Button、Checkbox、Radio、Segmented Control。
  - InputField/TextField。
  - NumberField：支持键入、提交、上下/左右拖拽改值、step、large step、min/max clamp、非法输入提示。
  - RangeField：支持 min/max/value/step，可和 NumberField 共享数值 schema。
  - ColorField、Vector2/3/4 Field。
- 选择类：
  - Dropdown/ComboBox：支持单选、多选、flags、多状态特殊选项、禁用选项、可选搜索。
  - EnumField、FlagsField、SearchSelect。
- 引用类：
  - AssetField：可拖入资产、清空、定位、打开、显示拒绝原因。
  - InstanceField/ObjectField：可拖入场景实例、资产实例或允许的引用类型。
- 结构/数据类：
  - Group/Foldout：标题、右侧状态、折叠/展开状态。
  - PropertyRow、InspectorSection。
  - ArrayField：根据元素类型生成子字段，支持增删改、移动、空状态。
  - MapField：根据 key/value 类型生成键值字段，校验 key 唯一性。
  - List/Tree Row、Context Action Menu。

### `.ui.toml` 与 Showcase 资产

- 新增组件展示窗口资产：
  - `zircon_editor/assets/ui/editor/component_showcase.ui.toml`
  - `zircon_editor/assets/ui/editor/component_widgets.ui.toml`
  - 必要时新增 Material 风格主题资产，例如 `zircon_editor/assets/ui/theme/editor_material.ui.toml`
- Showcase 布局采用 Rider/Unity inspector 参照：
  - 左侧：组件分类导航。
  - 中部：ScrollableBox 展示所有组件，按 Visual、Input、Numeric、Selection、Reference、Collections、Feedback 分组。
  - 右侧：当前选中组件状态、事件日志、验证状态、drag/drop payload 预览。
- 示例窗口自身不写业务 `.slint`；所有展示内容由 `.ui.toml` 声明，Slint 只接收 Runtime UI projection。

### Editor Slint Host

- 将 Slint host 样式切到 Material；实现前确认当前 Slint 版本支持的 `material` style 名称，若本地 API 有差异，采用 Slint 1.16 支持的 Material 等价路径。
- 扩展 generic host projection，使 `ViewTemplateNodeData` 能表达：
  - control role、value state、validation state、popup state、selection state、drag/drop state、disabled/focused/hover/pressed state。
- Slint 侧只扩展通用 primitive 渲染能力：
  - 文本、图片、图标、按钮外观、输入框外观、滑条、列表行、弹出候选层、折叠组。
  - 不在 `.slint` 内写 Component Showcase 的结构、字段列表或业务交互。
- Editor host 维护 showcase 的 transient demo state：
  - 当前数值、输入文本、dropdown 选择、array/map 内容、foldout 展开状态、模拟拖入引用、事件日志。

## Public Interfaces / Types

- 新增或扩展 Runtime UI 公共接口：
  - `UiComponentDescriptor`
  - `UiComponentDescriptorRegistry`
  - `UiPropSchema`
  - `UiValue`
  - `UiComponentState`
  - `UiComponentEvent`
  - `UiDragPayload`
  - `UiDropPolicy`
  - `UiValidationState`
- `.ui.toml` native node 的 `props` 保持向后兼容，但允许组件声明 typed props，例如：
  - `value`, `min`, `max`, `step`, `options`, `multiple`, `accepts`, `expanded`, `element_type`, `key_type`, `value_type`
- Editor 侧新增 window id/content kind：
  - `editor.ui_component_showcase`
  - 对应 Activity Window descriptor、默认标题、默认 docking 约束和菜单/命令入口。

## Test Plan

- Runtime tests:
  - `.ui.toml` 能解析并编译所有 V1 组件示例。
  - 组件注册表包含 V1 必需组件，默认 props/layout 合法。
  - Showcase asset 编译后能生成 retained `UiSurface` 和 render extract。
  - NumberField 拖拽、键入提交、min/max clamp、非法输入验证正确。
  - Dropdown 单选、多选、禁用选项、特殊选项事件正确。
  - Group 展开/折叠、Array 增删改移、Map key 唯一性校验正确。
  - AssetField/InstanceField 接收与拒绝 drag payload 的规则正确。
- Editor tests:
  - `editor.ui_component_showcase` 能注册为 Activity Window 并加载对应 `.ui.toml`。
  - Slint projection 能携带组件 role、state、validation、popup、drag/drop metadata。
  - Showcase 不依赖业务 `.slint` 文件；`.slint` 仅作为 generic host。
  - 现有 UI Asset Editor fixture 和已有 `.ui.toml` 资产继续编译通过。
- Validation commands:
  - 运行 runtime UI 相关 cargo tests。
  - 运行 editor UI/host/template runtime 相关 cargo tests。
  - 如 Slint host 或 build script 改动较大，再运行 editor 集成契约测试。

## Assumptions

- V1 聚焦编辑器屏幕空间 UI，不把 world-space UI、复杂虚拟列表、大型数据分页作为首批目标。
- 组件真实数据绑定先使用 editor showcase transient state；后续再接入通用属性反射、资产编辑和 inspector 数据源。
- 资产/实例拖入先实现 typed payload 与接受/拒绝语义；真实资源浏览器、场景树联动在已有 editor 数据源上逐步接入。
- 现有 `workbench.slint` 可以继续作为 generic host bootstrap，但不得重新成为业务 UI 真源。
