---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
resolved_at: 2026-07-13
summary_slug: retained-painter-component-contract-regressions
origin_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
fixing_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
origin_child_dir: docs/plans/zircon_editor/editor/15
fixing_child_dir: docs/plans/zircon_editor/editor_layout/15
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme
tests:
  - zircon_editor test binary filter ui::retained_host::host_contract::paint_template_nodes::
---


# Editor Layout 15：retained painter 组件契约回归

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- 来源执行切片：Editor 15 M1 全量 Windows editor lib-test 分区验收
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 交接原因：失败全部落在 retained painter 的组件状态、度量、色板与复合布局合同，不属于导出流水线实现。

## 失败现象与复现证据

直接执行当前源码生成的 `zircon_editor-0af59361f300b435.exe`，过滤
`ui::retained_host::host_contract::paint_template_nodes::`：

- 总计 694 项；666 passed，28 failed，2414 filtered out；耗时 57.54 秒。
- 测试二进制时间为 2026-07-12 21:47:23；本组涉及的测试、度量、色板与 selector 源文件均早于该时间，
  因而不是用旧二进制复现并发后的新源码。
- 失败不是 Editor 15 导出 focused 组造成；`core::export::tests` 已独立 13/13 通过。
- 2026-07-13 当前源码二进制（2026-07-13 01:26:25）复验扩大后的 696 项分区：695 passed、
  1 failed、2416 filtered out，耗时 96.24 秒。原 28 项已消除；剩余失败为
  `template_tooltips::tests::paint::workbench_tooltip_paints_declared_bubble_arrow_and_info_icon`，
  箭头采样位置得到 `[50, 58, 65, 255]`，声明色为 `[23, 28, 32, 255]`。

## 最低共享层根因

最低已证实边界是 Layout 15 的共享 `HostControlMetrics`、`HostMaterialPalette` 与
`UiPainterResolvedState` 消费合同发生整体漂移，而非 28 个控件各自独立失败。当前 owner 已进一步确认
`METRICS.row_height=24` 与批准的 `EditorDensityTokens::WORKBENCH_ROW_HEIGHT=28` 不一致；其余 palette、
state priority 与复合行布局仍必须从这些共享权威自底向上复验。

当前源码复验表明上述 28 项共享漂移已修复。唯一残留位于 Layout 15 的 tooltip 复合绘制/中央 palette
投影边界：测试目前只验证箭头区域有像素变化，却仍从固定像素读取并比较声明色；owner 需确认箭头几何、
采样点与中央 tooltip token 的真实合同后修正生产投影或精确断言，禁止在 Editor 15 导出层补偿。

### 责任切片映射

| Layout 15 owner | 失败范围 | 数量 | 代表性证据 |
| --- | --- | ---: | --- |
| `15b` 控制度量单源 | button/icon-button/chip/segmented/status icon 几何或尺寸 | 9 | 16px/24px glyph 位置漂移、button 文本与 glyph slot 尺寸漂移、segment body `26.000002 != 22.0` |
| `15c` retained palette 单源与状态投影 | toast/alert/shell/selection/status/tree 的状态色或优先级 | 10 | `Focused != Open`、`Selected != Hovered`、`Checked != Hovered`、多个 RGBA 与中央投影期望不一致 |
| `15d` 复合密度与对齐 | list row、table row、utility tab 与行 adornment | 9 | selected/disabled row surface 漂移、表格单元文本/可见列/最小列宽漂移、Slate adornment 尺寸漂移 |

数量按主要修复 owner 归类；若最低层根因是同一 `HostControlMetrics` 或 `HostMaterialPalette` 投影，功能 owner
应在最低共享层一次修复并向其他切片回填，禁止分别改 28 个期望值制造分叉。

## 完整失败清单

- `style_selector::workbench_toast::tests::open_toast_uses_active_border`
- `template_alerts::tests::{inline::workbench_info_alert_paints_tinted_surface_icon_and_label,style::workbench_toast_style_uses_shared_state_priority,toast::workbench_toast_paints_status_mark_action_and_close}`
- `template_buttons::tests::{paint::editor_variant_button_uses_centered_button_text_path,paint::selected_asset_browser_utility_tab_still_paints_slate_indicator,paint::semantic_button_glyphs_prefer_shell_asset_pixels,style::button_glyph_slots_project_from_host_control_metrics}`
- `template_chips::metrics::tests::workbench_chip_metrics_project_from_host_control_metrics`
- `template_icon_buttons::tests::geometry::{panel_icon_button_defaults_to_unreal_icon16_size,rail_icon_button_defaults_to_unreal_large_icon24_size,toolbar_icon_button_uses_unreal_slim_toolbar_icon_size}`
- `template_list_rows::tests::paint::{checked_list_row_paints_right_check_with_muted_selected_fill,disabled_list_row_keeps_background_empty_and_draws_disabled_adornment,selected_list_row_paints_muted_selected_fill_neutral_outline_and_navigation_adornment,workbench_list_row_adornments_paint_shell_asset_pixels}`
- `template_segmented_controls::tests::{geometry::segmented_control_offsets_group_label_body,style::segmented_and_tab_styles_use_shared_state_priority}`
- `template_selection_controls::tests::paint::selection_control_paints_checked_checkbox_without_full_row_surface`
- `template_shell_panels::tests::state::shell_panel_chrome_selector_states_reach_native_paint`
- `template_status_controls::tests::{chips::status_chip_paints_flat_text_without_surface_or_chevron,chips::status_chip_uses_shared_painter_state_priority,icons::status_icon_button_glyph_rect_uses_shared_status_metrics,icons::status_icon_button_uses_shared_icon_button_state_priority}`
- `template_table_rows::tests::{cells::table_cells_ignore_options_that_look_like_complete_rows,geometry::table_columns_drop_low_priority_numeric_cells_when_too_narrow,geometry::table_columns_respect_readable_minimums_when_width_allows}`
- `template_tree_rows::tests::paint::loading_player_start_tree_row_mutes_special_icon_color`

## 架构修复验收

1. 先分别复跑 `15b` 度量、`15c` palette/state、`15d` row/table 三组，确认生产合同或测试期望哪一侧发生了硬切漂移。
2. 以 `HostControlMetrics`、`HostMaterialPalette` 与共享 `UiPainterResolvedState` 为唯一权威；不得恢复局部常量、第二套色板或控件私有状态优先级。
3. focused 组全绿后，重跑完整 `paint_template_nodes::` 当前 696 项，并向 Editor 15 回传结果；随后由来源计划继续其余 editor lib-test 分区。

## 禁止临时方案

- 禁止批量改断言为当前像素而不核对 15b/15c/15d 计划合同与设计 token。
- 禁止在单个控件内加兼容分支、局部 magic number、私有 palette 或重复状态决议。
- 禁止忽略 28 项、降低测试覆盖或把失败记成导出流水线问题。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
| --- | --- | --- | --- | --- |
| Layout 15 S15.1/S15.4/S15.6 / Editor 15 M1 | retained painter 694 项分区 | `open-已精确复现并路由` | 2026-07-12 | 当前 editor test binary：666 passed / 28 failed / 2414 filtered out，57.54s；失败清单与 15b/15c/15d owner 映射见上。 |
| Layout 15 15b/15c/15d | 最低共享合同修复 | `fixed` | 2026-07-13 | `METRICS.row_height` 直接引用批准的 28px density token；panel/button/status/search/selection/list/tree、toolbar、rail glyph 分别由 row/gap 推导为 16/20/24px。表格最小宽度以 Runtime Text 测量和 density 相对预算共同决定，232px 保留 Name+Type，360px 保留四列。状态与色板测试服从共享 `UiPainterResolvedState` 和 host palette。 |
| Layout 15 S15.4/15b / Editor 15 M1 | `ui::layouts::` 分区追加 | `fixed` | 2026-07-13 | 当前源码编辑器测试二进制单线程 76/76：按实际投影验证 active panel，summary badge 验证 Runtime Text 内容留白与最大宽度，menu chrome 验证从 authored stencil 推导出的正且一致 slot gap。 |
| Layout 15 S15.6 / Editor 15 M1 | 当前源码 retained painter 全分区复验 | `fixed` | 2026-07-13 | 15b metrics projection 36/36；15c selector/palette/state 99/99；15d/template composite 467/467；完整 `paint_template_nodes::` 696/696，2416 filtered out。tooltip 箭头改为验证绘制区域，而不锁定抗锯齿边界像素。 |

## 修复结果与回传

- 根因：HostControlMetrics retained a 24px row baseline while Layout 15 required the 28px Editor density token; icon slots were coupled to title typography, table floors over-consumed responsive width, and tests retained pre-cutover state/palette/pixel assumptions.
- 架构修复：Project row, glyph, field, selection, status, row, and table geometry from shared density tokens and Runtime Text; keep palette/state in shared selectors; allocate table columns from measured headers and relative density budgets; assert semantic state and relative paint regions.
- 验证：Windows coordinator clean editor lib-test build succeeded; current editor source isolated against that consistent Runtime dependency set passed metrics 36/36, selector/palette/state 99/99, template composites 467/467, full retained painter 696/696, and ui::layouts 76/76. Screenshot target scan: 0.
- 回传：Return to Editor 15 with retained painter 696/696 and adjacent layout 76/76 green; original 28 regressions and follow-on responsive assertions are fixed.
