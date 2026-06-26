---
related_code:
  - zircon_runtime_interface/src/ui/layout/style.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime_interface/src/ui/layout/constraints.rs
  - zircon_runtime_interface/src/ui/layout/linear_sizing.rs
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime_interface/src/ui/layout/scroll.rs
  - zircon_runtime_interface/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/mod.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/compute.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime/src/ui/layout/pass/pipeline.rs
design_references:
  - docs/ui-and-layout/ai-workbench-style/prototype/README.md
plan_sources:
  - docs/plans/zircon_editor/editor_layout/01-design-tokens-and-language-contract.md
  - docs/plans/zircon_editor/editor_layout/02-declarative-layout-interface.md
  - docs/plans/zircon_editor/editor_layout/12-widget-slot-componentization.md
  - docs/plans/zircon_editor/editor_layout/16-relative-layout-and-resolution-adaptation.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
status: planned
---
# 13 Taffy 类 CSS 约束语言规范(flex / grid / block + 约束 token)

## 1. 目标

把"编辑器布局用一套**类 CSS 的约束语言**声明,底层由 **Taffy** 求解"沉淀为一份**约束语言规范**:作者用 flex/grid/block + min/max/gap/padding/align 这套熟悉的类 CSS 词汇 + 01 的尺寸 token,描述每个 slot 内子节点如何排布,而不是手算坐标。本计划只定**约束语言映射与 token 化规范**,不改 Taffy bridge 求解内部(已存在)。与 02 的关系:02 定"区域级"声明(哪个面板入哪个区域),13 定"区域/slot 内"子节点的类 CSS 排布约束。

> **相对优先,见 `16`**:本约束语言是 `16`(相对布局与多分辨率自适应规范)落地的词汇层。硬约束:尺寸**优先用相对档**(`flex-grow` 权重 / `flex-basis` 百分比 / `auto` / `min`-`max`),其次才是 **DPI 无关逻辑单位 token**(`$--left-drawer-width` 等,渲染前乘 `scale_factor`),物理像素裸值**仅** `center` 自由区。约束语言不是"用 token 写死像素",而是"用相对比例 + 逻辑 token 表达自适应"。

## 2. 现状(按代码核实)

### 2.1 已存在的设施(Taffy 后端已成立,不重做)

| 能力 | 落点 | 证据 |
| --- | --- | --- |
| 布局后端选择 | `iface .../layout/engine.rs` | `UiLayoutEngineBackend{Zircon,Taffy}`、`UiLayoutEngineFamily{Free,Canvas,Flex,Grid,Block,Scroll,Wrap,Masonry,VirtualList}` |
| 布局样式 DTO | `iface .../layout/style.rs` | `UiLayoutStyle`:flex_grow/shrink/basis、grid_template_*、margin、padding;`UiLayoutDisplay{Flex,Grid,Block,Overlay,Canvas,Scroll,Virtual,None}` |
| 约束 DTO | `iface .../layout/constraints.rs` | 尺寸/min/max 约束 |
| 线性排布 | `iface .../layout/linear_sizing.rs` | 线性 sizing |
| 几何 | `iface .../layout/geometry.rs` | rect/size/point |
| 滚动/虚拟化 | `iface .../layout/scroll.rs` / `virtualization.rs` | scroll/virtual list |
| Taffy 桥 | `runtime .../layout/taffy_bridge/compute.rs` | `taffy_style_for_container()`:container → Taffy Style |
| 度量/排布 pass | `runtime .../layout/pass/measure.rs` / `taffy_arrange.rs` / `pipeline.rs` | `measure_node()` → DesiredSize;Taffy 排布;pass 顺序 |
| 引擎能力 | `iface .../layout/engine.rs` | `UiLayoutEngineCapability::zircon()` 全 family;`taffy_flex_grid_wrap_block()` Taffy 子集 |

### 2.2 真实缺口

- 缺**面向作者的类 CSS 约束词汇规范**:`UiLayoutStyle` 字段齐全,但没把"作者写什么 → 映射到哪个字段 → 走哪个 family"写成一份可读规范(目前需读 Rust DTO)。
- 缺**约束 token 化规范**:尺寸/gap/padding 应引用 01 token(`$gap.m`/`$--left-drawer-width`),缺"约束值优先 token、裸像素仅 center 自由区"的硬规范。
- 缺**family 选择规范**:何时 Flex、何时 Grid、何时 Block、何时走 Zircon-owned(Overlay/Canvas/Scroll/Virtual),缺决策表。
- 缺**约束语言与 slot 的衔接**:12 的 slot 用 `UiSlotKind` 定排布算法,需明确 `UiSlotKind` ↔ `UiLayoutDisplay`/family 的映射,使 slot 排布与类 CSS 约束统一。

## 3. 设计

### 3.1 类 CSS 约束词汇 → DTO 映射

作者用类 CSS 词汇声明,规范固定其到 `UiLayoutStyle` 的映射:

| 类 CSS 词汇 | 含义 | 映射字段 | family |
| --- | --- | --- | --- |
| `display: flex` + `direction: row/col` | 弹性行/列 | `UiLayoutDisplay::Flex` + flex_direction | Flex(Taffy) |
| `grow/shrink/basis` | 弹性伸缩(**basis 支持百分比**;**优先 grow/basis% 而非固定尺寸**,对标 UE 锚点拉伸/`FillWidth`) | flex_grow/shrink/basis | Flex |
| `display: grid` + `grid-template-cols/rows` | 网格 | `Grid` + grid_template_* | Grid(Taffy) |
| `display: block` | 块流 | `Block` | Block(Taffy) |
| `gap` | 间距(引用 `$gap.*` 逻辑 token,替代手工 `x += sep` 累加) | gap | Flex/Grid |
| `padding`/`margin` | 内/外边距(token) | padding/margin | 全 |
| `margin: auto` | 主轴/交叉轴居中(对标 bevy `margin: auto()`) | margin = auto | Flex |
| `min/max-width/height` | 尺寸约束(防裁,接 15d 列最小宽) | constraints(token) | 全 |
| `aspect-ratio` | 等比自适应(对标 UE `EStretch::ScaleToFit`) | aspect_ratio | Flex/Grid |
| `align/justify` | 对齐 | `UiAlignment2D` | Flex/Grid |
| `overflow: scroll` | 滚动 | `Scroll`(Zircon-owned) | Scroll |
| `position: absolute`/`overlay` | 叠放/锚定(anchor 0~1 → basis%/inset%,对标 UE `FAnchors`) | `Overlay`/`Canvas`(Zircon-owned) | Free/Canvas |

### 3.2 约束 token 化(相对优先 + 逻辑单位,接 01/16)

- **相对优先**:能用相对档(`grow`/`basis%`/`auto`/`min`-`max`)就不写固定尺寸。区域、抽屉、center 主区一律先用 grow/basis% 表达比例自适应(对标 `16` §3.2)。
- **token 是 DPI 无关逻辑单位**:固定厚度才引用 01 token——`$--left-drawer-width`、`$--right-drawer-width`、`$--bottom-output-height`、`$gap.s/m/l`、`$control.height`、`$pad.*`。token 值是**逻辑单位**,渲染前统一乘 `scale_factor`(`16` §3.4),**不得当物理像素直接用**。
- 物理像素裸值**仅允许在 center 自由区 / 用户内容**;chrome 资产的约束值须 token 化(配合 01 资产扫描 + 10 渲染契约,三处一致禁裸物理像素)。
- token 值变 / scale 变 → 全局布局重算受影响子树(接 09/10 增量),不需改资产。

### 3.2a flex 充分利用与像素反模式(接 16 §3.5/§3.6)

**充分利用(正面)**:
- `flex-grow` 权重让主带/center 瓜分窗口富余,窗口拉宽按比例伸,而非反算 `height − Σ固定`。
- `flex-basis` 百分比表达"占父 N%"(对标 UE 锚点 `Anchor × 父尺寸`)。
- `min/max` 防裁:收窄时列/抽屉到最小宽即停、富余从 `shrink` 列等比回收(接 `15d`)。
- `gap`(逻辑 token)做分隔/间距,替代逐项 `x += separator_thickness` 像素累加。
- `aspect-ratio` 做等比缩放(对标 `EStretch::ScaleToFit`)。

**反模式(禁止)**:
- ✗ 硬编码裸物理像素厚度并**手工竖向累加**算几何(壳层 `region_frames.rs` 现状,见 `16` R2)。
- ✗ 把逻辑 token 当物理像素直接绘制、绕过 `scale_factor`(见 `16` R1/R4)。
- ✗ 断点阈值用物理像素而非逻辑宽度(见 `16` R3 / `15e`)。

### 3.2b 文本节点的 measure(接 17)

文本节点参与 Flex/Grid/Block 求解时,其期望尺寸与 min/max 宽度必须来自**真实字形度量**(advance/kerning/ascent),而非 `font_size*0.5` 等宽近似——否则约束求解拿到错误的内容尺寸,导致文本溢出容器或过早换行。规范:文本节点的 Taffy measure callback 经 `17` 的字形度量提供者求 `(min_content, max_content, preferred)` 宽,换行/省略/自适应策略由 `17` 决定;`13` 只负责把这些尺寸喂进约束求解。详见 `17` §3.1/§3.3。

### 3.3 family 选择决策表

| 场景 | 选 family | 理由 |
| --- | --- | --- |
| 工具条/行/列表行/抽屉竖排 | Flex | 一维弹性,最常用 |
| 属性表/网格面板/对齐表单 | Grid | 二维对齐 |
| 文档流式文本块 | Block | 块流 |
| 浮层/徽标/角标/绝对定位 | Overlay/Canvas | 脱流叠放(Zircon-owned) |
| 长列表(资产/日志) | VirtualList | 虚拟化,接 `virtualization.rs` |
| 可滚动区 | Scroll | 溢出滚动 |

规范:**优先 Flex/Grid/Block(Taffy 标准)**;只有脱流/虚拟/滚动等 Taffy 不擅长的才走 Zircon-owned family。

### 3.4 UiSlotKind ↔ family 映射(衔接 12)

| `UiSlotKind`(12) | `UiLayoutDisplay`/family(13) |
| --- | --- |
| Linear | Flex |
| Grid | Grid |
| Container | Block |
| Overlay | Overlay |
| Canvas / Free | Canvas / Free |
| Scrollable | Scroll |
| Flow | Wrap |
| Splitter | Flex + 可拖分隔(center 分屏,接 03/07) |
| Scale | Scale(等比) |

12 声明组件 slot 用哪种 `UiSlotKind`,13 据此映射到布局 family,二者统一:slot = 排布容器,family = 算法。

### 3.5 约束语言 token 资产形态

约束以 `.zui` 内联属性 + token 引用表达,示例(规范形态,非实现):

```toml
# 抽屉竖排 slot:flex 列、token 间距、token 宽度
[node.left_drawer]
display = "flex"
direction = "column"
gap = "$gap.m"
width = "$--left-drawer-width"
padding = "$pad.s"
```

## 4. 接口与数据结构草案(Rust)

```rust
// 类 CSS 约束 → UiLayoutStyle 的映射器(只读规范化,不改 Taffy 求解)
pub struct CssLikeConstraint {
    pub display: UiLayoutDisplay,
    pub gap: Option<ConstraintTokenName>,        // 引用 01 token
    pub padding: Option<ConstraintTokenName>,
    pub size: SizeConstraintTokens,              // min/max/basis 走 token
    pub align: Option<UiAlignment2D>,
}
impl CssLikeConstraint {
    pub fn into_layout_style(&self, tokens: &EditorDesignTokens) -> UiLayoutStyle; // token → 值
    pub fn family(&self) -> UiLayoutEngineFamily;
}
// slot kind → family(衔接 12)
pub fn family_for_slot_kind(kind: UiSlotKind) -> UiLayoutEngineFamily;
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增 | `zircon_editor/src/ui/workbench/autolayout/css_like_constraint.rs` | 类 CSS 约束 → UiLayoutStyle 映射 + family 选择 |
| 修改 | `taffy_bridge/compute.rs`(只读消费,不改求解) | 确认 token 化约束喂入 `taffy_style_for_container` |
| 新增 | `docs/ui-and-layout/css-constraint-language.md` | 约束词汇表 + token 化 + family 决策 + slot 映射 |

## 6. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
| -- | --- | --- | --- | --- |
| S1 | 约束词汇映射 + token 化 + family 决策表 | css_like_constraint.rs / css-constraint-language.md | `cargo test -p zircon_editor --lib css_like_constraint --locked` | 新建 |
| S2 | slot↔family 映射接入 + chrome 约束 token 化复核 | css_like_constraint.rs / chrome 资产 | `cargo test -p zircon_editor --lib --locked` | chrome 约束裸像素→token,衔接 12 slot kind |

## 7. 测试矩阵

- `display: flex/grid/block` 映射到正确 `UiLayoutDisplay` 与 family。
- 约束 token(`$gap.m`/`$--left-drawer-width`)解析为 01 token 值并喂入 Taffy。
- chrome 资产无裸像素约束(扫描证明),center 自由区允许裸值。
- `UiSlotKind` → family 映射与 12 一致。
- family 决策:脱流/虚拟/滚动走 Zircon-owned,其余走 Taffy。

## 8. 风险与对策

- 风险:类 CSS 词汇与 Taffy 语义细节不完全对齐。对策:规范只覆盖 Taffy 已支持子集(`taffy_flex_grid_wrap_block`),超出部分走 Zircon-owned 并标注。
- 风险:token 化约束改动引发全局布局回退。对策:token→值变更走 09/10 增量重算,先在单 slot 验证再铺开。

## 9. 完成定义

类 CSS 约束词汇映射成文;约束尺寸 token 化(chrome 禁裸值);family 选择决策表落地;`UiSlotKind`↔family 与 12 统一。

## 10. 边界约束

不改 Taffy bridge 求解算法(已存在);区域级声明归 02,slot 契约归 12,token 值归 01;裸像素仅 center 自由区。

## 11. 参考实现对照(dev/ 源码锚点)

- `dev/bevy/crates/bevy_ui/src/layout/convert.rs`:声明式 Style → `taffy::Style` 转换样板(类 CSS → Taffy 的直接对照)。
- `dev/slint/internal/compiler/passes`(layout 相关 pass):约束语言 → 布局求解的 pass 化参考。
- `dev/slint/internal/core/layout.rs`:`LayoutInfo{min,max,min_percent,max_percent,stretch}`——绝对值 + 百分比 + stretch 因子混合约束(对标 grow/basis%/min-max)。
- `dev/UnrealEngine/.../SlateCore/Public/Layout`(`FFlexLayout`/`SGridPanel`/`SBoxPanel`):flex/grid/box 容器约束语义参考(非 CSS,但容器约束思想一致)。
- `dev/UnrealEngine/.../Slate/Public/Widgets/Layout/Anchors.h:78-81` + `SScaleBox.h:31-68`:锚点 0~1 拉伸判定与 `EStretch` 模式,约束语言 `basis%`/`aspect-ratio` 的语义来源(详见 `16` §3.3)。
- `dev/UnrealEngine/.../SlateCore/Public/Fonts/FontCache.h:153-165`:`FShapedGlyphEntry{XAdvance,XOffset,YOffset,Kerning}`——文本节点 measure 的真实字形度量来源,measure 不取整(详见 `17` §3.1)。
- `dev/material-ui/packages/mui-system`(spacing/breakpoints/grid):spacing token + grid 约束的 token 化参考。

## 12. 状态与产出记录

planned。后续项:S1 约束词汇映射 + token 化 + family 决策表。
