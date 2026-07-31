---
related_code:
  - zircon_runtime/src/ui/layout/mod.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/mod.rs
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/layout/pass/mod.rs
  - zircon_runtime/src/ui/layout/pass/engine.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime/src/ui/layout/pass/incremental.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime_interface/src/ui/layout/mod.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime_interface/src/ui/layout/constraints.rs
  - zircon_runtime_interface/src/ui/layout/slot.rs
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/workbench_shell_geometry.rs
  - zircon_editor/src/ui/workbench/autolayout/axis_constraint_override.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/Geometry.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/ArrangedChildren.h
plan_sources:
  - .codex/plans/UI Layout 架构评审与 Taffy 收敛计划.md
  - .codex/plans/布局系统.md
  - .codex/plans/Bevy-Informed Zircon UI 架构优化里程碑计划.md
  - .codex/plans/Zircon UI 增量布局、增量重绘与控件池优化计划.md
status: planned
---

# 02 布局系统：Taffy 权威与特殊容器

## 1. 目标

把「类 CSS/HTML 的组件化布局」做成唯一描述方式：Flex/Grid/Block/Wrap 由 Taffy 权威求解，多个相似布局用一种声明统一表达；Overlay、Canvas、Scroll、Virtual、editor docking 等特殊容器保留 Zircon 自有布局路径，但都写回同一棵 `UiArrangedTree`。布局失败必须记录 fallback reason，不允许静默退回。补齐布局调试器（归档 M20）。

## 2. 现状（按代码核实修正）

### 2.1 已存在的设施

| 能力 | 落点 | 证据 |
|------|------|------|
| 引擎选择与回退建模 | `zircon_runtime_interface/src/ui/layout/engine.rs` | `UiLayoutEngineBackend`（:11）、`UiLayoutEngineFamily`（:18）、`UiLayoutEngineCapability::taffy_flex_grid_wrap_block`（:88）、`UiLayoutEngineSelection::select`（:239，带 per-node `fallback_reason`）、`UiLayoutEngineSelectionReport::from_selections/recompute_counts`（:283–:341） |
| Fallback 原因枚举（13 variant） | 同上 :171 | `UiLayoutEngineFallbackReason`{UnsupportedFamily, MissingContentMeasure, MissingDpiScaling, ZirconOwnedSemantics, UnsupportedChildVisibility, ChildPlacementPolicy, AxisConstraintPriority, InvalidLayoutValue, SlotFramePolicy, SlotCanvasPlacement, TaffyStyleUnavailable, TaffyTreeBuildFailed, TaffyComputeFailed} |
| 约束词汇（legacy） | `zircon_runtime_interface/src/ui/layout/constraints.rs` | `StretchMode`（:4）、`AxisConstraint`（:11）、`BoxConstraints`（:66）、`DesiredSize`（:74）、`LayoutBoundary`（:86） |
| taffy 映射 | `zircon_runtime/src/ui/layout/taffy_bridge/mod.rs` | `taffy_style_for_container`（:8）、`taffy_display_for_family`（:100）——容器级映射已单点化 |
| 布局 pass | `zircon_runtime/src/ui/layout/pass/` | arrange、axis、child_frame、clip、engine、incremental、layout_tree、material、measure、responsive_mui、slot、taffy_arrange 共 13 文件 |
| 滚动/虚拟化 | `zircon_runtime/src/ui/layout/{scroll,virtualization}.rs` | `virtual_window_for_scrollable_box`（scroll.rs:6）、`compute_virtual_list_window`（virtualization.rs:3） |
| editor docking 几何 | `zircon_editor/src/ui/workbench/autolayout/` | workbench_shell_geometry.rs、region/、constraints/、axis_constraint_override.rs、pane_constraint_override.rs、floating_window.rs |

### 2.2 真实缺口

1. **布局描述分散**：声明词汇有三套并存——interface 的 `AxisConstraint`/`StretchMode`（legacy）、v2 style 布局字段、`.zui` 布局约束；无一份对齐 CSS/MUI 的统一属性集，`taffy_style_for_container` 只覆盖容器级，子项级（grow/shrink/basis/align-self/inset）解释散在 pass/axis.rs 与 slot 路径。
2. **fallback 有枚举无闸门**：per-node reason 已记录，但未聚合进帧报告对外暴露，无「无静默 fallback」CI 断言，非法值用例未系统化。
3. **Scroll/Virtual 边界未定稿**：嵌套滚动消费次序、虚拟列表动态行高（固定/测量缓存/估算修正三档）无书面契约与测试。
4. **docking 接缝无契约**：autolayout 输出 shell 几何，但「pane 内容区根节点如何成为 Taffy root 约束」没有显式接口，editor 侧仍有逐 pane 的约束 override（axis_constraint_override.rs）。
5. **布局调试器缺失**：constraint diagnostics、style 来源链、debug packet（归档 M20）未实现。

## 3. 设计

### 3.1 布局描述统一（类 CSS 组件化）

- `zircon_runtime_interface::ui::layout` 新增统一布局属性集 `UiLayoutStyle`（见 §4），对齐 MUI/CSS 命名，作为 `.zui`、v2 style、组件 descriptor 的共同词汇表。（2026-07-02 评审收口）作者词汇/token 规范权威 = editor_layout/13（含 `$token` 文法）；本计划持有的是 **DTO→taffy 映射与运行时求解**，词汇集定义以 13 为准，本文 §4 草案是其运行时投影。
- `zircon_runtime/src/ui/layout/style_mapping.rs`（新增）持有唯一「`UiLayoutStyle` → `taffy::Style`」逐字段映射；`taffy_bridge::taffy_style_for_container` 收编进来；任何组件不得自带私有布局解释。
- legacy `AxisConstraint`/`StretchMode` 经 adapter 折算为 `UiLayoutStyle`，声明源逐步迁移后删除 adapter。
- 相似布局靠组件 + class 复用（计划 04 的 style 级联），明令禁止逐节点像素硬编码作为对齐手段。

### 3.2 引擎选择与特殊容器

- `UiLayoutEngineSelectionReport` 接入 surface 帧报告：每帧聚合 reason counts + 首例 node id，对外可查询。
- 特殊容器职责定稿：
  - **Overlay**：z 序层叠 + anchor 对齐（popup/tooltip/拖拽影子），子项独立测量。
  - **Canvas/Free**：绝对坐标 + pivot/anchor（图编辑器、HUD 画布）。
  - **Scroll**：视口裁剪 + 内容测量 + 滚动条合成；嵌套滚动按「最内层先消费、未消费冒泡」定稿（与 01 的 wheel 沿 hit path 冒泡协同）。（2026-07-02 评审收口）消费语义**按轴部分消费**：wheel delta 按 x/y 轴分别判定消费，某轴到达滚动边界时该轴剩余 delta 冒泡外层，另一轴不受影响。
  - **Virtual**：窗口化行生成 + 行高三档策略（固定 / 测量缓存 / 估算修正）。（2026-07-02 评审收口）测量缓存档与 03 的文本 measure 缓存为**同一缓存**（text/09 两级缓存的 LayoutCache），不另建行高专用缓存；估算修正档的实测行高**回写**估算模型；行内容变更时经同一失效联动使缓存条目与估算失效。
  - **EditorDocking**：drawer/splitter/tab-stack 框架几何归 editor autolayout；每个 pane 内容区根节点经 `PaneContentRootConstraint` 契约交回 Taffy。
- fallback：Taffy 求解失败（非法约束、NaN、循环依赖）记录 reason 并以安全尺寸排布；CI 测试断言「白名单之外无 fallback」。

### 3.3 增量与失效

- 脏标记传播沿用既定规则：向上冒泡至 `LayoutBoundary` 或外部尺寸边界停止。
- 增量布局（pass/incremental.rs）保持默认路径；全量重排只在 root 约束变化或 surface 重建时发生，帧报告记录重排节点数。

### 3.4 布局调试器（M20 补课）

- `UiLayoutDebugPacket`：节点 geometry、输入约束、引擎、style 来源链（asset → class → inline，对接计划 04）、fallback reason 的序列化导出；默认关闭、按帧开启。
- editor 侧 Widget Tree Debugger 面板（计划 09 批次 3）消费该 packet；本计划只落 runtime 数据面。

### 3.5 DPI 与取整（2026-07-02 评审收口）

- 遵 editor_layout/16 §3.4：布局求解全程使用**逻辑坐标**，物理像素换算集中在 editor_layout/21 的**单点换算**（GPU 提交侧），布局层不出现散落的 `* scale_factor`。
- 取整策略随 16 §3.4 定稿执行（边界对齐取整，避免相邻节点缝隙/重叠）；本计划不自定取整规则。
- 16 §3.6 留给本计划的「taffy 桥限制评估」事项列入切片表（见 M1.S5）。

## 4. 接口与数据结构草案

```rust
// 新增 zircon_runtime_interface/src/ui/layout/style.rs（2026-07-02 勘误：已落地）
pub struct UiLayoutStyle {
    pub display: UiLayoutDisplay,                  // 引擎选择的声明源
    pub direction: UiFlexDirection,                // Row | Column | RowReverse | ColumnReverse
    pub wrap: UiFlexWrap,                          // NoWrap | Wrap | WrapReverse
    pub justify_content: Option<UiJustify>,        // Start|End|Center|SpaceBetween|SpaceAround|SpaceEvenly
    pub align_items: Option<UiAlign>,              // Start|End|Center|Stretch|Baseline
    pub align_self: Option<UiAlign>,
    pub align_content: Option<UiAlign>,
    pub gap: UiGap,                                // { row: UiDimension, column: UiDimension }
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: UiDimension,
    pub grid_template_columns: Vec<UiGridTrack>,   // Px | Percent | Fr | Auto | MinMax
    pub grid_template_rows: Vec<UiGridTrack>,
    pub grid_row: Option<UiGridPlacement>,
    pub grid_column: Option<UiGridPlacement>,
    pub size: UiSize2,                             // { width: UiDimension, height: UiDimension }
    pub min_size: UiSize2,
    pub max_size: UiSize2,
    pub aspect_ratio: Option<f32>,
    pub margin: UiEdges,                           // { left/right/top/bottom: UiDimension }
    pub padding: UiEdges,
    pub position: UiPositionMode,                  // Relative | Absolute
    pub inset: UiEdges,
    pub overflow: UiOverflowPair,                  // { x, y: Visible | Hidden | Scroll }
}

pub enum UiLayoutDisplay { Flex, Grid, Block, /* Wrap 经 wrap 字段表达 */ Overlay, Canvas, Scroll, Virtual, None }
pub enum UiDimension { Auto, Px(f32), Percent(f32) }

// 新增 zircon_runtime/src/ui/layout/style_mapping.rs（唯一映射）（2026-07-02 勘误：已落地）
pub fn taffy_style_from_ui_layout_style(style: &UiLayoutStyle) -> Result<taffy::Style, UiLayoutEngineFallbackReason>;
pub fn ui_layout_style_from_axis_constraints(                   // legacy adapter（迁移期）
    horizontal: AxisConstraint, vertical: AxisConstraint, stretch: StretchMode,
) -> UiLayoutStyle;

// 新增 zircon_runtime_interface/src/ui/layout/debug.rs（2026-07-02 勘误：已落地）
pub struct UiLayoutDebugPacket {
    pub frame_index: u64,
    pub selection_report: UiLayoutEngineSelectionReport,  // 现有类型
    pub nodes: Vec<UiLayoutDebugNode>,
}
pub struct UiLayoutDebugNode {
    pub node_id: UiNodeId,
    pub geometry: UiRect,
    pub constraints: BoxConstraints,                      // 现有类型
    pub engine: UiLayoutEngineBackend,                    // 现有类型
    pub fallback_reason: Option<UiLayoutEngineFallbackReason>,
    pub style_sources: Vec<UiLayoutStyleSourceRef>,       // 新增：asset → class → inline 来源链
}

// 新增（M4）zircon_editor/src/ui/workbench/autolayout/pane_content_contract.rs
pub struct PaneContentRootConstraint {
    pub pane_region: ShellRegionId,        // 现有类型（shell_region_id.rs）
    pub content_rect: UiRect,              // autolayout 求出的内容区矩形
    pub min_content_size: UiSize2,
}
pub fn pane_content_root_constraints(geometry: &WorkbenchShellGeometry) -> Vec<PaneContentRootConstraint>;
```

属性 → `taffy::Style` 映射表（style_mapping 单点实现，每行一组对拍测试）：

| UiLayoutStyle | taffy::Style |
|---------------|--------------|
| display(Flex/Grid/Block) | `display` |
| direction / wrap | `flex_direction` / `flex_wrap` |
| justify_content / align_items / align_self / align_content | 同名字段（Baseline 对齐（2026-07-02 评审收口）：文本 `first_baseline` 数据来源 = runtime text 布局结果（text/03），不由布局层自行估算） |
| gap | `gap` |
| flex_grow / flex_shrink / flex_basis | 同名字段 |
| grid_template_columns/rows、grid_row/column | `grid_template_*`、`grid_row/column` |
| size / min_size / max_size / aspect_ratio | 同名字段 |
| margin / padding / inset / position | 同名字段 |
| overflow | `overflow` |
| display(Overlay/Canvas/Scroll/Virtual) | 不进 Taffy——`UiLayoutEngineSelection` 选 Zircon 引擎并记录 `ZirconOwnedSemantics` 为正常选择（非 fallback） |

## 5. 模块与文件落点

**新增**：`zircon_runtime_interface/src/ui/layout/style.rs`（已落地）、`zircon_runtime_interface/src/ui/layout/debug.rs`（已落地）、`zircon_runtime/src/ui/layout/style_mapping.rs`（已落地）、`zircon_runtime/src/ui/layout/debug_packet.rs`、`zircon_editor/src/ui/workbench/autolayout/pane_content_contract.rs`（2026-07-02 勘误：前三项撰写后已在码，读作「已存在，按切片继续演进」）

**修改**：

| 路径 | 改什么 |
|------|--------|
| `zircon_runtime/src/ui/layout/taffy_bridge/mod.rs` | `taffy_style_for_container` 收编进 style_mapping，桥只做 tree build |
| `zircon_runtime/src/ui/layout/pass/{axis,slot,taffy_arrange,engine}.rs` | 改读 `UiLayoutStyle`（经 adapter 过渡） |
| `zircon_runtime/src/ui/layout/{scroll,virtualization}.rs` | 嵌套消费次序、行高三档策略 |
| `zircon_runtime/src/ui/surface/`（帧报告处） | SelectionReport 聚合 + debug packet 采集开关 |
| `zircon_editor/src/ui/workbench/autolayout/{mod,workbench_shell_geometry}.rs` | 输出 PaneContentRootConstraint |

**删除（硬切换义务）**：legacy adapter（声明源迁完后，M1.S4）；`pass/material.rs`、`pass/responsive_mui.rs` 中与统一属性集重复的解释段（保留响应式断点语义本身）；editor `axis_constraint_override.rs`/`pane_constraint_override.rs` 中被 PaneContentRootConstraint 取代的逐 pane override（M4.S2）。

## 6. 管线时序

布局阶段内部次序不变：dirty 收集 → 引擎选择（per 容器，记录 Selection）→ Taffy 子树求解 / Zircon 容器排布 → 写回 `UiArrangedTree` → measure 缓存供 render extract 复用。本计划新增的只有：选择报告聚合点（布局 pass 末尾）与 debug packet 采集点（同处，开关控制）。

## 7. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
|---|------|---------|---------|--------|
| M1.S1 | `UiLayoutStyle` DTO 定稿 + serde/默认值测试 | interface layout/style.rs | `cargo test -p zircon_runtime_interface --locked layout` | 无删除 |
| M1.S2 | style_mapping 唯一映射 + taffy_bridge 收编 | style_mapping.rs、taffy_bridge.rs | `cargo test -p zircon_runtime --lib style_mapping --locked` | taffy_bridge 内散落映射删除 |
| M1.S3 | legacy adapter：AxisConstraint/StretchMode → UiLayoutStyle；pass/axis.rs、slot.rs 改读统一属性 | pass/axis.rs、pass/slot.rs | `cargo test -p zircon_runtime --lib layout --locked` | 无删除（adapter 留到 S4） |
| M1.S4 | v2 style 与 `.zui` 布局字段切到统一词汇（与 04 M3、05 M1 协同）；属性矩阵对拍 | v2 style 编译路径 | 同上 + `.zui` 治理测试 | 声明源迁完删 adapter |
| M1.S5 | （2026-07-02 评审收口新增）定稿 **Taffy measure 回调契约**：回调签名、`known_dimensions`/`available_space` 语义（Definite/MinContent/MaxContent 三值如何映射到文本 measure 的 min-content/max-content/preferred）、与文本 measure 缓存（text/09 两级缓存）的接缝。契约规范权威 = editor_layout/13 §3.8，本切片为实现落点；03 M3 与 07 M1 依赖此切片。同切片完成 16 §3.6 留给本计划的「taffy 桥限制评估」（taffy 对逻辑坐标/取整/measure 精度的限制清单） | pass/measure.rs、style_mapping.rs、评估记录 | `cargo test -p zircon_runtime --lib measure --locked` | 无删除 |
| M2.S1 | SelectionReport 聚合进 surface 帧报告（reason counts + 首例 node） | surface 帧报告、debug_packet.rs | `cargo test -p zircon_runtime --lib engine_selection --locked` | 无删除 |
| M2.S2 | 非法值用例系统化：NaN/负尺寸/循环约束 → reason 而非 panic/静默 | style_mapping、pass/engine.rs 测试 | 同上 | 无删除 |
| M2.S3 | 「无静默 fallback」CI 断言：合法模板全量布局后 reason counts 为白名单子集 | 测试 + 白名单清单 | `cargo test -p zircon_runtime --lib --locked` | 无删除 |
| M3.S1 | 嵌套滚动定稿：最内层先消费、未消费冒泡（与 01 wheel 路由协同） | scroll.rs + dispatch wheel 路径 | `cargo test -p zircon_runtime --lib scroll --locked` | 无删除 |
| M3.S2 | 虚拟行高三档：compute_virtual_list_window 扩展估算修正档（带阻尼收敛） | virtualization.rs | `cargo test -p zircon_runtime --lib virtual --locked` | 无删除 |
| M3.S3 | 1k 行虚拟列表 + 嵌套 Scroll 行为测试 | 测试 | 同上 | 无删除 |
| M4.S1 | PaneContentRootConstraint 契约落地：autolayout 输出 → pane 内容子树 root 约束 | pane_content_contract.rs、workbench_shell_geometry.rs | `cargo test -p zircon_editor --lib autolayout --locked` | 无删除 |
| M4.S2 | drawer 拖拽改宽 → 内容区 Taffy reflow 实机验证；删除被取代的逐 pane override | axis_constraint_override.rs、pane_constraint_override.rs | `cargo test -p zircon_editor --lib --locked` + 实机 | 删被取代 override |
| M5.S1 | debug DTO + 采集（开关控制，默认关） | interface layout/debug.rs、debug_packet.rs | `cargo test -p zircon_runtime --lib debug_packet --locked` | 无删除 |
| M5.S2 | 导出口（host request / dump）+ 快照测试 + `docs/zircon_runtime/ui/layout.md` 更新 | debug_packet.rs、文档 | 同上 | 无删除 |

## 8. 测试矩阵（代表性用例）

- **M1**：`ui_layout_style_maps_every_field_to_taffy`（逐字段对拍）、`axis_constraint_adapter_preserves_stretch_semantics`、`percent_basis_resolves_against_parent`
- **M2**：`invalid_size_records_invalid_layout_value_reason`、`taffy_compute_failure_falls_back_with_reason`、`legal_templates_produce_no_unexpected_fallback`
- **M3**：`nested_scroll_inner_consumes_before_outer`、`unconsumed_wheel_bubbles_to_outer_scroll`、`virtual_list_estimated_row_height_converges_after_measure`、`virtual_window_covers_viewport_plus_overscan`
- **M4**：`pane_content_root_constraint_matches_shell_geometry`、`drawer_resize_reflows_pane_content`
- **M5**：`layout_debug_packet_snapshot_stable`、`debug_packet_off_has_zero_capture_cost`

落点：runtime 侧模块内 `#[cfg(test)]`（沿 layout 现状）；editor 侧 autolayout 邻近测试。

## 9. 风险与对策

| 风险 | 对策 / 探测信号 |
|------|----------------|
| 统一词汇迁移触及全部模板与组件声明 | adapter 先行（M1.S3），声明源在 M1.S4 与 04/05 协同分批迁；属性矩阵对拍守每一步 |
| taffy 语义细节差异（percent 基准、auto margin、min-content） | 每字段一组对拍测试；差异项显式记录在 style_mapping 注释与文档 |
| 虚拟列表估算修正引发滚动抖动 | 修正档带阻尼；测试断言行高估算单调收敛 |
| docking 接缝改动与 08 区域切换重叠 | M4 排在 08 M2 之前完成并被其消费；同区域不双改 |
| debug packet 采集拖慢帧 | 默认关闭；开启路径有零成本断言测试 |

## 10. 里程碑级依赖表

| 里程碑 | 前置 | 被依赖 |
|--------|------|--------|
| M1 | 无（与 01 并行） | 02 M2–M4、03 M3（measure 回调接口）、04 M3（统一词汇）、06 全部 |
| M2 | 02 M1 | 02 M5 |
| M3 | 02 M1 | 06 M2（Tree/Table 虚拟化） |
| M4 | 02 M1 | 08 M2（drawer 框架迁移） |
| M5 | 02 M2 | 09 批次 3（Widget Tree Debugger 数据面） |

## 11. 完成定义

- 任意一处布局声明只用 `UiLayoutStyle` 词汇；taffy 映射只有 style_mapping 一份。
- 帧报告含引擎选择聚合；CI 断言合法模板零意外 fallback。
- 嵌套滚动 / 1k 行虚拟列表行为测试全绿；drawer 改宽实机 reflow 正确。
- debug packet 可开关导出且有快照基线。
- 验收命令组：`cargo test -p zircon_runtime --lib --locked`（layout/scroll/virtual/style_mapping 过滤）、`cargo test -p zircon_runtime_interface --locked`、`cargo test -p zircon_editor --lib --locked`。

## 12. 边界约束

- 不把 Overlay/Canvas/Scroll/Virtual 塞进 Taffy；也不为 Flex/Grid 保留 Zircon 手写路径。`ZirconOwnedSemantics` 是正常选择记录，不算 fallback。
- Taffy 仍是直接依赖（非插件）；不引入其他布局库。
- 布局结果只经 `UiArrangedTree` 对外暴露；hit/render 不得另行计算几何。
- `UiLayoutStyle` 进 interface 层即受 ABI 约束：字段增删走 serde 默认值兼容，集中在 M1.S1 定稿。

## 13. 参考实现对照（dev/ 源码锚点）

实现切片前先读对应锚点，不确定的行为语义以参考实现为准（在 PR 说明中注明出处）；禁止凭印象实现、禁止引用未核实路径。

| 设计点 | 主参考 | 次参考 | 参考什么 |
|--------|--------|--------|---------|
| 属性 → taffy::Style 唯一映射 | `dev/bevy/crates/bevy_ui/src/layout/convert.rs` | `dev/bevy/crates/bevy_ui/src/layout/{mod.rs, ui_surface.rs}` | bevy 的「声明属性 → taffy::Style」逐字段折算是 style_mapping 的直接样板（含 percent/auto 边界处理） |
| 布局调试导出 | `dev/bevy/crates/bevy_ui/src/layout/debug.rs` | — | debug packet 的节点几何/树形 dump 形态 |
| arranged geometry 概念 | `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/{Geometry.h, ArrangedChildren.h, ArrangedWidget.h}` | `LayoutUtils.h`、`Visibility.h`、`Clipping.h`（同目录） | arranged tree 为唯一空间事实、可见性/裁剪如何影响排列与命中（不取手写 OnArrangeChildren） |
| 增量失效 | `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/FastUpdate` | — | Slate invalidation 系统的脏域划分（layout/paint/volatility） |
| 自有容器（Canvas/Grid） | `dev/Fyrox/fyrox-ui/src/canvas.rs`、`grid.rs` | `dev/slint/internal/core/layout.rs` | 绝对定位容器与 Rust 端布局求解器的接口形态 |
| 嵌套滚动/容器语义 | `dev/godot/scene/gui/{container.cpp, box_container.cpp, scroll_bar.cpp}` | `dev/godot/scene/gui/split_container.cpp` | 容器 re-sort 时机、滚动条合成、splitter 拖拽几何 |
| docking 接缝 | `dev/Fyrox/fyrox-ui/src/dock/{tile.rs, config.rs}` | `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking` | docking 树与内容区约束的分界（框架几何 vs 内容布局） |

## 14. 状态与产出记录

- 2026-07-02（评审收口）：勘误 §4/§5——`UiLayoutStyle`（interface layout/style.rs）、`style_mapping.rs`、`layout/debug.rs` 已在码；新增 M1.S5（Taffy measure 回调契约 + taffy 桥限制评估）、§3.5（DPI 与取整，遵 editor_layout/16 §3.4/21）；§3.1 补词汇规范权威=editor_layout/13；§3.2 Scroll 补按轴部分消费、Virtual 补与 03/text 09 measure 缓存接缝、Baseline 补 first_baseline 来源=text/03。后续切片执行时在此回写状态。
- 2026-07-23 performance handoff：interface `ui/layout/**` 11/11确认每container selection构建两份supported-family Vec，incremental report clone所有untouched rows并用BTreeMap重算；PERF-MVP-263要求static capability mask、generation report原位patch/发布、detail仅debug gate。`UiLayoutStyle` grid tracks按compiled generation共享（261/274/312），slot建共享edge index（260），persistent Taffy tree与scratch参考Bevy `UiSurface`（261），fixed-window O(1)保留且offscreen arrange=0（262）。补1/100/10k nodes/slots/tracks与100k rows的alloc/clone/probe/upsert/visible visits/p95门，current-source Cargo和F4 resize/scroll实机待完成。
