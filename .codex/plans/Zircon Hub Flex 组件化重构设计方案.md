# Zircon Hub Flex 组件化重构设计方案

## Summary
以用户提供的参考图作为唯一视觉基准，把 Hub UI 收敛为“组件库先行 + 页面组合”的 Slint Flex/相对布局系统。目标是：窗口可缩放、可拖拽，页面不使用业务级绝对坐标，所有图标改为 SVG 资源，Projects 页恢复参考图中的四列卡片、双行工具栏、Recent Projects 表格、Quick Actions 面板和底部组件验证区。

## Key Changes

- 新建/扩展 Hub 组件库：
  - `HubShell`：无边框窗口主体、顶部栏、侧边栏、内容区、状态栏。
  - `HeaderBar` / `HeaderGroup`：品牌、引擎下拉、状态胶囊、工具按钮、用户区、窗口按钮，全用 `HorizontalLayout/FlexboxLayout` 对齐。
  - `NavRail` / `NavItem`：左侧导航，图标+文本统一基线，折叠态也用布局居中，不用 `x/y`。
  - `Toolbar` / `ToolbarGroup`：搜索、筛选、排序、视图切换、导入、新建按钮按参考图分左右两组，窄窗口时允许换行。
  - `HubPanel` / `PanelHeader` / `ScrollablePanel`：统一面板、标题栏、右侧按钮/徽章、内部滚动。
  - `ProjectCard`：参考图样式的竖向卡片，封面在上，标题/路径/时间/标签在下。
  - `DataTable` / `TableRow`：Recent Projects 改为表格布局，列宽按比例伸缩，行内图标、文本、更多按钮稳定对齐。
  - `ActionRow`：Quick Actions 的左图标、中间标题描述、右箭头固定贴右。
  - `Badge` / `StatusPill` / `IconButton` / `PillButton` / `DropDownButton` / `SearchBox`：按钮、标签、下拉、状态全部统一组件。

- 全部图标 SVG 化：
  - 禁止 UI 里继续用 `"+"`、`">"`、`"[]"`、`"::"`、`"=="`、`"v"`、`"!"`、`"?"` 等字符当图标。
  - 新增 SVG 图标集：`chevron-down`、`chevron-right`、`plus`、`download/import`、`grid`、`list`、`folder`、`sort`、`bell`、`help`、`settings`、`minimize`、`maximize`、`close`、`more-vertical`、`refresh`、`collapse`。
  - 扩展按钮组件接口，从 `icon: string` 迁移到 `icon-image: image` + `has-icon-image`，字符只允许作为开发期 fallback，不作为正常渲染路径。

- Projects 页面按参考图重排：
  - 顶部标题区保留在内容页上方，右侧放 Import Project / New Project。
  - 工具栏拆成两行语义：左侧 SearchBox，右侧 filter/sort/view buttons；窗口变窄时用 Flex 换行。
  - 项目卡片区使用 `FlexboxLayout`：卡片 `flex-basis` 约 296px，`min-width` 260px，`flex-grow: 1`，宽屏显示 4 张，窄屏自动降为 3/2/1 张。
  - `Show More` 只控制卡片区可见行数；默认一行，展开后显示更多行并在卡片区内部滚动或扩展到上限。
  - 下方使用 `TwoColumnSurface`：Recent Projects 左侧 `flex-grow: 1`，Quick Actions 右侧 `preferred-width` 约 420px，窄窗口时可垂直堆叠。
  - Recent Projects 改为真正表格：Name / Engine Version / Last Modified / Location / Action，各列相对伸缩。
  - “Open Path / View All Projects” 放在 Recent Projects 面板底部或 Header 右侧，按参考图优先采用底部左侧按钮，但不碰状态栏。

- 移除业务级绝对定位：
  - `app.slint` 中顶部拖拽层改为 HeaderBar 内的 `DragRegion` 组件，不再用覆盖式 `x: 0/y: 0`。
  - 底部状态栏文本改为 `HorizontalLayout` padding，不再用 `x/y`。
  - 引擎下拉改为 `PopupWindow` 挂在 `DropDownButton` 内，相对按钮定位；不再在窗口根部写固定 `x/y`。
  - `InfoRow` 右侧徽章/箭头改为 `HorizontalLayout` 固定尾部槽，不再用 `parent.width - self.width`。
  - `ProjectCover` 内品牌标记改为嵌套布局居中/贴边，不再用 `x: 3/y: 3`。

- 组件验证先行：
  - 新增 `ComponentGallery` / `DesignSystemPreview` 页面，用参考图底部 Button States 的方式展示按钮、页签、卡片、表格、工具栏、下拉、状态栏、文本标题、滚动面板。
  - 组件验证页可作为隐藏开发页或 Settings 内入口，不进入默认用户流程。
  - 每个组件先在 Gallery 中验证，再迁移 Projects / Settings / Cloud / Team / Editor / Builds。

## Interfaces

- Slint 组件接口调整：
  - `IconButton`, `PillButton`, `ToolbarSelect`, `StatusPill`, `NavButton` 增加 `icon-image: image`、`has-icon-image: bool`。
  - 新增 `DropDownButton`，接收 `label`、`leading-icon-image`、`chevron-icon-image`、`opened`。
  - 新增 `DataTable` / `TableColumnHeader` / `ProjectTableRow`，Recent Projects 不再由 `InfoRow` 假装表格。
  - 新增 `FlexProjectGrid`，只负责卡片流式换行和 Show More 状态。
  - `ProjectCard` 改为竖向卡片组件，封面比例固定，文本区域 elide，标签行不改变卡片高度。

- Rust/ViewModel 调整：
  - 现有 `NavItemData`、`QuickActionData`、`HeaderStatusData` 继续使用 `icon-image` 字段。
  - 补齐 UI 图标资源映射，所有按钮/工具栏/窗口按钮都从 Rust 或 Slint 静态 `@image-url` 获得 SVG。
  - 不新增业务数据模型，除非表格列需要新增可显示字段；Recent Projects 仍复用现有 `RecentProjectRowData`。

## Test Plan

- 静态检查：
  - 搜索 `zircon_hub/ui/*.slint`，确认业务页面没有 `x:` / `y:` 布局定位；允许 `PopupWindow` 内相对锚点和 SVG 内部坐标。
  - 搜索 `icon: "..."` 和纯符号 `Text`，确认正常 UI 不再使用字符图标。

- 构建验证：
  - `cargo fmt -p zircon_hub --check`
  - `cargo check -p zircon_hub --locked --offline --jobs 1`
  - `cargo test -p zircon_hub --locked --offline --jobs 1`
  - `cargo build -p zircon_hub --locked --offline --jobs 1`

- 视觉验收：
  - 截图 Projects、Settings、Cloud、Team、Editor、Builds、ComponentGallery。
  - 检查 1320x820、1600x900、1100x760、900x720 四档窗口。
  - Projects 宽屏应接近参考图：四张竖向项目卡、Recent 表格左、Quick Actions 右、顶部工具栏垂直居中。
  - 窄窗口应自然换行/堆叠，不重叠、不裁剪、不压住底部状态栏。
  - 拖拽窗口、最小化、最大化、关闭保持可用。

## Assumptions

- 参考图是最终视觉基准，当前横向大项目卡需要改回参考图中的竖向卡片。
- “全部 Flex 布局”指业务 UI 不用手算像素坐标；固定尺寸图标、按钮高度、卡片最小宽度、SVG path 坐标允许存在。
- ComponentGallery 用于先验证组件体系，不一定默认显示给最终用户。
- SVG 图标采用项目内自绘简洁线性图标，不引入外部图标库。
