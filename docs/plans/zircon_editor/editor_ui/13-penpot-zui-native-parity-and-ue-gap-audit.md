---
related_code:
  - zircon_app/src/entry/runtime_entry_app/host_requests/ui_action.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/hud.rs
  - zircon_runtime/src/dynamic_api/session/menu.rs
  - zircon_runtime/src/ui/surface/input/window_pump.rs
  - zircon_runtime/src/ui/template/asset/binding/validation.rs
  - zircon_runtime/src/ui/platform_input/winit_translation.rs
  - zircon_runtime/src/ui/event_ui/manager/invocation.rs
  - zircon_runtime/src/ui/surface/navigation_index/semantics.rs
  - zircon_runtime/src/ui/dispatch/input_manager/routing.rs
  - zircon_runtime/src/text/font/shared.rs
  - zircon_runtime/assets/fonts/editor-ui.font.toml
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/screen_space_ui.wgsl
  - zircon_runtime/src/ui/icon_atlas/svg.rs
  - zircon_runtime/src/ui/icon_atlas/atlas.rs
  - zircon_runtime_interface/src/ui/surface/render/brush.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime/src/ui/layout/pass/clip.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/compute.rs
  - zircon_runtime/src/ui/layout/style_mapping.rs
  - zircon_runtime/src/ui/layout/pass/responsive_mui.rs
  - zircon_runtime/src/ui/v2/surface_tree/layout.rs
  - zircon_runtime/src/ui/v2/surface_tree/parse.rs
  - zircon_runtime/src/ui/template/build/slot_contract.rs
  - zircon_runtime/src/ui/surface/invalidation.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/core/framework/input/input_action_map.rs
  - zircon_runtime/tests/zui_penpot_bridge_contract.rs
  - zircon_runtime/tests/zui_native_visual_acceptance/preview.rs
  - zircon_runtime/tests/fixtures/ui/penpot_roundtrip.zui
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/bridge/zui-prefab-system.ts
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/bridge/penpot-projection.ts
  - dev/penpot/plugins/apps/zircon-zui-plugin/tools/zui-layout-semantic-parity.ts
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/FastUpdate/SlateInvalidationRoot.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/InvalidateWidgetReason.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/HittestGrid.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/ElementBatcher.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElements.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/Clipping.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/UnrealEngine/Engine/Shaders/Private/SlateShaderCommon.ush
  - dev/bevy/crates/bevy_ui/src/layout/convert.rs
plan_sources:
  - "user: 2026-10-04 将 Penpot 网页对 .zui 的渲染一模一样实现到 zircon app，修复渲染、交互、布局问题，并参照 dev/UnrealEngine 检查 UI 漏洞"
  - docs/plans/designment/02-milestone-execution-and-evidence.md
  - docs/plans/zircon_editor/editor_ui/2026-09-02-penpot-slate-zui-convergence.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
  - docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
doc_type: plan
status: planned
last_refined: 2026-10-04
---

# 计划 13：Penpot ↔ Zircon App `.zui` 原生一致性与 UE Slate 漏洞审计

## 1. 目标与边界

目标：同一份 `.zui` 在 Penpot 网页投影与 `zircon_app` 原生窗口中，结构、几何、文本、颜色和交互结果一致。“一模一样”的验收口径固定为：**结构差异为零**（节点集合、父子、顺序、frame、clip、文本行框误差 ≤1 逻辑像素），之后才比较像素，像素差只允许 AA 级。截图不能替代结构证据。

边界（继承 designment 02 与 editor_ui index）：

- `.zui` v2 是唯一事实源；Penpot 只是可逆 authoring 投影，不新增第二套 schema。
- 不新增 crate；契约 DTO 只进 `zircon_runtime_interface::ui`，行为留在 `zircon_runtime::ui`。
- 硬切换：新 owner 落地即删除旧路径，不留兼容桥。
- 不越过 MVP F0–F4 gate 宣称 A2-P 或产品完成。
- 验证节奏遵 `docs/plans/milestone-validation-policy.md`：切片只做格式、结构和聚焦检查，里程碑末批量 Cargo。

## 2. 现状核实（2026-10-04 实读）

探索阶段有三处误判，已按源码更正，避免重复建设：

| 原判断 | 源码事实 | 本计划处理 |
| --- | --- | --- |
| 没有空间命中网格 | `ui/tree/hit_test.rs` 已有 `UiHitTestGrid`，每轴 ≤128 cell，单项 ≤4096 cell | 只补 viewport 索引域、查询预算、cursor radius |
| `UiInvalidationGraph` 未实现 | 已实现，但只是资产编译缓存级分类器（`ui/template/asset/invalidation/graph.rs`） | 新增 surface 级 fast/slow path，不动资产失效图 |
| UI 动作层不存在 | `core/framework/input/input_action_map.rs` 已有 action/context/binding；App 侧 `ui_action.rs` 只做采样日志 | 补“UI action → InputAction / 内置命令”桥 |

硬前提：`zircon_runtime` lib 当前不能干净编译。A2-C 证据记录 246 个共享编译错误；2026-10-01 另有三份 Editor 构建 failure（render/01 execution packet、optimize/11a session timer visibility、astra plugins/02 trust admission）。`docs/layout` 双端目录的 Penpot 侧有截图，**引擎侧捕获为 0**。在此之前任何“一致性”都无法被测量。

已存在、可直接复用的资产：

- Penpot 插件 `dev/penpot/plugins/apps/zircon-zui-plugin`：parser、projection、reconcile、CLI、SES host contract、浏览器契约，以及 semantic/text parity 比较器（`tools/zui-layout-semantic-parity.ts`、`tools/zui-layout-text-parity.ts`）。
- 引擎侧 `zircon_runtime/tests/zui_penpot_bridge_contract.rs` 与 `zui_native_visual_acceptance/`（28 个 review state、DPI、evidence 字段）。
- Editor 侧 `retained_host/ui/pane_data_conversion/zui_visual_acceptance/`（geometry、painter receipts、readiness）。
- 容器 padding：`nodes.<id>.layout.padding` 已经进入 `layout_padding`，并在 `layout/pass/arrange.rs` 的 `inset_frame` 消费。designment A2 文档中“Runtime 不消费容器 padding”的结论已过期，需要在 parity harness 上验证并回写。

## 3. 问题诊断

### 3.1 渲染不一致

| ID | 根因 | 证据 | 级别 |
| --- | --- | --- | --- |
| R1 | 运行时默认字体是等宽 `Fira Mono`，Penpot 用比例字体测量，换行与宽度全部漂移 | `text/font/shared.rs:19` | P0 |
| R2 | 屏幕空间 UI 渲染器按资源类别拆成多组数组提交，不消费 `UiBatchPlan::ordered_element_indices`，重叠层级被打乱 | `graphics/scene/scene_renderer/ui/render.rs`；11c P0-1 | P0 |
| R3 | 语义图标没有生产渲染器，画成实心矩形；`svg.rs` 只提取 path 且属性按子串匹配；图集槽位可越界 | `ui/icon_atlas/svg.rs:179`、`atlas.rs:134`；11c P0-2 | P0 |
| R4 | Penpot 侧在 TS 中重做组件展开与基础样式（硬编码 `PENPOT_PALETTE`、注入 prefab stylesheet），引擎侧用 Rust instancer 和 style resolver，两套语义必然漂移 | `zui-prefab-system.ts` | P0（架构） |
| R5 | 动态会话 HUD 和菜单的 `raster_scale` 固定 1.0，DPI 变化不进入 UI | `dynamic_api/session/hud.rs:96`、`menu.rs:230` | P0 |
| R6 | 颜色未线性化就写入 sRGB attachment，与浏览器混合结果不一致 | 11c P1-25、P1-26 | P1 |
| R7 | 画刷与解析样式没有 box-shadow，Material elevation 与 Penpot 阴影无法表达 | `ui/surface/render/brush.rs`、`resolved_style.rs` | P1 |
| R8 | 子节点裁剪与祖先无交集时回退为自身矩形，重新打开不可见区域 | `ui/layout/pass/clip.rs:3` | P1 |
| R9 | 描边对齐未在 bridge 固定；引擎是 inside，Penpot 可为 center 或 outer | `screen_space_ui.wgsl` | P2 |

SVG 结论：不需要通用 SVG 渲染器。Penpot 矢量只出现在图标与装饰形状两处。图标走“SVG 子集解析 → 按 frame×DPI 栅格 → 图集”；装饰形状在 bridge 层降级为 Rounded/Border 画刷，或拒绝导出。

### 3.2 交互无响应

| ID | 根因 | 证据 | 级别 |
| --- | --- | --- | --- |
| I1 | App 收到 UI 动作后只计数并按 2 的幂采样日志，没有任何消费者 | `zircon_app/src/entry/runtime_entry_app/host_requests/ui_action.rs` | P0 |
| I2 | `route` 与 `action.action` 并存时不拒绝，派发返回 None，点击静默失效 | `ui/template/asset/binding/validation.rs:91` | P0 |
| I3 | 失焦、遮挡、DPI、销毁只进 core input，不进 UI window pump，capture、IME 和 popup 滞留 | `dynamic_api/session/events.rs`；`ui/surface/input/window_pump.rs` 已实现但未接线 | P0 |
| I4 | winit 滚轮事件位置填原点，按原点命中并覆盖光标位置 | `ui/platform_input/winit_translation.rs:77` | P1 |
| I5 | 事件管理器订阅 diff 时立即丢弃 Receiver | `ui/event_ui/manager/invocation.rs:121` | P1 |
| I6 | 导航索引签名判定取反，tab 和方向变化时反而跳过重建 | `ui/surface/navigation_index/semantics.rs:139` | P1 |
| I7 | 组件 reducer 缺陷：blur 清空文案、嵌套通知索引错误、滑块无变化仍发事件、NaN 写入、虚拟窗口负起点 | 各文件 `BUG:` 注释 | P1 |
| I8 | modal、focus、popup 依赖组件名与属性别名字符串，自定义组件会绕过 modal trap | 11a P1-16、P1-19 | P2 |

路由次序 `PointerCapture → PopupStack → PreviewTunnel → DirectTarget → Bubble → FocusPath → DefaultAction` 已在 `dispatch/input_manager/routing.rs` 单点定义，骨架与 Slate 一致。缺的是矩阵测试，以及 editor_ui 01 的 open failure。

### 3.3 布局计算错误

| ID | 根因 | 证据 | 级别 |
| --- | --- | --- | --- |
| L1 | Taffy 桥按容器临时建树、只放直接子叶、算完即丢；嵌套 min/max/content、百分比、auto margin 与 CSS flex 不同 | `ui/layout/taffy_bridge/compute.rs` | P0 |
| L2 | `main_axis_alignment_supported` 只接受 Start 和 Fill，Penpot 的 center、end、space-between 静默回退到 Zircon arrange | `compute.rs:309` | P0 |
| L3 | typed `UiLayoutStyle` 与 `style_mapping.rs` 已存在，但 v2 surface tree 仍走 `UiContainerKind + AxisConstraint` 旧词汇 | `ui/v2/surface_tree/layout.rs` | P0 |
| L4 | 文本测量受 R1 影响；shaping 全部失败时仍发布猜测 advance 的“成功”结果 | 11b P0-2 | P0 |
| L5 | Grid 只有均匀 `fr(1)` 轨道；MUI offset 与 responsive columns 没有上界，会溢出 | `compute.rs`；`template/build/slot_contract.rs:264`；`responsive_mui.rs:633` | P1 |
| L6 | `row-reverse` 被映成普通横向盒；order、z、priority 超出 i32 时静默截断 | `responsive_mui.rs:499`；`v2/surface_tree/parse.rs:48` | P1 |
| L7 | Overlay slot `z_order`、Canvas anchor/pivot、嵌套滚动消费次序、虚拟列表全量物化均未定稿 | editor_ui 02 §2.2；11a P1-17 | P1/P2 |
| L8 | 根原点：Penpot board 的 x/y 是画布坐标，引擎归一化到 (0,0)，fixture 的 `position` 在两端含义不同 | designment 02 §1.2 规则 1 | P1（契约） |

## 4. UE Slate 对照与漏洞清单

| UE 机制 | Zircon 现状 | 漏洞 | 决策 |
| --- | --- | --- | --- |
| `FSlateApplication` 统一入口与路由 | `UiInputManager` 七阶段已定 | 窗口生命周期不进 UI（I3） | 采用；补生命周期接线 |
| `FReply` 声明式副作用 | `UiDispatchReply`、`UiDispatchEffect` 已定 | 动作结果没有回执 | 采用；新增 action receipt |
| Enhanced Input | `InputActionMap` 已存在 | UI 动作不能成为输入源（I1） | 调整：UI action 作为新的 `InputBinding` 来源，不另建系统 |
| `FSlateInvalidationRoot` 与 `EInvalidateWidgetReason` | 7 域 dirty flag、reason bitset、局部 rebuild | 没有 paint-only fast path | 采用结构：proxy 数组加更新标志；fast path 直接修补 render command |
| `FHittestGrid::SetHittestArea` | 网格已存在，但索引域随内容扩张 | 没有查询预算与 cursor radius | 采用：由窗口定义索引域 |
| 每个 Panel 手写 `OnArrangeChildren` | Taffy 加自有容器 | 临时树导致 CSS 语义偏差（L1） | 不采用：Flex/Grid 交给持久 Taffy 树，参照 Bevy `UiSurface` |
| `FSlateElementBatcher` 有序批次 | 计划已排序，渲染器不消费 | 层级错乱（R2） | 采用：单一有序 draw-op 流，仅不重叠时合批 |
| `FSlateBrush` 含阴影 | 没有阴影 | R7 | 部分采用：单层 SDF 阴影 |
| `SlateShaderCommon.ush` 解析覆盖 | 圆角 SDF 已一致 | 缺线性颜色（R6） | 沿用；补颜色空间策略 |
| `FSlateFontCache` | 分层存在 | 默认字体与 shaping 失败（R1、L4） | 沿用分层；两端使用同一字节的字体 |
| `Clipping.h` 裁剪区交集 | 存在 | 无交集时回退错误（R8） | 修复为空裁剪 |

## 5. 里程碑

依赖：`M0 → {M1, M2, M3 并行} → M4`；M5 与 M2、M3 并行启动；M6 在 M3 之后收口。

### M0 — 可验证基线（阻塞全部）

- 关闭 2026-10-01 三份 Editor 构建 failure；246 个共享错误按 owner 归档并交回 owner，不越权批改。
- 退出条件之一：`cargo check -p zircon_runtime --lib --locked` 与 `cargo test -p zircon_runtime --test zui_penpot_bridge_contract --locked` 通过。
- 让 `zui_native_visual_acceptance` 对 `penpot_roundtrip` 与 8 个 `reactbits_*` fixture 产出引擎截图、geometry JSON 和 text JSON，回填 `docs/layout` 结果表的 Engine 列。
- 复用插件的 semantic/text parity 比较器，把引擎输出接为第二个 renderer，生成“结构差异”和“像素差异”分离的报告。
- 快修包，每项一个 RED→GREEN 聚焦测试，并删除对应 `BUG:` 注释：I2、I4、I5、I6、I7、L5、L6、R8、`svg.rs` 属性边界匹配、`atlas.rs` 槽位上界。
- I2 改为拒绝之前，先扫描全部仓库 `.zui`，列出双目标绑定并修正资产，再启用拒绝。

### M1 — 交互闭环

- **UI 动作桥**：`InputActionMap` 增加 UI action 绑定来源。`ui_action.rs` 改为先查 action map 并注入 InputAction，再查内置 UI 命令（如 `dialog.cancel`、popup、导航），都未命中才计数告警。每个动作产生 receipt 回流会话，供 pressed 和 loading 状态使用。
- **窗口生命周期**：`events.rs` 把 ScaleFactorChanged、Focused、Occluded、CloseRequested、Destroyed 同时送入 UI window pump。失焦时取消 capture 与拖拽、结束 IME 组合、关闭临时 UI。
- **DPI**：删除 `hud.rs`、`menu.rs` 的 1.0 常量，`raster_scale` 来自窗口度量。缩放后第一个指针事件必须命中新的布局代次。
- **路由与焦点**：七阶段 × handled/passthrough/blocked × capture/popup/modal 的矩阵测试。modal trap 改为编译后的类型化行为句柄，不再匹配组件名。
- 退出：载入 fixture 后，点击、悬停、拖拽、滚轮、键盘、双击、tooltip 都有可观察回执。1.5 DPI 下命中帧与绘制帧一致。失焦后没有滞留 capture。

### M2 — 渲染一致性

- **字体**：分两步。先让主题 `.zui` 显式声明 `font.family` token 并刷新 golden，再把运行时默认切到 `editor-ui.font.toml` 的比例复合字体，Fira Mono 只用于 code run。Penpot 投影加载同一份 TTF 字节，并记录 family、PostScript 名与 SHA-256。shaping 失败改为类型化错误。
- **有序绘制**：屏幕空间 UI 渲染器改为单一有序 draw-op 流，消费 `ordered_element_indices`。用 solid、image、文本、SDF、caret 两两重叠的矩阵做像素测试。
- **图标**：把 Editor 侧 SVG 栅格器下沉为运行时 icon atlas 的生产栅格，按 frame×DPI 分桶。`svg.rs` 改为支持 `g transform`、基本形状和 stroke 的子集解析。
- **阴影、颜色、描边**：`UiResolvedStyle` 增加单层 `box_shadow`，画刷增加 Shadow，shader 用 SDF 近似。颜色先线性化再写 sRGB，Editor 与运行时共用一个颜色空间策略。bridge 固定 inside 描边，center 和 outer 转换或给出警告。字体切换与颜色线性化合并为一次 golden 刷新窗口。
- 退出：9 个 fixture × 4 个视口/DPI 用例结构差异为零，像素差仅为 AA 级。

### M3 — 布局对齐

- **typed 布局成为 v2 权威**：`v2/surface_tree/layout.rs` 直接编译出 `UiLayoutStyle`，Flex、Grid、Block、Wrap 一律经 `style_mapping.rs` 进入 Taffy。旧词汇只保留给 Overlay、Canvas、Scroll、Virtual、docking。解除对齐限制，补齐 `row-reverse`、space-between/around/evenly、align-self 和百分比。
- **持久 Taffy 树**：每个 surface 代次持有一棵 Taffy 树，树事务直接增删改。文本叶子用 measure 闭包接入文本测量缓存。用 feature flag 与旧临时树双跑 parity，一期后删除旧路径。
- **特殊容器契约**：Overlay 的 slot `z_order` 成为 z 权威；Canvas 定稿 anchor、pivot、offset；嵌套滚动按轴“最内层先消费，剩余冒泡”；docking pane 根作为 Taffy root 约束。
- **根原点**：parity harness 把 board 原点归一化为 (0,0)，以 board 逻辑宽高作为视口。验证容器 padding 在 Taffy 路径与 Penpot auto-layout padding 等价，并回写 designment A2 文档。
- **无静默 fallback**：布局引擎选择报告进入帧报告；Penpot 子集 fixture 断言 fallback 为 0。
- 退出：Penpot 导出子集零 fallback，几何误差 ≤1px；`workbench_window.zui` 三档分辨率加 1.5 DPI 结构一致。

### M4 — 产品验收（A2-P）

- 受 MVP F0–F4 gate 约束。用 `tools/analysis/visual/capture-editor-ui-visual.ps1 -DpiProfile 100` 和 `-DpiProfile 150` 采集产品窗口，与同一 Penpot 状态并排比较，提供 Penpot geometry、Runtime structured frame、Editor 截图和容差报告四件证据。

### M5 — 性能与失效（与 M2、M3 并行）

- 新增 surface 级失效根：按绘制序排列的 proxy 数组与更新标志。仅 repaint 或 transform 时走 fast path，直接修补 render command 与 damage；有 layout 或 structure 变化时走现有局部 rebuild。重建报告改为 full、subtree、fast-path、no-op 四态。
- 命中网格的索引域由窗口定义，加入查询候选预算、cursor radius 和 layer range。
- 有序 draw-op 流按代次缓存；同目标 pointer move 必须为 O(1) no-op。

### M6 — 架构收敛

- **v2 为唯一生产入口**：`template/asset` 的 binding 编译、热重载事务、包校验和资产失效图改为 v2 调用的服务模块。旧 `UiAssetLoader → UiDocumentCompiler → UiTemplateSurfaceBuilder` 入口在调用方迁完后删除。判据是 M3 结束时 v2 已覆盖这三项能力。
- **Bridge 单一权威**：新增 `zircon_app` commandlet，由引擎完成 import、组件展开、样式与 token 解析和布局，输出 design projection `.zui` 与 geometry JSON。Penpot catalog 改为消费该输出，删除 TS 侧第二套展开逻辑。
- `zui_penpot_bridge_contract.rs` 扩为能力矩阵：designment A0 表的每一行都有一个正例和一个负例。

## 6. 与现有计划的分工

- designment 02 的 A2-C、A2-P 引擎侧工作由本计划 M0–M4 承担，状态仍记录在 designment evidence。
- editor_ui `2026-09-02-penpot-slate-zui-convergence.md` 的 M2、M3、M4 分别对应本计划 M1、M5、M2。
- editor_ui 01、02 的 Tier 1 open failure 随 M1、M3 关闭；editor_ui 12 的 token、focus、palette 漂移随 M2 关闭。
- optimize 11a、11b、11c 的 P0 实施由本计划认领，复核结果回写原计划。

## 7. 验证

| 层 | 方法 | 标准 |
| --- | --- | --- |
| 单元/契约 | 里程碑末批量 `cargo test -p zircon_runtime --lib --locked`，按过滤词合并；接口改动加 `cargo test -p zircon_runtime_interface --locked` | 每个修复一个 RED→GREEN 测试，对应 `BUG:` 注释删除 |
| 结构 parity | 插件 semantic/text parity 比较器，引擎作为第二 renderer | 节点、顺序、frame、clip、行框 ≤1px；Penpot 子集 fallback 为 0 |
| 像素 parity | 9 fixture × 4 视口/DPI × 28 review state | 结构差为零后才比较；记录 renderer、源码、字体 SHA-256 |
| 交互 trace | App 层注入事件序列 | 所有动作 receipt 非 unhandled；失焦后 capture、IME、popup 为空 |
| 插件 | `pnpm --filter zircon-zui-plugin test`、`lint`、`build` | 全部通过，并记录真实输出 |
| 产品 | `tools/analysis/visual/capture-editor-ui-visual.ps1` 100 和 150 两档 | 证据齐全后人工逐图复核 |

所有 Cargo 验证走 coordinator 分配的 target-dir，保留 `--locked`，不并行运行重型构建。

## 8. 风险

| 风险 | 缓解 |
| --- | --- |
| 编译基线不稳定，工作树有多会话并行 | M0 独占；只修最低共享层；每个里程碑重取 fingerprint，只提交本计划文件 |
| 默认字体切换让全部 golden 失效 | 先加显式 token，再切默认；与颜色线性化合并为一次刷新 |
| 持久 Taffy 树引发嵌套布局回归 | feature flag 双跑 parity，先 fixture 后产品 |
| 双目标绑定由静默改为拒绝，旧资产加载失败 | 先扫描并修资产，再启用拒绝 |
| 删除 TS 展开逻辑后插件回路变慢 | commandlet 输出缓存与 watch |
| 阴影增加填充开销 | 单层近似、按 bounds 扩展 damage，无阴影路径零成本 |

## 9. 状态与产出记录

> 请将产出记录放置在子计划目录 `13/` 中，此处仅展示当前现状的概述。

- 2026-10-04：计划建立。关键结论已逐条对照源码核实；尚未修改任何生产代码，未运行 Cargo。
