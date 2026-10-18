# Zircon Hub 响应式组件化重构计划

## Summary
- 范围按你确认的 **Hub 优先** 执行：不做完整通用 MUI 复刻库，先把 Zircon Hub 重构成 MUI-inspired、Slint-native、响应式的组件系统。
- 保留 Slint 1.16.1，不引入 HTML/CSS 引擎；用 Slint `FlexboxLayout`、`HorizontalLayout`、`VerticalLayout`、`min/preferred`、`flex-grow/shrink/basis` 封装 CSS-like primitives。
- 目标是让页面只表达语义区域：shell、header、nav、toolbar、card flow、panel grid、list/table、detail/action 区；页面级不再散落固定 `width/height` 和断点公式。

## Key Changes
- 拆分当前超过 1000 行的 `zircon_hub/ui/components.slint`：
  - 新增 `tokens.slint`：统一颜色、spacing、padding、row height、control height、icon size、panel min width、breakpoint。
  - 新增 `layout.slint`：`Stack`、`Row`、`Column`、`Flow`、`Page`、`PanelGrid`、`ResponsiveSlot`、`Fill`、`Divider`。
  - 新增 `surfaces.slint`、`inputs.slint`、`navigation.slint`、`data_display.slint`、`overlays.slint`，按 MUI 分类承载 Hub 组件。
  - 保留 `components.slint` 作为 re-export 入口，避免一次性改爆所有页面 import。
- Token 默认值固定：
  - spacing scale：`0/4/8/12/16/24/28`。
  - breakpoints：`compact < 1040px`、`medium < 1280px`、`wide >= 1536px`。
  - page padding：28px；panel gap：14px；toolbar gap：10px。
  - atom fixed sizes 只放在 token/原子组件：icon 16/18/20/28/34px、button 34/38/42px、table row 28px、list row 56/72/86px、border 1px。
- Layout primitive 语义固定：
  - `Stack` 用 `FlexboxLayout`，默认 `flex-wrap: nowrap`，用于一维布局；`Row`/`Column` 是固定方向包装。
  - `Flow` 用 `FlexboxLayout` + wrap，承载卡片流、toolbar wrap、metric flow。
  - `PanelGrid` 只负责响应式面板排列，子项必须通过 `ResponsiveSlot` 提供 `flex-basis`、`flex-grow`、`flex-shrink`、`min-width`。
  - `Page` 统一页面背景、滚动、padding、底部留白和 viewport 宽度状态。
- MUI 参考映射：
  - Box/Container/Stack/Grid -> `Page`、`Stack`、`Row`、`Column`、`Flow`、`PanelGrid`。
  - Paper/Card -> `HubPanel`、`HubCard`。
  - AppBar/Drawer/List -> `HubHeader`、`NavRail`、`NavButton`、`InfoRow`。
  - Button/IconButton/Select/TextField/ToggleButton -> `HubButton`、`IconButton`、`ToolbarSelect`、`SearchBox`、`SegmentButton`。
  - Chip/Badge/Table/List/Alert -> `Badge`、`StatusPill`、`DataTable`、`ListPanel`、`StatusBanner`。
- 页面迁移顺序固定：
  - `app.slint`：先收束 shell、top header、nav rail、status area、window controls。
  - `projects.slint`：作为首个验收页，重排为 `PageHeader + Toolbar + ProjectFlow + PanelGrid(RecentProjects, QuickActions)`。
  - `assets.slint`、`plugins.slint`、`learn.slint`：统一为 `CatalogPage/ListPanel`。
  - `cloud.slint`、`team.slint`：统一 metric flow + list panel。
  - `editor.slint`、`builds.slint`、`settings.slint`：统一 workspace panel grid，不再手写每个 flow 的固定高度。
- 页面级约束：
  - 允许：`min-width`、`preferred-width`、`flex-basis`、`flex-grow`、`flex-shrink`、语义 row height 参数。
  - 禁止：页面直接写固定 `width:`/`height:` 做布局、页面散落 `root.width < ...` 断点、业务区域 `x:`/`y:` 定位。
  - 例外只保留在原子层：1px divider、border、图标、按钮、表格行高、popup anchor、窗口拖拽坐标换算。

## Interfaces
- Rust/ViewModel 不新增业务 API；现有 `NavItemData`、`ProjectCardData`、`RecentProjectRowData`、`QuickActionData`、`UiTextData` 等继续作为页面输入。
- Slint 新增公共组件接口：
  - `ResponsiveSlot { basis, min-width, grow, shrink }`
  - `PanelGrid { gap, row-gap, align-items }`
  - `Page { scroll-y, padding, bottom-padding }`
  - `HubButton/IconButton/SearchBox/ToolbarSelect/SegmentButton`
  - `HubPanel/HubCard/PanelHeader/ListPanel/DataTable/StatusBanner`
- `ComponentSamples` 升级为 Hub 内部组件验收页，覆盖 layout primitives、buttons、badges、cards、tables、toolbar、nav、popup、empty/error/loading 状态。

## Test Plan
- 静态合同测试：新增 Hub layout contract 测试，扫描 `zircon_hub/ui/*.slint`，阻止页面级固定 `width/height`、散落断点、业务 `x/y` 定位、字符图标回流。
- 编译验证：
  - `cargo fmt -p zircon_hub --check`
  - `cargo check -p zircon_hub --locked --offline --jobs 1`
  - `cargo test -p zircon_hub --locked --offline --jobs 1`
  - `cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1`
- 视觉验收：
  - 用 `.codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-window.ps1` 捕获真实 Hub 窗口。
  - 用 multi-page 脚本覆盖 Projects dashboard、新建项目、项目浏览、项目详情。
  - 固定检查 1600x1024、1366x820、1100x760、960x640 四档：无重叠、无文本压盖、无状态栏遮挡、toolbar 可换行、panel 可堆叠。
- 回归场景：
  - 长路径、长项目名、空列表、无 source engine、构建中/失败/成功状态、nav collapsed、窗口最小尺寸、popup 打开/外部关闭。

## Assumptions
- 本轮不实现完整 MUI 全组件库，只借鉴 [MUI All Components](https://mui.com/material-ui/all-components/) 与本地 `dev/material-ui/packages/mui-system` 的 Stack/Grid/breakpoint/spacing 思路。
- Slint `FlexboxLayout` 已可用，且当前 `zircon_hub/build.rs` 已启用 experimental flexbox 编译路径。
- 视觉风格保持 Zircon Hub 当前深色工程工具取向，不追求 MUI light theme 像素级复刻。
- 实施时同步更新 `docs/zircon_hub/ui/` 下的组件目录和布局契约文档。
