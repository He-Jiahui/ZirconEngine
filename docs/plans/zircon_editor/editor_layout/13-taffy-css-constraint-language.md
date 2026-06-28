---
related_code:
  - zircon_runtime_interface/src/ui/layout/style.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime_interface/src/ui/layout/constraints.rs
  - zircon_runtime_interface/src/ui/layout/linear_sizing.rs
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime_interface/src/ui/layout/scroll.rs
  - zircon_runtime_interface/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/layout/style_mapping.rs
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

> **关键事实(本次细化的前提)**:`UiLayoutStyle` → `taffy::Style` 的**完整映射已落地**(`style_mapping.rs::taffy_style_from_ui_layout_style()`,`style_mapping.rs:14-70`),逐字段翻译了 display / direction / wrap / justify-content / align-items / align-self / align-content / gap(row+column)/ flex-grow/shrink/basis / grid-template-rows/columns(Px/Percent/Fr/Auto/MinMax)/ grid-row/column(line/span/auto)/ size·min·max / aspect-ratio / margin(含 auto)/ padding / position(relative/absolute)/ inset(含 auto)/ overflow(x,y)。**所以"求解能力"不缺**——缺的是作者侧的类 CSS 词汇规范、token 化规范,以及少量 DTO 扩展候选(§3.6 T3)。

## 2. 现状(按代码核实)

### 2.1 已存在的设施(Taffy 后端已成立,不重做)

| 能力 | 落点 | 证据 |
| --- | --- | --- |
| 布局后端选择 | `iface .../layout/engine.rs` | `UiLayoutEngineBackend{Zircon,Taffy}`(`engine.rs:9-14`);`UiLayoutEngineFamily{Free,Canvas,Container,Overlay,Flex,Grid,Block,Scrollable,Wrap,Masonry,VirtualizedList}`(`engine.rs:16-30`) |
| family 归属判定 | `iface .../layout/engine.rs` | `is_taffy_owned()` = `Flex\|Grid\|Block\|Wrap`(`engine.rs:46-48`);`is_zircon_owned()` = 其余(`engine.rs:33-44`) |
| 布局样式 DTO(完整 CSS 字段) | `iface .../layout/style.rs` | `UiLayoutStyle`:`style.rs:3-30`;`UiLayoutDisplay{Flex,Grid,Block,Overlay,Canvas,Scroll,Virtual,None}`(`style.rs:63-75`) |
| **DTO → Taffy 完整映射** | `runtime .../layout/style_mapping.rs` | `taffy_style_from_ui_layout_style()`(`style_mapping.rs:14-70`)逐字段翻译整个 `UiLayoutStyle` |
| 容器/约束 → DTO 投影 | `runtime .../layout/style_mapping.rs` | `ui_layout_style_from_container()`(`:72-128`)、`ui_layout_style_from_axis_constraints()`(`:130-160`) |
| 约束 DTO | `iface .../layout/constraints.rs` | 尺寸/min/max/priority/weight/stretch_mode |
| 线性排布 | `iface .../layout/linear_sizing.rs` | `UiLinearSlotSizing`(rule=Auto/Stretch/StretchContent + value/shrink/min/max),桥接见 `compute.rs:471-498` |
| 几何 | `iface .../layout/geometry.rs` | rect/size/point |
| 滚动/虚拟化 | `iface .../layout/scroll.rs` / `virtualization.rs` | scroll/virtual list(Zircon-owned) |
| Taffy 桥 | `runtime .../layout/taffy_bridge/{mod,compute}.rs` | `taffy_style_for_container()`(`mod.rs:22-30`)→ DTO → Taffy;`compute_taffy_child_frames()`(`compute.rs:69-140`)子节点排布 |
| 度量/排布 pass | `runtime .../layout/pass/{measure,taffy_arrange,pipeline}.rs` | `measure_node()` → DesiredSize;Taffy 排布;pass 顺序 |
| 引擎能力 | `iface .../layout/engine.rs` | `UiLayoutEngineCapability::zircon()` 全 family(`:67-86`);`taffy_flex_grid_wrap_block()` Taffy 子集(`:88-100`) |

### 2.2 真实缺口

- 缺**面向作者的类 CSS 约束词汇规范**:`UiLayoutStyle` 字段齐全、映射齐全(`style_mapping.rs`),但没把"作者写什么 → 落到哪个字段 → 走哪个 family → 现状是否已实现"写成一份可读规范(目前需读 Rust DTO + 映射函数)。本文 §3.1/§3.6 补齐。
- 缺**约束 token 化规范**:尺寸/gap/padding 应引用 01 token(`$gap.m`/`$--left-drawer-width`),缺"约束值优先 token、裸像素仅 center 自由区"的硬规范,以及**取值规范化边界**(Percent 归一化 0..1、gap/padding 不可 auto、align-content 无 baseline)。本文 §3.2/§3.7 补齐。
- 缺**family 选择规范**:何时 Flex、何时 Grid、何时 Block、何时走 Zircon-owned(Overlay/Canvas/Scrollable/Virtualized/Masonry),缺决策表。本文 §3.3 补齐。
- 缺**约束语言与 slot 的衔接**:12 的 slot 用 `UiSlotKind` 定排布算法,需明确 `UiSlotKind` ↔ `UiLayoutDisplay`/family 的映射。本文 §3.4 补齐。
- 缺**CSS 覆盖度盘点**:哪些 CSS 规则现已可表达、哪些受限、哪些需 DTO 扩展(vw/vh、justify-items/self、grid-auto-flow、overflow:clip、box-sizing、direction、fit-content)。本文 §3.6 三档矩阵补齐,并给跨引擎源码依据。

## 3. 设计

### 3.1 类 CSS 约束词汇 → DTO 映射(完备表,带现状映射证据)

作者用类 CSS 词汇声明,规范固定其到 `UiLayoutStyle` 字段的映射;"现状映射证据"列指向已落地的 `style_mapping.rs` 行号,"跨引擎对标"列给出同语义在其他引擎的实现锚点(详见 §11)。

| 类 CSS 词汇 | `UiLayoutStyle` 字段 | 取值/单位 | 现状映射证据 | family | 跨引擎对标 |
| --- | --- | --- | --- | --- | --- |
| `display: flex/grid/block/none` | `display` | `UiLayoutDisplay` | `style_mapping.rs:184-197` | Flex/Grid/Block | bevy `Display`(convert.rs:66) |
| `flex-direction: row/column[-reverse]` | `direction` | `UiFlexDirection` | `:199-206` | Flex | bevy(convert.rs:76);slint `flex-direction`(builtins.slint:460) |
| `flex-wrap: nowrap/wrap/wrap-reverse` | `wrap` | `UiFlexWrap` | `:208-214` | Flex/Wrap | UE `SWrapBox`;bevy(convert.rs:77) |
| `justify-content` | `justify_content` | `UiJustify`(Start/End/Center/Space*) | `:216-225` | Flex/Grid | slint `alignment`(layout.rs:1224-1261) |
| `align-items` | `align_items` | `UiAlign`(含 Stretch/Baseline) | `:227-235` | Flex/Grid | godot `SIZE_*`(control.h:79-86) |
| `align-self` | `align_self` | `UiAlign` | `:237-245` | Flex/Grid | slint `align-self`(layout.rs:1168) |
| `align-content` | `align_content` | `UiAlign`(**无 Baseline**) | `:247-255` | Flex/Grid | — |
| `gap` / `row-gap` / `column-gap` | `gap{row,column}` | `UiDimension`(**不可 Auto**) | `:26-29,272-280` | Flex/Grid | slint `spacing`(builtins.slint:437-439);godot `separation`(box_container.cpp:104) |
| `flex-grow` | `flex_grow` | `f32` 权重(≥0) | `:30,418-427` | Flex | UE `FStretch`/`SizeRule_Stretch`(SlateStructs.h:100);godot `stretch_ratio`(box_container.cpp:93) |
| `flex-shrink` | `flex_shrink` | `f32`(≥0) | `:31` | Flex | slint `flex_shrink`(layout.rs:1163) |
| `flex-basis`(**支持 %**) | `flex_basis` | `UiDimension`(Auto/Px/Percent) | `:32,264-270` | Flex | UE 锚点拉伸/`FillWidth`;slint `flex_basis`(layout.rs:1166) |
| `grid-template-columns/rows` | `grid_template_columns/rows` | `Vec<UiGridTrack>` | `:33-42,336-367` | Grid | bevy `grid_template_*`(convert.rs:126) |
| └ track `<len>` | `UiGridTrack::Px` | `f32` | `:346-349` | Grid | bevy `GridTrack::px` |
| └ track `<pct>` | `UiGridTrack::Percent` | `f32`(0..1) | `:350-353` | Grid | bevy `GridTrack::percent` |
| └ track `Nfr` | `UiGridTrack::Fr`(min→auto) | `f32` | `:354-357` | Grid | bevy `GridTrack::fr` |
| └ track `auto` | `UiGridTrack::Auto` | — | `:358-361` | Grid | bevy `GridTrack::auto` |
| └ track `minmax(a,b)` | `UiGridTrack::MinMax` | `UiGridTrackBreadth` | `:362-397` | Grid | bevy `GridTrack::minmax` |
| `grid-row` / `grid-column`(line/span) | `grid_row/column` | `UiGridPlacement{start,end}` | `:43-52,399-416` | Grid | slint `row/col/colspan/rowspan`(layout.rs:475-480);UE `SGridPanel` ColumnSpan/RowSpan |
| `width`/`height` | `size{width,height}` | `UiDimension` | `:53,257-262` | 全 | UE `SBox::WidthOverride`(SBox.h:55) |
| `min-width`/`min-height` | `min_size` | `UiDimension` | `:54` | 全 | UE `MinDesiredWidth`(SBox.h:61);slint `min`/`min_percent`(layout.rs:29,32) |
| `max-width`/`max-height` | `max_size` | `UiDimension` | `:55` | 全 | UE `MaxDesiredWidth`(SBox.h:67);slint `max`/`max_percent`(layout.rs:26,28) |
| `aspect-ratio` | `aspect_ratio` | `Option<f32>` | `:56` | Flex/Grid | UE `SScaleBox::EStretch::ScaleToFit`(SScaleBox.h:44);godot `AspectRatioContainer::ratio`(aspect_ratio_container.h:57) |
| `margin`(**支持 auto 居中**) | `margin` | `UiEdges`(Auto/Px/Percent) | `:57,303-312` | 全 | bevy `margin: auto`(convert.rs:91);bevy `UiRect` |
| `padding` | `padding` | `UiEdges`(**不可 Auto**) | `:58,294-301` | 全 | godot `MarginContainer`;slint `padding`(layout.rs:380-410) |
| `position: relative/absolute` | `position` | `UiPositionMode` | `:59,314-319` | 全(Taffy 绝对定位) | godot `anchor[]`+`offset[]`(control.h:56-57) |
| `top/right/bottom/left`(inset,**支持 auto/%**) | `inset` | `UiEdges` | `:60,303-312` | 全 | UE `FAnchors`+Offset(Anchors.h);godot `anchor 0~1 + offset`(control.h:56-57) |
| `overflow: visible/hidden/scroll` | `overflow{x,y}` | `UiOverflowPair` | `:61,321-334` | 全(scroll 行为另见 §3.7) | bevy `overflow.x/y`(convert.rs:70-72) |

> 说明:上表所有行的"现状映射证据"都落在 `style_mapping.rs`,即**这些 CSS 词汇现在就能被引擎求解**;13 的工作是把它们规范化为作者词汇 + token 化,而非新增求解能力。受限项与扩展候选见 §3.6/§3.7。

### 3.2 约束 token 化(相对优先 + 逻辑单位,接 01/16)

- **相对优先**:能用相对档(`grow`/`basis%`/`auto`/`min`-`max`)就不写固定尺寸。区域、抽屉、center 主区一律先用 grow/basis% 表达比例自适应(对标 `16` §3.2)。
- **token 是 DPI 无关逻辑单位**:固定厚度才引用 01 token——`$--left-drawer-width`、`$--right-drawer-width`、`$--bottom-output-height`、`$gap.s/m/l`、`$control.height`、`$pad.*`。token 值是**逻辑单位**,渲染前统一乘 `scale_factor`(`16` §3.4),**不得当物理像素直接用**。
- 物理像素裸值**仅允许在 center 自由区 / 用户内容**;chrome 资产的约束值须 token 化(配合 01 资产扫描 + 10 渲染契约,三处一致禁裸物理像素)。
- token 值变 / scale 变 → 全局布局重算受影响子树(接 09/10 增量),不需改资产。
- **token 的尺度基数对标 Material-UI**:01 的 `$gap.*`/`$pad.*` 8px 基数节拍可对标 `mui-system` 的 `spacing` 默认基数 8(`createSpacing.ts:29`);断点 token 对标 `mui` 的 `xs/sm/md/lg/xl`(`createBreakpoints.js:20-24`),但本规范断点用**逻辑宽度**(`16` §3.4 / R3),不用物理像素。

### 3.2a flex 充分利用与像素反模式(接 16 §3.5/§3.6)

**充分利用(正面)**:
- `flex-grow` 权重让主带/center 瓜分窗口富余,窗口拉宽按比例伸,而非反算 `height − Σ固定`(对标 UE `SizeRule_Stretch` / godot `stretch_ratio`)。
- `flex-basis` 百分比表达"占父 N%"(对标 UE 锚点 `Anchor × 父尺寸`)。
- `min/max` 防裁:收窄时列/抽屉到最小宽即停、富余从 `shrink` 列等比回收(接 `15d`;对标 slint `min/max` + `min_percent/max_percent`)。
- `gap`(逻辑 token)做分隔/间距,替代逐项 `x += separator_thickness` 像素累加。
- `aspect-ratio` 做等比缩放(对标 UE `EStretch::ScaleToFit` / godot `AspectRatioContainer`)。
- `margin: auto` 做主轴/交叉轴居中(已映射 `style_mapping.rs:57,303-312`,对标 bevy `margin: auto()`)。

**反模式(禁止)**:
- ✗ 硬编码裸物理像素厚度并**手工竖向累加**算几何(壳层 `region_frames.rs` 现状,见 `16` R2)。
- ✗ 把逻辑 token 当物理像素直接绘制、绕过 `scale_factor`(见 `16` R1/R4)。
- ✗ 断点阈值用物理像素而非逻辑宽度(见 `16` R3 / `15e`)。

### 3.2b 文本节点的 measure(接 17)

文本节点参与 Flex/Grid/Block 求解时,其期望尺寸与 min/max 宽度必须来自**真实字形度量**(advance/kerning/ascent),而非 `font_size*0.5` 等宽近似——否则约束求解拿到错误的内容尺寸,导致文本溢出容器或过早换行。规范:文本节点的 measure 经 `17` 的字形度量提供者求 `(min_content, max_content, preferred)` 宽,换行/省略/自适应策略由 `17` 决定;`13` 只负责把这些尺寸喂进约束求解。详见 `17` §3.1/§3.3。对标 UE `FShapedGlyphEntry{XAdvance,XOffset,YOffset,Kerning}`(`17` §3.1)。

### 3.3 family 选择决策表

| 场景 | 选 family | 理由 |
| --- | --- | --- |
| 工具条/行/列表行/抽屉竖排 | Flex | 一维弹性,最常用 |
| 属性表/网格面板/对齐表单 | Grid | 二维对齐 |
| 文档流式文本块 | Block | 块流 |
| 自动换行标签流 | Wrap | flex-wrap(Taffy native) |
| 浮层/徽标/角标/绝对定位 | Overlay/Canvas | 脱流叠放(Zircon-owned) |
| 长列表(资产/日志) | VirtualizedList | 虚拟化,接 `virtualization.rs`(Zircon-owned) |
| 瀑布流 | Masonry | 瀑布(Zircon-owned) |
| 可滚动区 | Scrollable | 溢出滚动(Zircon-owned) |
| 自由容器/尺寸盒 | Container/Free | 单子/裸尺寸(Zircon-owned) |

规范:**优先 Flex/Grid/Block/Wrap(Taffy 标准,`is_taffy_owned()`,`engine.rs:46-48`)**;只有脱流/虚拟/滚动/瀑布等 Taffy 不擅长的才走 Zircon-owned family(`is_zircon_owned()`,`engine.rs:33-44`)。请求落到非支持后端时由 `UiLayoutEngineSelection::select()`(`engine.rs:238-269`)按 fallback reason 回退。

### 3.4 UiSlotKind ↔ family 映射(衔接 12)

| `UiSlotKind`(12) | `UiLayoutDisplay`/family(13) |
| --- | --- |
| Linear | Flex |
| Grid | Grid |
| Container | Block / Container |
| Overlay | Overlay |
| Canvas / Free | Canvas / Free |
| Scrollable | Scrollable |
| Flow | Wrap |
| Splitter | Flex + 可拖分隔(center 分屏,接 03/07;对标 UE `SSplitter`) |
| Scale | Scale(等比,对标 UE `SScaleBox` / godot `AspectRatioContainer`) |

12 声明组件 slot 用哪种 `UiSlotKind`,13 据此映射到布局 family,二者统一:slot = 排布容器,family = 算法。

### 3.5 约束语言 token 资产形态

约束以 `.zui` 内联属性 + token 引用表达,示例(规范形态,非实现):

```toml
# 抽屉竖排 slot:flex 列、token 间距、token 宽度、相对优先
[node.left_drawer]
display = "flex"
direction = "column"
gap = "$gap.m"            # 逻辑 token,渲染前 × scale_factor
width = "$--left-drawer-width"
padding = "$pad.s"
min_width = "$--left-drawer-min"   # 防裁(接 15d)

# 属性表:2 列网格、列宽 minmax、行 gap token
[node.inspector_grid]
display = "grid"
grid_template_columns = ["minmax($--label-min, auto)", "1fr"]
row_gap = "$gap.s"

# center 主区:grow 吃富余,允许内容裸像素(自由区)
[node.viewport]
display = "flex"
flex_grow = 1.0
```

### 3.6 CSS 属性完备覆盖矩阵(三档 + 引擎依据)

把"尽可能多适配 CSS 描述规则"落成可盘点的三档。**T1/T2 在 13 范围内**(词汇 + token 化,不改求解);**T3 是 DTO 扩展候选**,超出 13 的"不改求解"边界,列出以便后续单独立项(各带跨引擎依据,证明其语义有现实需求与参照)。

#### T1 已实现(DTO 字段 + Taffy 映射齐全,直接规范化即可用)

`display(flex/grid/block/none)`、`flex-direction`、`flex-wrap`、`justify-content`、`align-items`、`align-self`、`align-content(除 baseline)`、`gap/row-gap/column-gap`、`flex-grow`、`flex-shrink`、`flex-basis(含 %)`、`width/height`、`min/max-width/height`、`aspect-ratio`、`margin(含 auto)`、`padding`、`position(relative/absolute)`、`top/right/bottom/left(inset,含 auto/%)`、`overflow(visible/hidden/scroll)`、`grid-template-rows/columns(px/%/fr/auto/minmax)`、`grid-row/column(line/span/auto)`。证据见 §3.1 各行 `style_mapping.rs` 行号。

#### T2 受限/部分(可表达但有边界,规范须显式标注)

| CSS 规则 | 限制 | 证据 | 规范处置 |
| --- | --- | --- | --- |
| `<percent>`(所有百分比) | 值**归一化 0.0..=1.0**,非 0..100 | `style.rs:139`;`taffy_dimension`(`style_mapping.rs:264-270`) | token/parser 把 `50%` 解析为 `0.5`;§3.7 |
| `align-content: baseline` | 不支持,报 `InvalidLayoutValue` | `style_mapping.rs:247-255` | 规范禁用 baseline 于 align-content |
| `gap: auto` / `padding: auto` | 不支持(`taffy_length_percentage` 拒 Auto) | `:272-280,294-301` | 仅 margin/inset 可 auto |
| `1fr` 轨道 min | 强制为 `auto`(= `minmax(auto,1fr)`) | `:354-357,379` | 与 CSS `1fr` 语义一致,文档说明 |
| `overflow: scroll` | Taffy 侧记为 scroll 行为,真正滚动/虚拟由 Zircon-owned `Scrollable`/`VirtualizedList` 接管 | `:328-334`;`engine.rs:33-44` | §3.7:声明 overflow vs. 选 family 的分工 |
| `compute.rs` 旧投影路径的 Grid | 子节点排布走 `compute.rs`,父轨道按 `fr(1.0)` 均分(`compute.rs:301-302`),不读 `grid_template_*` 的 px/%/minmax | `compute.rs:295-303` | 需精确轨道时走 `taffy_style_for_container`(DTO 路径,`mod.rs:22-30`)而非容器投影路径 |

#### T3 DTO 扩展候选(当前 DTO 无字段,超出 13 求解边界,后续单独立项)

| CSS 规则 | 现状 | 跨引擎依据(证明有需求/参照) |
| --- | --- | --- |
| 视口单位 `vw/vh/vmin/vmax` | `UiDimension` 仅 Auto/Px/Percent(`style.rs:134-141`) | bevy `Val::{Vw,Vh,VMin,VMax}`(geometry.rs:50-56)。**本仓库优先用 相对% + `scale_factor`(`16` §3.4)覆盖该需求**,vw/vh 仅在确有必要时再扩展 |
| `justify-items` / `justify-self` | `UiLayoutStyle` 无字段(grid 项对齐目前仅经 slot 在 `compute.rs:436-441` 设 `justify_self`) | bevy `justify_items`/`justify_self`(convert.rs:79,81);Taffy 原生支持 |
| `grid-auto-flow` / `grid-auto-rows/columns` | 无字段(隐式轨道不可控) | bevy `grid_auto_flow`/`grid_auto_rows/columns`(convert.rs:120,131-140) |
| `fit-content()` 轨道 | `UiGridTrackBreadth` 无 fit-content | bevy `GridTrack::fit_content_px/percent` |
| `repeat(auto-fill/auto-fit)` | `UiGridTrack` 无重复语义 | bevy `RepeatedGridTrack` + `AutoFill/AutoFit`(convert.rs) |
| `overflow: clip` + clip-margin | `UiOverflow` 仅 Visible/Hidden/Scroll(`style.rs:240-247`) | bevy `OverflowAxis::Clip` + `overflow_clip_margin` |
| `box-sizing` | 无字段(默认按 Taffy 行为) | bevy `box_sizing`(convert.rs:67) |
| `direction: ltr/rtl` | 无字段 | bevy `direction`(convert.rs:84) |
| `object-fit`(cover/contain/fill) | 仅 `aspect-ratio`,无 fit 模式 | UE `SScaleBox::EStretch{ScaleToFit/ScaleToFill/Fill}`(SScaleBox.h:44-61);godot `AspectRatioContainer::StretchMode`(aspect_ratio_container.h:44-48) |

规范:T3 项**不在 13 内实现**;若后续需要,按"DTO 扩展 + `style_mapping.rs` 映射 + 测试"独立切片立项,并复用此表的跨引擎依据作为设计参照。13 文档负责把它们登记为"已知不支持",避免作者误用。

### 3.7 取值规范化与边界(parser/token 必须遵守)

1. **百分比归一化**:CSS 作者写 `50%`,token 解析层须产出 `UiDimension::Percent(0.5)`(0.0..=1.0)。grid 轨道百分比同此(`UiGridTrack::Percent` → `MinMax`,`style_mapping.rs:350-353`)。越界/负值/NaN 由 `finite_non_negative()`(`:464-468`)拒绝并回退。
2. **auto 适用面**:仅 `size/min/max/flex-basis/margin/inset` 可取 `auto`;`gap/padding` 取 auto 会触发 `InvalidLayoutValue`(`:272-280`)。
3. **align-content 无 baseline**:见 T2。
4. **overflow 声明 ≠ 选 family**:`overflow: scroll` 是节点样式位;真正的滚动条/视口裁剪/虚拟化由 family `Scrollable`/`VirtualizedList`(Zircon-owned)负责。作者要"可滚动区"时应选对应 `UiSlotKind`(§3.4),而非只写 `overflow: scroll`。
5. **裸像素禁区**:chrome 约束值禁裸物理像素(§3.2),由 01 资产扫描 + 10 渲染契约 + 本规范三处一致校验;center 自由区与用户内容豁免。

### 3.8 Taffy 求解协议:增量缓存 / measure / rounding(接 09/10/16/17)

把约束喂进 Taffy 后,**增量、内容尺寸、像素吸附**三件事由 Taffy 0.10 的内部协议决定,规范须据此对齐(源码:cargo registry `taffy-0.10.1`,已核实):

1. **增量缓存(接 09/10)**:Taffy 每节点有 9 槽缓存(`tree/cache.rs:24-197`),键 = `known_dimensions × available_space`(`compute_cache_slot` `:73-107`)。**父尺寸不变 → available_space 不变 → 子节点缓存命中、跳过重算**;父尺寸变(如窗口拉宽)→ `AvailableSpace::Definite(new)` 变 → 槽变 → 失效重算。规范:09 的 `ViewDirtySet` 标脏后,**只对脏子树 `cache_clear` 并重 `compute_layout`**;Taffy 无显式 dirty 位(`taffy_tree.rs`),失效靠键变化 + 手动清缓存,故样式/内容变更(11/20)必须显式触发对应子树清缓存,否则 measure 值变而键不变会命中旧缓存。
2. **measure 协议(接 17 文本)**:叶子节点经 measure 回调提供内容尺寸(`compute/leaf.rs:15-164`);`available_space` 为 `MinContent`/`MaxContent`/`Definite` 时分别求最小内容宽/最大内容宽/给定宽下尺寸。规范:文本节点的 measure(17 §3.1 真实字形度量)必须按这三种 available_space 正确返回 `(min_content, max_content, preferred)`,否则 flex/grid 的内容尺寸求解(13 §3.2b)拿到错值。measure 返回的是 **content box** 尺寸(不含 padding/border)。
3. **rounding 归属(接 16/21)**:已 `disable_rounding()`(保真 30.5px 分数控件)。Taffy 的 `round_layout` 用累积坐标避免取整间隙(`compute/mod.rs:219-274`),关闭后**像素吸附责任移交渲染层**——文本/1px 边框在顶点装配时整像素吸附(`21` §3.5),自由内容不吸附;分数缩放(`16` §3.4a)同此归属。13 只负责喂分数逻辑值,不在此吸附。

## 4. 接口与数据结构草案(Rust)

```rust
// 类 CSS 约束 → UiLayoutStyle 的映射器(只读规范化,不改 Taffy 求解)
pub struct CssLikeConstraint {
    pub display: UiLayoutDisplay,
    pub direction: UiFlexDirection,
    pub gap: Option<ConstraintTokenName>,        // 引用 01 token,渲染前 × scale
    pub padding: Option<ConstraintTokenName>,
    pub size: SizeConstraintTokens,              // min/max/basis 走 token,相对优先
    pub justify: Option<UiJustify>,
    pub align: Option<UiAlign>,
}
impl CssLikeConstraint {
    /// token → 值,产出可直接喂 `taffy_style_from_ui_layout_style` 的 DTO。
    pub fn into_layout_style(&self, tokens: &EditorDesignTokens) -> UiLayoutStyle;
    pub fn family(&self) -> UiLayoutEngineFamily;
}
// slot kind → family(衔接 12)
pub fn family_for_slot_kind(kind: UiSlotKind) -> UiLayoutEngineFamily;
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增 | `zircon_editor/src/ui/workbench/autolayout/css_like_constraint.rs` | 类 CSS 约束 → `UiLayoutStyle` 映射 + family 选择;`into_layout_style` 产出的 DTO 直接喂现有 `style_mapping.rs` |
| 只读消费 | `runtime .../layout/style_mapping.rs`(不改) | `into_layout_style` 输出对接 `taffy_style_from_ui_layout_style`,确认 token 化约束被正确翻译 |
| 只读消费 | `taffy_bridge/{mod,compute}.rs`(不改求解) | 确认 token 化约束喂入 `taffy_style_for_container` |
| 新增 | `docs/ui-and-layout/css-constraint-language.md` | 约束词汇表 + token 化 + family 决策 + slot 映射 + T1/T2/T3 覆盖矩阵 |

## 6. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
| -- | --- | --- | --- | --- |
| S1 | 约束词汇映射(§3.1)+ token 化(§3.2)+ family 决策表(§3.3)+ 取值规范化(§3.7) | css_like_constraint.rs / css-constraint-language.md | `cargo test -p zircon_editor --lib css_like_constraint --locked` | 新建 |
| S2 | slot↔family 映射接入(§3.4)+ chrome 约束 token 化复核(§3.2) | css_like_constraint.rs / chrome 资产 | `cargo test -p zircon_editor --lib --locked` | chrome 约束裸像素→token,衔接 12 slot kind |
| S3 | T1/T2/T3 覆盖矩阵成文 + "已知不支持"登记(§3.6) | css-constraint-language.md | 文档评审 | 不写代码;T3 各项各自独立立项 |

## 7. 测试矩阵

- `display: flex/grid/block` 映射到正确 `UiLayoutDisplay` 与 family(对照 `style_mapping.rs:184-197`)。
- 约束 token(`$gap.m`/`$--left-drawer-width`)解析为 01 token 值并喂入 Taffy;百分比归一化为 0..1(§3.7-1)。
- `margin: auto` 居中生效;`gap: auto`/`padding: auto` 被拒(§3.7-2)。
- `align-content: baseline` 被拒(§3.7-3)。
- `flex-grow`/`flex-basis%`/`min`-`max` 组合:窗口拉宽按权重伸、收窄到 min 即停(接 15d)。
- grid `minmax`/`fr`/`%` 轨道正确翻译(`style_mapping.rs:336-397`)。
- chrome 资产无裸像素约束(扫描证明),center 自由区允许裸值。
- `UiSlotKind` → family 映射与 12 一致;脱流/虚拟/滚动/瀑布走 Zircon-owned,其余走 Taffy。
- T3 项(vw/justify-items/grid-auto-flow/overflow:clip 等)在解析层被识别为"已知不支持"并给出明确诊断,不静默吞掉。

## 8. 风险与对策

- 风险:类 CSS 词汇与 Taffy 语义细节不完全对齐。对策:T1/T2 覆盖 Taffy 已支持子集(`is_taffy_owned`),超出走 Zircon-owned 或 T3 扩展候选并标注(§3.6)。
- 风险:百分比单位制(0..1 vs 0..100)在 parser 与 DTO 间不一致导致布局错位。对策:§3.7-1 固定归一化契约 + 测试。
- 风险:token 化约束改动引发全局布局回退。对策:token→值变更走 09/10 增量重算,先在单 slot 验证再铺开。
- 风险:作者只写 `overflow: scroll` 期望出现滚动但未选 `Scrollable` family。对策:§3.7-4 显式分工 + lint。

## 9. 完成定义

类 CSS 约束词汇映射成文(§3.1,带 `style_mapping.rs` 现状证据);约束尺寸 token 化(chrome 禁裸值,§3.2/§3.7);family 选择决策表落地(§3.3);`UiSlotKind`↔family 与 12 统一(§3.4);T1/T2/T3 CSS 覆盖矩阵成文并带跨引擎源码依据(§3.6/§11);取值规范化边界确立(§3.7)。

## 10. 边界约束

不改 Taffy bridge 求解算法与 `style_mapping.rs` 映射(已存在);区域级声明归 02,slot 契约归 12,token 值归 01;裸像素仅 center 自由区;T3 扩展候选不在本计划实现,仅登记。

## 11. 参考实现对照(dev/ 源码锚点,已核实)

> 行号以 `dev/` 当前快照为准,均已 grep 核实;UE 路径在 `Engine/Source/Runtime/{SlateCore,Slate}/Public/...`。

**Bevy(声明式 Style → `taffy::Style`,与本仓库 `style_mapping.rs` 最直接对照)**
- `dev/bevy/crates/bevy_ui/src/layout/convert.rs:64` `from_node()`:Style→taffy::Style 总入口;`:70-72` overflow、`:85-89` inset、`:102` flex_basis、`:115` aspect_ratio、`:126` grid_template_columns。
- `dev/bevy/crates/bevy_ui/src/geometry.rs:31` `enum Val`;`:50/52/54/56` `Vw/Vh/VMin/VMax`(T3 视口单位依据)。

**Slint(约束语言 → 求解,绝对值 + 百分比 + stretch 混合)**
- `dev/slint/internal/core/layout.rs:24` `struct LayoutInfo`;`:26` max、`:28` max_percent、`:29`(min)、`:32` min_percent、`:34` preferred、`:36` stretch —— 对标本规范 min/max(绝对+%)与 flex-grow 的混合表达。
- `dev/slint/internal/compiler/passes/lower_layout.rs`:约束语言 → 布局求解的 pass 化参考。
- 对齐枚举/求解:`layout.rs:1224-1261`(start/center/end/space-*)、per-item flex `:1161-1170`(flex_grow/shrink/basis/align-self)、grid cell `:475-480`(col/row/colspan/rowspan)。

**Unreal Slate(容器约束语义,非 CSS 但思想一致)**
- `.../SlateCore/Public/Layout/SlateStructs.h:97-101` `enum ESizeRule{SizeRule_Auto,SizeRule_Stretch,SizeRule_StretchContent}` —— 对标 `flex-basis:auto` / `flex-grow` / 内容比例拉伸。
- `.../Slate/Public/Widgets/Layout/Anchors.h` `FAnchors`(Min/Max 0~1)—— 对标 `position:absolute` + `inset%` / `basis%`。
- `.../Slate/Public/Widgets/Layout/SScaleBox.h:44/50/56/61` `EStretch{ScaleToFit/ScaleToFitX/ScaleToFitY/ScaleToFill}` —— 对标 `aspect-ratio` / `object-fit`(T3)。
- `.../Slate/Public/Widgets/Layout/SBox.h:55` WidthOverride、`:61` MinDesiredWidth、`:67` MaxDesiredWidth、`:72` MinAspectRatio —— 对标 `width`/`min-width`/`max-width`/`aspect-ratio`。
- `.../SlateCore/Public/Types/SlateEnums.h:173` `EHorizontalAlignment`(`:176` HAlign_Fill、`:182` HAlign_Center)、`:193` `EVerticalAlignment`(`:196` VAlign_Fill)—— 对标 `align-items`/`justify-content` 的 fill/center。
- `SGridPanel`(Column/Row/ColumnSpan/RowSpan)、`SWrapBox`(flex-wrap)、`SSplitter`(可拖分隔)、`SConstraintCanvas`(FAnchors+Offset 绝对定位)。

**Godot(锚点 + 容器 size flags)**
- `dev/godot/scene/gui/control.h:56-57` `ANCHOR_BEGIN/END`(0/1)、`:60` `GrowDirection`、`:79-86` `SizeFlags{SHRINK_BEGIN,FILL,EXPAND,SHRINK_CENTER,SHRINK_END,EXPAND_FILL}` —— 对标 `position:absolute`+`inset%`、`flex-grow`/`align`。
- `dev/godot/scene/gui/box_container.cpp:84`(EXPAND 判定)、`:93/138` `stretch_ratio`(flex-grow 权重)、`:104` `separation`(gap)。
- `dev/godot/scene/gui/aspect_ratio_container.h:44-48` `StretchMode{...,STRETCH_FIT,STRETCH_COVER}`、`:57/63` `ratio`/`set_ratio` —— 对标 `aspect-ratio`/`object-fit`(T3)。

**Material-UI(spacing/breakpoints/sizing 的 token 化)**
- `dev/material-ui/packages/mui-system/src/createTheme/createSpacing.ts:29` `spacingInput = 8`(8px 基数)—— 对标 `$gap.*`/`$pad.*` 节拍。
- `dev/material-ui/packages/mui-system/src/createBreakpoints/createBreakpoints.js:20-24` `xs:0/sm:600/md:900/lg:1200/xl:1536` —— 对标断点 token(本仓库用逻辑宽度)。
- `dev/material-ui/packages/mui-system/src/sizing/sizing.js:5-6` `sizingTransform`:`value<=1 ? value*100% : value` —— 对标 0..1 归一化百分比与 px 混写(§3.7-1)。

## 12. 状态与产出记录

planned。本次细化产出:§3.1 完备词汇映射表(带 `style_mapping.rs` 现状证据)、§3.6 T1/T2/T3 CSS 覆盖矩阵(带跨引擎依据)、§3.7 取值规范化边界、§11 已核实的多引擎源码对照表。后续项:S1 约束词汇映射 + token 化 + family 决策表;S3 覆盖矩阵成文 + T3 各项独立立项。
