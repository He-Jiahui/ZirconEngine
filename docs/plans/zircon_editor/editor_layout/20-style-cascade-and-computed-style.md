---
related_code:
  - zircon_runtime_interface/src/ui/style.rs
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector
plan_sources:
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_layout/01-design-tokens-and-language-contract.md
  - docs/plans/zircon_editor/editor_layout/13-taffy-css-constraint-language.md
  - docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md
  - docs/plans/zircon_editor/editor_layout/19-focus-and-navigation-model.md
status: planned
---
# 20 样式系统:USS/CSS 级联、选择器与计算样式

## 1. 目标

把"作者写的样式规则如何匹配节点、如何级联合成最终样式"沉淀为一份**类 USS/CSS 的级联样式规范**。当前只有**固定优先级状态选择器**(`UiPainterStyleSelector`,按硬编码 disabled>pressed>… 折叠)与一个**无伪状态**的 v2 resolver(`editor_ui/04` §2.1/2.2),**没有真正的选择器匹配、specificity、级联、计算样式、自定义属性、继承、transition**——这是"style 不成熟"的根因。本计划把样式从"每控件取一档预设色"升级为 **stylesheet → 选择器匹配 → 级联 → computed style** 的引擎,对标 Unity UI Toolkit 的 USS。

> 工程化硬目标(接 `index` §4.0):样式是**层叠数据**,不是调用点的一次性视觉值。改一处 token/规则,所有匹配节点随动;状态(hover/focus/checked)由伪状态选择器表达,不在绘制族里写 `if state == ...` 分支。

## 2. 现状(按代码核实)

- `style.rs` 有 `UiPainterStyleSelector`(:272)、`UiPainterResolvedState`(:233)按 family 折叠的**固定优先级**解析;**这是"状态档选择",不是 CSS 级联**——没有选择器语言、没有 specificity、没有多规则合成。
- `zircon_runtime/src/ui/v2/style.rs` 的 `UiV2StyleResolver::resolve` **无伪状态**(`editor_ui/04` §2.2.2);双路(retained 软绘 / render extract)状态样式不同源。
- 组件绘制族仍有内联状态分支(`editor_ui/04` §2.2.4)。
- `01` 有设计 token,但 token **未作为可被规则引用的"自定义属性/CSS 变量"**统一进样式解析(13 §3.2 引用 token 是布局侧,视觉 token 散在选择器)。

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

### 3.4 computed style 与继承

规范:节点最终样式 = **computed style**(级联后逐属性定值 + 继承解析),对标 CSS computed value、Unity `ICustomStyle`/resolvedStyle:

- 可继承属性(字体族/字号/色/对齐等,接 17 文本)沿树继承;不可继承属性(背景/边框/布局)不继承,对标 CSS 继承模型。
- computed style 是渲染(10/21)与命中(18 视觉无关)的输入;一节点一份缓存,脏时重算(接 09)。

### 3.5 transition(状态过渡,接动画)

规范:属性可声明 `transition`(property + duration + easing),状态切换时插值,对标 CSS transition、Unity USS `transition-property`/`transition-duration`(`SlideToggle.uss:19-20,33-34`)。MVP 可只支持色/不透明/位移过渡,驱动接 `editor_ui/07`(theatre 动画)。非硬需求,可后置切片。

### 3.6 双路同源(治"retained/extract 不同源")

规范:**级联引擎只有一处**;retained 软件绘制族与 render extract 都消费同一 computed style(治 `editor_ui/04` §2.2.2 双路不同源)。v2 resolver 升级为"调用统一级联引擎",**禁止再写第二份优先级表**(沿用 `editor_ui/04` §3.3 的硬规则)。

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
| S1 | 选择器解析 + specificity + 级联引擎 + var/token + computed | `cargo test -p zircon_runtime --lib style_cascade --locked` |
| S2 | v2 resolver 接级联引擎(伪状态);retained/extract 同源 | `cargo test -p zircon_editor --lib --locked` |
| S3 | 组件内联状态分支清除 → 伪状态选择器(收编固定优先级) | `cargo test -p zircon_editor --lib style_selector --locked` |
| S4(可选) | transition 过渡(接 editor_ui/07) | `cargo test -p zircon_editor --lib style_transition --locked` |

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

planned。后续项:S1 选择器解析 + specificity + 级联引擎 + var/token + computed。
