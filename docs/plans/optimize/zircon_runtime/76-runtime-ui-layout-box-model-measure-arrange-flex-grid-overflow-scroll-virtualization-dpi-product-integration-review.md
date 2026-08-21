---
title: Runtime UI Layout、Box Model、Measure/Arrange、Flex/Grid、Overflow/Scroll、Virtualization、DPI 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime76
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime_interface/src/ui/layout
  - zircon_runtime_interface/src/ui/tree/node/layout_cache.rs
  - zircon_runtime/src/ui/layout
  - zircon_runtime/src/ui/template/build
  - zircon_runtime/src/ui/v2/surface_tree
  - zircon_runtime/src/ui/surface/node_pool.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/surface/virtual_window.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/table/virtualization.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/tree_view_virtualization.rs
  - zircon_runtime/src/ui/component/state_reducer/windowing.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_editor/src/ui/workbench/autolayout
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/virtual_rows.rs
  - zircon_editor/assets/ui
  - examples/woc/assets/ui
tests:
  - zircon_runtime/src/ui/tests/layout_slots
  - zircon_runtime/src/ui/tests/taffy_layout_pass
  - zircon_runtime/src/ui/tests/scroll_virtualization.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains/incremental_layout.rs
  - zircon_runtime/src/ui/tests/surface_node_pool.rs
  - zircon_runtime/src/ui/tests/runtime_ui_layout_routes.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/73-runtime-ui-style-theme-token-cascade-selector-pseudo-state-invalidation-transition-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Layout/SScrollBox.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Views/STableViewBase.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/Blueprint/UserWidgetPool.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Public/Blueprint/UserWidgetPool.h
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
  - dev/bevy/crates/bevy_ui/src/measurement.rs
  - dev/bevy/crates/bevy_ui_widgets/src/scrollarea.rs
  - dev/Fyrox/fyrox-ui/src/widget.rs
  - dev/Fyrox/fyrox-ui/src/grid.rs
  - dev/Fyrox/fyrox-ui/src/list_view.rs
  - dev/Fyrox/fyrox-ui/src/scroll_panel.rs
  - dev/Fyrox/fyrox-ui/src/scroll_viewer.rs
  - dev/godot/scene/gui/container.cpp
  - dev/godot/scene/gui/control.cpp
  - dev/godot/scene/gui/grid_container.cpp
  - dev/godot/scene/gui/scroll_container.cpp
  - dev/godot/scene/gui/tree.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugUI.Containers.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 76 · Runtime UI Layout、Box Model、Measure/Arrange、Flex/Grid、Overflow/Scroll、Virtualization、DPI 与 Product Integration 工程化差距

## 1. 结论

Zircon 当前并非没有 UI layout。公开接口已经定义 `AxisConstraint`、`BoxConstraints`、九类 container、slot、geometry、layout metrics、CSS-like `UiLayoutStyle`、backend capability/report、scroll state 与 virtual window；Runtime 有递归 measure/arrange、局部 dirty rebuild、Taffy bridge、Grid/Masonry/Wrap/Canvas/Overlay/ScrollableBox、text measure cache 和 node pool；Editor 与 WOC 也已经用这些类型承载真实窗口、资产编辑器、物品栏、角色创建和锁匠界面。上述类型、局部算法和 218 项聚焦测试可保留，不能退回硬编码 frame。

真正的工程级缺口是“公开 layout contract、编译资产、backend graph、measure protocol、scroll/virtual collection、DPI geometry 与产品 viewport”没有形成单一权威执行链。公共 `UiLayoutStyle` 能表达 percentage、MinMax Grid、absolute inset、justify/align、reverse flow 与双轴 overflow，Editor autolayout也能生成它；但实际 retained tree没有 `UiLayoutStyle` 字段，产品 pass仍从 `UiContainerKind + BoxConstraints` 临时合成一个缩水 style。Taffy graph又在每个容器每次 arrange时新建，只包含 parent和预先测量好的 leaf children，计算后立即丢弃。公开能力和实际产品语义因此不是同一件事。

当前所谓 virtualization仍先递归测量全部 materialized child，再为全部 child计算位置，最后只对窗口外子树清空 frame。`UiScrollableBox` 的固定 `item_extent` window、Table/Tree 的字符串 metadata reducer和 surface node pool是三套分离机制；它们没有 logical item provider、generation、materialize/recycle transaction或变量高度索引。DPI侧也只在 window state和render raster scale局部存在，dynamic product用 camera physical `UVec2`直接作为 layout root size，未建立 logical layout、physical render和pointer geometry barrier。

本轮登记 **3 项 Runtime76 独有 P0、48 项 P1、12 项 P2 与 48 项资格门**。Runtime11A继续拥有总体 UI service/tree/input/focus/window lifecycle，并继续拥有 P0-3 DPI事件未进入产品、P0-6 authored geometry无界分配和其既有通用 virtualization问题；Runtime73拥有 style/theme/cascade；Runtime74拥有 template/binding/reload；Runtime75拥有 component behavior与collection component state；Runtime11B/11C拥有 text shaping和GPU clip/render。Runtime76只拥有 layout schema到真实 measure/arrange/backend、scroll viewport和virtual collection geometry的闭环。

在 bounded layout admission、单一 compiled style、persistent backend graph、constraint-aware intrinsic measure、真实 data virtualization、logical/physical DPI闭环、incremental dependency graph与真实产品scale receipt全部通过前，不得把当前 backend report中的 `Native/Fallback` 当作能力资格，也不得宣称布局功能、性能或表现达到或超过当前 Unreal。

## 2. 审查边界与物理冻结

### 2.1 Owner 边界

| 领域 | Canonical owner | Runtime76 责任 | 不重复登记 |
|---|---|---|---|
| UI service/tree/input/window lifecycle | Runtime11A | layout graph消费sealed tree、geometry generation和window metrics | surface registry、输入dispatch、focus/IME、window event pump与总体tree安全 |
| Text shaping/layout/edit | Runtime11B | backend向text measurer提供known/available constraints并消费intrinsic result | font fallback、shaping、line break、selection、IME与glyph cache |
| GPU UI render/clip | Runtime11C | layout产生typed clip/scroll viewport与logical geometry | GPU batch、atlas、SDF、stencil/scissor和submit |
| Style/theme/cascade | Runtime73 | 消费唯一computed layout style及其generation | selector、cascade、theme、token、pseudo-state与transition scheduler |
| Template/binding/reload | Runtime74 | layout schema canonicalize、compiled artifact和reload layout migration | expression/model/event/command与整体reload transaction |
| Component/collection behavior | Runtime75 | collection provider与viewport/layout transaction的typed接口 | control reducer、selection、component event与per-component a11y |
| Dynamic product session | Runtime43 | logical viewport、surface identity、layout generation的消费点 | FFI/session lifetime、frame/event和host request总链 |
| Editor authoring | Editor01/23 | Editor必须预览同一compiled layout artifact并展示预算/错误 | document UX、undo/save/cook、native window host与palette |

本篇不把 Runtime11A P0-3 的“窗口 DPI/销毁事件未进入产品”重新计数；本篇只展开事件进入后仍缺失的 logical-layout/physical-render契约。本篇也不把 Runtime11A P0-6 的 hit-grid无界分配重复计数；`RUL-P0-001`是另一个可由 `.zui` track count/span直接触发的独立分配和整数溢出路径。

### 2.2 Zircon 物理冻结

本轮核心冻结 107 个 production/cross-product Rust 文件、20,378 行、702,381 bytes，manifest fingerprint 为 `851ff50c4e5e575057e547339c18dae1f9f4d3c03527b4fc00f216c22ce2c2a3`；聚焦 54 个测试文件、13,783 行、468,773 bytes，fingerprint 为 `c4bba75f0d627f5b09a81423084f298c3d4d822f7c98f3981ed5d14f2184910b`。算法为对排序后的 `relative-path=per-file-SHA-256` 以 LF 连接、末尾不附加 LF，再做 SHA-256。结论绑定当前共享 working copy，而不只绑定 baseline HEAD；实施前必须重取指纹。

| 范围 | 文件 / 行 / bytes | fingerprint / 本轮证据 |
|---|---:|---|
| Public layout contracts | 12 / 1,855 / 52,728 | `d3193c73625c18cb8e0afae62a9e920bf3b0a252166d376d2228aeeb1e0c1da8`；constraints、style、engine、geometry、metrics、scroll、slot与cache |
| Runtime layout execution | 46 / 11,923 / 416,858 | `d48fa1b58cfd0d7dc52cd9b72dc94e02900191d71456e54971f7c96869550a6a`；parser、measure/arrange、Taffy、incremental、scroll、pool、window与dynamic product |
| Editor product Rust consumers | 49 / 6,600 / 232,795 | `d1eb395909b07f0560a2d6b945121d415de5a2b5ba88ce067454270a6b2dcc5c`；CSS-like autolayout、virtual rows、view materialization与workbench layout |
| 聚焦测试 | 54 / 13,783 / 468,773 | `c4bba75f0d627f5b09a81423084f298c3d4d822f7c98f3981ed5d14f2184910b`；218项test、2项ignored |
| 真实产品 `.zui` | 48 / 14,910 / 854,682 | `795c8a1238a205cbba62b54e6119989040907f0e6b2e5eebea97b58dedabc444`；36份Editor、12份WOC布局消费者 |

聚焦测试含 6 次 `f32::INFINITY`、3 次 `f32::NAN`、7 次 `scale_factor` 和 8 次 `item_extent`，说明局部桥接极值与window metrics已有测试；但没有任何 variable-height/heterogeneous-extent命中，也没有巨大 `columns/rows/span`、`UiLayoutStyle`到live surface、logical size到dynamic product `compute_layout`闭环测试。48份产品资产中至少7份直接使用 `GridBox`、3份直接使用 `ScrollableBox`、5份声明virtualization/item extent；Editor的 `MaterialMasonry` 和 MUI X DataGrid又通过component metadata走另一条布局/窗口语义。

本轮只做 review，不修改 production/tests/assets，不运行 Cargo、Editor、WOC、真实窗口、fuzz、fault、soak或benchmark。共享工作树有其他 Session 修改；本报告因此保持 `source_recheck_required: true`。

### 2.3 参考物理冻结

参考侧冻结 21 个文件、30,794 行、1,095,023 bytes，manifest fingerprint 为 `7ba80d528f7a1096c07e45cd672ba3871878e6b3426cf657efae53b2d208c51a`。

| 参考 | 文件 / 行 / bytes | 采用的工程事实 | 对 Zircon 的约束 |
|---|---:|---|---|
| Unreal Slate/UMG | 6 / 9,153 / 316,882 | `SlatePrepass/CacheDesiredSize/ArrangeChildren`与invalidate reason闭合；`SScrollBox`有惯性、overscroll、drag/wheel/analog；`SListView`按items source生成/释放视窗行并配合widget pool | layout invalidation、prepass、scroll physics、data source、row generation/release与pool必须是同一生命周期 |
| Bevy UI | 3 / 874 / 32,441 | `UiSurface`持久保存entity-to-Taffy映射与Taffy tree，upsert style/context、更新children，再以known/available space调用typed `NodeMeasure` | 不得每个container每次pass临时建浅树；intrinsic measure必须由backend constraint驱动 |
| Fyrox UI | 5 / 4,518 / 170,938 | Widget、Grid和ScrollPanel实现真正measure/arrange override；Grid区分Strict/Auto/Stretch并多阶段分配；ScrollPanel支持双轴和bring-into-view | measure/arrange、track sizing、双轴scroll和focus reveal需要稳定control合同 |
| Godot Control | 5 / 14,861 / 523,222 | Container监听minimum-size/size-flags并deferred `queue_sort`；Grid处理min/max/expand/RTL/剩余像素；ScrollContainer把scrollbar reserve、双轴模式、deadzone和ensure-visible纳入布局 | 最小/最大尺寸传播、deferred coalescing、RTL、rounding和scrollbar占位不能是metadata装饰 |
| Unity Graphics | 2 / 1,388 / 51,540 | 本地仓库只有SRP DebugUI的observable children、panel dirty、query path和table row shape检查 | 仅借鉴DebugUI层级失效和容器不变量；该仓库不是UI Toolkit layout/virtualization源码，不作为通用布局parity证据 |

参考引擎不是复制清单。Zircon可以保留Taffy和Rust数据导向设计，但必须达到相同级别的权威graph、constraint propagation、invalidation、scroll/collection lifecycle和可验证产品行为。

## 3. 当前可保留底座

| 底座 | 当前价值 | 重构要求 |
|---|---|---|
| `AxisConstraint/BoxConstraints` | 已有min/preferred/max/stretch/weight与resolve测试 | 升级为finite、unit-aware、source-bound compiled constraint，禁止lossy parse和NaN传播 |
| `UiLayoutStyle` | 已覆盖Flex/Grid/Block、percentage、MinMax、absolute、overflow | 变成retained node唯一computed layout输入，不再只作Editor DTO/Taffy adapter测试 |
| `UiLayoutEngineRequest/SelectionReport` | 已能记录requested/selected backend、fallback reason与tree stats | capability必须可证且fallback重新完整校验；report绑定artifact/backend generation |
| Taffy bridge | Flex/Grid/Wrap/Block局部映射和失败回退测试真实存在 | 改为persistent full graph、typed measure context和dirty update，不再per-container shallow rebuild |
| measure/arrange pass | container分流、slot order、clip、content size与desired size已经可运行 | 建立available-space依赖、intrinsic protocol、cycle/depth/budget和incremental cache |
| layout dirty/rebuild | 有structured dirty flags、layout boundary、partial rebuild report | 变为依赖图和generation transaction，不能仅靠祖先递归重测或序列化derived cache |
| scroll state/window | offset、viewport/content extent、overscan与visible-range dirty已有类型 | 合并为双轴scroll viewport与logical collection window，不再按materialized child index裁frame |
| node pool | detach/reuse/report与transient focus reset已有基础 | 加预算、qualified reuse key、完整state/cache reset并接入virtual provider lifecycle |
| metrics/geometry | logical frame、pixel snapping、layout/render transform已有类型 | 贯穿window metrics、layout、hit、render和accessibility generation |
| 产品资产与测试 | Editor/WOC真实使用Grid/Scroll/virtual list，聚焦测试数量可观 | 升级为同artifact、同backend、同viewport contract的golden/scale/product qualification |

## 4. 三项新增 P0

### RUL-P0-001 · `.zui` track count与slot span可直接决定无界分配，并存在未检查整数加法

`template/build/parsers.rs` 的 `parse_usize`只拒绝负数，不限制 `columns/rows/overscan/span`。随后 fallback measure/arrange执行 `vec![0.0; columns]`、`vec![0.0; rows]`和Masonry多数组分配；Taffy bridge执行 `vec![fr(1.0); columns/rows]`。`placement.column + column_span`、`row + row_span`和Taffy grid line也使用普通 `usize`加法。内容资产可因此触发OOM、debug/release差异溢出、panic或极长pass。

必须在compile/import admission建立每surface node/track/span/depth/temporary-bytes预算，所有推导用checked/saturating policy并返回带source span的typed diagnostic；backend allocation前仍需二次budget guard。不得依赖正常产品当前只有2到16列，也不得以Runtime11A的hit-grid修复替代本路径。

### RUL-P0-002 · 公共 `UiLayoutStyle` 与真实 retained layout tree断链，编译和Editor可接受的语义在产品运行时静默消失

`UiLayoutStyle`公开承诺percent 0..=1、MinMax tracks、absolute inset、justify/align、reverse flow和双轴overflow；Editor `css_like_constraint`能生成这些字段，`taffy_style_from_ui_layout_style`测试也能映射它们。但 `UiTreeNode`产品布局不保存/消费该style，`taffy_bridge`只由 `UiContainerKind + BoxConstraints`重新构造缩水style；Grid再被强制生成相同`fr(1)` tracks。`row-reverse`在responsive MUI中甚至映射为普通HorizontalBox。

这不是缺少若干CSS特性，而是公开ABI与产品执行语义分裂。必须由style/compiler owner产生versioned `CompiledUiLayoutStyle`，retained node持有qualified handle/generation，所有backend消费同一style；无法表达的字段应在compile时失败或显式fallback，不能只在Editor预览/单元测试成功。

### RUL-P0-003 · Backend selection只校验preferred完整能力，却把未满足request的fallback标为有效

`UiLayoutEngineSelection::select`对preferred调用`unsupported_reason`，切换fallback后却只检查`supports_family`，不再检查`needs_content_measure`与`needs_dpi_scaling`。同时 `UiLayoutEngineCapability::zircon()`无条件宣称Flex/Grid/Block/Wrap/VirtualizedList、content measure和DPI全部支持，而实际fallback是局部手写算法，DPI又未进入product root size。report因而可能把不满足请求的执行标成`Fallback`或`Native`。

必须让capability由实际backend version与feature matrix生成，对preferred/fallback使用同一完整admission；selection绑定compiled artifact requirements，并以conformance corpus证明每个family/feature。未资格backend必须返回Unsupported/Rejected，不能通过名称为Zircon就默认全能力。

## 5. P1 工程化重构清单

### 5.1 Schema、编译与资源门禁

| ID | 必须重构的内容 |
|---|---|
| RUL-P1-001 | 建立唯一 `CompiledUiLayoutStyle`，合并container/constraints、cascade style、slot和responsive override |
| RUL-P1-002 | 定义logical px、percent、fraction、auto、min/max-content等typed unit，禁止裸f32跨层猜单位 |
| RUL-P1-003 | 对所有尺寸、gap、weight、offset、scale验证finite与合法范围；NaN/Inf不得进入tree |
| RUL-P1-004 | 落实percentage 0..=1合同，超范围必须diagnostic；不得只做non-negative检查 |
| RUL-P1-005 | 明确negative margin、negative scale和overflow extent政策，合法语义与非法输入分开 |
| RUL-P1-006 | 对node/children/track/span/overscan/temporary bytes建立per-artifact和per-surface预算 |
| RUL-P1-007 | 所有track/span/index/bytes推导使用checked arithmetic并返回稳定错误码/source span |
| RUL-P1-008 | compiled artifact记录layout schema version、style/theme generation、backend requirements与hash |
| RUL-P1-009 | slot placement验证parent family、line/span范围、重叠政策和auto-placement，不在arrange时clamp掩盖 |
| RUL-P1-010 | 定义显式layout root/containing block；多个roots不得默认叠在同一full viewport frame |
| RUL-P1-011 | pass stage变为release可验证的typed state machine，不能只依赖`debug_assert`与名称列表 |
| RUL-P1-012 | 删除按component字符串增加Button/Material常量尺寸的leaf measure，改为resolved style/painter metrics合同 |

### 5.2 Measure、Arrange 与 Backend Graph

| ID | 必须重构的内容 |
|---|---|
| RUL-P1-013 | 每个surface持久拥有backend graph与node mapping，style/children/context增量upsert/remove |
| RUL-P1-014 | backend node id携带surface/node generation，防止pool/reload后旧layout结果写回新实例 |
| RUL-P1-015 | intrinsic measure接收known width/height、available space、min/max-content mode与DPI context |
| RUL-P1-016 | 文本、图片、自定义控件实现typed `MeasureProvider`，backend不能只消费预先算好的leaf desired size |
| RUL-P1-017 | 文本必须在Flex/Grid实际分配宽度变化时重新measure wrapping，而不是只在Fixed preferred width包装 |
| RUL-P1-018 | 建立baseline、first/last baseline、min-content/max-content和aspect ratio的一致传播 |
| RUL-P1-019 | Taffy必须计算完整嵌套graph；禁止per-container创建parent+leaf浅树后立即drop |
| RUL-P1-020 | Zircon fallback与Taffy对共同能力运行differential corpus，差异必须有显式owned semantics |
| RUL-P1-021 | Grid实现显式tracks、Auto/Fr/Percent/MinMax、span contribution、auto-placement、RTL与deterministic rounding |
| RUL-P1-022 | Flex实现row/column reverse、order、wrap-reverse、justify/align-content/self、baseline与shrink min-size语义 |
| RUL-P1-023 | Block/Canvas/Overlay建立containing block、absolute inset、z/order与out-of-flow measure规则 |
| RUL-P1-024 | layout输出保留logical unsnapped geometry，render/hit/a11y按同generation派生physical/snapped geometry |

### 5.3 Overflow、Scroll 与 Virtual Collection

| ID | 必须重构的内容 |
|---|---|
| RUL-P1-025 | Scroll viewport支持X/Y独立overflow、双轴offset/content extent与axis policy |
| RUL-P1-026 | `Auto/Always/Never/Reserve/Overlay` scrollbar进入measure/arrange，不能只解析后不消费 |
| RUL-P1-027 | 建立nested scroll chain、wheel/touch/analog handoff、remaining delta和boundary propagation |
| RUL-P1-028 | 建立惯性、overscroll、drag threshold、cancel/interrupt和reduced-motion策略 |
| RUL-P1-029 | 提供typed ensure-visible/bring-into-view、focus reveal、alignment与animation receipt |
| RUL-P1-030 | 内容插入/删除/resize时提供scroll anchoring和stable item identity，避免viewport跳动 |
| RUL-P1-031 | `VirtualizedCollection`必须消费logical item provider/count/key，不以materialized `children.len()`冒充数据集 |
| RUL-P1-032 | 支持变量高度/宽度的measured extent cache、prefix index和anchor correction，固定extent只是快路径 |
| RUL-P1-033 | measure/arrange/build/render只materialize visible+overscan窗口，scale成本与logical count解耦 |
| RUL-P1-034 | node pool设总量/bytes/idle-generation预算，qualified reuse key不含动态path，并完整清理state/cache/subscription |
| RUL-P1-035 | 合并ScrollableBox window、Table/Tree metadata reducer和node pool为单一viewport transaction与receipt |
| RUL-P1-036 | selection/focus/a11y支持未materialize item的logical identity、set size/position和按需realize |

### 5.4 Incremental、Responsive、Product 与证据

| ID | 必须重构的内容 |
|---|---|
| RUL-P1-037 | 建立measure/arrange/style/content/viewport依赖图；dirty leaf只重算受影响的祖先与后代 |
| RUL-P1-038 | derived `UiLayoutCache`不作为可恢复truth序列化；cache key包含artifact/style/text/font/DPI/backend generation |
| RUL-P1-039 | root resize、style/theme reload、font generation、content change和scroll只触发精确失效域 |
| RUL-P1-040 | 多root、popup、overlay、embedded target与split viewport使用显式composition layout，不默认重叠 |
| RUL-P1-041 | layout/render transform升级为origin+matrix/rotation/skew能力并定义bounds、hit和clip映射 |
| RUL-P1-042 | layout clip产生typed rect/rounded/path/transform viewport；GPU实现仍由Runtime11C拥有 |
| RUL-P1-043 | arranged tree和layout report建立node-id index，公共查询与merge不得反复线性扫描 |
| RUL-P1-044 | responsive/media query编译为typed rule与dependency index，不每pass全树扫描TOML metadata |
| RUL-P1-045 | MUI spacing、breakpoint、direction/reverse、display与unit转换必须有单一adapter和parity测试 |
| RUL-P1-046 | dynamic product以window logical size布局、physical size渲染，并在resize/DPI后发布geometry barrier |
| RUL-P1-047 | 记录visited/measured/arranged/backend-updated/materialized/pool bytes与budget rejection typed metrics |
| RUL-P1-048 | Editor/WOC真实窗口运行同compiled artifact/backend，以golden、fault和scale receipt关闭产品资格 |

## 6. P2 收敛项

| ID | 收敛项 |
|---|---|
| RUL-P2-001 | 将`Native/Fallback`展示名改为可追溯backend id/version/capability tier，避免诊断误导 |
| RUL-P2-002 | 合并重复的virtual-window helper和18个metadata alias，保留一个typed schema adapter |
| RUL-P2-003 | 把`ZR_UI_LAYOUT_PROFILE`与`eprintln!`迁入统一profiling/diagnostic sink |
| RUL-P2-004 | 为measure/arrange/Taffy update复用surface scratch arena，减少逐容器Vec分配 |
| RUL-P2-005 | 避免递归measure复制每个node的children Vec；使用sealed snapshot slice/arena traversal |
| RUL-P2-006 | selection report与arranged output使用stable index，移除replace/lookup线性扫描 |
| RUL-P2-007 | layout diagnostic采用稳定code、source path/span、node id和deterministic排序 |
| RUL-P2-008 | 为Grid/Flex/scroll生成最小化golden corpus并保存backend differential seed |
| RUL-P2-009 | 提供Editor layout inspector显示constraint、intrinsic size、track、clip、scroll window和dirty原因 |
| RUL-P2-010 | 对reference citation记录snapshot/hash/适用claim，source drift时自动标记recheck |
| RUL-P2-011 | 清理“virtual/native/full support”等超过当前事实的注释、测试名和catalog文案 |
| RUL-P2-012 | 建立共享layout workload catalog，benchmark只引用versioned dataset而不复制临时fixture |

## 7. 当前差距矩阵

| 能力 | 当前实现 | 工程级目标 | 状态 |
|---|---|---|---|
| Layout style authority | 公共style、container/constraints、responsive metadata三套输入 | 单一compiled style+generation | 阻断 |
| Backend graph | 每容器每次pass临时parent+leaf Taffy tree | persistent full surface graph | 阻断 |
| Intrinsic measure | 先递归算desired，再作为leaf固定尺寸喂Taffy | known/available-space typed measure callback | 不完整 |
| Flex | 基本横/纵/wrap与gap | reverse/order/baseline/min-content/完整alignment | 不完整 |
| Grid | 等分cell/`fr(1)` tracks | typed tracks、span contribution、auto placement、RTL | 不完整 |
| Overflow/scroll | 单轴offset、frame clip、固定window | 双轴、scrollbar layout、chain、physics、anchor | 不完整 |
| Virtualization | 全树materialize+measure，只隐藏窗口外frame | data provider驱动的bounded materialization/recycle | 假能力 |
| Node pool | 无界bucket、动态path key、保留cache/部分state | budgeted qualified pool+generation reset | 不完整 |
| Incremental | dirty flags和部分subtree skip | dependency graph+generation cache | 不完整 |
| Responsive | 全树扫描TOML、硬编码MUI unit/breakpoint | compiled typed responsive rules | 不完整 |
| DPI | metrics局部存在，dynamic product用physical viewport布局 | logical layout/physical render/hit barrier | 阻断，继承Runtime11A P0-3 |
| Product evidence | 单元测试与静态ZUI | real window golden/scale/fault/differential | 缺失 |

## 8. 目标架构

### 8.1 `CompiledUiLayoutArtifact`

Template、style和component compiler共同产出只读artifact：node topology、qualified style handle、slot、typed units、responsive rule、backend requirement、source span、resource budget和schema/style generation。运行时不得重新解析TOML或按component字符串猜layout。artifact admission先验证finite、cycle/depth、track/span、budget和backend capability，再允许mount。

### 8.2 `UiLayoutSurface`

每个surface持有generational node arena、persistent backend graph、measure provider table、dirty dependency graph、logical geometry store、scroll viewport store和bounded scratch/cache。insert/remove/reparent/style/content/DPI变化作为layout transaction提交；backend node mapping和结果都携带surface generation，旧pass不能写回新tree。

### 8.3 Measure/Arrange Protocol

backend以known dimensions、available space、intrinsic mode、resolved style、DPI/layout scale调用typed provider；Text/Image/Custom返回带dependency key的intrinsic result。measure产生desired/min/max/baseline，arrange产生logical frame/clip/scroll viewport；pixel snapping和physical transform是后续派生，不回写logical truth。

### 8.4 Scroll 与 Virtualized Collection

ScrollViewport独立维护双轴offset、extent、scrollbar、chain、physics和anchor。VirtualizedCollection提供logical count/key、estimate/measure extent、materialize/update/recycle和selection/a11y identity；窗口计算支持固定extent快路和变量extent索引。materialization transaction与node pool、component reducer、layout和render同代提交。

### 8.5 Product Geometry Barrier

Window owner发布 `UiWindowMetricsGeneration`，layout消费logical size，render消费physical size/scale，pointer从physical映射到同代logical geometry。resize/DPI/camera target变化后，旧geometry不可继续接受input或publication；dynamic Runtime、Editor和embedded host都适配同一barrier。

## 9. 分层实施里程碑

### M0 · Admission 与 capability硬门

关闭三项P0：typed/finite/bounded schema、checked allocation、单一compiled style、fallback完整校验和真实capability matrix。先阻止坏资产进入运行时，再迁移算法。

### M1 · Persistent layout graph

建立`UiLayoutSurface`、generational node mapping、full nested Taffy graph和增量style/children/context update；旧per-container shallow bridge硬切删除，不保留双执行路径。

### M2 · Constraint-aware measure/arrange

接入Text/Image/Custom measure provider、known/available space、intrinsic/baseline/min-content；完成Flex/Grid/Block/Canvas/Overlay conformance和differential corpus。

### M3 · Scroll viewport

实现双轴overflow、scrollbar reserve/overlay、chain、bring-into-view、inertial/overscroll、anchor和typed receipt；layout与input owner按同代geometry交接。

### M4 · Data virtualization

引入logical provider/key、variable extent index、bounded materialization、qualified node pool和logical focus/selection/a11y；合并Table/Tree/ScrollableBox三套window语义。

### M5 · Incremental、responsive 与 DPI

建立dependency cache、compiled responsive rules和logical/physical geometry barrier；关闭root resize、theme/font/DPI、reload与scroll的精确失效矩阵。

### M6 · 产品迁移与资格

Editor component lab、UI asset editor、workbench virtual rows以及WOC inventory/character/lockpick使用同一artifact/backend；运行real window golden、scale、fault、soak和performance receipt。

## 10. 资格门

### 10.1 Schema、admission 与 backend门

| Gate | 必须证明 |
|---|---|
| RUL-GATE-001 | NaN/Inf/negative-illegal尺寸在compile阻断并报告source span |
| RUL-GATE-002 | 巨大columns/rows/span/overscan在allocation前被budget拒绝 |
| RUL-GATE-003 | 所有index/track/bytes乘加使用checked policy，无debug/release差异 |
| RUL-GATE-004 | percentage合同与合法negative margin policy有正反测试 |
| RUL-GATE-005 | compiled artifact携带schema/style/backend requirement hash |
| RUL-GATE-006 | public `UiLayoutStyle`每个accepted字段在live surface可观察 |
| RUL-GATE-007 | unsupported style字段compile失败或显式qualified fallback |
| RUL-GATE-008 | preferred和fallback使用相同完整`unsupported_reason`校验 |
| RUL-GATE-009 | capability matrix由backend build/version产生，不由默认构造宣称 |
| RUL-GATE-010 | backend selection report绑定node/artifact/surface generation |
| RUL-GATE-011 | 多root containing block/composition规则确定且不默认重叠 |
| RUL-GATE-012 | release build pass stage顺序和状态转换可验证 |

### 10.2 Measure、Flex、Grid 与 geometry门

| Gate | 必须证明 |
|---|---|
| RUL-GATE-013 | backend graph跨frame持久，单style change不重建无关container tree |
| RUL-GATE-014 | nested Flex/Grid通过full graph计算，不是parent+leaf浅树 |
| RUL-GATE-015 | text measure收到known/available width并在wrap width变化时重排 |
| RUL-GATE-016 | image/custom measure provider与artifact generation一致 |
| RUL-GATE-017 | min/max-content、aspect ratio和baseline conformance通过 |
| RUL-GATE-018 | Flex reverse/order/wrap/alignment/baseline矩阵通过 |
| RUL-GATE-019 | Grid Auto/Fr/Percent/MinMax tracks和span contribution通过 |
| RUL-GATE-020 | Grid auto-placement、RTL和剩余pixel分配deterministic |
| RUL-GATE-021 | Block/absolute/Canvas/Overlay containing block矩阵通过 |
| RUL-GATE-022 | Zircon/Taffy共同能力differential无未解释差异 |
| RUL-GATE-023 | logical frame不因pixel snapping被覆盖 |
| RUL-GATE-024 | transform/clip/hit/render/a11y消费同一geometry generation |

### 10.3 Scroll、virtualization 与 pool门

| Gate | 必须证明 |
|---|---|
| RUL-GATE-025 | 双轴overflow和independent scroll offset通过 |
| RUL-GATE-026 | scrollbar Auto/Always/Never/Reserve/Overlay改变正确measure/arrange |
| RUL-GATE-027 | nested wheel/touch/analog chain保留remaining delta |
| RUL-GATE-028 | inertial/overscroll/drag可中断且reduced-motion可降级 |
| RUL-GATE-029 | ensure-visible/focus reveal在nested scroll中到达目标 |
| RUL-GATE-030 | prepend/remove/resize保持stable anchor和offset correction |
| RUL-GATE-031 | 100万logical rows只materialize bounded visible+overscan nodes |
| RUL-GATE-032 | variable-height rows滚动窗口与实际arranged frames一致 |
| RUL-GATE-033 | collapsed/hidden item不破坏logical index和extent索引 |
| RUL-GATE-034 | node pool达到count/bytes预算后deterministic evict/reject |
| RUL-GATE-035 | reused node不保留旧hover/selected/focus/cache/subscription |
| RUL-GATE-036 | logical未materialize item仍有正确selection/focus/a11y identity |

### 10.4 Incremental、DPI、产品与性能门

| Gate | 必须证明 |
|---|---|
| RUL-GATE-037 | single leaf size change只访问dependency closure |
| RUL-GATE-038 | stable frame rebuild访问量与tree size解耦 |
| RUL-GATE-039 | serialized/restored surface不复用无generation的derived geometry |
| RUL-GATE-040 | responsive breakpoint只重算受规则影响nodes |
| RUL-GATE-041 | MUI row-reverse和numeric/string spacing语义一致 |
| RUL-GATE-042 | 1x/1.25x/1.5x/2x DPI下logical layout、physical render和pointer命中一致 |
| RUL-GATE-043 | resize/scale后旧geometry被barrier拒绝，不接受stale input |
| RUL-GATE-044 | Editor与dynamic Runtime使用同一compiled artifact/backend hash |
| RUL-GATE-045 | WOC inventory/character/lockpick真实产品golden通过 |
| RUL-GATE-046 | Grid/Flex/scroll/virtual workload有p50/p95/max/alloc/visited receipt |
| RUL-GATE-047 | malformed/over-budget资产fault test无panic、OOM或partial publication |
| RUL-GATE-048 | parity/performance声明绑定硬件、build、workload、reference snapshot与原始证据 |

## 11. 测试与证据缺口

| 缺口 | 当前证据 | 必须补齐 |
|---|---|---|
| Layout admission safety | 有部分Inf/NaN bridge测试，无巨大track/span | compile/runtime budget property test、OOM前拒绝与checked arithmetic |
| Public style live semantics | style mapping单测与Editor parser测试 | artifact到surface到frame的每字段conformance |
| Persistent graph | Taffy tree stats只证明临时浅树构建 | multi-frame node identity、增量update和full nested graph测试 |
| Intrinsic text | 有wrapping与text cache测试 | known/available constraint、min/max-content、baseline、font generation矩阵 |
| Grid/Flex | happy-path slot和fallback测试 | WPT-like differential、reverse/RTL/MinMax/span/rounding corpus |
| Scroll | wheel/scrollbar/固定extent窗口测试 | 双轴、nested chain、physics、anchor、bring-into-view、DPI input |
| Virtualization | metadata transaction数量可bounded | logical provider、bounded node count/measure count、变量extent、recycle state |
| Incremental | non-auto parent scale matrix有2项ignored | auto/nested/backend/DPI/style/font/virtual dependency workload，required lane |
| DPI | window pump与raster scale局部测试 | dynamic product logical root size、geometry barrier、real monitor move |
| 产品 | 48份真实ZUI静态存在 | Editor/WOC real window screenshot、hit/a11y、fault、soak、scale和performance |

静态源码审查不能证明真实性能优于Unreal。后续性能对照必须至少固定硬件、OS、release/LTO配置、DPI、viewport、tree形状、dirty比例、文本/图片混合、logical item count、input trace和采样窗口，并同时保存visited nodes、materialized nodes、backend updates、allocation bytes、frame p50/p95/max及视觉/命中正确性。仅比较单个`compute_layout`微基准或测试transaction count不具备产品资格。

## 12. Owner 与状态

- Review owner：`zircon_runtime` UI layout；公共layout DTO由`zircon_runtime_interface`承载，不能在Interface另建执行owner。
- Parent owners：Runtime11A负责UI service/tree/input/window lifecycle；Runtime11B负责text shaping/layout；Runtime11C负责GPU clip/render；Runtime73/74/75负责style、template/binding/reload和component behavior。
- Product owners：Runtime43负责dynamic session接线；Editor01/23负责authoring/preview；WOC产品owner负责真实HUD/window acceptance。
- 当前状态：`review_complete`，`implementation_status: pending`，`source_recheck_required: true`。
- 本轮未修改Rust、测试或资产，未运行Cargo和动态产品验证；所有性能、parity与稳定性声明仍为未资格。

Runtime76关闭的是“当前源码被逐项审查并形成可执行重构合同”，不是“布局系统已经完成”。实施必须按M0到M6硬切收敛，不得长期并存public style、container fallback、responsive metadata和component-specific layout四套authority，也不得以更多happy-path测试掩盖无界分配、假capability或全树假virtualization。
