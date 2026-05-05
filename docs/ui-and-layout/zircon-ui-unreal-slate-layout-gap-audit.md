---
related_code:
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime_interface/src/ui/layout/metrics.rs
  - zircon_runtime_interface/src/ui/layout/slot.rs
  - zircon_runtime_interface/src/ui/layout/mod.rs
  - zircon_runtime_interface/src/tests/ui_layout.rs
  - zircon_runtime_interface/src/ui/surface/arranged.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/child_frame.rs
  - zircon_runtime/src/ui/layout/pass/clip.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SWidget.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/Geometry.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/ArrangedWidget.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/ArrangedChildren.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/LayoutUtils.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/BasicLayoutWidgetSlot.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/WidgetSlotWithAttributeSupport.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/Clipping.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SOverlay.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SOverlay.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SBoxPanel.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Layout/SConstraintCanvas.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Layout/SConstraintCanvas.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Layout/SGridPanel.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Layout/SWrapBox.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Layout/SScrollBox.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Layout/SSplitter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Layout/SScaleBox.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Layout/SDPIScaler.cpp
implementation_files:
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime_interface/src/ui/layout/metrics.rs
  - zircon_runtime_interface/src/ui/layout/slot.rs
  - zircon_runtime_interface/src/ui/layout/mod.rs
  - zircon_runtime_interface/src/tests/ui_layout.rs
  - zircon_runtime_interface/src/ui/surface/arranged.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/child_frame.rs
  - zircon_runtime/src/ui/layout/pass/clip.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
plan_sources:
  - user: 2026-05-06 完善布局方面内容，参照 dev 下虚幻源码
  - .codex/plans/布局系统.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - cargo test -p zircon_runtime_interface --lib ui_layout_geometry_slot_and_metrics_contracts_construct --locked --target-dir E:\zircon-build\targets\ui-layout-dto-contracts
doc_type: milestone-detail
---

# Zircon UI 与 Unreal Slate 布局差异审计

## Scope

本文补齐 `Zircon UI 与 Unreal Slate 差异审计及后续里程碑` 中“布局”部分的细化内容。目标不是复制 Unreal C++ API，而是把 `dev/UnrealEngine` 的 Slate 布局职责拆成 Zircon 可落地的 retained `.ui.toml -> UiTree -> UiSurfaceFrame` 契约。

当前 Zircon 已经具备基础 measure/arrange、`UiArrangedTree`、clip frame、z/paint order、scrollable virtual window 和 shared hit/render 消费同一 arranged frame 的起点。L1 起步契约已在 `zircon_runtime_interface::ui::layout` 中加入 `UiGeometry`、`UiLayoutMetrics`、`UiPixelSnapping`、`UiSlot` 和基础 slot padding/alignment DTO；这些类型先作为中立 contract 存在，后续 runtime/template/editor 迁移必须继续从该 contract 往上接入，不能重新把 parent slot 语义塞回 child node。与 Slate 相比，剩余缺口集中在 transform 运算、slot policy 消费、panel 算法、DPI/scale 脏传播、pixel snapping 渲染接入和布局失效边界。

## Unreal Slate 布局源码基线

| Slate 源码 | 布局职责 | 对 Zircon 的约束 |
|---|---|---|
| `SWidget.cpp` | `SlatePrepass()` 先更新属性，再对子树执行 `Prepass_Internal()`，子节点先缓存 desired size，父节点最后 `CacheDesiredSize(ComputeDesiredSize(layout_scale))`；`Collapsed` 子节点跳过递归但标记 prepass dirty。 | Zircon 需要把“反向 desired-size prepass”明确成一等阶段，并让 layout scale、visibility、attribute invalidation 进入同一个 cache key。 |
| `Geometry.h` | `FGeometry` 同时持有 local size、accumulated layout transform、accumulated render transform；`MakeRoot()` / `MakeChild()` 生成父子几何；`LocalToAbsolute()`、`AbsoluteToLocal()`、layout/render bounding rect 用同一几何源。 | `UiFrame` 只能表达绝对轴对齐矩形，无法覆盖 local geometry、layout transform、render transform、DPI scale、popup rounded local 和 transformed bounds。 |
| `ArrangedChildren.h` / `ArrangedWidget.h` | `OnArrangeChildren()` 输出 `FArrangedChildren`，并通过 visibility filter 决定布局、paint、hittest 等调用者要的子集。 | `UiArrangedTree` 已接近该职责，但还需要记录调用目的、local frame、layout scale 和 rejected/collapsed 诊断。 |
| `BasicLayoutWidgetSlot.h` / `WidgetSlotWithAttributeSupport.h` | slot 是 parent-owned attribute container；padding、alignment 等 slot 属性变化直接产生 layout invalidation。 | Zircon 当前 `slot_attributes.layout` 会并入 child layout contract，缺少可持久、可诊断、可独立失效的 parent slot DTO。 |
| `SOverlay.h` / `SOverlay.cpp` | overlay slot 继承基础 padding/alignment，并单独持有 `ZOrder`；构造和运行时变更都会稳定排序 children。 | Zircon `Overlay` 当前接近 Free placement，缺 overlay slot z-order、alignment、padding 与 paint layer 分组的专门语义。 |
| `SConstraintCanvas.h` / `SConstraintCanvas.cpp` | canvas slot 持有 offset、anchors、alignment、auto-size、z-order；arrange 时从 parent allotted size 计算 anchored local position/size，并输出 layer breaks。 | Zircon 的 anchor/pivot/position 存在于 child node，缺 parent-owned canvas slot；这会阻碍 Canvas/Free/Overlay 与其他 parent slot 语义共存。 |
| `LayoutUtils.h` / `SBoxPanel.cpp` | stack panel 按 slot padding、alignment、size rule、stretch coefficient、min/max、flow direction 和 `AllowShrink` 求解，再输出 child geometry。 | Zircon 线性布局目前只有 gap、基础 axis constraints 和 stretch 记录；缺 per-slot padding/alignment、RTL flow、StretchContent、min/max 冻结迭代和 scroll/non-scroll shrink 差异。 |
| `SGridPanel.cpp` | grid slot 拥有 column/row/span/layer/nudge；列/行 desired size 与 fill coefficient 分离；arrange 通过 partial sums 求 span frame；paint 以 grid layer 分组提升 layer id。 | Zircon 还没有正式 `GridBox` container 和 grid slot metadata，后续不能用 flat linear/overlay 临时模拟。 |
| `SWrapBox.cpp` | wrap/flow 维护 line state、preferred wrap size、inner slot padding、force new line、fill empty space、line max cross size、horizontal alignment 和 RTL 翻转。 | `FlowBox` 需要作为独立 panel 算法，而不是 HorizontalBox 溢出后的特殊分支。 |
| `SScrollBox.cpp` | `SScrollPanel` 以 `PhysicalOffset` 偏移 stack arrange，支持 back/front pad scrolling，scroll panel arrange 禁止 shrink，并把滚动状态、focus/navigation、scrollbar 行为组合在同一控件。 | Zircon 已有 offset clamp 和 visible window；仍缺 scroll anchoring、pad scrolling、overscroll/animation、focus-to-scroll 策略、scrollbar slot 和虚拟行稳定 key。 |
| `SSplitter.cpp` | splitter 把 child size rule、resizable flag、min size、handle physical/hit size 和 user-resize callback 作为 panel slot/paint/input 组合；handle geometry 与 child geometry同源。 | Zircon editor workbench splitters 还没有进入 shared layout panel；需要把 splitter region、handle hit rect 和 resize policy 纳入 shared arranged tree。 |
| `SScaleBox.cpp` | ScaleBox 可能需要 custom prepass，因 desired size 取决于上一帧 allotted area 和 computed content scale；arrange 输出 child scale，paint 时可能强制 clip。 | Zircon 缺 area-dependent prepass 和 scale container，后续图片/viewport/preview fitting 不能只靠 render-side缩放。 |
| `SDPIScaler.cpp` / `SWindow.cpp` | root prepass 使用 application scale 与 window DPI；`SDPIScaler` 通过 `GetRelativeLayoutScale()` 改变子树 desired size 与 child geometry。 | Zircon 需要 surface/window 级 `layout_scale`，并保证 scale 变化触发 prepass/layout dirty，而不是只改变最终渲染尺寸。 |
| `SWidget.h` / `DrawElementTypes.cpp` / `Geometry.h` | widget 可设置 `EWidgetPixelSnapping`；draw element 默认 snap-to-pixel，可显式关闭；`LocalToRoundedLocal()` 用于 popup/tooltip 避免半像素抖动。 | Zircon 需要定义 pixel snapping 在 layout、render extract、hittest 中的边界：render 可 snap，但 hit 与 debug 必须能追溯 unsnapped arranged geometry。 |

## 当前 Zircon 布局状态

`zircon_runtime_interface::ui::layout::UiFrame` 是当前已被 runtime/editor 广泛消费的轴对齐矩形基础，只保存 `x/y/width/height` 和 intersection/contains helpers。L1 起步契约新增的 `UiGeometry` 在不破坏现有 `UiFrame` 消费者的前提下，把 local size/offset、layout transform、render transform、absolute layout frame、render bounds、clip frame 和 pixel snapping policy 放到同一 DTO 中；默认 `UiGeometry::from_frame()` 保持 render bounds 等于 layout frame，保证后续迁移可以逐步替换而不改变现有 arranged tree 行为。

`UiLayoutMetrics` 现在记录 root logical/physical size、DPI scale、font scale、layout scale、flow direction 和默认 pixel snapping policy；这是 future prepass/arrange 根输入，而不是渲染端私有参数。`UiSlot` 现在作为 parent-owned placement identity 存在，包含 parent/child id、slot kind、padding、alignment、order 和 dirty revision；它还没有接入 template build 或 runtime arrange pass，但已经固定了 slot 不应在编译阶段被 merge 到 child node 后消失的 contract。

`UiTreeNode` 仍保存 constraints、anchor、pivot、position、container、scroll state、clip、z/paint order 和 dirty flags。`UiArrangedNode` 仍保存 arranged `frame`、`clip_frame`、visibility、input policy 和 control id，成为 render extract 与 hit grid 的共享空间输入。后续迁移必须把 slot/pixel/layout scale 等新 contract 接到这些现有输出，而不是平行制造 editor-only 或 renderer-only 坐标表。

`zircon_runtime::ui::layout::pass::measure_node()` 已实现自底向上的 desired size 计算：`Collapsed` 类 visibility 不参与布局，linear/scrollable 容器会累加主轴、取交叉轴最大值，leaf 节点可从文本与 Material 元组件推导内容尺寸。`arrange_node()` 已实现自顶向下 frame 传递：Free/Container/Overlay、HorizontalBox、VerticalBox、ScrollableBox 和 Space 分支共享 clip chain，scrollable 分支会 clamp offset 并产出 virtual window。

这说明 Zircon 已经落在 Slate 的“prepass -> arrange -> arranged frame -> paint/hittest”主方向上，但还没有达到 Slate 的 `FGeometry + slot policy + panel family` 深度。

## 布局差异细化

| 领域 | Slate 参考行为 | Zircon 当前状态 | 缺口 |
|---|---|---|---|
| Prepass / desired size | `SWidget::SlatePrepass()` 携带 layout scale，先递归子节点，再缓存父节点 desired size；collapsed 子树不会贡献 desired size。 | `measure_node()` 已自底向上测量并清空 collapsed subtree cache。 | 缺 layout scale multiplier、prepass cache generation、属性更新/dirty reason 与 desired-size 失效边界；文本真实测量后还需要把 font/shaping key 纳入 cache。 |
| Geometry authority | `FGeometry` 区分 local size、layout transform、render transform 和 accumulated transforms；同一几何可服务布局、绘制、坐标转换和 bounds。 | `UiFrame` 仍是现有 arranged/render/hit 基础；`UiGeometry`、`UiLayoutTransform`、`UiRenderTransform` 已进入 interface contract，但尚未被 runtime arrange/render/hit 消费。 | 缺 local-to-parent accumulated chain、transform composition、坐标转换 helpers、旋转/pivot render transform、popup rounding 和 transformed culling 接入。 |
| Arranged output | `FArrangedChildren` 带 visibility filter，panel 输出 widget + geometry。 | `UiArrangedTree` 带 frame、clip、z/order、visibility、input policy。 | 缺 local frame、layout scale、render transform、arrange purpose/filter、rejected node reason；debug reflector 不能完整复盘 arrange 决策。 |
| Slot policy | `TBasicLayoutWidgetSlot` / `LayoutUtils` 统一处理 padding、horizontal/vertical alignment、size rule、min/max、flow direction。 | `.ui.toml` 和 tree node 当前主要表达 node-level constraints、gap、anchor/pivot/position；`UiSlot` / `UiMargin` / `UiAlignment2D` 已进入 interface contract，但 template build 和 arrange pass 尚未消费。 | 需要把 `slot` schema 编译为 parent-owned `UiSlot`，并逐步加入 main-axis size rule、cross-axis fill、grid row/column/span、flow flags、layer/nudge 等 payload。 |
| Slot invalidation | Slate slot attributes 是 `TSlateContainedAttribute`，slot padding/alignment 等变化能直接打 layout dirty。 | `slot_attributes.layout` 在 template build 时被 merge，失去独立 slot identity。 | 需要 slot revision/dirty reason；slot mutation 不应伪装成 child node 自身 layout mutation。 |
| Canvas / free placement | `SConstraintCanvas` 用 slot offset、anchors、alignment、auto-size、z-order 从 parent geometry 派生 child local geometry。 | `free_child_frame()` 用 child anchor/pivot/position 直接求绝对 frame。 | 需要把 Free/Canvas placement 升级为 parent slot；保留 child 默认 intent，但 parent slot 覆盖必须可追踪。 |
| Linear panel | `SBoxPanel` 支持 Auto、Stretch、StretchContent、grow/shrink coefficient、min/max clamp 和 RTL iteration。 | HorizontalBox/VerticalBox 已支持 gap、stretch preservation、基础 axis solve。 | 缺 StretchContent、min/max 多轮冻结、cross-axis alignment、per-slot padding、RTL 和 `AllowShrink` 差异。 |
| Overlay / Container | `SOverlay`/single-child helpers 通过 slot padding/alignment 把 child 放入 allotted geometry。 | Free/Container/Overlay 使用 `free_child_frame()` 和 anchor/pivot/position。 | 需要把 overlay slot alignment 从 child absolute placement 中分离；Container 与 Overlay 应共享 slot policy，但保留不同 paint/z 语义。 |
| Grid | `SGridPanel` 拥有 row/column desired sizes、fill coefficients、span、layer、nudge 和 partial-sum arrange。 | 尚无正式 GridBox container。 | 需要新增 grid container 与 grid slot，支持 row/column fill、span、layer order、nudge、collapsed 子节点跳过和 golden layout。 |
| Flow / Wrap | `SWrapBox` 维护 line arrangement，支持 preferred wrap size、inner padding、force new line、fill line/empty space 和 line alignment。 | `.codex/plans/布局系统.md` 已定义 FlowBox 目标，但 runtime pass 尚未实现。 | 需要独立 FlowBox algorithm；不能用 HorizontalBox + manual newline 兼容分支替代。 |
| Scroll / virtual | `SScrollPanel` 以 `PhysicalOffset` 偏移 arranged stack，`AllowShrink=false`，desired size 可加入 front/back padding。 | ScrollableBox 已 clamp offset、记录 viewport/content extent、产出 virtual visible window。 | 缺 scroll anchor、front/back pad scrolling、overscroll/animation、keyboard/focus scroll request、scrollbar slot 与 virtual item stable-key。 |
| Splitter / resize | `SSplitter` 把 child sizing rule、resizable、min size、handle geometry 和 paint/input path 绑定在同一 panel。 | Workbench/autolayout 有 splitters，但 shared UI layout core 没有 SplitterBox panel。 | 需要 `SplitterBox` 或 `ResizableStack` panel，保证 handle frame、paint order 和 pointer hit route 都来自 arranged tree。 |
| Scale / fit containers | `SScaleBox` 的 desired size 和 child scale 可依赖 allotted geometry，需要 custom prepass 和 paint-time clip。 | 当前没有 scale/fill/fit container；图片和 viewport fitting 更多在宿主/渲染侧处理。 | 需要 `ScaleBox` / `FitBox` 语义，至少覆盖 `Fill`、`ScaleToFit`、`ScaleToFill`、user scale 和 clipping。 |
| DPI / scale | Window/application scale 进入 prepass；`SDPIScaler` 作为 relative layout scale 改变子树 desired size。 | `UiLayoutMetrics` 已记录 logical/physical size、DPI scale、font scale、layout scale 和 flow direction，但 runtime layout pass 尚未使用。 | 需要把 metrics 作为 surface root prepass/arrange 输入；scale 变化必须 invalidate prepass/layout/hit/render。 |
| Pixel snapping | Widget 与 draw element 可继承/启用/禁用 pixel snapping；render 默认 snap，可通过 draw effect 禁止。 | `UiPixelSnapping` 已进入 metrics/geometry contract；render extract 仍未根据该 policy 输出 snapped paint geometry。 | 需要 render extract 或 final paint stage 应用 snapping，并保留 unsnapped layout geometry 给 hit/debug。 |
| Clipping / culling | `FSlateClippingZone` 可从 geometry/paint geometry 建立 scissor/stencil clip，支持 transformed quads。 | `resolve_clip_frame()` 做轴对齐 frame intersection。 | V1 可继续 axis-aligned scissor，但 DTO 要预留 transformed clip zone；render/hit/debug 必须从 arranged clip chain 派生。 |

## DTO 分层落点

Zircon 不应把所有布局字段继续塞进 `UiTreeNode`。按照 Slate 的 slot/container 分工，下一轮应把 DTO 拆成四层：

| DTO 层 | 建议归属 | 说明 |
|---|---|---|
| Node layout intent | `UiTreeNode` / `.ui.toml layout` | 节点自己的 width/height、boundary、default anchor intent、visibility、clip、input policy 和 intrinsic content metadata。 |
| Parent slot | `zircon_runtime_interface::ui::layout::slot` | parent 对 child 的放置规则：padding、alignment、order、z/layer、linear size rule、grid row/column/span、flow flags、canvas anchors/offset、splitter resize policy。 |
| Surface metrics | `zircon_runtime_interface::ui::surface` 或 `ui::layout` | root logical size、physical size、dpi scale、font scale、layout scale、pixel snapping default、locale flow direction。 |
| Arranged geometry | `UiArrangedNode` / `UiLayoutCache` | prepass desired size、local geometry、absolute layout frame、render transform, clip chain, snapped paint frame, rejection reason 和 source slot id。 |

建议最小类型集合：

| 类型 | 最小字段 | 用途 |
|---|---|---|
| `UiMargin` | left/top/right/bottom | Slate `FMargin` 对等；slot padding、culling extension、popup inset 都复用。 |
| `UiAlignment2D` | horizontal, vertical | 表达 start/center/end/fill；支持 RTL 时 horizontal start/end 需由 flow direction 解析。 |
| `UiLayoutMetrics` | logical_size, physical_size, dpi_scale, font_scale, layout_scale, flow_direction, pixel_snapping | 每个 surface/frame 的 prepass 和 arrange 根输入。 |
| `UiGeometry` | local_size, local_offset, layout_transform, render_transform, absolute_frame, render_bounds | 替代单一 `UiFrame` 作为 long-term geometry authority。 |
| `UiSlot` | parent_id, child_id, kind, padding, alignment, order, dirty_revision | parent-owned slot identity；template build 不能只 merge 进 child node 后丢弃。 |
| `UiSlotKind` | Free/Overlay/Linear/Grid/Flow/Canvas/Scrollable/Splitter/Scale | parent-specific payload enum；每种 payload 单独文件，避免一个巨型 slot.rs。 |
| `UiArrangeDiagnostic` | source_slot, filter, rejected_reason, layout_input, resolved_policy | Widget Reflector 和 golden tests 复盘用，不参与正常渲染热路径。 |

## `.ui.toml` Schema 收敛方向

当前 `layout` 和 `slot_attributes.layout` 已经存在，但 build 阶段会把二者 merge 成 child layout contract。后续需要硬切为“node 自身 layout”和“parent slot layout”两条通道：

```toml
[[nodes.children]]
id = "ViewportToolbar"
type = "HorizontalBox"
layout = { width = { stretch = "Stretch" }, height = { preferred = 40.0, stretch = "Fixed" } }
slot = { kind = "Overlay", h_align = "Fill", v_align = "Start", padding = { left = 8.0, top = 8.0 }, z_order = 10 }
```

迁移规则：

- `layout.width` / `layout.height` 继续表示 child 自身约束。
- `slot` 表示 parent 如何放置这个 child；slot 不得被 merge 到 child node 后消失。
- 旧 `slot_attributes.layout` 只允许作为一次性迁移输入；编译输出必须保留为 `UiSlot`。
- Linear parent 下的 main-axis size rule 归 slot；cross-axis default 可从 child constraints 推导，但显式 alignment 必须归 slot。
- Canvas/Free parent 下的 anchor/offset/alignment 归 slot；child node 只保留默认 placement intent，避免同一个 child 在不同 parent 中语义冲突。

## Zircon 布局目标契约

布局真源仍然是 `.ui.toml` 描述和 `zircon_runtime_interface::ui` DTO，而不是 C++ live widget subclass。参考 Slate 时只采用职责划分：prepass 计算 desired size，arrange 输出几何，paint/hit/input/debug 只消费 arranged output。

目标数据流固定为：

1. `.ui.toml` 编译出 tree node、parent slot、style/text/image descriptors。
2. `UiSurface::compute_layout()` 使用 root `UiLayoutMetrics` 运行 prepass 和 arrange。
3. Layout pass 输出 `UiArrangedTree`，节点携带 layout geometry、render geometry policy、clip chain、visibility、slot-derived order 和 diagnostics。
4. Render extract、hit grid、pointer route、focus/navigation、debug reflector 都只消费该 arranged tree。
5. Editor Slint/native host 只能投影 arranged output，不能重新手写 toolbar、pane、popup 或 list 的坐标表。

## 实现顺序约束

这部分布局工作必须从 contract 和 lower-layer layout core 往上走，不能从 editor 截图或某个面板特例倒推：

1. 先添加 `UiMargin`、`UiAlignment2D`、`UiLayoutMetrics`、`UiSlot`、`UiGeometry` 的 interface contract 与 serde/golden tests。当前已完成起步版本：`UiGeometry`、layout/render transform、`UiPixelSnapping`、`UiLayoutMetrics`、`UiFlowDirection`、`UiSlot`、`UiSlotKind`、`UiMargin`、`UiAlignment2D` 和 `ui_layout_geometry_slot_and_metrics_contracts_construct` contract test。
2. 再让 template build 保留 parent-owned slot 输出，同时维持当前 node constraints 行为不变。
3. 然后只改 Linear/Overlay/Free 三个已有 panel 去消费 slot padding/alignment/order，保持 render/hit 仍来自同一 arranged tree。
4. 最后新增 Grid/Flow/Splitter/Scale 等 panel；每个新 panel 必须先有 measure/arrange golden tests，再接 editor/runtime fixture。

如果跳过第 1-2 步直接写 Grid/Flow，必然会继续扩大 `UiTreeNode` 和 `layout_contract.rs` 的职责，后续要为 slot identity、debug reflector 和 invalidation 返工。

## Layout Milestones

1. **L0 Slate 布局证据冻结**：保留本文列出的 Unreal 源码路径，补充 Zircon 当前 layout pass 的 source map；所有后续 layout PR 都必须声明参考的 Slate panel/geometry 行为和有意差异。
2. **L1 Geometry DTO 扩展**：新增 `UiGeometry`、`UiLayoutTransform`、`UiRenderTransform`、`UiLayoutMetrics`、`UiPixelSnapping`；`UiArrangedNode` 同时保留 layout frame、local frame、clip zone 和 render transform policy。
3. **L2 Prepass / Cache 收敛**：把 `measure_node()` 提升为显式 prepass，cache key 包含 layout scale、font scale、visibility、style/text/material revision；collapsed 子树与 hidden 子树按 Slate 可见性边界分开测试。
4. **L3 Slot Model**：引入 parent-owned slot DTO，覆盖 padding、alignment、main-axis size rule、cross-axis fill、grid row/column/span、flow flags、layer/nudge；`.ui.toml` 中 child 节点不再承担 parent-specific slot 语义。
5. **L4 Linear / Overlay Parity**：补齐 Auto/Stretch/StretchContent、min/max freeze pass、RTL flow、per-slot alignment、overlay alignment 和 scroll `AllowShrink=false` 差异。
6. **L5 Grid / Flow Panels**：实现 GridBox 与 FlowBox 的独立 measure/arrange；GridBox 覆盖 fill coefficient、span、layer；FlowBox 覆盖 wrap size、line alignment、force new line 和 fill empty space。
7. **L6 Scroll Anchoring / Virtual Stability**：在 ScrollableBox 中加入 anchor id、stable item key、front/back pad scrolling、focus-to-scroll request、scrollbar geometry slot 和 virtualization debug dump。
8. **L7 Splitter / Scale / Fit Panels**：把 editor workbench splitter、resize handle hit rect、preview/image/viewport fitting 升级为 shared panel，而不是继续散落在 host/autolayout 特例中。
9. **L8 DPI / Pixel / Debug**：surface root 持有 logical/physical size 与 DPI/font scale；render extract 记录 snapped paint geometry；Widget Reflector 输出 prepass cache、slot policy、layout/render transform、clip chain、pixel snapping 和 rejected reason。

## Acceptance Tests

布局验收应按底层到上层排列，避免只用 editor 截图覆盖底层错误：

- `zircon_runtime_interface` contract：`UiGeometry` 能表达 local/layout/render transform、scale、pixel snapping 和 axis-aligned clip fallback。
- Prepass tests：collapsed 子树 desired size 归零、hidden 子树仍占布局、scale/font/style/text revision 会触发重新测量。
- Linear tests：Auto、Stretch、StretchContent、min/max clamp、gap、padding、cross-axis alignment、RTL、scroll allow-shrink 差异均有 golden frame。
- Grid tests：row/column fill、span、layer、nudge、collapsed slot、paint order 与 hit order 使用同一个 arranged tree。
- Flow tests：wrap threshold、force new line、fill empty space、line max cross size、center/right/fill alignment 和 RTL 翻转。
- Scroll tests：offset clamp、front/back pad scrolling、anchor preservation、virtual window stable-key、focus-to-scroll、scrollbar frame。
- Splitter tests：fraction/content size rule、min-size stealing、collapsed child skip、physical handle frame、hit handle frame、resize callback target 都来自 arranged geometry。
- Scale/Fit tests：Fill、ScaleToFit、ScaleToFill、user scale、clip-needed modes、area-dependent prepass invalidation 和 snapped render bounds。
- DPI/pixel tests：DPI scale 改变 desired size 与 arranged frame；pixel snapping 只影响 render paint geometry，不改变 hit debug layout geometry。
- Editor/runtime golden：同一 `.ui.toml` 在 editor native host 和 runtime renderer 中输出同一 arranged frame、hit path、render extract count 和 debug snapshot。

## Assumptions

- V1 仍以 screen-space、axis-aligned UI 为主；transform DTO 先承载 layout/render scale、translation 和 future rotation，不要求一次实现 3D/world-space hit。
- `.ui.toml` 是 Zircon 的 authored truth；Unreal `SWidget` subclass 体系只作为职责参考，不引入 live widget virtual override 模型。
- Slot policy 属于 parent container。节点可声明默认 layout intent，但 Grid/Flow/Overlay/Linear 的 parent-specific slot 必须由 parent-owned slot DTO 表达。
- Pixel snapping 不能破坏 hit-test 一致性；若 render snapped geometry 与 layout geometry不同，debug snapshot 必须同时显示两者。
