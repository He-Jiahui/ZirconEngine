---
related_code:
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/template/asset/compiler/ui_style_resolver.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdowns.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer.rs
  - docs/zircon_editor/ui/retained_host/host_contract/paint_template_nodes/style_selector.md
  - dev/material-ui/packages/mui-material/src/styles
plan_sources:
  - .codex/plans/Material UI 共享组件风格收束计划.md
  - .codex/plans/Material UI 全组件样式设计与验证计划.md
  - .codex/plans/Editor 基础组件 Material 化视觉优化计划.md
  - .codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md
status: planned
---

# 04 样式主题与中立绘制状态选择器

## 1. 目标

两件事：(a) 建立中央主题（theme token）治理——目前没有任何 theme 文档/资产类型（归档 M11 未完成项）；(b) 把已存在于接口层的 `UiPainterStyleSelector` 推到全组件覆盖——组件逻辑只产出语义状态，最终视觉样式由 selector 按固定优先级解析，逐组件删除中立 `paint_template_nodes/` 绘制族里的 hovered/pressed/disabled 内联分支（归档 M2 伪状态样式应用的正面解决）。

## 2. 现状（按代码核实修正）

### 2.1 已存在的设施

| 能力 | 落点 | 证据 |
|------|------|------|
| **状态选择器（已在接口层落地一轮）** | `zircon_runtime_interface/src/ui/style.rs` | `UiPainterStyleSelector`（:272）、`UiPainterFamily`（:209）、`UiPainterResolvedState`（:233，含 Disabled/Loading/Pressed/Focused/Hovered/DropHovered/Selected 等）、`UiPainterState`（:254，`normal()`/`is_active`/`is_focus_visible`/`is_pointer_hot`）；按 family 折叠：`resolved_state_for_family`/`interactive_resolved_state`/`selection_control_resolved_state`/`slider_resolved_state`/`button_resolved_state`（:275–:455） |
| Button 样式语义 | 同上 | `ButtonVariant/ButtonColor/ButtonSize/ButtonIconPlacement/ButtonInteractionState`（:105–:207）、`ResolvedButtonStyle`（:471） |
| 基础样式 DTO | 同上 | `UiRgbaColor`（:4）、`UiStyleColor`（:52）、`StyleDimension`（:61）、`UiResolvedElementStyle`（:77） |
| v2 样式解析骨架 | `zircon_runtime/src/ui/v2/style.rs` | `UiV2StyleResolver::resolve`（:16/:19）——**无伪状态（hover 等）处理** |
| 模板编译样式链 | `zircon_runtime/src/ui/template/asset/compiler/` | `ui_style_resolver.rs`、`style_apply.rs` + `style_apply/` 目录 |
| editor 中立软件绘制族 | `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/` | `style_selector/`（已消费接口 selector）、`material_state_layer.rs`、`template_buttons.rs`、`template_icon_buttons.rs(+tests)`、`template_dropdowns.rs`、`template_popup_rows.rs`、`template_list_rows.rs`、`template_property_rows.rs`、`template_inspector_rows.rs`、`template_fields.rs`、`template_chips.rs`、`template_alerts.rs`、`material_primitives/`、`mui_x_primitives/` 等；旧 `host_contract/painter/` 目录已在 08 M3.S2 硬删除 |

即：**优先级共享单实现已成立**（selector 在 interface，editor retained-host 软件绘制族经 `paint_template_nodes/style_selector/` 消费同一份）。

### 2.2 真实缺口

1. **无中央主题**：仓内不存在 `UiThemeDocument`/theme 资产类型（grep 无命中）；颜色/尺寸散落在模板与 `paint_template_nodes/` 常量里，无 token source-chain 可回溯，无裸 hex 禁令，无主题热重载。
2. **v2 伪状态解析缺失**：`UiV2StyleResolver` 无 `:hover/:pressed/:focused/...` 匹配；render extract 侧拿不到状态分档样式，状态视觉目前只在 retained-host 软件绘制一侧成立——双路不同源。
3. **状态集不全且生产者未收口**：`UiPainterState` 覆盖主要交互态，但 checked/open/dragging 等语义态的写入路径分散（依赖 01 M3 的 reply 统一后才有唯一生产者）。
4. **组件内联分支未清**：`template_buttons.rs` 等投影文件仍有按状态硬编码的颜色/边框分支，未全部改经 selector + theme token。

## 3. 设计

### 3.1 主题 token 治理（M11 补课）

- `zircon_runtime_interface/src/ui/style.rs` 升级为 `style/` 目录（mod.rs 薄声明 + `base.rs`（现有类型平移）+ `theme.rs`（新增）+ `selector.rs`（selector 平移）），新增 `UiThemeDocument`：palette（surface 0–3 层、text primary/secondary/disabled、accent、success/info/warning/error、separator）、typography 阶（对齐 MUI variant：body/caption/subtitle/title）、shape（圆角档 4/5/8/12）、spacing 阶、控件尺寸档（default 40 / compact 32 / dense 28）、elevation。
- 主题作为资产进资产管线，支持热重载：theme 变更 → style 解析缓存指纹失效 → 全表面 restyle，不重建树。（2026-07-02 评审收口）`.zui` 是唯一 UI 资产后缀：theme 文档 = `asset.kind = theme_tokens` 的 **`.zui` profile**（生产已有 `editor_tokens.zui`），序列化形态对齐 `editor_tokens.zui`；原「TOML theme 文档 / `.theme.toml`」新载体方案作废，计划 05 的注册按 theme_tokens profile 物化/facade 执行。
- token source-chain 校验：每个最终颜色/尺寸能回溯到 token id；CI 测试拒绝模板与 painter 源中的裸 hex（白名单除外）。
- （2026-07-02 评审收口）token 权威让渡：token 命名/值的规范权威 = editor_layout/01（design tokens 语言契约）+ editor_layout/15c（retained palette 单一来源）；本计划只做 **ThemeRegistry 运行时消费与热重载**，不定义新 token 命名；token 引用语法遵 editor_layout/01 统一文法（含 `$token` 形式）。

### 3.2 统一状态模型

- 以现有 `UiPainterState` 为底盘补全 12 态清单：`normal, hovered, pressed, focused, focus-visible, disabled, checked, selected, open, dragging, drop-hover, loading`（DropHovered/Loading/Selected 已在 ResolvedState）。
- 状态生产者唯一：hover/press/focus/capture/dragging 由计划 01 的 input manager 经 reply 写入 component state；checked/selected/open/loading 由组件语义状态机（06 的 state reducer）写入。painter 与 render extract 都只读。

### 3.3 selector 全覆盖

- runtime 侧：v2 style resolver 补伪状态匹配（`:hover/:pressed/:focused/:disabled/:checked/:selected/:open` 与组合），折叠规则**直接调用接口层 `UiPainterStyleSelector` 的 resolved-state 函数**——禁止在 v2 内再写一份优先级表。
- editor painter 侧：已有 `style_selector/`，任务是把 template_* 投影文件中残余内联分支逐组件切到 selector + theme token。
- 切换顺序与文件映射：Button（template_buttons.rs）→ IconButton（template_icon_buttons.rs）→ Checkbox/Radio/Toggle（material_primitives/ 内选择控件段）→ Slider（material_primitives/ 滑杆段）→ Dropdown/PopupRow（template_dropdowns.rs、template_popup_rows.rs）→ Tab/SegmentedControl（按盘点定位）→ ListRow/TreeRow/TableRow（template_list_rows.rs、template_property_rows.rs、template_inspector_rows.rs）→ Tooltip/Toast（template_alerts.rs 与相邻文件）。每组件切换同变更删除旧分支。

### 3.4 验证

- 状态矩阵快照测试：每组件 × 每状态组合的 resolved style 快照（runtime extract 与 native painter 双路对拍，保证视觉同源）。
- 复用 component-prototype 的 `verify-native-component-contract.mjs` retained selector-state 覆盖检查。

### 3.5 与 editor_layout/20 的 gating（2026-07-02 评审收口）

- `UiPainterStyleSelector` 的**固定优先级折叠是 editor_layout/20 级联引擎落地前的过渡形态**；20 是 style 级联/computed style 的规范权威。
- M3 的 v2 伪状态折叠按 20 的 **computed style 形态**实施（解析产物直接采用 20 定义的 computed style 结构），避免「先按旧形态迁一次、20 落地后再迁一次」的双次迁移。
- 20.S2 验收后，`UiPainterStyleSelector` 的固定优先级表**降级为内置默认 stylesheet**（级联引擎的最低优先级层），不再是独立解析路径。
- transition 声明字段（2026-07-02 评审收口，U7）：schema 归 editor_layout/20（style 规范权威）；本计划 M3 的解析链**承接实现**（伪状态折叠时携带 transition 声明），计划 07 消费驱动；三方按此引用，不各自定义字段。

## 4. 接口与数据结构草案

```rust
// 新增 zircon_runtime_interface/src/ui/style/theme.rs
pub struct UiThemeDocument {
    pub id: String,                           // "zircon.dark"（浅色主题结构预留）
    pub palette: UiThemePalette,
    pub typography: Vec<UiThemeTypographyVariant>,
    pub shape: UiThemeShape,
    pub spacing: Vec<f32>,                    // 4/8/12/16/24…
    pub control_sizes: UiThemeControlSizes,
    pub elevation: Vec<UiThemeElevation>,
}
pub struct UiThemePalette {
    pub surface: [UiRgbaColor; 4],            // #111416 / #171a1d / #1b1f23 / #252b31
    pub text_primary: UiRgbaColor,
    pub text_secondary: UiRgbaColor,
    pub text_disabled: UiRgbaColor,
    pub accent: UiRgbaColor,                  // teal #3cc7d6，仅激活/选中/焦点
    pub success: UiRgbaColor, pub info: UiRgbaColor,
    pub warning: UiRgbaColor, pub error: UiRgbaColor,
    pub separator: UiRgbaColor,
}
pub struct UiThemeTypographyVariant { pub variant: String /* body|caption|subtitle|title */, pub family: String, pub size: f32, pub weight: u16, pub line_height: f32 }
// （2026-07-02 评审收口）`family: String` 语义：解析经字体 token → text/01 CompositeFont/FontFaceId 回退链，
// 不直持裸字符串语义——family 字符串仅是 token/字体名查询键，最终以 FontDatabase 解析结果为准。
pub struct UiThemeShape { pub radius_small: f32, pub radius_medium: f32, pub radius_large: f32, pub radius_panel: f32 }   // 4/5/8/12
pub struct UiThemeControlSizes { pub default_height: f32, pub compact_height: f32, pub dense_height: f32 }                 // 40/32/28
pub struct UiThemeTokenRef(pub String);       // 可回溯 token id，如 "palette.surface.1"

// theme 文档载体（2026-07-02 评审收口，U6）：asset.kind = theme_tokens 的 `.zui` profile，
// 序列化形态对齐生产已有的 editor_tokens.zui（不再引入 .theme.toml 第二载体）。
// 示例（.zui theme_tokens profile，字段以 editor_tokens.zui 为准）：
// [asset]              kind = "theme_tokens"  id = "zircon.dark"
// [palette]            surface = ["#111416", "#171a1d", "#1b1f23", "#252b31"]
// accent = "#3cc7d6"   text_primary = "#e8ecee" ...
// [[typography]]       variant = "body"  family = "Inter"  size = 13.0  weight = 400  line_height = 1.45
// [shape]              radius_small = 4.0  radius_medium = 5.0  radius_large = 8.0  radius_panel = 12.0
// [control_sizes]      default_height = 40.0  compact_height = 32.0  dense_height = 28.0

// 新增 zircon_runtime/src/ui/theme/{mod.rs, loader.rs, registry.rs, token_check.rs}
pub struct UiThemeRegistry { active: UiThemeDocument, fingerprint: u64 }
impl UiThemeRegistry {
    pub fn resolve_token(&self, token: &UiThemeTokenRef) -> Option<UiStyleColor>;   // 现有 UiStyleColor
    pub fn apply_document(&mut self, doc: UiThemeDocument) -> UiThemeReloadOutcome; // 指纹变更 → restyle 失效
}

// 修改 zircon_runtime/src/ui/v2/style.rs：伪状态匹配
pub struct UiV2PseudoState(pub UiPainterState);   // 复用现有状态结构
impl UiV2StyleResolver {
    pub fn resolve_with_state(
        &self, /* 既有参数, */ state: UiPainterState,   // 现有类型
    ) -> UiV2ResolvedStyle;   // 内部调 UiPainterStyleSelector::resolved_state_for_family 折叠
}
```

## 5. 模块与文件落点

**新增**：`zircon_runtime_interface/src/ui/style/{mod.rs, base.rs, theme.rs, selector.rs}`（由 style.rs 拆分，mod.rs 薄）、`zircon_runtime/src/ui/theme/{mod.rs, loader.rs, registry.rs, token_check.rs}`、默认 theme 文档 = `.zui` theme_tokens profile（对齐生产 `editor_tokens.zui`；2026-07-02 评审收口，原 `zircon.dark.theme.toml` 载体作废）

**修改**：

| 路径 | 改什么 |
|------|--------|
| `zircon_runtime/src/ui/v2/style.rs` | `resolve_with_state` 伪状态匹配 + selector 折叠 |
| `zircon_runtime/src/ui/template/asset/compiler/{ui_style_resolver.rs, style_apply.rs}` | token 引用解析（颜色字段接受 token id）、伪状态类生成 |
| `zircon_editor/.../paint_template_nodes/style_selector/` | 接 theme registry token 表 |
| `zircon_editor/.../paint_template_nodes/template_buttons.rs` 等切换清单（§3.3） | 状态分支改 selector + token，逐文件 |

**删除（硬切换义务）**：每个组件切换时，对应 template_* 文件内的 hovered/pressed/disabled 颜色硬编码分支与裸 hex 常量；`material_state_layer.rs` 中与 selector 重复的状态判定段。

## 6. 管线时序

状态写入（input manager reply / state reducer）→ component state → **样式解析**（v2 resolver 伪状态折叠 / native painter style_selector，同源 selector 常量）→ render extract / painter 输出。theme 热重载：asset watch（05）→ `UiThemeRegistry::apply_document` → 指纹变更 → style 解析缓存失效 → 全表面 restyle（不重建树）→ damage。

## 7. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
|---|------|---------|---------|--------|
| M1.S1 | style.rs 拆 style/ 目录（纯平移，零行为变化） | interface style/ | `cargo test -p zircon_runtime_interface --locked` | style.rs 删除（被目录取代） |
| M1.S2 | `UiThemeDocument` + 默认 dark theme（`.zui` theme_tokens profile，对齐 editor_tokens.zui；2026-07-02 评审收口，原 TOML 新载体作废）+ ThemeRegistry/loader | theme.rs、runtime ui/theme/ | `cargo test -p zircon_runtime --lib theme --locked` | 无删除 |
| M1.S3 | 散落 hex 收编进 token + 裸 hex 扫描测试（扫 `.zui` UI 文档与 `paint_template_nodes` 源，白名单清单显式） | token_check.rs、各模板 | `cargo test -p zircon_runtime --lib token_check --locked` | 中立绘制族内重复常量表删除 |
| M2.S1 | `UiPainterState` 状态集盘点补全（checked/open/dragging 字段与折叠规则） | interface style/selector.rs | `cargo test -p zircon_runtime_interface --locked` | 无删除 |
| M2.S2 | 生产者唯一化：交互态只由 01 reply 写入、语义态只由 reducer 写入（依赖 01 M3） | surface 组件状态写入点 | `cargo test -p zircon_runtime --lib --locked` | 散落写入点删除 |
| M3.S1 | v2 伪状态匹配 + `resolve_with_state`（折叠调 selector，不复制优先级） | v2/style.rs、ui_style_resolver.rs | `cargo test -p zircon_runtime --lib v2_style --locked` | 无删除 |
| M3.S2 | 状态矩阵快照（runtime 侧）：family × 状态组合 resolved style 基线 | v2 测试 | 同上 | 无删除 |
| M4.S1 | Button 切换：template_buttons.rs 全分支改 selector+token，删内联 | template_buttons.rs | `cargo test -p zircon_editor --lib template_buttons --locked` | 删内联分支 |
| M4.S2 | IconButton 切换（同模式，template_icon_buttons.rs + tests） | template_icon_buttons* | `cargo test -p zircon_editor --lib icon_buttons --locked` | 删内联分支 |
| M4.S3 | 双路对拍：Button/IconButton 状态矩阵 extract vs painter 同源断言 + 实机 hover/press 无回归 | 对拍测试 | `cargo test -p zircon_editor --lib --locked` + 实机 | 无删除 |
| M5.S1 | Checkbox/Radio/Toggle + Slider 切换（material_primitives/ 内） | material_primitives/ | `cargo test -p zircon_editor --lib material_primitives --locked` | 删内联分支 |
| M5.S2 | Dropdown/PopupRow + Tab/Segmented 切换 | template_dropdowns.rs、template_popup_rows.rs 等 | `cargo test -p zircon_editor --lib --locked` | 删内联分支 |
| M5.S3 | Rows（list/property/inspector）+ Tooltip/Toast 切换；全组件状态矩阵 + 契约脚本 | template_*_rows.rs、template_alerts.rs | 同上 + `verify-native-component-contract.mjs` | 删内联分支 |
| M6.S1 | 主题热重载（依赖 05 M2）：watch → registry → restyle 失效 → damage | ui/theme/、05 watch 链 | `cargo test -p zircon_runtime --lib theme_reload --locked` | 无删除 |
| M6.S2 | 实机改 theme 文件即时生效验收 | 实机 | editor 实机 | 无删除 |

## 8. 测试矩阵（代表性用例）

- **M1**：`theme_document_round_trips_toml`、`token_resolves_to_palette_color`、`bare_hex_scan_rejects_unlisted_literal`
- **M2**：`painter_state_priority_disabled_over_pressed`、`interaction_state_written_only_by_reply`、`semantic_state_written_only_by_reducer`
- **M3**：`v2_pseudo_hover_matches_only_when_hovered`、`v2_state_fold_uses_selector_priority`、`state_matrix_snapshot_button_family`
- **M4/M5**：`button_resolved_style_same_for_extract_and_painter`、`dropdown_open_state_styles_from_selector`、每组件 `*_state_matrix_snapshot`
- **M6**：`theme_reload_invalidates_style_cache_without_tree_rebuild`

落点：interface/runtime 模块内 `#[cfg(test)]`；editor 侧沿 `paint_template_nodes` 既有 `template_icon_buttons_tests.rs` 同级惯例。

## 9. 风险与对策

| 风险 | 对策 / 探测信号 |
|------|----------------|
| 全组件视觉基线被 token 化「顺手改色」 | 切换切片只允许等值替换；状态矩阵快照先建基线，diff 必须为空 |
| v2 与 retained-host 绘制折叠规则漂移（两份优先级） | 折叠只调 interface selector 函数；对拍测试断言双路同 resolved state |
| 裸 hex 禁令误伤合理常量（调试色、图标原色） | 白名单文件 + 行级 allow 注释机制，白名单变更需评审 |
| 状态生产者收口依赖 01 M3 进度 | M2.S2 排在 01 M3 后；之前 M1/M3 可先行 |
| theme 拆目录引发 interface 大量 use 路径变化 | M1.S1 纯平移 + re-export 保持公开路径不变（curated re-export，非兼容桥） |

## 10. 里程碑级依赖表

| 里程碑 | 前置 | 被依赖 |
|--------|------|--------|
| M1 | 无 | 04 M3/M4、05 M1（UiThemeAsset 注册）、06 全部（token 消费） |
| M2 | 01 M3 | 04 M3、06 M1 |
| M3 | 04 M1、04 M2 | 04 M4/M5、06 M1 |
| M4 | 04 M3 | 04 M5、06 M1（Button 族 DoD 衔接） |
| M5 | 04 M4 | 06 M1–M3 |
| M6 | 04 M1、05 M2（watch 级联） | 09 批次 3（Theme Token 预览） |

## 11. 完成定义

- 仓内任何最终颜色/尺寸可回溯 token id；裸 hex 扫描常绿。
- v2 与 retained-host 绘制状态折叠同源；全组件状态矩阵快照双路一致。
- template_* 投影文件无状态硬编码分支。
- 实机：hover/press/focus/disabled 视觉无回归；改 theme 文件即时生效。
- 验收命令组：`cargo test -p zircon_runtime_interface --locked`、`cargo test -p zircon_runtime --lib --locked`（theme/v2_style/token_check 过滤）、`cargo test -p zircon_editor --lib --locked`、`node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-component-contract.mjs`。

## 12. 边界约束

- 第一轮不做 ripple/motion 动画，只做静态状态反馈（动画归计划 07）。
- 不新增公开 DTO 字段超出状态集与 theme 文档所需；`TemplatePaneNodeData` 既有字段约束沿用。
- 浅色主题暂不实现，但 `UiThemeDocument` 结构必须可表达第二主题（id 区分、结构预留、内容后置）。
- 优先级表只存在于 interface selector 一处；v2/painter/任何新消费方只许调用、不许复制。

## 13. 参考实现对照（dev/ 源码锚点）

实现切片前先读对应锚点，不确定的行为语义以参考实现为准（在 PR 说明中注明出处）；禁止凭印象实现、禁止引用未核实路径。

| 设计点 | 主参考 | 次参考 | 参考什么 |
|--------|--------|--------|---------|
| theme 结构与命名权威 | `dev/material-ui/packages/mui-material/src/styles` | — | createTheme 的 palette/typography/shape/spacing/transitions 字段组织——`UiThemeDocument` 字段命名以此对齐 |
| 组件 variant/状态样式 | `dev/material-ui/packages/mui-material/src/Button`（及任一组件目录） | `dev/material-ui/packages/mui-material/src/Collapse` | ownerState → 样式折叠的模式、状态类（Mui-disabled 等）与本计划伪状态的对应 |
| 注册式样式集 | `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling` | — | FSlateStyleSet/Brush 注册表：token id → 视觉资源的回溯链样板 |
| 控件主题查询链 | `dev/godot/scene/gui/control.cpp` | — | Godot theme 项的 control → theme owner → default 级联查找（token source-chain 对照） |
| 状态装饰器 | `dev/Fyrox/fyrox-ui/src/{decorator.rs, brush.rs}` | — | hover/pressed/selected 的装饰器切换实现（selector 的另一种形态，对照取舍） |

## 14. 状态与产出记录

| 日期 | 范围 | 状态 | 完成项目 | 验证 |
| --- | --- | --- | --- | --- |
| 2026-06-28 | Plan 11 M5 style/theme token scan `.zui` target guard | editor_ui_11_m5_style_theme_token_scan_zui_guard_passed | M1.S3 的裸 hex 扫描目标已从 `.zui` + `*.v2.ui.toml` 双后缀改为 `.zui` UI 文档与 `paint_template_nodes` 源；后续 token 检查不再把退役 `.v2.ui.toml` 写成未来扫描目标。 | 新增 `test_style_theme_plan_token_scan_targets_zui_documents_only`；RED 先失败列出旧 `*.v2.ui.toml` 扫描目标，GREEN 后通过。该切片不改生产代码、不运行 Cargo。 |
| 2026-07-02 | 评审收口（文档修订） | editor_ui_04_review_alignment_recorded | theme 载体定稿 `.zui` theme_tokens profile（U6，.theme.toml 作废）；新增 §3.5 与 editor_layout/20 的 gating（含 transition schema 归属 U7）；token 命名/值权威让渡 editor_layout/01+15c；typography family 补 text/01 FontFaceId 回退链语义。 | 文档修订，无代码变更。 |
| 2026-07-23 | Runtime interface skin performance handoff | editor_ui_04_skin_generation_catalog_performance_pending | `ui/skin/**` 2/2当前无产品consumer，但每次preset构造会重建全部String/Vec/token且token lookup线性。EditorUI04按PERF-MVP-264在首次接线时建立single static/theme-generation catalog与compact token index，paint/pane稳定代不得调用构造器或clone descriptor；参考UE `FSlateStyleSet`长期FName-keyed registry的owner原则。 | presets/tokens 1/31/1k/10k、1M lookups记录constructor、String/Vec bytes、comparisons与p95；stable constructor/clone=0、lookup近O(1)，reload原子发布、合同/Cargo/F4待验收。 |
| 2026-07-23 | Runtime interface design/style performance handoff | editor_ui_04_theme_document_generation_performance_pending | `ui/{design_tokens,style}.rs`确认workbench/theme document构造会重建id/font/variant String与typography/spacing/elevation Vec，palette default还重复生成surface数组；painter selector本身为Copy/const正向基线。EditorUI04沿PERF-MVP-157/251/264让theme generation长期拥有document、font/token identity与resolved style，stable paint不得调用constructor/to_theme_document或复制family。 | themes/variants/tokens 1/100/10k、1M stable resolves记录constructor、String/Vec/family clone、resolve与p95；stable constructor/clone=0、state selector无堆分配，reload一次发布且视觉/serde/Cargo/F4等价。 |
| 2026-07-23 | Runtime interface template selector/invalidation handoff | editor_ui_04_template_selector_generation_performance_pending | `ui/template/**`确认selector先建compound String、再建char Vec/token String，且document/style/component/invalidation多路重复parse；selector/style impact无条件扩散到style/layout/hit/render/text。EditorUI04按PERF-MVP-307/309/311让style generation持budgeted parsed selector与候选index，由matched rule/property effect产精确node/domain dirty，stable resolve不得parse或重建path。 | nodes/rules/selectors各1/100/10k、depth 1/64/1k记录parse/index builds、char/token/path bytes、rule probes、dirty domains与p95；同generation selector parse=1、stable parse=0、局部style不触发无关layout/hit/text，视觉/Cargo/F4待验收。 |
| 2026-07-23 | Runtime interface v2 style clean handoff | editor_ui_04_v2_style_delta_performance_pending | `ui/v2/style.rs`的resolved style仍以三张String/TOML BTreeMap拥有结果，`merge_block()`整表clone后extend；当前未找到产品调用，作为接线前门禁。EditorUI04按PERF-MVP-275让compile generation发布typed resolved style，runtime state变化只合并changed fields，不把wide map merge接入pointer/paint稳定路径。 | nodes/rules/keys各1/100/10k、1M state resolves记录map/value clone bytes、changed keys/nodes与p95；stable map clone=0、merge随delta、视觉/serde/Cargo/F4等价。 |
