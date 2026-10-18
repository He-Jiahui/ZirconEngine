# Zircon Hub 典型组件样例设计

## Summary
先新增 `zircon_hub/ui/components.slint`，把页面重复手写的布局抽成稳定组件，再迁移页面组合。参考 MUI 的分类思路：Inputs、Data display、Surfaces、Navigation、Layout；其中 List 是“连续的文本或图片垂直索引”，Card 是“单一主题的内容与操作表面”，Stack 用于一维水平/垂直排列。参考来源：[MUI All Components](https://mui.com/material-ui/all-components/)、[MUI List](https://mui.com/material-ui/react-list/)、[MUI Card](https://mui.com/material-ui/react-card/)、[MUI Stack](https://mui.com/material-ui/react-stack/)。

## Component Catalog
- `HubPanel`
  - 用途：统一面板表面，替代散落的 `Panel`。
  - 固定规则：`border-radius: 8px`、统一背景/边框、`clip: true`、默认 padding 16/18px。
  - 变体：`default / elevated / interactive / danger`，只影响 hover、边框和背景，不改变尺寸。

- `PanelHeader`
  - 用途：标题 + 右侧徽章/按钮的稳定布局。
  - 结构：左侧 `title/subtitle`，中间 `horizontal-stretch: 1`，右侧 `Badge`、`PillButton` 或 `IconButton`。
  - 规则：高度固定 42 或 52px；右侧永远贴右，不允许漂到中间。

- `SearchBox`
  - 用途：Projects 搜索框。
  - 结构：左图标 + `LineEdit`。
  - 规则：固定高度 42px，默认宽 330px，可配置宽度；焦点只改变边框颜色；输入内容 `overflow: elide`，不挤压相邻工具按钮。

- `InfoRow`
  - 用途：左右靠边排列的通用列表行，覆盖最近项目、Cloud 服务、Team 成员、引擎记录。
  - 结构：左侧 thumbnail/icon 44px，中间 `title/detail/meta`，右侧 `Badge / StatusBadge / IconButton / ActionArrow`。
  - 规则：行高 56/72/88 三档；中间文本 `horizontal-stretch: 1`；右侧区域固定宽度，永远贴右。

- `ActionRow`
  - 用途：Quick Actions。
  - 结构：左图标 32px，中间标题+说明，右侧箭头按钮 28px。
  - 规则：箭头固定在最右；标题和说明只能 elide，不能推挤箭头；禁用态降低颜色但保持布局尺寸。

- `Badge` / `StatusBadge`
  - 用途：版本、平台、状态、计数。
  - 规则：小号 `52x24`，中号 `96/128x30`，不随文本变宽导致布局跳动；长文本 elide。
  - 状态色：ok 绿、warning 黄、error 红、info 蓝、accent 青绿、neutral 灰。

- `ScrollablePanel`
  - 用途：内部列表超出时只滚动面板内部。
  - 结构：`HubPanel` + `PanelHeader` + 内部 `ScrollView`。
  - 规则：外层页面不因内容增高被撑爆；`viewport-y` 只属于内部滚动区域；底部状态栏不被覆盖。

- `TwoColumnSurface`
  - 用途：Settings、Editor、Builds 这类两列固定工作区。
  - 规则：左右列间距 14px；左列固定或比例宽，右列 `horizontal-stretch: 1`；每列内部用 `ScrollablePanel` 或固定面板，不互相压住。

- `HeaderGroup`
  - 用途：app 顶栏元素分组。
  - 结构：品牌组、引擎选择、状态胶囊组、工具按钮组、用户组、窗口按钮组。
  - 规则：顶栏高度 66px，所有元素垂直居中；窗口按钮固定 36px；状态胶囊不挤压品牌和窗口按钮。

## Migration Plan
- 第一阶段只新增组件样例，不重做全部页面：在 `components.slint` 中实现组件，并做一个临时 `ComponentSamples` 页面或隐藏测试区域用于视觉验收。
- 第二阶段迁移 `projects.slint`：`ProjectCard` 使用 `HubPanel + Badge`，最近项目表行使用 `InfoRow`，Quick Actions 使用 `ActionRow`，搜索使用 `SearchBox`，底部 `Open Path` 放进固定 footer 区。
- 第三阶段迁移 `settings.slint`、`cloud.slint`、`team.slint`：Settings 使用 `TwoColumnSurface + ScrollablePanel`；Cloud/Team 的服务行和成员行统一用 `InfoRow`，状态徽章固定贴右。
- 第四阶段回看 `app.slint`、`editor.slint`、`builds.slint`：顶栏迁移到 `HeaderGroup`；Editor/Builds 的引擎记录、构建步骤可逐步复用 `InfoRow` 和 `StatusBadge`。

## Test Plan
- 编译检查：`cargo check -p zircon_hub --locked --offline --jobs 1`。
- 构建验收：`cargo build -p zircon_hub --locked --offline --jobs 1`。
- 截图验收页面：Projects、Settings、Cloud、Team、Editor、Builds。
- 重点检查：
  - 顶栏元素垂直居中。
  - 搜索框不裁剪、不漂移。
  - Quick Actions 箭头固定最右。
  - `Open Path` 不碰底部状态栏。
  - Cloud/Team 状态徽章靠右。
  - 面板内容越界时内部滚动，不压住其它面板。

## Assumptions
- 本轮先做 Slint 组件系统和典型样例，不新增 Rust 数据模型。
- `shared.slint` 继续保留数据结构和低级基础控件；新布局组件放入 `components.slint`。
- 页面迁移以视觉稳定优先，保持现有业务回调和数据绑定不变。
