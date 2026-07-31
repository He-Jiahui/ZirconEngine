---
related_code:
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/template/asset/compiler/ui_style_resolver.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector
  - zircon_editor/assets/ui/editor/components
design_references:
  - docs/ui-and-layout/editor-workbench-designs/STYLE-NOTES.md
  - docs/ui-and-layout/ai-workbench-style/ai-workbench-web-framework.png
plan_sources:
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
status: in_progress
---
# 01 设计 Token 与设计语言契约

## 1. 目标

把 `editor-workbench-designs/STYLE-NOTES.md` 与 `ai-workbench-web-framework.png` 里散落在设计图中的视觉规则,沉淀为**一份中央设计 token 资产 + 一份设计语言契约文档**,让"一致的设计风格"成为可被编辑器全局引用、可被样式选择器消费、可被验收检查的单一事实源,而不是每个面板各写各的色值。本计划只做**设计语言的定义与喂入**,不改样式选择器机制本身(那在 `editor_ui/04`)。

## 2. 现状(按代码核实)

### 2.1 已存在的设施

| 能力 | 落点 | 证据 |
| --- | --- | --- |
| 状态选择器(按 family 折叠,优先级单源) | `zircon_runtime_interface/src/ui/style.rs` | `UiPainterStyleSelector`、`UiPainterResolvedState`、`button_resolved_state` 等 |
| 基础样式 DTO | 同上 | `UiRgbaColor`、`UiStyleColor`、`StyleDimension`、`UiResolvedElementStyle` |
| 中立绘制族(消费样式产出语义状态) | `paint_template_nodes/style_selector` | 选择器解析的最终视觉态 |

### 2.2 真实缺口

- 缺中央 theme/token 资产类型(`editor_ui/04` 已标注 theme 文档/资产类型缺失,本计划补设计 token 这一具体形态)。
- 缺把 STYLE-NOTES 的色板/控件规格/密度规则固化为可引用 token 的契约文档。
- 缺 token → 选择器的喂入路径(色值不应硬编码在组件 `.zui` 或绘制族里)。

## 3. 设计

### 3.1 设计 token 分组

按 STYLE-NOTES 的视觉规则,token 分四组,均为中央单源:

| Token 组 | 内容 | 来源规则 |
| --- | --- | --- |
| **色板(palette)** | 近黑面板色阶 `surface.0=#111416`、`surface.1=#171a1d`、`surface.2=#1b1f23`、`surface.3=#252b31`;强调色 `accent=#3cc7d6`;边框/分隔/文本前景色 | STYLE-NOTES 色板;teal 仅用于激活/选中/焦点/关键态 |
| **控件规格(controls)** | 控件高 `control.height=28..32px`、`control.radius=低圆角`、`border.width=1px`、扁平态 | STYLE-NOTES 控件规则;禁止渐变/辉光/阴影 |
| **密度与间距(density)** | 行高、间隙(`gap.s/m/l`)、抽屉内边距 | STYLE-NOTES 密度 + 工作台壳布局 |
| **字排(typography)**(2026-07-02 评审收口新增) | 字号阶(`font.body/strong/code/...`)、行高比(`line_height_ratio`)、字重 | 17 §3.6 与 15b 的裁决:行高/字号唯一权威=本组 token,runtime `DEFAULT_FONT_SIZE`/`DEFAULT_LINE_HEIGHT_SCALE` 与 15b `METRICS.font_body/line_height_ratio` 均改为本组投影 |
| **状态语义(state-roles)** | 各状态用哪组色(default/hovered/focused/selected/pressed/disabled) | 与选择器优先级 disabled>pressed>selected/focused>hovered>default 一致 |

### 3.2 设计语言契约文档

一份契约 markdown,把"一致的设计风格"写成可执行规则:每条规则引用具体 token、给出适用与禁用场景(对应 STYLE-NOTES 的 NO 列表:无渐变/辉光/阴影/嵌套卡片/英雄字号),作为 02-06 全部子计划的视觉验收基线。

### 3.3 Token 喂入路径

token 资产 → 样式选择器输入(不绕过 `UiPainterStyleSelector`)→ 绘制族消费。组件 `.zui` 引用 token 名而非裸色值,保证改一处 token 全局生效。

### 3.4 Token 引用文法与自定义属性注册表(2026-07-02 评审收口)

评审发现 token 引用写法在各计划中已出现五种并存:`$--left-drawer-width`(02/13)、`$gap.m`(13)、`$editor.surface.1`(10)、`var(--editor-surface-1)`(20)、`editor.surface.recessed`(15c `.zui`)。本计划作为 token 语言权威,在此定稿**唯一引用文法**:

1. **规范 token 名**:点分层级小写名(`editor.surface.1`、`gap.m`、`control.height`),这是 token 的唯一身份,资产/文档/诊断一律用它。
2. **内联形态** `$<token名>`(如 `$gap.m`、`$editor.surface.1`):用于 `.zui` 属性值与约束 token 位;`$--x` 前缀写法(02/13 早期)登记为兼容别名,收束期解析器双收,收束后仅规范形态。
3. **级联形态** `var(--<token名,点换连字符>)`(如 `var(--editor-surface-1)`):仅用于 20 的 USS 级联规则文本;点分名↔连字符名的映射是**机械双射**(`.`→`-`),不允许手工命名偏离。
4. **token → 自定义属性注册表**(20 §3.3 依赖的交付项,在此立项):S2 增补交付——`EditorDesignTokens` 全量字段自动注册为级联引擎的自定义属性(`--editor-surface-1` 等),20 的 `var()` 解析只查此注册表,禁止第二份 token→值映射。

资产扫描 / 渲染 guard / 级联解析三处校验统一以本节文法为准。

## 4. 接口与数据结构草案(Rust)

```rust
// zircon_runtime_interface/src/ui/style.rs 旁,token 中央定义
pub struct EditorDesignTokens {
    pub palette: PaletteTokens,      // surface[0..4], accent, border, text
    pub controls: ControlTokens,     // height, radius, border_width
    pub density: DensityTokens,      // gaps, paddings, row_height
    pub state_roles: StateRoleTokens,// 各状态 -> 色角色映射
}
// 喂入选择器,而非选择器内联默认色
pub fn apply_tokens_to_selector(tokens: &EditorDesignTokens, selector: &mut UiPainterStyleSelector);
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增 | `zircon_runtime_interface/src/ui/design_tokens.rs` | 中央 token 类型 |
| 新增 | `zircon_editor/assets/ui/editor/theme/editor_tokens.zui` | token 资产 |
| 新增 | `docs/ui-and-layout/design-language-contract.md` | 设计语言契约文档 |
| 修改 | `paint_template_nodes/style_selector` | 接受 token 喂入 |

## 6. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
| -- | --- | --- | --- | --- |
| S1 | token 资产骨架 + 契约文档 | design_tokens.rs / editor_tokens.zui / design-language-contract.md | `cargo check -p zircon_runtime_interface --locked` | 新建,无旧路径 |
| S2 | token 喂入选择器 + 组件引用 token 名;(2026-07-02 评审收口)增补交付:token → 自定义属性注册表(§3.4-4,供 20 级联 var() 消费)+ token 引用文法统一(§3.4,兼容别名双收) | style_selector / 组件 `.zui` | `cargo test -p zircon_editor --lib --locked` | 删除组件内联裸色值 |

## 7. 测试矩阵

- token 资产可加载且字段完整。
- 选择器在 token 喂入后,各状态解析色与 STYLE-NOTES 色板一致。
- 组件 `.zui` 中无裸十六进制色值(改引用 token)。

## 8. 风险与对策

- 风险:token 与现有内联色不一致导致视觉回退。对策:S2 前先做一次色值对照表,逐项对齐 STYLE-NOTES。

## 9. 完成定义

token 中央化、契约文档落地、组件引用 token、选择器消费 token,且全局改一处 token 生效。

## 10. 边界约束

不改选择器优先级机制(属 `editor_ui/04`);不内嵌设计 PNG;不引入渐变/辉光/阴影。

## 11. 参考实现对照(dev/ 源码锚点)

- `dev/material-ui/packages/mui-material/src/styles`:theme 结构(palette/typography/spacing/components)作为 token 分组参考,不取 sx 运行时。

## 12. 状态与产出记录

| 日期 | 切片 | 状态 | 产出/证据 | 后续项 |
| --- | --- | --- | --- | --- |
| 2026-06-23 | 01.S1 token 资产骨架 + 契约文档 | completed | 已新增 `zircon_runtime_interface/src/ui/design_tokens.rs`、`zircon_editor/assets/ui/editor/theme/editor_tokens.zui`、`docs/ui-and-layout/design-language-contract.md` 与 `zircon_runtime_interface/src/tests/editor_design_tokens.rs`;`zircon_runtime_interface/src/ui/mod.rs` 已导出 token 模块。`cargo test -p zircon_runtime_interface --lib editor_design_tokens --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-runtime-interface-0623` 3/3 通过;scoped rustfmt 与 `git diff --check` 通过。 | 01.S2:把 token 喂入样式选择器,并清理组件 `.zui` 内裸色值/裸控件规格引用。 |
| 2026-06-23 | 01.S2 token 喂入选择器 + 布局新增资产引用 token 名 | partial-runtime-interface-passed-editor-cargo-blocked | 已新增 `EditorResolvedPainterStyle` 与 `EditorDesignTokens::resolve_painter_style(...)`,通过 `UiPainterStyleSelector::resolved_state_for_family(...)` 后再映射 palette/foreground/border/radius/height,不改 selector 优先级。`editor_tokens.zui` 增加 `editor.*` token 名表;`workbench_skeleton.zui`、`command_palette.zui`、`preferences.zui` 导入 token 资产并引用 `editor.surface.*`/`editor.text.*`/`editor.border`;`editor_layout_contracts.rs` 增加新增布局资产不得回退裸 hex 的静态契约。RED: focused runtime-interface 测试先因缺 `resolve_painter_style` 失败;GREEN: `cargo test -p zircon_runtime_interface --lib editor_design_tokens_feed_painter_styles_through_selector_state --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-token-feed-0623` 1/1 通过。 | 继续 01.S2 的 wider cleanup:旧 `components/workbench/shell` 与 `components/workbench/modules` 仍有历史裸 hex/裸规格值,需要在 editor Cargo lane 恢复后按资产族逐步替换为 token 引用并接入 retained painter 端验收。当前 `zircon_editor` Cargo gate 仍被 active render mesh import 漂移阻塞。 |
| 2026-06-23 | 01.S2 density token lookup + editor lane restored | partial-editor-verified | `EditorDesignTokens::density_value_for_token_name(...)` 已把 `editor.density.*` 与 `--left-drawer-width`/`--right-drawer-width`/`--bottom-output-height` 映射到中央 density token,供 02.S2 壳声明投影消费。为恢复 editor 验证,最小修复下层 render mesh owner split 后的 re-export/import 漂移:`MeshPassCommandBuffers`、`CachedMeshDrawLookup` 与 `mesh_draw_command_list::builder` 上级路径。验证:`cargo test -p zircon_runtime_interface --lib editor_design_tokens --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-token-feed-0623 --message-format short --color never` 5/5 通过;`cargo check -p zircon_editor --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never` 通过;`cargo test -p zircon_editor --lib editor_layout_contracts --no-run --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never` 通过,随后直接运行测试二进制 `editor_layout_contracts --test-threads=1 --nocapture` 8/8 通过。 | 01.S2 仍未关闭整个资产族 hard cutover:旧 shell/module 资产的历史裸 hex/裸规格值和 retained painter 端视觉验收继续保留为后续项。 |
| 2026-07-27 | 01.S2 通用按钮原子控件 token 投影 + 资产收束 | partial-static-verified | `buttons.rs` 新增缓存的 `ButtonVisual`,由 `EditorDesignTokens` 与 `EditorTypographyTokens` 投影 Button/ToggleButton/IconButton 的 normal/hover/pressed/selected/focus/disabled 状态、低圆角、密度尺寸与逻辑文本行高;经 `style_overrides` 接收有效 CSS 色和有限度量,非法色、负间距、非正尺寸回退中央 token。`workbench_button.zui` 与 `workbench_icon_button.zui` 已移除局部颜色/像素规格并声明 palette/control/density/typography token;`render_buttons.rs` 覆盖有效覆盖优先和非法回退;`editor_design_token_contracts.rs` 校验资产令牌输入。已执行 scoped `rustfmt`,无旧按钮色板/裸 hex/模块超限/文档和 diff 静态守卫。 | 仍需在桌面宿主稳定后经受管 Cargo 验证当前 Rust 测试，并以当前源启动 retained host 后把真实截图写到 `docs/tests/editor/`；同时继续按原子控件优先顺序清理余下历史资产。 |
| 2026-07-27 | 01.S2 文本字段原子控件 token 投影 + 资产收束 | partial-static-verified | `text_fields.rs` 新增缓存的 `TextFieldVisual`,以 `EditorDesignTokens` 的 recessed/hover/pressed/focus/disabled 色阶、边框、密度和 `EditorTypographyTokens` 的逻辑 body 行高驱动输入面;字段文本继续经 `UiTextLayoutRequest`、共享测量缓存、选择和 preedit 接口，不绕开 runtime text 权威路径。Search/Field/NumberField 资产已声明统一 state/control/density/typography token 并移除原始布局像素值;数值 min/max/step 保留为领域数据。`render_text_fields.rs` 新增有效 style override 优先与非法值回退契约;`editor_design_token_contracts.rs` 扩展资产输入断言。已执行 scoped `rustfmt`,无旧字段色板、裸 hex、原始 layout 度量、模块超限和 diff 静态守卫。 | 仍需在桌面宿主稳定后经受管 Cargo 验证当前 Rust 测试，并以当前源启动 retained host 后把真实截图写到 `docs/tests/editor/`；继续按原子控件优先顺序处理选择控件、分段控件和滑块。 |
| 2026-07-27 | 01.S2 选择控件原子 token 投影 + 相对几何 | partial-static-verified | `selection_controls.rs` 以缓存的 `SelectionVisual` 替换 Checkbox/Radio/Toggle/Switch 的局部色板、11px 标签和固定标记规格。选中、焦点、不可用与 hover/pressed 状态统一投影 palette；checkbox mark、radio dot、toggle track/thumb、标签间距从 control height、density gap 和 border width 计算。Checkbox/Radio/Toggle 资产已声明 palette/control/density/typography token，移除本地色值和原始 layout 规格；测试更新为中央 accent/disabled 契约并加入令牌边界守卫。已执行 scoped `rustfmt`,无旧选择控件色板、裸 hex、原始 layout 度量、模块超限和 diff 静态守卫。 | 仍需在桌面宿主稳定后经受管 Cargo 验证当前 Rust 测试，并以当前源启动 retained host 后把真实截图写到 `docs/tests/editor/`；继续处理分段控件、滑块和容器原子控件。 |
| 2026-07-27 | 01.S2 拖拽预览覆盖层 token 投影 | partial-static-verified | `drag_overlay.rs` 新增缓存的 `DragOverlayVisual`，allowed/blocked preview、drop indicator、低圆角、图标与文字间距、鼠标偏移和 overlay 字体均来自 `EditorDesignTokens` 与 `EditorTypographyTokens`。拖放目标坐标保留为运行时数据；样式覆盖仅接受有效 CSS 颜色和有限非负度量，非法输入回退中央默认。`render_drag_overlay.rs` 更新中央 token 断言和渲染器边界守卫。已执行 scoped `rustfmt`,无旧拖拽色板、模块超限和 diff 静态守卫。 | 仍需在桌面宿主稳定后经受管 Cargo 验证当前 Rust 测试，并以当前源启动 retained host 后把真实截图写到 `docs/tests/editor/`；继续处理分段控件、滑块和容器原子控件。 |
| 2026-07-27 | 01.S2 分段控件资产 token 输入收束 | partial-asset-verified | `workbench_segmented_control.zui` 已声明 normal/hover/pressed/selected/disabled 表面、focus/selected underline、正文与禁用文字、border/radius 及 logical body line-height 的统一 token；`editor_design_token_contracts.rs` 增补完整输入断言。资产无裸 hex、无原始 layout 度量，布局继续用容器 Stretch 约束以保持等分自适应。 | `segmented_controls.rs` 仍需把历史默认常量硬切换为 `EditorDesignTokens`/`EditorTypographyTokens` 投影，保留其现有选项等分与 focused-only 中性表面行为。 |
| 2026-07-27 | 01.S2 分段控件 runtime token 投影 + 兼容性回归 | partial-static-verified | `segmented_controls.rs` 以缓存的 `SegmentedVisual` 取代局部调色板与字号常量；normal/hover/pressed/disabled 表面、边框、正文/标签/Tab 字排、等分选项内边距与下划线由 `EditorDesignTokens`、`EditorTypographyTokens`、density 和 control token 推导。焦点边框、选中段边框及下划线保留独立的有效覆盖通道，`selected_border_width` 与既有 `selected_border_color` 语义未被合并；无效 CSS 色和非法度量回退中央默认。`render_segmented_controls.rs` 覆盖默认色阶、不可用状态、有效覆盖优先、非法回退与选中段边框兼容性。已执行 scoped `rustfmt`、无遗留常量/令牌钩子/测试覆盖静态守卫与 `git diff --check`。 | 仍需在桌面宿主稳定后经受管 Cargo 验证当前 Rust 测试，并以当前源启动 retained host 后把真实截图写到 `docs/tests/editor/`；继续处理滑块与容器原子控件。 |
| 2026-07-27 | 01.S2 Slider/RangeSlider token 投影 + 适配布局 | partial-static-verified | `sliders.rs` 以缓存的 `SliderVisual` 取代历史色板、11px 字号与固定视觉度量；轨道、值框、标签/数值文字、thumb、tick、disabled、warning/error、4px Slate 轨道/8px thumb/弱文字 halo 从 `EditorDesignTokens`、`EditorTypographyTokens`、density 和 control token 推导。默认 fill 保持 `separator.strong` 中性语义，避免把焦点或常规调整误渲染为高饱和 accent；tick 预算和 RangeSlider 双 thumb 数据路径保持不变。命令构造与状态色裁决分别移至 `sliders/commands.rs`、`sliders/state_colors.rs`，令主渲染模块保持在大小阈值内。两份 slider 资产现引用统一 token；RangeSlider 使用 Stretch 最小高度，在不足以放置第二值框时由 runtime 几何自然折叠。`render_sliders.rs` 覆盖令牌钩子、默认/不可用/焦点状态、有效覆盖和非法回退，资产契约覆盖两份输入。已执行 scoped `rustfmt`、无历史常量/裸 hex/模块超限静态守卫与 `git diff --check`。 | 仍需在桌面宿主稳定后经受管 Cargo 验证当前 Rust 测试，并以当前源启动 retained host 后把真实截图写到 `docs/tests/editor/`；继续处理容器、列表行和窗口组合。 |
