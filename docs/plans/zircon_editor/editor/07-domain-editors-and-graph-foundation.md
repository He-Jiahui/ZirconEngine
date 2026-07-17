---
related_code:
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/ui/animation_editor/session.rs
  - zircon_editor/src/ui/material_editor
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/src/ui/layouts/views/animation_editor.rs
  - zircon_editor/src/ui/template_runtime
  - zircon_plugins/animation
reference_sources:
  - dev/Fyrox/editor/src/plugins/animation/mod.rs
  - dev/Fyrox/editor/src/plugins/absm/mod.rs
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Public
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor
  - dev/theatre
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor_ui/07-ui-animation-theatre.md
status: in_progress
---

# 07 图编辑基座与领域编辑器（动画 / Montage / 状态机 / 行为树 / 预览）

| 2026-07-12 | Editor08 M1.2 失败移交：focused document kind 权威投影 | 已修复（fixed，2026-07-15） | Editor08 已落地 `FocusedDocumentKind(DocumentKind)` when 谓词并禁止由 project-open 猜测 scene focus；Editor07 完成 typed `ViewDescriptor.document_kind`、唯一 `focused_view` 和浮动窗口焦点生命周期，最终 current-source 16/16 通过，详见 [fixed 回传](08/fixed-2026-07-15-command-eval-focused-document-projection.md)。 |
- fixed 已修复：[irradiance-volume-shader-ide-validation-dependency](07/fixed-2026-07-15-irradiance-volume-shader-ide-validation-dependency.md)
- fixed 已修复：[failure-return-plan-table-row-corruption](07/fixed-2026-07-15-failure-return-plan-table-row-corruption.md)
- Editor05 移交（`open / viewport SelectionModel consumer hard cut`）：[`07/failure-2026-07-16-viewport-selection-model-consumer-hard-cut.md`](07/failure-2026-07-16-viewport-selection-model-consumer-hard-cut.md)
- fixed 已修复：[ui-root-owner-boundary-migration-debt](07/fixed-2026-07-17-ui-root-owner-boundary-migration-debt.md)
- Editor07 current-source 上行门发现的文本 fixture 失败（并入既有 `open / retained text`）：[`../editor_ui/03/failure-2026-07-11-retained-text-family-and-subpixel-contracts.md`](../editor_ui/03/failure-2026-07-11-retained-text-family-and-subpixel-contracts.md)

## 参照证据（dev/）

**Fyrox 动画编辑器**（`plugins/animation/mod.rs:121-132`）——轨道式编辑器最小完备件：`window / animation_player: ErasedHandle / animation: ErasedHandle / track_list: TrackList / curve_editor / toolbar / content / ruler: Ruler / thumb: Thumb`（播放头）。

**Fyrox 状态机编辑器**（`plugins/absm/mod.rs:204-213`）五件套：`state_graph_viewer`（状态图画布）/ `state_viewer`（选中态内部 blend tree）/ `parameter_panel` / `toolbar` / `blend_space_editor`。要点：**状态层与 blend 层是两个联动面板而非一个图**；参数面板与预览联动是核心交互。

**UE Persona 共享预览**（`IPersonaPreviewScene.h:83-95`）：`GetPersonaToolkit / InvalidateViews / FocusViews / GetPreviewMeshComponent / GetAllPreviewMeshComponents`——Skeleton/AnimBP/Montage 多编辑器**共用一个预览场景**，编辑哪个资产都作用在同一具预览体上。

**UE BehaviorTreeEditor**：composite/decorator/service/task 四类；装饰器/服务是**节点附着子项**而非独立图节点——图基座需「节点内子列表」。

**theatre**：轨道折叠/关键帧聚合/吸附交互由 `editor_ui/07` 引入为 UX 参照，本计划直接消费结论。

## 现状与证据（zircon，2026-07-05 实读）

### 图/时间轴注册词汇已定型（v3 补全字段）

四张表（06 store 收编对象）+ 描述符字段（`editor_authoring_extension.rs:130-474` 实读）：

```rust
pub struct GraphPinDescriptor    { name, value_type: String, required: bool }          // :130-134
pub struct GraphNodeDescriptor   { id, display_name, category,
                                   inputs/outputs: Vec<GraphPinDescriptor> }           // :165-176
pub struct GraphNodePaletteDescriptor { /* 节点面板=palette 词汇表 */ }                 // :222
pub struct GraphEditorDescriptor { asset_kind, view_id, display_name,
                                   open_operation: EditorOperationPath,
                                   validate_operation: EditorOperationPath,            // 校验流已预期
                                   compile_operation: Option<EditorOperationPath>,     // 编译流已预期
                                   required_capabilities }                             // :273-283
pub struct TimelineTrackDescriptor  { id, display_name, value_kind: String, required_capabilities } // :348-353
pub struct TimelineEditorDescriptor { asset_kind, view_id, display_name, open_operation,
                                      track_types: Vec<String>, required_capabilities }             // :397-407
```

三个设计输入：(a) 端口类型是**字符串 `value_type`**——图基座默认连接校验=类型名相等，领域可覆写；(b) `validate/compile_operation` 双操作位说明注册面已预期「图→校验→编译」流水（absm/BT/材质图共用形状）；(c) `TimelineEditorDescriptor.track_types` 引用 `TimelineTrackDescriptor.id`——轨道类型是独立注册的可复用词汇。

### 编辑器实体

- `ui/animation_editor/` **合计仅 78 行**（mod 5 / presentation 16 / session 57，`AnimationEditorSession` + Error 两类型）——是占位骨架非半成品。
- `ui/material_editor/`：`{mod, projection, renderer_data_projection}` 同级占位。
- `ui/asset_editor/`：最成熟样板（session/undo/journal/theme/presentation_state 齐备），session 生命周期范式取自它。
- `template_runtime/component_adapter` 已有资源编辑器/动画编辑器/检查器适配位。

### runtime 侧

`zircon_plugins/animation` 有 runtime 动画插件（`runtime_physics_animation_tick_contract` 契约测试在案）；importer 含 `import_animation_asset.rs`。**montage/absm/BT 三资产族的 runtime 模型与求值器不存在。**

### 缺口

无图画布/连线/布线实体；无时间轴实体；无预览场景框架；三资产族缺失。

## 目标

1. **GraphEditorFoundation**：画布（平移/缩放/网格）、节点（端口/标题/体区委托/**附着子项**）、连线（`value_type` 默认校验 + 领域覆写/正交或贝塞尔布线）、选择/框选/复制粘贴/对齐、迷你图；数据模型 `GraphModel` trait；编辑全走 03 事务；物化经 `graph_editors/graph_node_palettes` 表；`validate/compile_operation` 接 08 命令（图工具栏「校验/编译」按钮=操作投影）。
2. **TimelineFoundation**：标尺/播放头/轨道列表/关键帧+区段双元素/框选/吸附；`timeline_*` 两表注册，`value_kind` 决定 lane 渲染器。
3. **PreviewSceneFramework**（Persona 等价）：复用 04 副 session 机制 + 预览体注入 + `invalidate_views/focus_views`；动画族编辑器共享同一预览 session。
4. **动画编辑器成型**：78 行骨架扩为 `DocumentToolkit` 实例（Fyrox 件清单：track_list+curve+ruler+thumb）+ 预览联动。
5. **Montage 编辑器**：资产模型（runtime 侧 `slots/sections（可重排跳转）/segments（引 clip）/notify tracks`，UE 词汇直译）；编辑器=三层 lane 时间轴实例。
6. **状态机编辑器**：absm 资产模型（states/transitions{条件}/layers/blend space）；编辑器复刻五件套（双 GraphModel 联动 + ParameterPanel + 活跃态高亮经 02 watch）。
7. **行为树编辑器**：BT 资产模型（四类节点）；`StructureConstraint::Tree` + 附着子项；PIE 活跃分支高亮预留。

## 非目标

- 蓝图式通用脚本图（基座留缩放位）；Sequencer 级过场编排；材质图编辑器（表可注册，待 shader/04 材质绑定契约稳定后立案）；骨骼编辑/蒙皮。

## 架构设计

### 模块布局

```
zircon_editor/src/ui/graph/
  mod.rs / model.rs / canvas.rs / node_widget.rs / routing.rs / commands.rs
zircon_editor/src/ui/timeline/
  mod.rs / model.rs / ruler.rs / track_list.rs / keyframe_lane.rs / section_lane.rs
zircon_editor/src/ui/curve/          # 曲线组件（动画曲线视图与 06 曲线字段编辑器共用底层）
zircon_editor/src/ui/preview_scene/
  mod.rs / preview_scene.rs / preview_subject.rs
zircon_editor/src/ui/animation_editor/   # 骨架扩为 toolkit 实例
zircon_editor/src/ui/absm_editor/        # 新
zircon_editor/src/ui/behavior_tree_editor/  # 新
# montage/absm/bt 资产模型住 zircon_plugins/animation（或 ai）runtime 资产族；编辑器只持创作投影
```

### 关键类型

```rust
// graph/model.rs
pub trait GraphModel: Send {
    type NodeId: Copy + Eq + Ord;
    fn nodes(&self) -> Vec<GraphNodeView<Self::NodeId>>;    // 含 attachments: Vec<AttachmentView>
    fn edges(&self) -> Vec<GraphEdgeView<Self::NodeId>>;
    fn palette(&self) -> &GraphNodePaletteDescriptor;       // 既有描述符=节点词汇表
    fn can_connect(&self, from: PortRef<Self::NodeId>, to: PortRef<Self::NodeId>) -> ConnectVerdict;
    // 缺省实现：GraphPinDescriptor.value_type 字符串相等 + required 端口悬空检查
    fn structure_constraint(&self) -> StructureConstraint;  // FreeGraph | Tree | Dag
    fn apply(&mut self, delta: GraphDelta<Self::NodeId>) -> Result<GraphDelta<Self::NodeId>, GraphError>;
    // apply 返回逆 delta —— commands.rs 的 revert 免费获得
}
// 附着子项：GraphNodeView.attachments（BT decorator/service）；palette 描述符不扩字段，
// 附着词汇由领域 palette 的 category 约定（"attachment/*"）承载——不动既有描述符形状。

// preview_scene/preview_scene.rs
pub struct PreviewScene {
    session: SecondarySessionHandle,     // 04 副 session
    subject: Option<PreviewSubject>,     // 骨架网格 + 参数覆盖
    viewport_doc: DocumentId,
}
impl PreviewScene {
    pub fn set_subject(&mut self, s: PreviewSubject);
    pub fn invalidate_views(&mut self);
    pub fn focus_views(&mut self);
    pub fn playback(&mut self) -> &mut PreviewPlayback;   // play/pause/step/rate
}
// AnimationFamilyToolkits 注入同一 Rc<RefCell<PreviewScene>>（Persona 共享语义）
```

### 领域→基座映射表

| 领域 | 图基座 | 时间轴 | 预览 | 资产模型归属 |
| --- | --- | --- | --- | --- |
| 动画 clip | — | 轨道+关键帧+曲线 | 必需 | 既有 animation 资产 |
| Montage | — | slot/section/notify 三层 lane | 必需 | `zircon_plugins/animation` 新增 |
| absm | StateGraph(Free)+BlendTree(Dag) 双实例 | — | 必需（活跃态高亮） | 同上新增 |
| 行为树 | Tree+附着子项 | — | PIE attach 预留 | animation 或新 ai 插件（M4 裁决） |
| 材质图（远期） | Dag | — | 材质球 | shader 计划集 |

### 深度测试

图基座以行为树（Tree+附着）为第二领域验收：接入时 `graph/` 零改动。时间轴以 montage（三层 lane）为第二领域验收（第一=动画 clip）。

## 里程碑

### M1 图基座

- 切片 1.1：`GraphModel` + canvas/node_widget/routing（渲染走 editor_ui 栈）；选择/框选/拖拽/缩放。
- 切片 1.2：连接校验（value_type 缺省 + 覆写）/复制粘贴（子图序列化为 delta）/对齐；`commands.rs` 接 03（逆 delta=revert）。
- 切片 1.3：`graph_editors/graph_node_palettes` 表接通物化（06 store）；`validate/compile_operation` 投影为图工具栏命令（08）。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（夹具 GraphModel：三约束矩阵/校验/delta 往返/粘贴幂等/撤销往返/required 端口悬空诊断）。更新 `docs/zircon_editor/ui/graph.md`。

### M2 时间轴基座与预览框架

- 切片 2.1：TimelineFoundation + `timeline_*` 两表接通（`value_kind`→lane 渲染器映射）。
- 切片 2.2：`PreviewScene`（依赖 04 M2）+ playback + subject；建/销 10 轮泄漏断言。
- 测试阶段：吸附边界/区段重叠规则/多轨选择单测；预览生命周期；手验播放控制。

### M3 动画编辑器与 Montage

- 切片 3.1：animation_editor 78 行骨架扩 toolkit（`AnimationEditorSession` 保留为 toolkit 的 session 层）；轨道时间轴+曲线视图（`ui/curve/` 落地，06 曲线字段编辑器同底）+ 预览联动。
- 切片 3.2：montage 资产模型（runtime 侧 + serde + 11 版本头）+ importer 接线；编辑器三层 lane；notify 与 runtime tick 契约对齐。
- 测试阶段：`cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked`（资产族往返 + tick 契约不回归）+ toolkit 测试；section 重排→运行序契约测试。

### M4 状态机与行为树

- 切片 4.1：absm 资产模型 + 五件套编辑器（双 GraphModel 联动 + ParameterPanel + 活跃态高亮经 02 `WatchKey::ComponentType`）；`validate_operation` 实装（悬空转移/不可达态诊断）。
- 切片 4.2：BT 资产模型 + 归属裁决（倾向新 `zircon_plugins/ai`，按 frameworks 计划 crate 化方向定）；Tree 实例 + 附着编辑；PIE 高亮接口预留。
- 测试阶段：absm 转移求值契约（runtime 侧）+ 图编辑撤销矩阵；BT 零改动验收断言；证据记状态节。

## 风险与开放问题

- 预览副 session 与 PIE 并存的图形资源预算：预览默认 30fps 上限纾解；04 设备共享风险同源，证据记状态节。
- montage/absm **求值器**是 runtime 侧计划外工作量——执行前与 `zircon_plugins/animation` owner 会签排期；编辑器按资产模型先行，预览联动降级 clip 级直至求值器落地。
- 附着子项经 palette category 约定承载是轻量方案；若 BT 实装时表达力不足（附着需独立端口/参数 schema），再提 `GraphNodeDescriptor` 扩字段的描述符演进案（11 迁移链配套），不预先扩。

## 产出记录与时间

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

| 日期 | 里程碑/切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-16 | Editor05 失败修复：viewport `SelectionModel` consumer hard cut | 代码完成 / review 0/0/0 / Coordinator 阻塞已修复，受管验证待执行 | 28 处 Workbench/binding 生产调用与 16 处 controller/test 调用已整体迁移到 `SelectionModel` active-domain API，controller 旧 getter/setter 已删除；非选择命令保持多选，删除保留存活集合，PIE 往返完整双域模型，history 为选择型命令保存有序 before/after snapshot。源码扫描除不同类型的 widget reflector 外为 0，`git diff --check` 与最终独立复审 `P0/P1/P2=0/0/0`；Coordinator01 已以 schema 41 完成 stale owner、绝对 expiry、orphan handoff 与 FIFO 生产回放，并回传 [fixed 已修复：stale-session-pending-cpu-reservation-starvation](07/fixed-2026-07-16-stale-session-pending-cpu-reservation-starvation.md)。当前仅保留 Editor07 current-source managed Cargo 验证门，未将尚未执行的 Rust 测试写成通过。详见 [子计划记录](07/2026-07-16-selection-model-consumer-hard-cut-output-records.md)。 |
| 2026-07-14 | Editor08 M1.2 回传修复：focused document kind 权威投影 | 已修复（fixed，2026-07-15） | `ViewDescriptor.document_kind` 成为 typed 领域 owner；session/workspace 无兼容字段地硬切到跨主页面和浮动窗口统一的 `focused_view`。补充修复 `None` 被默认 Scene 回退的问题后，runtime when 6/6、command/descriptor when 8/8、focused-owner hard-cut 1/1、Chrome typed projection 1/1 通过，详见 [fixed 回传](08/fixed-2026-07-15-command-eval-focused-document-projection.md)。 |
| 2026-07-13 | Editor09 M1 失败移交：动画资产打开测试夹具索引权威硬切 | 待修复（open） | animation/runtime/reflection 共 18 项失败已收敛到测试仍以未索引临时绝对路径派发 `OpenAsset`；当前入口正确要求 indexed `AssetTypeId`。修复要求见 [failure 交接](07/failure-2026-07-13-animation-asset-open-index-fixture-cutover.md)，禁止恢复 suffix toolkit 分派。 |
| 2026-07-14 | Text02 variable shaping 可见性编译失败回传 | 已修复（fixed） | Text02 已把旧 private flat helper 硬切到 folder-backed `shaping/horizontal/{backend,projection}`，并把 helper 可见性限制在 shaping 子系统；Editor paint fixture 同步采用含 `font_instance_id` 的规范 `ShapedGlyph`。Windows managed `text_horizontal_` 5/5 与本计划来源 exact 1/1（3172 filtered out）均通过，详见 [fixed 回传](07/fixed-2026-07-14-variable-shaping-visibility-compilation.md)。 |
| 2026-07-12 | Editor08 M1.2 失败移交：focused document kind 权威投影 | 已修复（fixed，2026-07-15） | Editor08 已落地 `FocusedDocumentKind(DocumentKind)` when 谓词并禁止由 project-open 猜测 scene focus；Editor07 完成 typed descriptor、唯一 `focused_view` 与浮动窗口焦点生命周期，最终 current-source 16/16 通过，详见 [fixed 回传](08/fixed-2026-07-15-command-eval-focused-document-projection.md)。 |
| 2026-07-14 | `engine-code-structure-convention` current structure audit | 已由 EditorUI10 修复并回传（2026-07-17） | `component_registry.rs` 与 `preferences.rs` 已物理删除并硬切为 folder-backed owner 树；Python audit 的迁移债/root owner violations 为 0，受管偏好 12/12、组件 1/1、结构 3/3 均 exit 0，独立复审 0/0/0。详见 [Editor07 fixed 回传记录](07/fixed-2026-07-17-ui-root-owner-boundary-migration-debt.md)。 |
| 2026-07-14 | Editor07 两项失败 current-source 第二轮上行门 | 未进入测试体 / 文本 owner 阻断 | 受管 Windows job `9cc782db74224c43887dfe73b46a4680` 在 focused-document exact 编译期产生 E0432 + E0063；本计划自有的 `EDITOR_MANAGER_NAME` 测试 import 已按唯一 `ui::host::module` owner 修正，不恢复 host-root re-export。剩余 E0063 是 retained paint-text fixture 构造 `ShapedGlyph` 时缺少已定稿 `font_instance_id`，已追加到 [EditorUI03 retained-text failure](../editor_ui/03/failure-2026-07-11-retained-text-family-and-subpixel-contracts.md)，日志 `.codex/tmp/editor07-focused-document-current-exact-r2-20260714.log`。Editor07 两个既有 failure 继续保持 open，禁止用未执行测试冒充通过。 |
- fixed 已修复：[irradiance-volume-shader-ide-validation-dependency](07/fixed-2026-07-15-irradiance-volume-shader-ide-validation-dependency.md)
