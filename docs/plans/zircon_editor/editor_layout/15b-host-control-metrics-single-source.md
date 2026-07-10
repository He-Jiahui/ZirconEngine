---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/tokens.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_control_geometry/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdowns/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdowns/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_icon_button/palette.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/CoreStyle.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Styling/StarshipCoreStyle.cpp
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/01-design-tokens-and-language-contract.md
  - docs/plans/engine-code-structure-convention.md
status: completed
---
# 15b 工作台 chrome 控件度量单源(S15.1 深化)

> 本文是 `15` 计划 **S15.1** 切片的实现就绪深化:把 retained-host **工作台 chrome 控件层**散落的度量常量收敛为**单一事实源** `HostControlMetrics`。父计划见 `15`。本文已用代码核实"应收敛/不应收敛"的二层边界,避免把它做成"集中 200 个常量"的不可能任务。

## 1. 关键边界:两层度量,只收敛 chrome 层

retained-host 的 `paint_template_nodes` 下其实有**两层**度量,职责不同,不可混并:

| 层 | 家族 | 设计系统 | 是否本计划收敛 |
| --- | --- | --- | --- |
| **A. 工作台 chrome 控件** | `template_buttons`/`template_segmented_control_geometry`/`template_icon_buttons`/`template_fields`/`template_dropdowns`/`template_status_controls`/`style_selector/workbench_*` | **Zircon 自有**(UE Slate 度量 + teal 色板) | **是**——`15` 的标准化目标,截图里的真实编辑器 chrome |
| **B. 组件展示库** | `material_primitives/*`、`mui_x_primitives/*`(alert/avatar/badge/chip/divider/paper/skeleton/timeline/charts/data_grid/pickers/tree_view…) | **MUI / MUI-X 移植**(忠实于 Material/MUI-X 自身规范) | **否**——这些是 MUI 移植件,强套 UE 度量会破坏其设计保真;保留各自 `metrics.rs` |

> 结论:S15.1 的"中央度量单源"只面向 **A 层 chrome 控件**(约 20 个常量,且已被 `15.S1a` 收敛到一致取值)。B 层(数十常量)是另一套 design system 的移植,**显式排除**,各自保留 owner。`15.S1a` 已把 chrome 取值统一到低圆角/薄边框/紧凑字号,本计划只是把它们从"多处一致"升级为"单处定义"。

## 2. chrome 层度量盘点(按代码核实)

| 现常量 | 值 | owner 文件 | 拟并入 `HostControlMetrics` 字段 |
| --- | --- | --- | --- |
| `BUTTON_FONT_SIZE` | 13.33 logical px | `template_buttons/content/metrics.rs` | `font_body`(UE Normal=10pt×96/72) |
| `BUTTON_LINE_HEIGHT` | 16 | 同上 | `line_height(font_body)`(=13.33×1.2) |
| `BUTTON_TEXT_INSET_X` | 12 | 同上 | `button_pad_x` |
| `BUTTON_ICON_GAP` | 7 | 同上 | `button_icon_gap` |
| `BUTTON_CHEVRON_RESERVE` | 18 | 同上 | `button_chevron_reserve` |
| `BUTTON_PRESSED_CONTENT_OFFSET_Y` | 1 | 同上 | `button_pressed_offset_y` |
| `BUTTON_RADIUS` | 4 | `template_buttons/geometry.rs` | `radius_control` |
| `SEGMENT_FONT_SIZE` | 13.33 logical px | `template_segmented_control_geometry/metrics.rs` | `font_body` |
| `SEGMENT_RADIUS` | 4 | 同上 | `radius_control` |
| `TAB_FONT_SIZE` | 13.33 logical px | 同上 | `font_body` |
| `SEGMENT_GROUP_LABEL_GAP` | 4 | 同上 | `gap_s` |
| `SEGMENT_SELECTED_INSET` | 2 | 同上 | `segment_selected_inset` |
| `TAB_TEXT_INSET_X` | 12 | 同上 | `button_pad_x`(页签/按钮共用横内边距) |
| `FIELD_RADIUS` | 4 | `template_fields/surface.rs` | `radius_control` |
| `FIELD_FONT_SIZE` | 13.33 logical px | `template_fields/text.rs` | `font_body` |
| `FIELD_LINE_HEIGHT` | 16 | 同上 | `line_height(font_body)` |
| `FIELD_TEXT_LEFT/RIGHT` | 8/8 | 同上 | `input_pad[0]/[1]` |
| `DROPDOWN_RADIUS` | 4 | `template_dropdowns/surface.rs` | `radius_control` |
| `DROPDOWN_FONT_SIZE` | 13.33 logical px | `template_dropdowns/text.rs` | `font_body` |
| `DROPDOWN_LINE_HEIGHT` | 16 | 同上 | `line_height(font_body)` |
| `DROPDOWN_TEXT_LEFT` | 8 | 同上 | `input_pad[0]` |
| `WORKBENCH_ICON_RAIL_RADIUS` | 4 | `style_selector/workbench_icon_button/palette.rs` | `radius_control` |
| `WORKBENCH_ICON_PANEL_RADIUS` | (核实) | 同上 | `radius_control`(或单列 `radius_icon_panel`) |

> 2026-07-10 单位复核修正:原实现把 UE Normal=10 **point** 误作 10 logical px。`SlateFontInfo.h` 明确 point 按 96 DPI 转 Slate Unit,故正确 body/line 为 13.33/16 logical px。radius/inset 等几何值仍 1:1;字体单位修正是有意视觉变化,不再声明“零视觉回归”。

## 3. owner API 草案

落点 `paint_theme/metrics.rs`(由 `paint_theme` owner 暴露,不进薄根):
```rust
pub(in crate::ui::retained_host::host_contract) struct HostControlMetrics {
    pub radius_control: f32,           // 4   (UE InputFocusRadius)
    pub border_width: f32,             // 1   (UE InputFocusThickness)
    pub font_small: f32,               // 10.67 logical px (UE 8pt @ 96 DPI)
    pub font_body: f32,                // 13.33 logical px (UE 10pt @ 96 DPI)
    pub font_large: f32,               // 18.67 logical px (UE 14pt @ 96 DPI)
    pub line_height_ratio: f32,        // 1.2
    pub button_pad_x: f32,             // 12  (UE ButtonMargins 横)
    pub button_icon_gap: f32,          // 7
    pub button_chevron_reserve: f32,   // 18
    pub button_pressed_offset_y: f32,  // 1   (UE PressedButtonMargins 下沉)
    pub input_pad: [f32; 4],           // 8/8/3/4 (UE SEditableTextBox)
    pub segment_selected_inset: f32,   // 2
    pub gap_s: f32,                    // 4
    pub gap_m: f32,                    // 8
    pub gap_l: f32,                    // 12
    pub row_height: f32,               // 28
}
pub(in crate::ui::retained_host::host_contract) const METRICS: HostControlMetrics = HostControlMetrics { /* 上表值 */ };
impl HostControlMetrics {
    pub fn line_height(&self, font: f32) -> f32 { font * self.line_height_ratio }
}
```
`paint_theme.rs` 增 `mod metrics;` + `pub(in ...) use metrics::{HostControlMetrics, METRICS};`(与现 `PALETTE` 同形)。

## 4. 硬切换(删除旧符号,迁移调用方)

按 `engine-code-structure-convention` 的 hard-cutover:同一变更内删本地常量、改读 `METRICS`,grep 旧名零命中。

- `template_buttons/content/metrics.rs`:删 6 个 `BUTTON_*`,函数改读 `METRICS`(`measured_label_width`/`content_offset_y` 保留,内部换源)。
- `template_buttons/geometry.rs`:删 `BUTTON_RADIUS`(`ADD_COMPONENT_OFFSET_Y` 属特例偏移,保留本地)。
- `template_segmented_control_geometry/metrics.rs`:删 `SEGMENT_FONT_SIZE/SEGMENT_RADIUS/TAB_FONT_SIZE/SEGMENT_GROUP_LABEL_GAP/SEGMENT_SELECTED_INSET/TAB_TEXT_INSET_X`,`*_line_height()` 改 `METRICS.line_height(...)`。
- `template_fields/{surface,text}.rs`、`template_dropdowns/{surface,text}.rs`:删 `*_RADIUS/*_FONT_SIZE/*_LINE_HEIGHT/*_TEXT_*`,改读 `METRICS`/`input_pad`。
- `style_selector/workbench_icon_button/palette.rs`:`WORKBENCH_ICON_RAIL_RADIUS`→`METRICS.radius_control`;`WORKBENCH_ICON_PANEL_RADIUS` 核实后并入或单列。
- B 层(`material_primitives`/`mui_x_primitives`)**不动**。

## 5. 测试矩阵
| 测试 | 断言 |
| --- | --- |
| 既有 `paint_template_nodes::template_buttons::tests`(17) | 硬切换后值不变,全绿(零回归) |
| 既有 `template_icon_buttons`(11) | 同上 |
| 段控/字段/下拉 几何测试 | 同上 |
| `host_control_metrics_match_unreal_baseline`(新) | `METRICS` 字段 == UE 派生基线(radius 4/font 10pt→13.33 logical px/pad 12/pressed 1…) |
| 静态守卫:`grep` chrome 旧常量名 | 仅出现在 `metrics.rs` 注释/无源码引用(零残留) |

## 6. 验收
- `cargo test -p zircon_editor --lib paint_template_nodes::template_buttons` / `template_icon_buttons` / 段控·字段·下拉,全绿。
- `capture_m3_gui_acceptance_visual_artifacts --ignored` + `capture_workbench_component_slate_atlas_visual_artifact --ignored` 刷新 `docs/tests/editor/`,人工确认字号按 UE point→96 DPI logical px 修正后可读性提升且控件未裁切。
- `grep -rn "BUTTON_RADIUS\|BUTTON_FONT_SIZE\|SEGMENT_RADIUS\|TAB_FONT_SIZE\|FIELD_RADIUS\|DROPDOWN_RADIUS"` 仅命中新 owner/注释。

## 7. 与 15.S1a / 01 的关系
- **15.S1a**:已把 chrome 取值收敛到一致(FiraSans + 低圆角/薄边框/紧凑字号);本计划把"一致取值"升级为"单源定义",不改像素。
- **01/S15.6**:度量单源(本文)与色彩单源(`S15.6` 把 `PALETTE` 接 `EditorDesignTokens`)是两条正交收口线;`HostControlMetrics` 后续也可由中央 `EditorDesignTokens` 的 `controls/density` 组投影(留作 `S15.6`/`01.S2` 衔接点)。
- 补(2026-07-02 评审收口):
  - **值语义**:`METRICS` 全部字段为**逻辑单位基准**(scale=1.0 取值),渲染前统一乘 scale(逻辑→物理换算单点在 21 顶点装配,遵 16 §3.4);任何调用方不得预乘 scale 存值。
  - **与 16 §5 合并**:`METRICS` 与 16 §5 的 `WorkbenchChromeMetricsLogical` 语义重叠,收编为**同一投影来源**(单表、双名先并存到切换点),挂 **16.S3** 切片执行合并。
  - **行高权威**:`line_height_ratio`/`line_height()` 仅为 chrome 层过渡值,行高权威 = **01 typography token**(与 17 §3.6 裁决一致);01.S2 typography 组落地后本表行高字段改为投影,不再自持比例。
  - **移交条款**:20.S2 级联 `var()` 通路验收后,`METRICS` 降级为级联引擎的内置默认值来源(见 20 §3.6 收编路线),删除时点以 20.S2 验收为准。

## 8. 结构纪律
owner 叶子 `paint_theme/metrics.rs`;根 wiring 薄;无 `unwrap/expect/TODO/allow(dead_code)/裸 Result`;文件 ≤800;硬切换删旧符号不留双轨;B 层不越界改。

## 9. 实现顺序
1. 建 `paint_theme/metrics.rs`(`HostControlMetrics` + `METRICS` + `line_height`)+ 基线测试(RED→GREEN)。
2. chrome 各家族逐个改读 `METRICS`,删本地常量。
3. 跑 chrome 几何/样式测试(零回归)+ 两张截图复验。
4. grep 守卫零残留;写状态。

## 10. 状态与产出记录
| 日期 | 切片 | 状态 | 产出/证据 | 后续项 |
| --- | --- | --- | --- | --- |
| 2026-06-25 | 15b/S15.1 深化立项 | planned | 代码核实 chrome 控件层 ~20 个度量常量(值已一致:radius 4/font 10/line 12/inset 8·12)与 B 层 MUI 移植件数十常量的**二层边界**;给出 `HostControlMetrics` owner API、逐常量→字段映射、硬切换删除清单、零回归测试矩阵、与 `15.S1a`/`01`/`S15.6` 衔接。明确只收敛 A 层、B 层不动。 | 按 §9 顺序实现;父计划 `15` 的 S15.1/D1 勾稽随之更新。 |
| 2026-06-25 | 15b/S15.1a chrome 控制度量硬切换 | implemented-focused-passed-build-screenshot-passed | `template_buttons/content/*`、`template_button_glyphs/*`、`template_fields/text.rs`、`template_dropdowns/text.rs`、`template_dropdown_glyphs/*`、`workbench_icon_button` selector、axis value field 和 inspector row primitives 改为直接消费 `paint_theme::METRICS`,删除本地 `BUTTON_*`/`FIELD_*`/`DROPDOWN_*`/icon radius/axis/inspector font-radius 旧别名。新增 `retained_host/asset_control_ids.rs` 统一 asset dispatch source 与 action/control id 映射,避免 surface binding 与 activation callback 保留两份表。验证:`cargo fmt -p zircon_editor --check`;focused tests 覆盖 `template_buttons` 21/21、`template_icon_buttons` 11/11、`template_fields` 12/12、`template_dropdowns` 8/8、`template_axis_value_fields` 5/5、`template_inspector_rows` 9/9、`template_activation_semantics` 7/7、`template_segmented` 11/11、`host_control_metrics_match_unreal_slate_baseline` 1/1;旧别名源码扫描和生产债务扫描零命中;`cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never` 通过;`capture_m3_gui_acceptance_visual_artifacts --ignored` 与 `capture_workbench_component_slate_atlas_visual_artifact --ignored` 均 1/1,截图刷新在 `docs/tests/editor/`。 | S15.1 chrome 度量硬切换关闭;整窗观感仍按 S15.4/S15.5 继续处理组合空隙、抽屉/窗口自适应与 popup anchor token 化。 |
