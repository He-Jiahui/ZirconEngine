# Material UI 全组件样式设计与验证计划

## Summary
以本地 `dev/material-ui/packages/mui-material` 的 `@mui/material` v9.0.1 源码和 [MUI All components](https://mui.com/material-ui/all-components/) 为参考，先完成“全组件样式设计表”，再按 Zircon retained UI 能力逐批实现样式和验证。

第一轮产物按你选择：不先做网页/PNG 图册，而是先产出完整设计文档表格。表格覆盖 MUI 公开组件族，并标明每个组件在 Zircon 中的视觉形态、状态、交互反馈、验证方式和落地优先级。

## Key Changes
- 建立 `MUI -> Zircon` 组件设计矩阵：
  - 来源：`dev/material-ui/docs/data/material/components/*` 作为公开组件目录，`dev/material-ui/packages/mui-material/src/*` 作为行为/样式源码参考。
  - 每行记录：MUI 组件名、源码路径、MUI variants/states、Zircon 对应 primitive/role、样式 tokens、默认/hover/pressed/focused/selected/disabled/error/open 状态、交互反馈、验证项。
  - 非视觉工具类如 `NoSsr`、`Portal`、`ClickAwayListener`、`useMediaQuery` 标为 behavior/utility，不强行画成控件。

- 设计分组按完整 MUI 组件族推进：
  - Inputs：Button、ButtonGroup、IconButton、Fab、TextField、Input、Select、Checkbox、Radio、Switch、Slider、ToggleButton、Autocomplete、Rating。
  - Data Display：Chip、Divider、Icon、SvgIcon、List、Table、Tooltip、Typography、Avatar、Badge、ImageList。
  - Feedback：Alert、Backdrop、Dialog、Modal、Popover、Popper、Progress、Skeleton、Snackbar。
  - Surfaces：Accordion、AppBar、Card、Paper、Drawer。
  - Navigation/Layout：Breadcrumbs、BottomNavigation、Menu、Tabs、Pagination、Stepper、Container、Grid、Stack、Box、Masonry。
  - Utils/Transitions：Collapse、Fade、Grow、Slide、Zoom、ClickAwayListener、Portal、NoSsr、CssBaseline、InitColorSchemeScript、useMediaQuery。

- Zircon 样式设计统一成 Editor dark Material：
  - Button：contained/primary、secondary、outlined、text/underline、disabled、loading、hover、pressed、focus-visible。
  - TextField：outlined、filled、standard，下划线态、错误态、helper text、multiline、select-mode。
  - Switch/Slider：track/thumb/value-label/marks/checked/drag/focus/disabled。
  - Select/Menu/Dialog/Tooltip/Drawer：open/closed、anchor/placement、backdrop、paper/elevation、focus route。
  - Table/List/Chip/Accordion/Pagination/Tabs：selected、hover、pressed、focus、disabled、expanded、active page。

- 验证标准采用“视觉 + 交互双验证”：
  - 所有视觉组件至少有默认状态和核心状态的样式/像素或快照验证。
  - 所有可交互组件增加 click/hover/press/focus/open/close/selected/checked 的 retained event route 或状态反馈验证。
  - 暂不要求完整 MUI ripple 动画；先验证状态层和触发反馈图示例。

## Interfaces
- 不直接复制 React/MUI API；只吸收稳定概念：`variant`、`color`、`size`、`disabled`、`focused`、`selected`、`checked`、`open`、`placement`、`anchor`、`elevation`。
- 优先复用现有 Zircon 字段：`button_variant`、`surface_variant`、`text_tone`、`validation_level`、`hovered`、`pressed`、`focused`、`selected`、`checked`、`disabled`、`popup_open`、`corner_radius`、`border_width`。
- 如某组件确实缺表达能力，先在设计表标记“needs support”，不要在第一步发明新 schema。

## Test Plan
- 设计表完整性测试：确保 MUI 公开组件目录中的每个组件族都有 Zircon 设计行、状态覆盖和验证策略。
- Theme contract 测试：冻结 Material tokens、variants、state priority、radius、border、tone、surface/elevation 约定。
- Painter/command 测试：覆盖 default/hover/pressed/focus/selected/checked/disabled/error/warning/success/info/open/backdrop/elevation 的颜色、边框、圆角。
- Showcase/interaction 测试：对 Button、TextField、Switch、Slider、Select、Toggle、Checkbox、Menu、Dialog、Tooltip、Drawer、Pagination 等交互组件验证点击反馈、打开/关闭、选择、切换、拖动和焦点反馈。
- 最终阶段运行 scoped checks：`cargo check -p zircon_editor --lib --tests --locked`，以及组件相关 focused tests；若当前 `Cargo.lock` 仍不一致，记录为外部阻塞，不自动改锁文件。

## Assumptions
- 第一轮先写完整样式设计表，不生成网页/PNG 图册，不直接改 Editor Showcase。
- 目标是 Material-inspired Editor dark 风格，不追求像素级复制 MUI light theme。
- MUI 源码只作为行为和视觉状态参考，Zircon 仍保持 retained host、`.ui.toml`、Rust painter/GPU command 的边界。
- 完整 MUI 范围很大，实施必须按组件族分批；每批完成后再进入测试阶段。
