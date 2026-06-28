---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/allocation.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_property_rows
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_row_metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels/frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls/chips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls/chips/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/menu_popup_metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus/geometry/row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus/geometry/popup.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus/geometry/submenu.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus/popup.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus/rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry/frames/row.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_window_menus.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/ellipsis.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_labels/input_kind.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_labels/focus.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_labels/values.rs
design_references:
  - docs/ui-and-layout/editor-workbench-designs/asset-browser-workbench.png
  - docs/ui-and-layout/editor-workbench-designs/inspector-drawer-content-spec.png
  - docs/ui-and-layout/editor-workbench-designs/console-drawer-content-spec.png
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15b-host-control-metrics-single-source.md
  - docs/plans/engine-code-structure-convention.md
status: in_progress
---
# 15d 复合控件密度与对齐(S15.4 深化)

> 本文是 `15` 计划 **S15.4** 切片的实现就绪深化:把搜索栏/工具栏/状态栏/列表·表格·树·属性行/弹层等**复合控件**的密度、对齐、列宽/省略统一到标准化原子之上。以截图里最糟的 **Asset Browser 表格列头/列宽**为锚点,顺带把同类复合件一并立规。父计划见 `15`,度量单源 `15b`,文本省略 `15`(S15.2,已实现)。

## 1. 锚点:Asset Browser 表格(按代码核实)

| 事实 | 落点 |
| --- | --- |
| 列宽=**固定比例** `[0.36, 0.27, 0.19, 0.18]` × 可用宽 | `template_table_rows/cells/metrics.rs::TABLE_COLUMN_RATIOS` |
| 可用宽 = `rect.width - 2·TABLE_CELL_INSET_X(9) - TABLE_ACTION_WIDTH` | `cells/geometry.rs::table_cell_rect` |
| 单元字号 `9.5`、内边距 `9/4`、行圆角 `3` | `cells/metrics.rs`、`surface.rs` |
| 列文本经 `draw_text`(现已接 S15.2 省略) | `cells/commands.rs::push_table_cells` |
| 表头与数据行**走同一分配**(`is_table_header`) | `cells/geometry.rs` |

### 1.1 缺陷
1. **无列最小宽**:窄表/抽屉收窄时,每列按比例同步缩小,`Size/Modified` 这类窄列先垮(`0.18×可用宽` 再减内边距 → 文本可用十几 px)→ 列头 `Modified` 被裁成 `Rev`、`Size` 成 `Siz`。S15.2 让它降级为 `…` 但仍过窄不可读。
2. **数值列不右对齐**:`Size`、`Modified`(数值/日期)与 `Name`(文本)一律左对齐,不符表格惯例(UE/Excel 数值右对齐),读数困难。
3. **字号漂移**:`9.5` 与 chrome `font_body=10`(`15b`)不一致。
4. **列头无差异样式**:表头与数据行同色,缺 `text_muted` 弱化与底分隔。

## 2. 设计:列分配算法(比例 + 最小宽 + 右对齐 + 省略)

新增/改 `template_table_rows/cells` 下分配 owner(纯函数,便于单测):
```text
fn allocate_columns(avail, ratios, mins, aligns):
    // 1) 比例初分
    raw[i] = avail * ratios[i]
    // 2) 钳到列最小宽;被抬高的列从富余列等比回收
    w = clamp_each_to_min(raw, mins)         // Σw 可能 > avail
    if Σw > avail:
        从 w[i] > mins[i] 的列按超出比例回收, 直到 Σw == avail 或全部触底
    // 3) 仍放不下 -> 丢弃最不重要列(优先级序),余列重分配(可见列 log 标注)
    return w, x_prefix_sum
fn cell_text_rect(col):
    align==Right -> 文本右对齐(x = col.right - measured_or_ellipsized_width - inset)
    align==Left  -> 左对齐
    文本统一走 ellipsize_single_line(S15.2)
```
- `mins`:`Name≥120 / Type≥56 / Size≥56 / Modified≥72`(可调,落 `cells/metrics.rs`)。
- `aligns`:`[Left, Left, Right, Right]`(Name/Type 左,Size/Modified 右)。
- 列优先级(丢弃序):`Modified < Size < Type < Name`。
- 字号 `9.5`→统一到 `15b` 的 `METRICS.font_body`(或新增 `font_table` 若刻意更紧,二选一并注明)。

## 3. 表头与行样式
- 表头单元:`text_muted` 弱化 + 底部 1px `separator`;数据行选中 `surface_selected`+`accent` 文本,悬停 `surface_hover`。
- 行高统一到 `METRICS.row_height`(`15b`),内边距 `9/4`→对齐 `gap` 体系。

## 4. 同类复合件一并立规(遵附录 B)
| 复合件 | 改动要点 | owner |
| --- | --- | --- |
| 搜索栏 | 高=`control.height`,内边距=`input.pad(8/8/3/4)`,占位 `text_muted`,左图标 | `template_fields` + 搜索 `.zui` |
| 状态栏 | 行高单源,左信息左对齐、右数值(Grid/Snap/100%)**右对齐**,段间 `gap.m` + 分隔 | `template_status_controls` + `chrome.rs::draw_status_bar_*` |
| 列表行 | 行高 `row_height`,单行文本省略,选中态统一 | `template_list_rows` |
| 树行 | 缩进 step 单源,展开箭头对齐,标签省略 | `template_tree_rows` |
| 属性行 | 左 label(`text_muted`,定宽)+ 右值(省略),冒号/对齐统一 | `template_property_rows` |
| 弹层/上下文菜单 | 行内边距 `8/3`,1px 边框无阴影,选中 `surface_selected` | 菜单指针 + `*_context_menu.zui` |
| 内容容器/面板 | 模块内容面板统一 `surface` 面色、1px 普通边框、4px 圆角;仅受控 `Workbench*LeftPanel/CenterPanel/RightPanel` 进入该路径,避免误伤列表/表格行 | `template_shell_panels` |

> 原则:这些复合件**不新增原子**(遵 `12`),只用标准化原子(按钮/文本/容器/间隙)+ 统一密度/对齐;差异靠 slot 填充。

## 5. 结构/债务纪律
- 列分配纯函数落 `cells` 下 owner 叶子 + 内联测试 ≤150 行;`cells/geometry.rs` 不超长。
- 无 `unwrap/expect/TODO/allow(dead_code)/裸 Result`;文件 ≤800。
- 列被丢弃时 `log` 标注(遵 `15` 附录 C "无静默截断")。
- 字号统一引 `15b` 的 `METRICS`,不再散落 `9.5`。

## 6. 测试矩阵
| 测试 | 断言 |
| --- | --- |
| `columns_respect_min_width_when_narrow` | 窄可用宽 → 每可见列 ≥ 其 min;富余列回收 |
| `columns_drop_least_important_when_overflow` | 极窄 → 丢弃 `Modified`(优先级序),余列重分配 |
| `numeric_columns_right_aligned` | `Size/Modified` 单元文本右对齐(x+width 对齐右内边距) |
| `header_uses_muted_style_and_separator` | 表头色=`text_muted`,有底分隔 |
| `cell_text_ellipsizes_not_clips` | 窄列文本以 `…` 结尾(走 S15.2) |
| `status_bar_right_values_right_aligned` | 状态栏右段数值右对齐 |

## 7. 验收
- `editor-window-m3-asset-browser-900x620` 截图:列头全称或 `…`、不再 `Siz/Rev`;`Size/Modified` 右对齐;行高一致;表头弱化。
- `editor-window-m3-assets-drawer-900x620`:抽屉收窄时列不糊(min 生效),溢出丢列有 log。
- `editor-window-m3-workbench-900x620`:状态栏右侧数值右对齐。
- 命令:`cargo test -p zircon_editor --lib template_table_rows`(列分配/对齐)+ `template_status_controls` + `capture_m3_gui_acceptance_visual_artifacts --ignored` 刷新 `docs/tests/editor/`。

## 8. 实现顺序
1. `cells/metrics.rs` 加列 min/对齐/优先级常量(或表),字号引 `METRICS`。
2. 新 `cells/allocation.rs` 纯函数(比例+min+回收+丢列)+ 单测(RED→GREEN)。
3. `table_cell_rect`/`push_table_cells` 改用分配 + 右对齐 + 省略;表头样式。
4. 状态栏右对齐、列表/树/属性/弹层密度统一(逐 owner)。
5. 测试 + 截图复验;写状态。

## 9. 边界
不改表格数据来源/排序(那是数据层);不动 B 层 MUI `data_grid`(那是 MUI-X 移植);列宽用户拖拽持久化属 `04`,本文只做自适应分配。

## 10. 状态与产出记录
| 日期 | 切片 | 状态 | 产出/证据 | 后续项 |
| --- | --- | --- | --- | --- |
| 2026-06-26 | 15d/S15.4aj table header/tail recessed surface | implemented-focused-passed-build-screenshot-passed | 按复杂列表从行原子继续收敛:参考 Unreal Slate `SetupTableViewStyles` 的 row/header recessed 背景,`workbench_table_row/palette.rs` 将 header/tail 背景统一到 `PALETTE.surface_inset`,避免表头和表格尾部空白区比普通行更黑/更亮而切成多层块。验证:`table_header_and_tail_use_recessed_table_surface` 先 RED 后 1/1;`cargo fmt -p zircon_editor --check`;`workbench_table_row` 8/8;`template_table_rows` 17/17;`cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never` 通过;M3 screenshot harness 1/1 刷新 `docs/tests/editor/editor-window-m3-asset-browser-900x620.png` 和 `editor-window-m3-workbench-900x620.png`,均使用外部 `D:\cargo-targets\zircon-editor-components-0626`,未写 repo `target`。 | 该行只关闭 table header/tail recessed surface 原子视觉;窗口级组合观感继续由父计划 `15` 跟进。 |
| 2026-06-26 | 15d/S15.4ai Asset Browser 短视口 utility duplicate projection collapse | implemented-focused-passed-build-screenshot-passed | 按复合控件从局部收敛的要求,继续修 Asset Browser utility 抽屉:focused regression 先证明 497px 高短视口中 `AssetBrowserPreviewPanel` 等同 `control_id` 的重复投影折叠不完整会残留可见内容;`asset_browser/compact_layout.rs` 的 frame/height setter 改为更新所有匹配节点,让 Preview panel/visual/text 在 utility content height=0 时全部折叠。验证:`asset_browser_projection_compacts_preview_utility_for_short_viewport` 先 RED 后 1/1;`cargo fmt -p zircon_editor --check`;`asset_browser` 16/16;`cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never` 通过;M3 screenshot harness 1/1 刷新 `docs/tests/editor/editor-window-m3-asset-browser-900x620.png`,均使用外部 `D:\cargo-targets\zircon-editor-components-0626`,未写入 repo `target`。 | 该行只关闭短视口 utility 重复投影残留;窗口级组合观感继续由父计划 `15` 跟进。 |
| 2026-06-25 | 15d/S15.4 深化立项 | planned | 代码核实 Asset Browser 表格列宽为固定比例 `[0.36,0.27,0.19,0.18]`、**无列最小宽**、数值列左对齐、字号 `9.5` 漂移、表头与行同样式 → 窄表列头被裁 `Siz/Rev`;给出"比例+最小宽+富余回收+丢列+右对齐+省略"列分配算法、表头/行样式、状态栏右对齐、列表/树/属性/弹层密度立规、6 条测试矩阵、实现顺序。 | 按 §8 实现 S15.4;字号统一引 `15b` `METRICS`;父计划 `15` 的 S15.4/D4 勾稽随之更新。 |
| 2026-06-25 | 15d/S15.4a 按钮组/页签文本语义首段 | implemented-focused-passed | 先从复合控件的最小语义错位修起:新增 `template_node_labels/input_kind.rs`,把 text input 与 Button/Tab/Segmented/Toggle/Dropdown 等命令控件分类分开;Change 绑定按钮和页签保留内部 `value_text` 作为派发值,显示面使用 authored text。`button_change_binding_label_prefers_authored_text_over_value`、`button_label_ignores_matching_text_input_focus_value`、`input_label_uses_matching_text_input_focus_value`、`asset_browser_projection_maps_bootstrap_asset_into_mount_nodes` 均 1/1 通过;`capture_m3_gui_acceptance_visual_artifacts --ignored` 1/1 通过并刷新 `docs/tests/editor/editor-window-m3-asset-browser-900x620.png`。 | 本行不关闭完整 S15.4:表格列分配(min/右对齐/丢列)、状态栏右对齐、复杂列表密度、弹层密度仍按 §8 继续。 |
| 2026-06-25 | 15d/S15.4b Asset Browser 表格列分配与数值右对齐 | implemented-focused-passed-screenshot-passed | 新增 `template_table_rows/cells/allocation.rs` 作为列分配 owner,由 `TABLE_COLUMN_RATIOS`、`TABLE_COLUMN_MIN_WIDTHS`、`TABLE_COLUMN_DROP_ORDER` 计算可见列宽;`geometry.rs` 改为消费分配结果,极窄时按 `Rev -> Size -> Type -> Name` 降级隐藏次要列;`commands.rs` 对 Size/Rev 使用测量宽度右贴齐并保持 S15.2 省略。RED/GREEN 后通过 `cargo test -p zircon_editor --lib table_columns_ --locked --jobs 1 --color never` 3/3、`cargo test -p zircon_editor --lib template_table_rows --locked --jobs 1 --color never` 15/15、`capture_m3_gui_acceptance_visual_artifacts --ignored` 1/1;截图刷新 `docs/tests/editor/editor-window-m3-asset-browser-900x620.png`,可见 Name/Type/Size/Rev 全列且 Size/Rev 右对齐。 | 表格列分配子项关闭;完整 S15.4 仍需状态栏右段对齐、list/tree/property/menu 复合密度和更窄抽屉列降级截图。 |
| 2026-06-25 | 15d/S15.4c 状态栏右段数值右对齐 | implemented-focused-passed-screenshot-passed | 新增 `template_status_controls/chips/text.rs` 作为 status chip 文本 owner,把 `Grid: 10 cm` 这类冒号文本拆成左 label 与右 value 两段,并按 `measure_fallback_text_width + text_clip_guard` 将 value frame 贴齐 `status_chip_text_rect` 右缘;`100%` 这类纯值 chip 整体右对齐。`chips.rs` 继续只负责 surface/chevron 和委托文本 owner。验证:`status_chip_right_aligns_` 2/2、`template_status_controls` 16/16、`capture_m3_gui_acceptance_visual_artifacts --ignored` 1/1 通过;截图刷新 `docs/tests/editor/editor-window-m3-workbench-900x620.png`,右侧 Grid/Snap/100% 值区对齐且截图不进 `target`。 | 状态栏右段子项关闭;完整 S15.4 仍需 list/tree/property/menu 复合密度和更窄抽屉列降级截图。 |
| 2026-06-25 | 15d/S15.4d 窗口菜单弹层边框/行文本/命中度量统一 | implemented-focused-passed-screenshot-passed | 新增 `menu_popup_metrics.rs` 作为 host-contract 中性 owner,统一窗口菜单绘制与 native pointer 命中的 edge inset、row height、row gap、shell margin 和 anchor gap;`paint_workbench_renderer/menus` 的主菜单/子菜单外框由 `focus_ring` 强调色改为普通 `border`,启用项文字由 `text_muted` 改为主 `text`,禁用项使用 `text_disabled`;native pointer row/popup/submenu frame 同步消费同一组度量。新增像素回归 `rust_owned_window_menu_popup_uses_muted_border_and_primary_item_text` 先 RED 捕获 `[60,199,214]` 亮青边框,后 GREEN 验证 `[52,60,66]` 普通 1px 边框和主文本色。验证:`cargo fmt -p zircon_editor --check`、`native_workbench_window_menus` 2/2、`shared_menu_pointer_bridge` 10/10、`capture_m3_gui_acceptance_visual_artifacts --ignored` 1/1 通过;截图刷新 `docs/tests/editor/editor-window-m3-menu-popup-svg-icons-900x620.png`,菜单弹层无亮青外框且截图未写入 repo `target`。 | 菜单弹层子项关闭;完整 S15.4 仍需 list/tree/property 复合密度和更窄抽屉列降级截图。 |
| 2026-06-25 | 15d/S15.4e list/tree/property 行密度单源 | implemented-focused-passed-screenshot-passed | 新增 `template_row_metrics.rs` 作为 retained-host 行控件密度 owner,从 `METRICS`/`PALETTE` 投影列表行、树行和属性行共用的 body font、line height、surface radius、文本 inset、树缩进/图标/动作间距和属性 label/field 度量;`template_list_rows`、`template_tree_row_geometry`、`template_property_rows/layout` 与 `template_property_rows/text` 删除本地散落常量并消费该 owner。新增 `list_tree_and_property_rows_use_shared_density_metrics` 先锁定三类行同源度量,并把 tree loading 图标像素回归改为按生产几何取样。验证:`cargo check -p zircon_editor --lib --tests --locked --jobs 1 --color never --message-format short` 通过,`list_tree_and_property_rows_use_shared_density_metrics` 1/1、`template_list_rows` 6/6、`template_tree_rows` 7/7、`template_property_rows` 2/2、`capture_m3_gui_acceptance_visual_artifacts --ignored` 1/1 通过;截图刷新 `docs/tests/editor/editor-window-m3-assets-drawer-900x620.png` 与 M3 全套 PNG,未写入 repo `target`。 | list/tree/property 行密度子项关闭;完整 S15.4 仅剩更窄抽屉列降级/组合容器观感复核,随后进入 S15.5 三档断点自适应。 |
| 2026-06-25 | 15d/S15.4f 搜索栏图标/占位/内边距标准化 | implemented-focused-passed-screenshot-passed | 新增 `template_fields/search.rs` 作为搜索 TextField 语义 owner,由 `METRICS.input_pad`、`font_large` 与 `gap_s` 派生左侧 magnifier glyph 和文本 inset;`SearchEdited` 等非 `Workbench*` 搜索输入通过 `is_search_field` 进入 `template_fields`,不再落回通用 fallback。`push_search_field_glyph` 用 retained-host quad 绘制搜索图标,空 value 的 `Search` 标签按占位色显示。验证:`cargo fmt -p zircon_editor --check`;`cargo test -p zircon_editor --lib template_fields --locked --jobs 1 --color never --message-format short` 12/12;`cargo check -p zircon_editor --lib --tests --locked --jobs 1 --color never --message-format short` 通过;`capture_m3_gui_acceptance_visual_artifacts --ignored` 1/1 通过。截图刷新 `docs/tests/editor/editor-window-m3-assets-drawer-900x620.png` 与 `editor-window-m3-asset-browser-900x620.png`,搜索栏左侧图标和文本右移可见,未写入 repo `target`。 | 搜索栏子项关闭;完整 S15.4 仍剩更窄抽屉列降级/组合容器观感复核,随后进入 S15.5 三档断点自适应。 |
| 2026-06-25 | 15d/S15.4g 内容容器/面板层级标准化 | implemented-focused-passed-screenshot-passed | 新增 `template_shell_panels/frame.rs` 作为 shell 内容容器框架 owner,由 `METRICS.radius_control`/`border_width` 与 Workbench chrome separator/fill 派生 4px 圆角、1px 普通边框和面板面色;`identity.rs` 将 `Workbench*LeftPanel`/`CenterPanel`/`RightPanel` 归为 `ContentPanel`,并保留 `WorkbenchAssetsTableRow01` 这类行节点不匹配;`surface.rs` 只委托 frame owner 选择边框/圆角,`separators/commands.rs` 不再为内容面板叠方向分隔。组件 atlas 新增 `WorkbenchAtlasLeftPanel` 无显式 surface 样例,证明 shell-panel owner 路径可见。验证:`cargo fmt -p zircon_editor --check`;`cargo test -p zircon_editor --lib template_shell_panels --locked --jobs 1 --color never --message-format short` 7/7;`capture_m3_gui_acceptance_visual_artifacts --ignored` 1/1;`capture_workbench_component_slate_atlas_visual_artifact --ignored` 1/1。截图刷新 `docs/tests/editor/editor-components-workbench-slate-atlas-900x620.png` 与 M3 全套 PNG,未写入 repo `target`。 | 内容容器观感子项关闭;完整 S15.4 的下一步应转入 S15.5 三档断点,处理更窄抽屉列降级与窗口组合复核。 |
| 2026-06-25 | 15d/S15.4h Workbench 模块 overflow 弹层/选择列表 | implemented-focused-passed-screenshot-passed | 沿用窗口菜单弹层度量收口后的 popup 基础,把上一轮 `WorkbenchModuleMore` 从入口升级为可交互选择列表:`workbench_window.zui` 新增 `WorkbenchModuleOverflowMenu`,并由 `template_bridge/workbench/module_overflow_menu.rs` 维护隐藏模块行、active hidden module 的 checked 标记和行选择到真实 tab control 的 dispatch;`window_menu_state.rs` 负责与其它 toolbar menus 互斥,`control.rs` 负责 popup item selected 后合并 module-tab effects。验证:`cargo fmt -p zircon_editor --check`;`compact_workbench_module_more_opens_overflow_menu_and_selects_hidden_module` 1/1;`workbench_toolbar` filter 5 passed/1 ignored;`capture_workbench_module_overflow_visual_artifact --ignored` 1/1,生成 `docs/tests/editor/editor-window-m3-workbench-module-overflow-900x620.png`,未写入 repo `target`。 | 该行只关闭 Workbench 顶部模块 overflow 弹层/选择列表;主页签 overflow 菜单已由后续 S15.4j/15a 行关闭,popup anchor token 化和 640/1260 组合断点仍在 15a/15e 后续。 |
| 2026-06-25 | 15d/S15.4j host 页签 overflow 弹层复用选择列表密度 | implemented-focused-passed-screenshot-passed-build-passed | 在 `15a` 的 host 页签 overflow 实现中复用 S15.4 菜单弹层密度:弹层使用普通 1px border、紧凑 row frame、hover/selected surface,并把隐藏页列表作为可交互选择列表绘制在 overlay 层。生产逻辑拆分在 `host_contract/host_page_overflow_menu.rs`、`paint_workbench_renderer/scene_layers/overlay/page_overflow.rs` 与 `native_pointer/button_dispatch/page_overflow_menu.rs`,没有把弹层样式塞回 host page 根入口。验证:`cargo build -p zircon_editor --locked --jobs 1 --message-format short --color never` 通过(仅既有 warning);`cargo fmt -p zircon_editor --check`;`cargo test -p zircon_editor overflow --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture` 15 passed/2 ignored;`capture_host_page_overflow_menu_visual_artifact --ignored` 1/1,截图 `docs/tests/editor/editor-window-m3-host-page-overflow-420x260.png` 未写入 repo `target`。 | 该行只记录复合弹层/选择列表密度复用;完整 S15.4 仍可继续更窄抽屉列降级和组合容器复核,S15.5 仍负责三档断点联动。 |
| 2026-06-25 | 15d/S15.4k 表格窄上下文列降级联动 | implemented-focused-passed-screenshot-passed-build-passed | 新增 `retained_host/ui/template_layout_context.rs` 作为模板布局上下文 owner,把 Workbench root 宽度和 Asset Browser pane 宽度映射为 `layoutNarrow`/`layoutRegular`/`layoutWide` variant token;`native_template_node_panes.rs` 与 `workbench_window_projection.rs` 只注入上下文,`template_table_rows/cells/allocation.rs` 只消费 `layoutNarrow` 来隐藏低优先级 Size/Rev 列,避免用局部行宽误判。验证:`cargo fmt -p zircon_editor --check`;`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never` 通过;`table_columns_drop_numeric_cells_for_narrow_layout_context`、`asset_browser_table_nodes_receive_narrow_context_variant`、`table_nodes_receive_context_tier_variant` 均 1/1;`cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never` 通过;`capture_m3_gui_acceptance_visual_artifacts --ignored` 1/1 刷新 `docs/tests/editor` PNG,没有写入 repo `target`。 | 表格列已接入领域断点上下文;S15.4 复合表格/列表主线已可作为下一轮抽屉/窗口组合复核基线,S15.5 仍需 breakpoint 默认 token 化、窗口 minimum 降低和 Ultra 档。 |
