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
| **状态语义(state-roles)** | 各状态用哪组色(default/hovered/focused/selected/pressed/disabled) | 与选择器优先级 disabled>pressed>selected/focused>hovered>default 一致 |

### 3.2 设计语言契约文档

一份契约 markdown,把"一致的设计风格"写成可执行规则:每条规则引用具体 token、给出适用与禁用场景(对应 STYLE-NOTES 的 NO 列表:无渐变/辉光/阴影/嵌套卡片/英雄字号),作为 02-06 全部子计划的视觉验收基线。

### 3.3 Token 喂入路径

token 资产 → 样式选择器输入(不绕过 `UiPainterStyleSelector`)→ 绘制族消费。组件 `.zui` 引用 token 名而非裸色值,保证改一处 token 全局生效。

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
| 新增 | `zircon_editor/assets/ui/editor/theme/editor_tokens.v2.ui.toml` | token 资产 |
| 新增 | `docs/ui-and-layout/design-language-contract.md` | 设计语言契约文档 |
| 修改 | `paint_template_nodes/style_selector` | 接受 token 喂入 |

## 6. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
| -- | --- | --- | --- | --- |
| S1 | token 资产骨架 + 契约文档 | design_tokens.rs / editor_tokens.v2.ui.toml / design-language-contract.md | `cargo check -p zircon_runtime_interface --locked` | 新建,无旧路径 |
| S2 | token 喂入选择器 + 组件引用 token 名 | style_selector / 组件 `.zui` | `cargo test -p zircon_editor --lib --locked` | 删除组件内联裸色值 |

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
| 2026-06-23 | 01.S1 token 资产骨架 + 契约文档 | completed | 已新增 `zircon_runtime_interface/src/ui/design_tokens.rs`、`zircon_editor/assets/ui/editor/theme/editor_tokens.v2.ui.toml`、`docs/ui-and-layout/design-language-contract.md` 与 `zircon_runtime_interface/src/tests/editor_design_tokens.rs`;`zircon_runtime_interface/src/ui/mod.rs` 已导出 token 模块。`cargo test -p zircon_runtime_interface --lib editor_design_tokens --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-runtime-interface-0623` 3/3 通过;scoped rustfmt 与 `git diff --check` 通过。 | 01.S2:把 token 喂入样式选择器,并清理组件 `.zui` 内裸色值/裸控件规格引用。 |
| 2026-06-23 | 01.S2 token 喂入选择器 + 布局新增资产引用 token 名 | partial-runtime-interface-passed-editor-cargo-blocked | 已新增 `EditorResolvedPainterStyle` 与 `EditorDesignTokens::resolve_painter_style(...)`,通过 `UiPainterStyleSelector::resolved_state_for_family(...)` 后再映射 palette/foreground/border/radius/height,不改 selector 优先级。`editor_tokens.v2.ui.toml` 增加 `editor.*` token 名表;`workbench_skeleton.zui`、`command_palette.zui`、`preferences.zui` 导入 token 资产并引用 `editor.surface.*`/`editor.text.*`/`editor.border`;`editor_layout_contracts.rs` 增加新增布局资产不得回退裸 hex 的静态契约。RED: focused runtime-interface 测试先因缺 `resolve_painter_style` 失败;GREEN: `cargo test -p zircon_runtime_interface --lib editor_design_tokens_feed_painter_styles_through_selector_state --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-token-feed-0623` 1/1 通过。 | 继续 01.S2 的 wider cleanup:旧 `components/workbench/shell` 与 `components/workbench/modules` 仍有历史裸 hex/裸规格值,需要在 editor Cargo lane 恢复后按资产族逐步替换为 token 引用并接入 retained painter 端验收。当前 `zircon_editor` Cargo gate 仍被 active render mesh import 漂移阻塞。 |
| 2026-06-23 | 01.S2 density token lookup + editor lane restored | partial-editor-verified | `EditorDesignTokens::density_value_for_token_name(...)` 已把 `editor.density.*` 与 `--left-drawer-width`/`--right-drawer-width`/`--bottom-output-height` 映射到中央 density token,供 02.S2 壳声明投影消费。为恢复 editor 验证,最小修复下层 render mesh owner split 后的 re-export/import 漂移:`MeshPassCommandBuffers`、`CachedMeshDrawLookup` 与 `mesh_draw_command_list::builder` 上级路径。验证:`cargo test -p zircon_runtime_interface --lib editor_design_tokens --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-token-feed-0623 --message-format short --color never` 5/5 通过;`cargo check -p zircon_editor --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never` 通过;`cargo test -p zircon_editor --lib editor_layout_contracts --no-run --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never` 通过,随后直接运行测试二进制 `editor_layout_contracts --test-threads=1 --nocapture` 8/8 通过。 | 01.S2 仍未关闭整个资产族 hard cutover:旧 shell/module 资产的历史裸 hex/裸规格值和 retained painter 端视觉验收继续保留为后续项。 |
