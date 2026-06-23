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
status: planned
---
# 13 Taffy 类 CSS 约束语言规范(flex / grid / block + 约束 token)

## 1. 目标

把"编辑器布局用一套**类 CSS 的约束语言**声明,底层由 **Taffy** 求解"沉淀为一份**约束语言规范**:作者用 flex/grid/block + min/max/gap/padding/align 这套熟悉的类 CSS 词汇 + 01 的尺寸 token,描述每个 slot 内子节点如何排布,而不是手算坐标。本计划只定**约束语言映射与 token 化规范**,不改 Taffy bridge 求解内部(已存在)。与 02 的关系:02 定"区域级"声明(哪个面板入哪个区域),13 定"区域/slot 内"子节点的类 CSS 排布约束。

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
| `grow/shrink/basis` | 弹性伸缩 | flex_grow/shrink/basis | Flex |
| `display: grid` + `grid-template-cols/rows` | 网格 | `Grid` + grid_template_* | Grid(Taffy) |
| `display: block` | 块流 | `Block` | Block(Taffy) |
| `gap` | 间距 | gap(引用 `$gap.*` token) | Flex/Grid |
| `padding`/`margin` | 内/外边距 | padding/margin(token) | 全 |
| `min/max-width/height` | 尺寸约束 | constraints(token) | 全 |
| `align/justify` | 对齐 | `UiAlignment2D` | Flex/Grid |
| `overflow: scroll` | 滚动 | `Scroll`(Zircon-owned) | Scroll |
| `position: absolute`/`overlay` | 叠放/锚定 | `Overlay`/`Canvas`(Zircon-owned) | Free/Canvas |

### 3.2 约束 token 化(尺寸单源,接 01)

- 所有尺寸约束**优先引用 01 token**:`$--left-drawer-width`、`$--right-drawer-width`、`$--bottom-output-height`、`$gap.s/m/l`、`$control.height`、`$pad.*`。
- 裸像素**仅允许在 center 自由区 / 用户内容**;chrome 资产的约束值须 token 化(配合 01 资产扫描 + 10 渲染契约,三处一致禁裸值)。
- token 值变 → 全局布局重算受影响子树(接 09/10 增量),不需改资产。

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
- `dev/UnrealEngine/.../SlateCore/Public/Layout`(`FFlexLayout`/`SGridPanel`/`SBoxPanel`):flex/grid/box 容器约束语义参考(非 CSS,但容器约束思想一致)。
- `dev/material-ui/packages/mui-system`(spacing/breakpoints/grid):spacing token + grid 约束的 token 化参考。

## 12. 状态与产出记录

planned。后续项:S1 约束词汇映射 + token 化 + family 决策表。
