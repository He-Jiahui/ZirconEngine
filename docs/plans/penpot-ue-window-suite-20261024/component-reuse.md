# 公共组件与全局样式交接

本约定描述新窗口应如何复用现有能力；不表示组件注册项已经具备完整的业务编辑行为。

## 已有权威入口

| 入口 | 已确认内容 | 集成要求 |
| --- | --- | --- |
| `zircon_editor/assets/ui/editor/components/catalog.toml` | 版本化 component ID、document、binding namespace、slots | 引用既有身份和 slot；新增共享组件先明确 owner |
| `zircon_editor/assets/ui/editor/theme/editor_tokens.zui` | palette、typography、controls、chrome、density | 新窗口优先使用共享语义值；自定义样式需要验证传播与重置 |
| `zircon_editor/assets/ui/editor/unreal/starship.zui` | 引入公共 tokens 的 UE 样式及控件状态 | 当前还含 UE 专用 token 值；不能假定所有值已与全局 token 单一归属 |
| `tools/penpot/apps/zircon-zui-plugin/src/penpot-native-components.ts` 及 ownership/copy 模块 | Penpot 原生组件、实例和归属相关实现 | 先查实际导出合同；不以外观相同的 detached 副本代替实例复用 |

## 复用候选

| 层 | 现有候选身份 | 使用前须核实 |
| --- | --- | --- |
| 窗口结构 | `UiHostWindow`、`DocumentHost`、`ActivityDrawerWindow`、`MenuBar`、`StatusBar` | 实际宿主、Dock/页签生命周期及命令绑定 |
| 区域组合 | `WorkbenchPanelHeader`、`WorkbenchPropertyEditorRow`、`WorkbenchDiagnosticRow` | slot 内容、状态及文本/布局约束 |
| 输入与数据 | `WorkbenchButton`、`WorkbenchIconButton`、`WorkbenchField`、`WorkbenchNumberField`、`WorkbenchDropdown`、`WorkbenchTreeRow`、`WorkbenchListRow` | 焦点、键盘、选择、禁用、提交、撤销及可访问性 |
| 专用表面 | `WorkbenchPreviewViewport`、`WorkbenchTimelineStrip`、Animation Sequence/Graph host bodies | 是否绑定真实渲染/时间轴/编辑数据；静态框架不等于完整编辑器 |

图编辑、时间轴、曲线、资产浏览和骨骼树的能力按真实实现继续核查。未查证的组件不写“已支持”。

## 每个窗口的组件记录

为重复控件记录 Penpot library/component ID、实例/变体、ZUI component ID/document、slot/参数、状态、样式 token、版本和本窗口的局部覆盖。窗口负责组合和数据，公共组件负责通用行为与样式。

全局样式修改需要同时验证 Penpot 和 Editor 中至少两个受影响窗口族；局部覆盖、恢复默认和持久化按实际支持验证。不能把“存在 token 文件”写成“全局设置已实现”。

## 一比一还原的记录

每个开放差异使用稳定编号，记录窗口/状态/DPI、UE 依据、Penpot 与 Editor 图像、差异、已证实原因或未知、依赖 owner、待用户方案和状态。真实字体、图标、间距、缩进、居中、裁剪、滚动和业务交互分别记录；不使用一个相似度分数覆盖未完成项。

统一操作流程见 [UE/Penpot fidelity](../../../.codex/skills/zircon-project-skills/zr-ui-layout-reference/references/ue-penpot-fidelity.md)。
