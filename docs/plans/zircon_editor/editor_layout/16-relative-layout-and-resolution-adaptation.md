---
related_code:
  - zircon_runtime/src/ui/layout/taffy_bridge/compute.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/child_frame.rs
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime_interface/src/ui/layout/style.rs
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/src/ui/workbench/autolayout/workbench_chrome_metrics.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/region_frames.rs
  - zircon_editor/src/ui/workbench/autolayout/layout_tier.rs
  - zircon_runtime/src/core/framework/window/resolution.rs
  - zircon_runtime/src/ui/surface/input/window_pump.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Layout/Anchors.h
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Public/Components/CanvasPanelSlot.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Layout/SConstraintCanvas.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Layout/SScaleBox.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SWindow.h
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Public/Blueprint/WidgetLayoutLibrary.h
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Public/Components/SafeZoneSlot.h
  - dev/material-ui/packages/mui-system/src/breakpoints/breakpoints.d.ts
  - dev/slint/internal/core/layout.rs
  - dev/bevy/examples/ui/layout/anchor_layout.rs
  - dev/godot/scene/gui/control.h
plan_sources:
  - docs/plans/zircon_editor/editor_layout/01-design-tokens-and-language-contract.md
  - docs/plans/zircon_editor/editor_layout/02-declarative-layout-interface.md
  - docs/plans/zircon_editor/editor_layout/13-taffy-css-constraint-language.md
  - docs/plans/zircon_editor/editor_layout/14-unreal-react-composition-thesis.md
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15d-composite-density-and-alignment.md
  - docs/plans/zircon_editor/editor_layout/15e-domain-breakpoint-adaptation.md
  - docs/plans/engine-code-structure-convention.md
status: planned
---
# 16 相对布局与多分辨率自适应规范(anchor / DPI 缩放 / stretch / flex 充分利用)

> 本文是布局**自适应**这条线的权威规范,与 `14`(组合/数据流/失效增量思想)平行:`14` 管"界面怎么长出来、数据怎么单向流",`16` 管"界面在不同分辨率/比例/DPI 下怎么不变形"。`16` 统领 `13`(类 CSS 约束)与 `15e`(断点),并修正壳 autolayout 当前的像素累加倾向。取虚幻 Slate/UMG 的**自适应思想**(锚点相对比例、DPI 缩放曲线、Stretch 模式),落到既有 **Taffy/flex + 壳 autolayout**,不照搬 Slate 运行时(与 `14` 同调)。

## 1. 目标

确立一条根规范:**编辑器布局是像素无关的相对布局**。任何区域、面板、控件的几何都来自"相对比例(flex 权重 / 百分比 basis)+ DPI 无关逻辑单位 token + 约束 min/max + 断点 tier",再由根 DPI 缩放统一换算到物理像素;**物理像素裸值只允许出现在 center 自由区 / 用户内容**。

要解决的真实问题:壳 autolayout 走向了"硬编码裸像素 + 手工像素累加",并且窗口已捕获的 `scale_factor` **完全没有参与布局**,导致界面"按像素摆放"、不随窗口比例与分辨率/DPI 自适应。本文给出三层自适应模型、相对优先单位规范、anchor/stretch→flex 映射、DPI 缩放规范、autolayout 计算规则修正,以及 flex 特性充分利用清单。

## 2. 现状(按代码核实)

### 2.1 已经做对的(相对布局基座,不重做)

| 能力 | 落点 | 证据 |
| --- | --- | --- |
| Taffy 求解保留小数精度 | `runtime .../layout/taffy_bridge/compute.rs` | `taffy.disable_rounding()`,子帧 = 父帧 + `layout.location`(相对偏移),非整像素对齐 |
| flex 样式齐全 | `iface .../layout/style.rs` | `UiLayoutStyle`:`flex_grow`/`flex_shrink`/`flex_basis`、`min/max`、`gap`、`grid_template_*` |
| 约束求解(权重/优先级/拉伸) | `runtime .../layout/constraints.rs` | `solve_axis_constraints(available, &[AxisConstraint])` 按权重分配富余/收缩 |
| 自由区锚点定位 | `runtime .../layout/pass/child_frame.rs` | `free_child_frame`:`x = parent.x + parent.width * anchor.x + position.x - width * pivot.x`(**已是虚幻式 anchor×父尺寸 + pivot**) |
| 线性容器 cursor 累加 | `runtime .../layout/pass/arrange.rs` | `arrange_linear_children` 用约束求解出的 `main_extents[i]` 推进 cursor,非硬编码 |

`free_child_frame` 的 `parent.width * anchor.x ... - width * pivot.x` 与虚幻 `SConstraintCanvas` 的 `AnchorPixels = Anchor × LocalSize` + pivot 是**同一公式**——说明 runtime 层方向正确,问题集中在 editor 壳层。

### 2.2 走偏的(像素反模式,本文要修正的目标)

| # | 反模式 | 落点 | 证据 |
| --- | --- | --- | --- |
| R1 | chrome 度量全硬编码裸像素,无 DPI | `editor .../autolayout/workbench_chrome_metrics.rs` | `top_bar_height: 25.0`、`host_bar_height: 32.0`、`status_bar_height: 24.0`、`rail_width: 34.0`、`separator_thickness: 1.0`、`splitter_hit_size: 8.0`(`Default` 全是字面量) |
| R2 | 竖直布局靠**手工像素累加**而非 flex 容器 | `editor .../autolayout/geometry/region_frames.rs` | `center_y = top_bar_height + separator_thickness + host_bar_height + separator_thickness`;`fixed_vertical` 6 项相加;`x += width + separator_thickness` 逐项推进 |
| R3 | 断点阈值是**固定物理像素**(虽 token 化但非逻辑单位) | `iface .../ui/design_tokens.rs`、`editor .../autolayout/layout_tier.rs` | `breakpoint_ultra_width: 480.0`/`narrow: 640.0`/`wide: 1260.0`、`compact_side_width: 1100.0`;`layout_tier` 直接拿物理宽度分档 |
| R4 | `scale_factor` 捕获但**不参与布局** | `runtime .../core/framework/window/resolution.rs`、`runtime .../ui/surface/input/window_pump.rs` | `resolution.rs::scale_factor()` 返回缩放、`window_pump.rs` 存 `metrics.scale_factor`,但 autolayout 输入里没有它——高 DPI 下 chrome 仍按 1.0× 物理像素绘制 |

> 注意:R2 中区域**宽/高本身已走 `solve_axis_constraints`(相对求解)**,这点是对的;问题是固定 chrome 条目(顶栏/host 栏/状态栏/分隔符)的厚度是硬编码像素并被**手工竖向累加**,而不是作为 flex 容器的 `basis(逻辑 token) + gap` 参与同一次求解。修正点是"把竖向也交给一次 flex 容器求解",而非推翻已有的横向约束求解。

## 3. 设计

### 3.1 三层自适应模型(自上而下)

```
① 根 DPI 缩放        scale_factor 注入视口根:logical → physical = logical × scale
   (对标 UE DPI curve / GetViewportScale)         ▼ 向下传播,全树共享
② 相对 / flex 布局   grow 权重 + basis 百分比 + min/max 约束 + gap(逻辑 token)
   (对标 UE Fill/SBox FillWidth、anchor 比例)      ▼ 产出逻辑几何
③ 断点 tier 降级     logical_width 落档 → 折叠/丢列/页签溢出(15e/15d/15a)
   (对标 material-ui xs/sm/md/lg/xl 响应式)
```

三层各司其职、互不替代:① 解决"同样布局在 4K/高 DPI 下不能变小"(等比放大);② 解决"窗口拉宽/变窄时区域按比例伸缩、不裁内容"(弹性);③ 解决"窄到一定程度要改变结构"(折叠降级)。当前实现①缺失、②在壳层被像素累加绕过、③已由 `15e` 部分落地但判定用的是物理像素(见 R3)。

### 3.2 相对优先单位规范(尺寸怎么写)

尺寸表达优先级,从高到低:

1. **相对**:`flex-grow` 权重(瓜分富余)/ `flex-basis` 百分比 / `auto` / `min`-`max` 约束。区域、抽屉、center 一律优先用这一档。
2. **DPI 无关逻辑单位 token**:`$--left-drawer-width`、`$gap.m`、`$control.height` 等(接 `01`)。token 值是**逻辑单位**,渲染前统一乘 `scale_factor`。固定厚度的 chrome 条目(顶栏/状态栏/分隔符)用这一档。
3. **物理像素裸值**:**仅** `region.center` 自由区 / 用户内容(接 `13` §3.2、`index` 全局约束)。chrome 三处(资产扫描 / 13 约束 / 10 渲染)一致禁裸物理像素。

硬规则:**逻辑单位 token ≠ 物理像素**。当前 `EditorDensityTokens` 把 token 当物理像素直接用是 R3 的根;本文把它重定义为逻辑单位,缩放在根。

### 3.3 anchor / stretch → Taffy/flex 映射(带虚幻源码证据)

虚幻用"锚点 0~1 相对比例 + Stretch 模式"做多分辨率自适应。证据与映射:

**(a) 锚点是 0~1 归一化比例,`Minimum != Maximum` 即拉伸** —— `dev/UnrealEngine/.../Slate/Public/Widgets/Layout/Anchors.h:17-23,78-81`:

```cpp
/** Holds the minimum anchors, left + top. */
FVector2D Minimum;
/** Holds the maximum anchors, right + bottom. */
FVector2D Maximum;
// ...
bool IsStretchedVertical()   const { return Minimum.Y != Maximum.Y; }
bool IsStretchedHorizontal() const { return Minimum.X != Maximum.X; }
```

**(b) 锚点比例 × 父尺寸 = 像素;Fill 模式尺寸 = 锚点间距 − 边距** —— `.../Slate/Private/Widgets/Layout/SConstraintCanvas.cpp:240-269`:

```cpp
const FMargin AnchorPixels =
    FMargin(Anchors.Minimum.X * AllottedGeometry.GetLocalSize().X,   // 比例 × 父宽
            Anchors.Minimum.Y * AllottedGeometry.GetLocalSize().Y,
            Anchors.Maximum.X * AllottedGeometry.GetLocalSize().X,
            Anchors.Maximum.Y * AllottedGeometry.GetLocalSize().Y);
const bool bIsHorizontalStretch = Anchors.Minimum.X != Anchors.Maximum.X;
// ...
if (bIsHorizontalStretch) {
    LocalPosition.X = AnchorPixels.Left + Offset.Left;
    LocalSize.X     = AnchorPixels.Right - LocalPosition.X - Offset.Right; // 拉伸:间距-边距
} else {
    LocalPosition.X = AnchorPixels.Left + Offset.Left - AlignmentOffset.X; // 非拉伸:固定尺寸+pivot
    LocalSize.X     = Size.X;
}
```

**(c) pivot/对齐是 0~1** —— `.../UMG/Public/Components/CanvasPanelSlot.h:24-38`:`FAnchorData{ FMargin Offsets; FAnchors Anchors; FVector2D Alignment; }`,`Alignment` 注释 "the pivot point ... upper left at (0,0), lower right at (1,1)"。

**(d) Stretch 模式枚举** —— `.../Slate/Public/Widgets/Layout/SScaleBox.h:31-68`:`None / Fill / ScaleToFit / ScaleToFitX/Y / ScaleToFill / ScaleBySafeZone / UserSpecified`,配 `EStretchDirection{Both, DownOnly, UpOnly}`。

映射到 Zircon Taffy/flex:

| 虚幻概念 | 证据 | Taffy/flex 等价 | Zircon 落点 |
| --- | --- | --- | --- |
| 锚点 `Minimum/Maximum` 0~1 | `Anchors.h:17-23` | `flex-basis` 百分比 / `position:absolute` 四向 inset(%) | `UiLayoutStyle.flex_basis`;`Overlay/Canvas` |
| `IsStretchedHorizontal`(min≠max) | `Anchors.h:80` | `flex-grow:1` + `align/justify: stretch` | `flex_grow` + `UiAlignment2D::Fill` |
| `AnchorPixels = Anchor × LocalSize` | `SConstraintCanvas.cpp:240-244` | 百分比 basis 由父尺寸求值(Taffy 内建) | `free_child_frame` 已同构 |
| `Alignment` pivot 0~1 | `CanvasPanelSlot.h:33-38` | `align-self` / `justify-self` / pivot | `child_frame.rs` node.pivot |
| `EStretch::Fill` | `SScaleBox.h:38` | `flex-grow + stretch`(非等比填满) | center/抽屉主区 |
| `EStretch::ScaleToFit` | `SScaleBox.h:41-44` | `aspect-ratio` + `max-width/height:100%`(等比) | 图标/缩略图/视口预览 |
| `ScaleBySafeZone` | `SScaleBox.h:62`、`SafeZoneSlot.h` | 安全区 `padding`/`inset` | 浮窗/全屏面板边距 |

### 3.4 DPI / scale_factor 缩放(核心节)

虚幻把 DPI 缩放作为**全局一处、向下传播**的根缩放,而不是逐控件改像素:

- `dev/UnrealEngine/.../SlateCore/Public/Widgets/SWindow.h:589-593`:
  ```cpp
  /** Returns the DPI scale factor of the native window */
  float GetDPIScaleFactor() const;
  /** Overrides the DPI scale factor of the native window */
  void  SetDPIScaleFactor(const float Factor);
  ```
- `dev/UnrealEngine/.../UMG/Public/Blueprint/WidgetLayoutLibrary.h:59-64,108`:
  ```cpp
  // "Gets the current DPI Scale being applied to the viewport and all the Widgets."
  static float GetViewportScale(const UObject* WorldContextObject);
  // 输入坐标按 DPI 反算
  static bool  GetMousePositionScaledByDPI(APlayerController* Player, float& X, float& Y);
  ```

Zircon 规范(对标上述):

1. **根注入**:autolayout 输入携带 `scale_factor`(来自 `resolution.rs::scale_factor()`),在视口根一次性持有,向下传播,全树共享——而不是各 owner 自己读。
2. **逻辑↔物理换算**:布局全程用**逻辑单位**;上屏前 `physical = logical × scale_factor`。token(§3.2 第 2 档)在解析点保持逻辑值;逻辑→物理换算**单点在 `21` 顶点装配阶段**(乘 scale + 像素吸附,`10` §3.1 已立条款),管线其余各段不得预乘 scale。(2026-07-02 评审收口)
3. **断点用逻辑宽度**:tier 判定用 `logical_width = physical_width / scale_factor`(修 R3)。保证 1920×1080@1.0 与 3840×2160@2.0 落同一 tier、同一观感——这正是"适配分辨率/DPI"的关键。
4. **输入命中逆缩放**:指针命中、splitter 拖拽把物理坐标 `÷ scale_factor` 回到逻辑空间再比较(对标 `GetMousePositionScaledByDPI`)。`18` 已同步此条款:hit_test 输入为逻辑坐标,物理→逻辑换算在输入边界一次完成。(2026-07-02 评审收口)
5. **closes R4**:`scale_factor` 从"仅 `window_pump.rs` 存着"升级为 autolayout 的一等输入。

> **文本延伸(见 `17`)**:本节的根缩放对**字体光栅化**同样适用——字形须按物理像素 `font_size_logical × scale_factor` 重栅格,而非固定字号拉伸。当前 `sdf_font_bake.rs` 的 atlas key 不含 scale,是 R4 在文本上的同源缺陷,详见 `17` §3.2。

### 3.4a 分辨率成熟度:scale 模式 / 多显示器 / 分数缩放 / 安全区(深化)

§3.4 的根缩放只解决"单窗口跟随系统 DPI"。一个成熟的分辨率方案还要处理 **scale 模式选择、跨显示器迁移、分数缩放吸附、安全区**。对标 Unity UI Toolkit `PanelSettings` 的三种 scale 模式(权威思想来源,见 `dev/ui-toolkit-manual-code-examples` + UIElements docs):

| Scale 模式 | 语义 | 适用 | 对标 |
| --- | --- | --- | --- |
| `constant-physical`(默认 chrome) | 逻辑单位即物理尺寸,scale = 系统 DPI;1pt 在任何屏物理大小一致 | 编辑器 chrome(工具条/抽屉/控件) | Unity `ConstantPhysicalSize`;UE `GetDPIScaleFactor` |
| `constant-pixel` | scale = 1,逻辑=物理像素,不随 DPI 变 | 像素精确内容、center 自由区可选 | Unity `ConstantPixelSize` |
| `scale-with-resolution` | 按参考分辨率比例缩放(`scale = f(actual/reference)`) | 需整体等比放大的全屏布局/预览 | Unity `ScaleWithScreenSize` + reference resolution |

Zircon 规范(在 §3.4 根缩放之上):

1. **scale 模式是 panel/根属性**:每个渲染根(主窗口、浮动窗口、视口)声明 scale 模式 + 参考分辨率,`effective_scale` = 模式 × 系统 DPI;chrome 默认 `constant-physical`,与 §3.4 一致。**(2026-07-02 评审收口)现状登记:本条为"后续扩展"——`ResolutionContext`(§4)暂无 scale mode 字段,当前仅 root `scale_factor` 一档(等价 `constant-physical`);落地时补切片并扩 `ResolutionContext`,在此之前 `constant-pixel`/`scale-with-resolution` 两档不可声明。**
2. **每显示器 DPI**:窗口跨显示器迁移时 `scale_factor` 随目标显示器更新(对标 Win32 per-monitor-V2 DPI、UE `SetDPIScaleFactor`),触发受影响子树重算(接 09/10),不重载资产。
3. **分数缩放吸附**:1.25×/1.5× 等分数 scale 下,逻辑→物理换算产生分数像素;**像素吸附归渲染/合成层**(顶点装配,见 `21` §3.5),Taffy 侧 `disable_rounding` 保留分数(见 `13`),由 21 在装配顶点时对文本/1px 边框整像素吸附,自由内容不吸附——避免 Taffy 取整与渲染取整双重误差。
4. **安全区 / 工作区**:布局根扣除系统保留区(任务栏、刘海、窗口装饰)得可用工作区,chrome 在工作区内布局(对标 Unity `safe-area`、移动端 safe area inset)。
5. **断点用逻辑宽度**(承 §3.4-3):tier 判定永远用 `logical_width = physical / effective_scale`,与 scale 模式无关,保证不同屏同观感。

> Taffy 取整归属(见 `13`/`21`):本仓库已 `taffy.disable_rounding()`(保真分数 30.5px 控件)。Taffy 0.10 的 `round_layout` 用累积坐标避免间隙(`compute/mod.rs:219-274`),关闭后**像素吸附责任移交渲染层**——这正是上面第 3 点的依据,`21` §3.5 落实。

### 3.5 autolayout 计算规则修正(R1/R2 → flex 容器)

把 `region_frames.rs` 的"竖向手工像素累加"改为"壳作为一个竖向 flex 容器,一次求解":

**修正前(R2,现状)**——固定厚度硬编码 + 手工累加:
```text
center_y      = top_bar_height + sep + host_bar_height + sep        // 4 项裸像素相加
fixed_vertical= top_bar + sep + host_bar + sep + sep + status_bar + (bottom? sep:0)
center_h      = size.height - fixed_vertical
x += width + separator_thickness                                    // 逐项推进
```

**修正后(规范)**——壳竖向 flex 容器,条目用逻辑 token,中间用 grow,分隔用 gap:
```text
shell = flex column, gap = $separator (逻辑 token × scale)
  ├─ top_bar     : basis $top-bar-height (逻辑), grow 0      // 固定条:basis
  ├─ host_bar    : basis $host-bar-height,        grow 0
  ├─ work_band   : grow 1                                    // 弹性主带:吃掉富余
  │     └─ row = flex row: [left grow/ basis%] [center grow 1] [right basis%]  // 横向已是约束求解
  ├─ bottom      : basis $bottom-output-height(逻辑), grow 0,可被 tier 折叠
  └─ status_bar  : basis $status-bar-height,      grow 0
```
要点:① 固定条目的厚度 = 逻辑 token(§3.2 第 2 档),根缩放统一乘 scale(§3.4);② 主带/center 用 `grow` 吃富余,不靠 `height - Σ固定` 反算;③ 分隔符是容器 `gap` 而非 `x += sep` 累加;④ chrome metrics(R1)整体重定义为逻辑单位,`Default` 值是逻辑设计基准(1.0× 下数值不变,高 DPI 下自动放大)。

### 3.6 flex 特性充分利用清单

taffy 已支持但壳层未用满的特性,本文要求优先采用:

| 特性 | 用途 | 接 |
| --- | --- | --- |
| `flex-grow` 权重 | center/主带瓜分富余,窗口拉宽时按比例伸 | R2 修正 |
| `flex-basis` 百分比 | 抽屉/列按父比例定宽(对标 anchor 0~1) | §3.3 |
| `flex-shrink` | 收窄时富余列等比回收 | `15d` 列分配 |
| `min/max` 约束 | 防裁:列/抽屉到最小宽就停(对标 slint `LayoutInfo.min`) | `15d`、`15e` |
| `gap`(逻辑 token) | 分隔/间距,替代 `x += sep` 累加 | R2 修正 |
| `aspect-ratio` | 等比缩放(对标 `EStretch::ScaleToFit`) | 图标/视口预览 |
| `margin: auto` 居中 | 主轴居中(对标 `dev/bevy/.../ui/layout/anchor_layout.rs` 的 `margin: auto()`) | 命令面板/对话框 |
| `wrap` | 工具条溢出换行 | `15a` |

当前未用满(已知 taffy 桥限制,记录待 `editor_ui/02` 评估,不在本文实现):

- 主轴对齐只支持 `Start/Fill`(`compute.rs::main_axis_alignment_supported`),`Center/End` 主轴对齐走不了 taffy。**脚注(2026-07-02 评审收口,与 `13` §3.1 互注)**:此限制仅指**容器投影路径**;DTO 路径(`taffy_style_from_ui_layout_style`)对 `justify-content` 完整支持——两条路径的支持面不同,勿混读。
- `Overlay/Canvas/Scroll/VirtualList` 不进 taffy(Zircon-owned),其相对定位走 `free_child_frame`(§2.1 已同构 anchor)。
- 子节点非零约束优先级不被 taffy 接受(`compute.rs::taffy_supports_axis_constraint_priority` 要求 `priority == 0`),高优先级约束回落 Zircon 求解。

## 4. 接口与数据结构草案(Rust,规范形态非实现)

```rust
/// 根分辨率上下文:视口根一次性持有,向下传播(对标 UE GetViewportScale)
pub struct ResolutionContext {
    pub scale_factor: f32,      // 来自 resolution.rs::scale_factor()
    pub logical_size: ShellSizePx, // 逻辑像素 = physical / scale_factor
}
impl ResolutionContext {
    pub fn to_physical(&self, logical: f32) -> f32 { logical * self.scale_factor }
    pub fn logical_width(&self) -> f32 { self.logical_size.width } // 断点判定用此值
}
// chrome 度量 = 逻辑单位(1.0× 下与现值一致),渲染前乘 scale
pub struct WorkbenchChromeMetricsLogical { /* top_bar_height: f32(逻辑) ... */ }
// 断点分档用逻辑宽度(修 R3)
pub fn workbench_layout_tier_for_logical_width(logical_w: f32) -> WorkbenchLayoutTier;
```

## 5. 模块与文件落点(后续代码切片的规范指引,本文不写代码)

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 注入 | `autolayout/` 新 owner(如 `resolution_context.rs`) | 持 `scale_factor` + 逻辑尺寸,喂入壳几何输入 |
| 改 | `workbench_chrome_metrics.rs` | 字段语义改为**逻辑单位**;数值不变(逻辑基准),渲染前乘 scale |
| 改 | `geometry/region_frames.rs` | 竖向改 flex 容器:固定条 basis + 主带 grow + gap,删手工累加 |
| 改 | `layout_tier.rs` | 分档输入改 `logical_width`,阈值为逻辑 token |
| 改 | `iface .../ui/design_tokens.rs` | breakpoint/drawer/chrome token 注释为逻辑单位 |

落点遵 `engine-code-structure-convention.md`:owner 叶子承载逻辑、根 wiring 保持薄、隐藏内容 `log` 标注(无静默截断)。

## 6. 里程碑切片化

| # | 切片 | 验证命令 |
| -- | --- | --- |
| S1 | 文档规范成文(本文 + 13/15e/02/index 对齐) | 交叉引用闭合;无新增代码 |
| S2 | `ResolutionContext` 注入壳几何(R4) | `cargo test -p zircon_editor --lib resolution_context --locked` |
| S3 | chrome metrics → 逻辑单位 + 渲染乘 scale(R1) | `cargo test -p zircon_editor --lib --locked` |
| S4 | `region_frames` 竖向 flex 容器化(R2) | `cargo test -p zircon_editor --lib workbench_shell_geometry --locked` |
| S5 | 断点判定改逻辑宽度(R3)+ 多 scale 一致性测试 | `cargo test -p zircon_editor --lib layout_tier --locked` |

## 7. 测试矩阵

| 测试 | 断言 |
| --- | --- |
| `tier_uses_logical_width_consistent_across_scale` | 1920@1.0 与 3840@2.0 落同一 tier |
| `region_ratio_invariant_under_scale` | scale 变化时各区域占父比例不变 |
| `chrome_metrics_scale_with_dpi` | scale=2.0 时 chrome 物理厚度翻倍、逻辑值不变 |
| `shell_vertical_uses_flex_not_pixel_sum` | 竖向几何由容器求解产出,无 `Σ固定 + x推进` |
| `no_bare_physical_px_in_chrome`(扫描) | chrome 资产无裸物理像素;center 自由区允许 |
| `grow_distributes_surplus_by_weight` | 主带/center 按 grow 权重瓜分富余宽 |

## 8. 风险与对策

- 风险:逻辑/物理双坐标引入换算 bug(命中错位)。对策:换算只在两处(根注入、上屏/命中),中途全逻辑;`GetMousePositionScaledByDPI` 式逆缩放集中在输入边界。
- 风险:竖向 flex 容器化触发壳几何回退。对策:S4 前先在 S2/S3 把 scale 注入与 metrics 逻辑化落稳,逐切片验证截图(接 `15e` 三档 harness)。
- 风险:与 `15e` 既有 tier 实现冲突。对策:`15e` 的 tier 决策改读 `logical_width` 即可复用,不重做折叠/降级。

## 9. 完成定义

三层自适应模型成文;相对优先单位规范确立;anchor/stretch→flex 映射带虚幻源码证据落表;DPI/scale_factor 缩放写成核心规范并点名 R4;autolayout 像素累加(R2)给出 flex 容器修正;flex 充分利用清单确立;`13`/`15e`/`02`/`index` 与本文对齐交叉引用。

## 10. 边界约束

不改 Taffy 求解算法(已存在,§2.1);不重做 docking 运行时(`03/07`);约束 token 值归 `01`、区域声明归 `02`、断点折叠实现归 `15e`;运行时能力(如 scale 注入管线)若属运行时构建则回流 `editor_ui/`。本文只立规范 + 指引落点,不产出代码。

## 11. 参考实现对照(dev/ 源码锚点)

| 维度 | 锚点 | 取什么 |
| --- | --- | --- |
| 锚点相对比例 | `dev/UnrealEngine/.../Slate/Public/Widgets/Layout/Anchors.h:17-23,78-81` | 0~1 归一化、min≠max 即拉伸 |
| 锚点求值 | `dev/UnrealEngine/.../Slate/Private/Widgets/Layout/SConstraintCanvas.cpp:240-281` | `Anchor × 父尺寸`、Fill=间距−边距、非拉伸=固定+pivot |
| pivot/对齐 | `dev/UnrealEngine/.../UMG/Public/Components/CanvasPanelSlot.h:24-38` | `FAnchorData{Offsets,Anchors,Alignment(0~1)}` |
| Stretch 模式 | `dev/UnrealEngine/.../Slate/Public/Widgets/Layout/SScaleBox.h:31-68` | Fill/ScaleToFit/ScaleToFill/ScaleBySafeZone |
| DPI 根缩放 | `dev/UnrealEngine/.../SlateCore/Public/Widgets/SWindow.h:589-593` | `GetDPIScaleFactor/SetDPIScaleFactor` |
| DPI 视口缩放/输入 | `dev/UnrealEngine/.../UMG/Public/Blueprint/WidgetLayoutLibrary.h:59-64,108` | "applied to the viewport and all the Widgets"、`GetMousePositionScaledByDPI` |
| 安全区 | `dev/UnrealEngine/.../UMG/Public/Components/SafeZoneSlot.h` | SafeAreaScale margin |
| 响应式断点 | `dev/material-ui/packages/mui-system/src/breakpoints/breakpoints.d.ts` | xs/sm/md/lg/xl + `ResponsiveStyleValue` |
| 约束 min/max+stretch | `dev/slint/internal/core/layout.rs` | `LayoutInfo{min,max,min_percent,max_percent,stretch}` |
| auto 居中 | `dev/bevy/examples/ui/layout/anchor_layout.rs` | `margin: auto()` 居中 |
| 控件锚点 | `dev/godot/scene/gui/control.h` | anchor 0/1 + offset 四向 |

## 12. 状态与产出记录

| 日期 | 切片 | 状态 | 产出/证据 | 后续项 |
| --- | --- | --- | --- | --- |
| 2026-06-26 | 16.S1 相对布局/多分辨率/DPI 规范立项 | planned | 代码核实:runtime taffy 桥相对布局正确(`compute.rs::disable_rounding`、`child_frame.rs` anchor×父尺寸),但壳层四反模式 R1–R4(`workbench_chrome_metrics.rs` 裸像素、`region_frames.rs` 像素累加、`design_tokens.rs` 断点物理像素、`scale_factor` 未参与布局)。给出三层自适应模型、相对优先单位、anchor/stretch→flex 映射(带虚幻 `Anchors.h`/`SConstraintCanvas.cpp`/`SScaleBox.h`/`SWindow.h`/`WidgetLayoutLibrary.h` 源码证据)、DPI 核心节、autolayout flex 容器修正、flex 充分利用清单、5 条切片 + 6 条测试矩阵。 | 按 §6 推进 S2–S5;`13`/`15e`/`02`/`index` 已同步对齐本文。 |
| 2026-07-05 | 16.S5 断点判定逻辑宽度 cutover | implemented-focused-passed-screenshot-passed | 按 R3/R4 先关闭 tier 输入的物理像素误判:`layout_tier.rs` 现在显式区分 logical/physical helper,physical helper 统一 `physical_width / scale_factor`;`compute_workbench_shell_geometry`、`window_minimums.rs`、componentized Workbench bridge 与 retained host `shell_scale_factor` 接线到 retained host window scale contract,真实 winit window 同步写入 `Window::scale_factor()`。新增三条 logical_width 回归覆盖 3840@2.0 与 1920@1.0 同档、1280@2.0 右抽屉折叠、1800@2.0 保持 regular 右抽屉,并新增窗口 scale 默认/非法值过滤回归。验证:`cargo fmt -p zircon_editor` 通过;旧 `*_for_width` API 扫描 0 命中;focused `cargo test -p zircon_editor --lib window_scale_factor_defaults_to_one_and_filters_invalid_values ...` 1/1 通过;focused `cargo test -p zircon_editor --lib logical_width ...` 3/3 通过;M3 screenshot harness `capture_m3_gui_acceptance_visual_artifacts` 1/1 通过并刷新 `docs/tests/editor` 的 640/900/1260 PNG,target 同名截图扫描无匹配。 | S2/S3/S4 的完整 `ResolutionContext`、chrome 逻辑单位上屏乘 scale、竖向 flex 化仍未关闭;继续按切片推进。 |
