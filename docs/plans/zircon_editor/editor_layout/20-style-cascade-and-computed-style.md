---
related_code:
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime_interface/src/ui/template/asset/style.rs
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/v2/style/runtime_state.rs
  - zircon_runtime/src/ui/v2/style/tokens.rs
  - zircon_runtime/src/ui/template/asset/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector
plan_sources:
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_layout/01-design-tokens-and-language-contract.md
  - docs/plans/zircon_editor/editor_layout/13-taffy-css-constraint-language.md
  - docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md
  - docs/plans/zircon_editor/editor_layout/19-focus-and-navigation-model.md
status: in_progress
---
# 20 样式系统:USS/CSS 级联、选择器与计算样式

## 1. 目标

把"作者写的样式规则如何匹配节点、如何级联合成最终样式"沉淀为一份**类 USS/CSS 的级联样式规范**。当前 v2 路径已具备 stylesheet parser、type/class/id/state/host/part token、child/descendant combinator、tuple specificity、源序级联、runtime pseudo-state 和 design-token `var()`；旧 retained painter 仍保留固定优先级状态选择。计划重点已从“从零创建选择器”转为收敛两条样式路径、补齐继承/computed-style 边界、清理绘制族内联状态分支和 transition。

> 工程化硬目标(接 `index` §4.0):样式是**层叠数据**,不是调用点的一次性视觉值。改一处 token/规则,所有匹配节点随动;状态(hover/focus/checked)由伪状态选择器表达,不在绘制族里写 `if state == ...` 分支。

## 2. 现状(按代码核实)

- `zircon_runtime_interface/src/ui/template/asset/style.rs` 已定义 `UiSelector`、child/descendant combinator、type/class/id/state/part/host token 与不可压平为整数的 `UiSelectorSpecificity`；`UiV2StyleResolver` 按 `(specificity, source order)` 排序并逐属性覆盖。
- `zircon_runtime/src/ui/v2/style.rs`/`runtime_state.rs` 已支持 authored 与 retained runtime pseudo-state、深层 descendant 状态匹配和 subtree dirty；`tokens.rs` 与 `register_editor_design_tokens` 已让 stylesheet/inline value 共享 token/`var()` 解析。`UiSelectorSpecificity` 的 interface export 已在源码恢复，但对应 open failure 仍需 fresh managed gate 后才能关闭。
- `zircon_runtime_interface/src/ui/style.rs` 的 `UiPainterStyleSelector`/`UiPainterResolvedState` 仍按 family 固定优先级折叠；retained 软绘与 v2 resolver 尚未完全同源。
- 组件绘制族仍有内联状态分支(`editor_ui/04` §2.2.4)。
- 设计 token 到 stylesheet/inline `var()` 的注册与解析已存在；仍需补主题切换后的精确失效、继承属性的 computed value 和 retained painter 消费同一 resolved artifact 的验收。

## 3. 设计

### 3.1 选择器语言(对标 USS/CSS)

规范支持的选择器子集(对标 Unity USS、CSS;Unity USS 证据见 §11):

| 选择器 | 例 | 对标 |
| --- | --- | --- |
| 类型/family | `Button` | CSS 类型选择器;Unity `Button` |
| 类(变体) | `.primary` `.compact` | CSS class;Unity `.unity-button`、`.slide-toggle__input`(`SlideToggle.uss:1`) |
| 名字/id | `#viewport` | CSS `#id`;Unity `#name` |
| 伪状态 | `:hover` `:active` `:focus` `:focus-visible` `:checked` `:disabled` `:selected` `:focus-within` | CSS 伪类;Unity `:hover/:active/:checked/:focus`(`SlideToggle.uss:54`) |
| 后代/子 | `.toolbar Button` `.list > .row` | CSS 组合子;Unity `.slide-toggle:focus .slide-toggle__input-knob`(`SlideToggle.uss:54`) |
| 修饰类(BEM) | `.slide-toggle__input--checked` | Unity BEM 约定(`SlideToggle.uss:46`) |

伪状态来源:18(hover/active/press)、19(focus/focus-visible/focus-within)、组件态(checked/selected/disabled,接 11 数据)。

### 3.2 specificity 与级联(治"固定优先级")

规范:多条规则命中同一节点时,按 **specificity →（同级)源序** 决定胜出值,逐属性级联,对标 CSS cascade、Unity USS specificity:

- specificity = (id 数, 类+伪类数, 类型数),高者胜;同 specificity 后出现者胜(对标 CSS)。
- 内联(节点上直接写的属性,接 13 `.zui` 内联)> stylesheet 规则 > 继承 > 初始值,对标 `UiLayoutStyleSourceKind{Inline,Class,Asset,RuntimeState}`(`style.rs:274`,已有的来源枚举正可承载层级)。
- **固定优先级 selector 收编**:现 `UiPainterStyleSelector` 的 disabled>pressed>… 不再是硬编码折叠,而是**伪状态选择器 + specificity** 的自然结果(`:disabled` 规则 specificity/源序高于 `:hover`)。保留其作为"内置默认 stylesheet",但走同一级联引擎。

### 3.3 自定义属性 / 设计 token 作 CSS 变量(接 01)

规范:01 的 token 表达为**自定义属性**(`--editor-surface-1`、`--accent`、`--gap-m`),规则用 `var(--…)` 引用,对标 CSS custom properties、Unity USS `var(--unity-colors-*)`(`SlideToggle.uss:2`)。token 变更 → 所有 `var()` 引用随动重算(接 09/10 增量)。这统一了 13 的布局 token 与本计划的视觉 token:**同一套自定义属性,布局/视觉规则都能引用**。

补注(2026-07-02 评审收口):`01` 需补一项交付——**"token → 自定义属性注册表"**(token 名到 `--custom-property` 名的单源映射表,含命名规范),此处先行登记,由 01 侧回填;本计划 S1 的 `var()` 解析以该注册表为输入。

主题切换条款(2026-07-02 评审收口):多主题 = **多 token 集切换、规则不变**——stylesheet 规则只引用 `var(--token)`,切主题即整体替换自定义属性取值集,不改任何选择器/规则;运行时热切换(切换后全量 `var()` 引用随动重算、无需重建规则索引)登记为 **20.S2 验收项之一**。

### 3.4 computed style 与继承

规范:节点最终样式 = **computed style**(级联后逐属性定值 + 继承解析),对标 CSS computed value、Unity `ICustomStyle`/resolvedStyle:

- 可继承属性(字体族/字号/色/对齐等,接 17 文本)沿树继承;不可继承属性(背景/边框/布局)不继承,对标 CSS 继承模型。
- computed style 是渲染(10/21)与命中(18 视觉无关)的输入;一节点一份缓存,脏时重算(接 09)。

### 3.5 transition(状态过渡,接动画)

规范:属性可声明 `transition`(property + duration + easing),状态切换时插值,对标 CSS transition、Unity USS `transition-property`/`transition-duration`(`SlideToggle.uss:19-20,33-34`)。MVP 可只支持色/不透明/位移过渡,驱动接 `editor_ui/07`(theatre 动画)。非硬需求,可后置切片。

补(2026-07-02 评审收口):transition 的**位移**=渲染层 transform 偏移(只改提取/绘制端偏移,**不进布局求解**);样式属性按 **paint-only**(色/不透明/transform 偏移/阴影等,过渡只触发 paint/提取)与 **layout-affecting**(宽高/margin/padding/gap 等,过渡逐帧触发 relayout)两类分类登记;布局属性动画**须显式声明**方可参与 transition,并遵 13 §3.8-4 的动画 relayout 预算条款(该条款已落 13)。

### 3.6 双路同源(治"retained/extract 不同源")

规范:**级联引擎只有一处**;retained 软件绘制族与 render extract 都消费同一 computed style(治 `editor_ui/04` §2.2.2 双路不同源)。v2 resolver 升级为"调用统一级联引擎",**禁止再写第二份优先级表**(沿用 `editor_ui/04` §3.3 的硬规则)。

收编路线(2026-07-02 评审收口):15c 的 `palette_projection`(OnceLock 静态调色板投影)与 15b 的 `METRICS` 常量表,在 **20.S2 落地后降级为级联引擎的"内置默认值来源"**(即内置默认 stylesheet 的取值后端,与 §3.2 的 `UiPainterStyleSelector` 收编同一模式);删除时点 = **级联 `var()` 通路(S1/S2)验收通过后**,由 15b/15c 各自的移交条款执行(见 15b §7、15c §5)。

滚动条视觉归属(2026-07-02 评审收口,遵 R4 滚动线):滚动条 chrome 的伪状态样式(`:hover`/`:active`、空闲隐藏计时淡出)归**本计划**(伪状态选择器 + transition)+ 15(组件形态);滚动/滚轮/裁剪感知命中归 18,虚拟化契约归 `editor_ui/02` M3。

## 4. 接口与数据结构草案(Rust)

```rust
pub enum UiSelectorPart { Type(String), Class(String), Name(String), Pseudo(UiPseudo) }
pub enum UiPseudo { Hover, Active, Focus, FocusVisible, FocusWithin, Checked, Disabled, Selected }
pub enum UiCombinator { Descendant, Child }
pub struct UiStyleRule { pub selector: Vec<(UiCombinator, Vec<UiSelectorPart>)>,
    pub declarations: Vec<(UiStyleProp, UiStyleValue)>, pub source_order: u32 }
pub enum UiStyleValue { Keyword(String), Length(UiDimension), Color(UiColor), Var(String) /* var(--x) */ }

pub fn specificity(sel: &[(UiCombinator, Vec<UiSelectorPart>)]) -> (u16, u16, u16);
// 级联:命中规则集 + 内联 + 继承 → computed style(单源,retained/extract 共用)
pub fn compute_style(node: &UiNode, sheet: &UiStyleSheet, inherited: &UiComputedStyle,
    pseudo: UiPseudoSet, tokens: &EditorDesignTokens) -> UiComputedStyle;
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增(契约) | `docs/ui-and-layout/style-cascade-contract.md` | 选择器/specificity/级联/var/computed/继承/transition |
| 升级 | `zircon_runtime/src/ui/v2/style.rs` | resolver → 调统一级联引擎(伪状态 + 选择器 + var) |
| 收编 | `style.rs` `UiPainterStyleSelector` | 作内置默认 stylesheet,走同一级联,不再硬编码折叠 |
| 接入 | 01 token | token 暴露为自定义属性供 `var()` 引用 |

## 6. 里程碑切片化

| # | 切片 | 验证命令 |
| -- | --- | --- |
| S1 | 加固并受管验收现有选择器解析 + tuple specificity + 源序级联 + var/token；补继承/computed 边界 | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter style_cascade` |
| S2 | v2 resolver 接级联引擎(伪状态);retained/extract 同源 | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests` |
| S3 | 组件内联状态分支清除 → 伪状态选择器(收编固定优先级) | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests -TestFilter style_selector` |
| S4(可选) | transition 过渡(接 editor_ui/07) | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests -TestFilter style_transition` |

## 7. 测试矩阵

- 选择器:类型/类/名/伪状态/后代/子 各正确匹配。
- specificity:`#id` > `.class` > `Type`;同 specificity 后者胜。
- 级联:内联 > 规则 > 继承 > 初始;`:disabled` 压过 `:hover`(等价旧固定优先级,但由 specificity/源序得出)。
- `var(--token)` 解析为 01 token 值;token 变更随动重算。
- computed style:可继承属性继承、不可继承不继承。
- retained 与 extract 拿到同一 computed style(双路同源)。

## 8. 风险与对策

- 风险:级联引擎替换固定优先级引发视觉回归。对策:S3 用现有 buttons/selection 等快照测试做等价基线,逐族切换。
- 风险:选择器匹配性能(每节点遍历规则)。对标 CSS/USS 用 key 桶按右端 family/class 索引规则;脏节点才重算(接 09)。
- 状态失效传播规范(2026-07-02 评审收口):含伪状态的**祖先选择器**(如 `.toolbar:hover Button`、`.panel:focus-within .row`)在规则索引期为祖先节点登记**"状态依赖边"**(伪状态 → 受影响后代规则集);运行时该伪状态翻转时,沿依赖边只标脏受影响后代并重算其 computed style——不整树重算(对标浏览器 invalidation sets)。`:focus-within` 特例:焦点变更沿祖先链**向上传播**翻转 `focus-within` 态(来源归 19),再按依赖边向下标脏。该机制为 S2 的一部分,与 09 增量脏传播共用通道。

## 9. 完成定义

选择器语言 + specificity + 级联 + var/token + computed style + 继承成文且落实现;v2 resolver 接统一级联引擎,retained/extract 同源;固定优先级 selector 收编为内置 stylesheet;组件内联状态分支清除。

## 10. 边界约束

token 数据归 01;布局属性求解归 13/Taffy(本计划只定样式匹配/级联,不重做布局求解);伪状态来源归 18/19;渲染消费 computed style 归 10/21;动画 transition 驱动归 `editor_ui/07`;单向受控遵 11。

## 11. 参考实现对照(dev/ 源码锚点,已核实)

- **Unity UI Toolkit(USS,最直接对照)**:`dev/ui-toolkit-manual-code-examples/slide-toggle/SlideToggle.uss` —— `.slide-toggle__input`(:1 类/BEM)、`var(--unity-colors-*)`(:2 自定义属性)、`.slide-toggle__input--checked`(:46 修饰类)、`.slide-toggle:focus .slide-toggle__input-knob`(:54 伪状态 + 后代组合子)、`transition-property`/`transition-duration`(:19-20,33-34);`create-custom-style-custom-control/ExampleElementCustomStyle.uss`(自定义样式属性);UIElements docs 的 USS selectors/specificity/inheritance/variables 章节为权威语义参照。
- **Slint**:`internal/core` 属性/样式与 `:hover`-类伪状态;property 求值模型。
- **Godot**:`scene/gui/control.h:308`(theme_type_variation,≈ CSS 变体类)、`theme.h:219`(set_type_variation)、per-control override maps(`control.h:311-316`);`StyleBox`(≈ 背景/边框/圆角声明)。
- **Unreal**:`SlateCore/.../Styling`(`FSlateStyleSet`/`ISlateStyle`,按 style name 取 widget style——非 CSS 级联,但"按选择键取样式"思想对照)。

## 12. 状态与产出记录

in_progress。v2 选择器/parser、tuple specificity、源序级联、runtime pseudo-state 与 token/`var()` 已有当前源码 owner；retained painter 同源、继承/computed 边界、主题精确失效、内联状态清退和 transition 仍未完成，不据此宣称里程碑完成。

- applicable open failure（保持 open）：[ui-selector-specificity-template-export-drift](20/failure-2026-07-27-ui-selector-specificity-template-export-drift.md)。
