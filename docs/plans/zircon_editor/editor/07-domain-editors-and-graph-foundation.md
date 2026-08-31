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
- 2026-07-18 pipeline report性能交接：Editor07不得按pane/UI tick从`RenderPipelineCompileReport`重建owned feature/pass/resource/diagnostic maps；按PERF-MVP-422消费generation-tagged compiled summary/detail Arc，if-newer更新并虚拟化大表。stable generation report build/deep clone=0，详情工作近visible rows，编译失败仍保留last-good与独立错误generation。
- 2026-07-18 Virtual Geometry diagnostics pane交接：runtime当前每VG camera frame无条件深建完整snapshot，Editor07不得以UI刷新率继续clone/全量投影。pane改为debug subscription generation+`query_if_newer` Arc handle，summary与可视detail分页/虚拟化，关闭pane立即停止诊断构建且history有界；见PERF-MVP-416。
- 2026-07-18 pipeline编辑器control-plane交接：Editor07对register/reload/profile切换只提交handle+revision generation请求，同revision pending去重并异步显示Ready/Error/last-good；不得在UI锁内等待validation compile或重复set。Render01/08发布validated revision artifact，见PERF-MVP-412。
- 2026-07-18 render diagnostics订阅交接：Editor07以viewport/report generation订阅`SealedRenderFrameDiagnostics`，summary只在if-newer时更新，graph/VG/UI/provider detail按可见pane和虚拟化范围请求；关闭pane即取消detail构建，不按UI tick深clone完整RenderStats或触发runtime重扫。见PERF-MVP-418并复用416。
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

## 现状与证据（zircon，2026-08-01 实读）

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

- `ui/animation_editor/` 已有 `graph/lifecycle/parameters/presentation/sequence/state_machine/support/tests` 八个 session 子模块，`AnimationEditorSession` 已直接承载 sequence/graph/state-machine 文档；仍缺共享 track list、curve、ruler、thumbnail 与 preview 联动。
- `ui/material_editor/`：`{mod, projection, renderer_data_projection}` 同级占位。
- `ui/asset_editor/`：最成熟样板（session/undo/journal/theme/presentation_state 齐备），session 生命周期范式取自它。
- `template_runtime/component_adapter` 已有资源编辑器/动画编辑器/检查器适配位。

### runtime 侧

`zircon_plugins/animation` 有 runtime 动画插件（`runtime_physics_animation_tick_contract` 契约测试在案）；importer 含 `import_animation_asset.rs`。`zircon_runtime::core::framework::animation::asset` 已有 `AnimationGraphAsset`、`AnimationSequenceAsset` 与 `AnimationStateMachineAsset`；仍缺 montage/BT 资产族，相关运行时求值能力仍需在实现切片前逐项核实和补齐。

### 缺口

无共享图画布/连线/布线基座；无共享时间轴实体；无预览场景框架；montage/BT 资产族缺失。动画编辑器已有私有 graph/state-machine session 逻辑，M1 必须先裁决其与未来共享基座的归属，避免重复实现。

## 目标

1. **GraphEditorFoundation**：画布（平移/缩放/网格）、节点（端口/标题/体区委托/**附着子项**）、连线（`value_type` 默认校验 + 领域覆写/正交或贝塞尔布线）、选择/框选/复制粘贴/对齐、迷你图；数据模型 `GraphModel` trait；编辑全走 03 事务；物化经 `graph_editors/graph_node_palettes` 表；`validate/compile_operation` 接 08 命令（图工具栏「校验/编译」按钮=操作投影）。
2. **TimelineFoundation**：标尺/播放头/轨道列表/关键帧+区段双元素/框选/吸附；`timeline_*` 两表注册，`value_kind` 决定 lane 渲染器。
3. **PreviewSceneFramework**（Persona 等价）：复用 04 副 session 机制 + 预览体注入 + `invalidate_views/focus_views`；动画族编辑器共享同一预览 session。
4. **动画编辑器成型**：在既有 `AnimationEditorSession` 与 graph/sequence/state-machine 子模块上形成 `DocumentToolkit` 实例，补齐 Fyrox 件清单中的 track_list+curve+ruler+thumb 与预览联动。
5. **Montage 编辑器**：资产模型（runtime 侧 `slots/sections（可重排跳转）/segments（引 clip）/notify tracks`，UE 词汇直译）；编辑器=三层 lane 时间轴实例。
6. **状态机编辑器**：复用并按需演进既有 `AnimationStateMachineAsset`/`AnimationGraphAsset`，不得平行新建 absm authority；编辑器复刻五件套（双 GraphModel 联动 + ParameterPanel + 活跃态高亮经 02 watch）。
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
zircon_editor/src/ui/animation_editor/   # 复用既有 session 子模块并补齐 toolkit
zircon_editor/src/ui/absm_editor/        # 新
zircon_editor/src/ui/behavior_tree_editor/  # 新
# graph/sequence/state-machine 复用 zircon_runtime 既有 animation::asset authority；montage/bt 归插件 runtime 资产族；编辑器只持创作投影
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
| absm | StateGraph(Free)+BlendTree(Dag) 双实例 | — | 必需（活跃态高亮） | 复用既有 `animation::asset::{graph,state_machine}` |
| 行为树 | Tree+附着子项 | — | PIE attach 预留 | animation 或新 ai 插件（M4 裁决） |
| 材质图（远期） | Dag | — | 材质球 | shader 计划集 |

### 深度测试

图基座以行为树（Tree+附着）为第二领域验收：接入时 `graph/` 零改动。时间轴以 montage（三层 lane）为第二领域验收（第一=动画 clip）。

## 里程碑

### M1 图基座

- 切片 1.1：`GraphModel` + canvas/node_widget/routing（渲染走 editor_ui 栈）；选择/框选/拖拽/缩放。
- 切片 1.2：连接校验（value_type 缺省 + 覆写）/复制粘贴（子图序列化为 delta）/对齐；`commands.rs` 接 03（逆 delta=revert）。
- 切片 1.3：`graph_node_palettes` descriptor 在 active catalog materialization 时绑定 package owner 与 schema version，随后接通 `graph_editors/graph_node_palettes` 表物化（06 store）；`validate/compile_operation` 投影为图工具栏命令（08）。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（夹具 GraphModel：三约束矩阵/校验/delta 往返/粘贴幂等/撤销往返/required 端口悬空诊断）。更新 `docs/zircon_editor/ui/graph.md`。

### M2 时间轴基座与预览框架

- 切片 2.1：TimelineFoundation + `timeline_*` 两表接通（`value_kind`→lane 渲染器映射）。
- 切片 2.2：`PreviewScene`（依赖 04 M2）+ playback + subject；建/销 10 轮泄漏断言。
- 测试阶段：吸附边界/区段重叠规则/多轨选择单测；预览生命周期；手验播放控制。

### M3 动画编辑器与 Montage

- 切片 3.1：在既有 `AnimationEditorSession` 及其 graph/sequence/state-machine 子模块上补齐 toolkit；增加轨道时间轴+曲线视图（`ui/curve/` 落地，06 曲线字段编辑器同底）+ 预览联动。
- 切片 3.2：montage 资产模型（runtime 侧 + serde + 11 版本头）+ importer 接线；编辑器三层 lane；notify 与 runtime tick 契约对齐。
- 测试阶段：`cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked`（资产族往返 + tick 契约不回归）+ toolkit 测试；section 重排→运行序契约测试。

### M4 状态机与行为树

- 切片 4.1：复用并补全既有 `AnimationStateMachineAsset`/`AnimationGraphAsset` + 五件套编辑器（双 GraphModel 联动 + ParameterPanel + 活跃态高亮经 02 `WatchKey::ComponentType`）；`validate_operation` 实装（悬空转移/不可达态诊断）。
- 切片 4.2：BT 资产模型 + 归属裁决（倾向新 `zircon_plugins/ai`，按 frameworks 计划 crate 化方向定）；Tree 实例 + 附着编辑；PIE 高亮接口预留。
- 测试阶段：absm 转移求值契约（runtime 侧）+ 图编辑撤销矩阵；BT 零改动验收断言；证据记状态节。

## 风险与开放问题

- 预览副 session 与 PIE 并存的图形资源预算：预览默认 30fps 上限纾解；04 设备共享风险同源，证据记状态节。
- montage 与状态机缺失的**求值能力**是 runtime 侧计划外工作量——执行前按 current source 逐项核实并与 `zircon_plugins/animation` owner 会签排期；编辑器复用既有资产模型先行，预览联动降级 clip 级直至求值器落地。
- 附着子项经 palette category 约定承载是轻量方案；若 BT 实装时表达力不足（附着需独立端口/参数 schema），再提 `GraphNodeDescriptor` 扩字段的描述符演进案（11 迁移链配套），不预先扩。

## 产出记录与时间

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

| 日期 | 里程碑/切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-24 | M1.3 图 palette provenance 基础 | 进行中 / 生产代码与静态门通过 / Cargo 行为门未执行 | `GraphNodePaletteDescriptor` 新增 schema version 与 effective owner；直接注册显式归属 `editor.extension.direct`，active plugin catalog materialization 覆盖为 package id，generation 重建时 inactive plugin palette 自然退出 catalog。两个注册入口均拒绝空 owner 与 version 0，并返回 `InvalidDescriptorSchemaVersion`；新增 descriptor owner/version 保留与非法 version 测试。`rustfmt --check --config skip_children=true`、`git diff --check`、旧并行 registry/锁/`Result<_, String>` 扫描通过；未运行 Cargo，且 per-node/pin registry、unknown-node migration、06 表物化和动画 `GraphModel` adapter 仍未完成，故 M1.3 保持进行中且不提交。 |
| 2026-07-18 | Performance failure：UI asset import physical generation cache | Editor07 import 子修复已实现 / parent failure 仍 open | 新 `UiAssetImportTraversal` 让 canonical physical source path 同时拥有单 generation 的成功/失败 read、parse、v2 projection cache key、实际读取目标与 parser mode；logical fragment aliases 逐条保留，physical expansion 去重终止 diamond/cycle，strict hydration 与 lossy refresh 共享 traversal。静态 TDD 4/4、rustfmt/diff check 与旧 visited 扫描通过；Rust tests 因共享 Cargo/source-bound 门未运行，delta projection、typing debounce、1k stress/p95/行为等价仍待完成。详见 [子计划记录](07/2026-07-18-ui-asset-import-physical-cache.md) 与 [open failure](07/failure-2026-07-17-ui-asset-editor-full-projection-and-import-rehydrate.md)。 |
| 2026-07-18 | Performance failure：sample-grid typed generation hard cut | Editor07 domain 修复已实现 / 下游 batch 与规模验证仍 open | 新 `ui::sample_grid::SampleGridGeneration` 在 attribute projection 阶段一次性生成预格式化 ticks、immutable points 及分离的 static/dynamic content generation；selection/drag 不再改变 static token，range 因同时改变 grid 与 point projection 而失效两个 token。host data 删除 9 个平行 ModelRc/raw 字段，painter 与既有视觉/投影测试整体迁移 typed slices，旧字段扫描为 0。静态合同 5/5、rustfmt/diff check 通过；Cargo 未运行，Render13 dashed/marker bounded batch 与 1/100/10,000 规模证据仍未完成，因此原 failure 保持 open。详见 [子计划记录](07/2026-07-18-sample-grid-generation-hardcut.md) 与 [open failure](07/failure-2026-07-17-sample-grid-command-amplification.md)。 |
| 2026-07-16 | Editor05 失败修复：viewport `SelectionModel` consumer hard cut | 代码完成 / review 0/0/0 / Coordinator 阻塞已修复，受管验证待执行 | 28 处 Workbench/binding 生产调用与 16 处 controller/test 调用已整体迁移到 `SelectionModel` active-domain API，controller 旧 getter/setter 已删除；非选择命令保持多选，删除保留存活集合，PIE 往返完整双域模型，history 为选择型命令保存有序 before/after snapshot。源码扫描除不同类型的 widget reflector 外为 0，`git diff --check` 与最终独立复审 `P0/P1/P2=0/0/0`；Coordinator01 已以 schema 41 完成 stale owner、绝对 expiry、orphan handoff 与 FIFO 生产回放，并回传 [fixed 已修复：stale-session-pending-cpu-reservation-starvation](07/fixed-2026-07-16-stale-session-pending-cpu-reservation-starvation.md)。当前仅保留 Editor07 current-source managed Cargo 验证门，未将尚未执行的 Rust 测试写成通过。详见 [子计划记录](07/2026-07-16-selection-model-consumer-hard-cut-output-records.md)。 |
| 2026-07-14 | Editor08 M1.2 回传修复：focused document kind 权威投影 | 已修复（fixed，2026-07-15） | `ViewDescriptor.document_kind` 成为 typed 领域 owner；session/workspace 无兼容字段地硬切到跨主页面和浮动窗口统一的 `focused_view`。补充修复 `None` 被默认 Scene 回退的问题后，runtime when 6/6、command/descriptor when 8/8、focused-owner hard-cut 1/1、Chrome typed projection 1/1 通过，详见 [fixed 回传](08/fixed-2026-07-15-command-eval-focused-document-projection.md)。 |
| 2026-07-13 | Editor09 M1 失败移交：动画资产打开测试夹具索引权威硬切 | 待修复（open） | animation/runtime/reflection 共 18 项失败已收敛到测试仍以未索引临时绝对路径派发 `OpenAsset`；当前入口正确要求 indexed `AssetTypeId`。修复要求见 [failure 交接](07/failure-2026-07-13-animation-asset-open-index-fixture-cutover.md)，禁止恢复 suffix toolkit 分派。 |
| 2026-07-14 | Text02 variable shaping 可见性编译失败回传 | 已修复（fixed） | Text02 已把旧 private flat helper 硬切到 folder-backed `shaping/horizontal/{backend,projection}`，并把 helper 可见性限制在 shaping 子系统；Editor paint fixture 同步采用含 `font_instance_id` 的规范 `ShapedGlyph`。Windows managed `text_horizontal_` 5/5 与本计划来源 exact 1/1（3172 filtered out）均通过，详见 [fixed 回传](07/fixed-2026-07-14-variable-shaping-visibility-compilation.md)。 |
| 2026-07-12 | Editor08 M1.2 失败移交：focused document kind 权威投影 | 已修复（fixed，2026-07-15） | Editor08 已落地 `FocusedDocumentKind(DocumentKind)` when 谓词并禁止由 project-open 猜测 scene focus；Editor07 完成 typed descriptor、唯一 `focused_view` 与浮动窗口焦点生命周期，最终 current-source 16/16 通过，详见 [fixed 回传](08/fixed-2026-07-15-command-eval-focused-document-projection.md)。 |
| 2026-07-14 | `engine-code-structure-convention` current structure audit | 已由 EditorUI10 修复并回传（2026-07-17） | `component_registry.rs` 与 `preferences.rs` 已物理删除并硬切为 folder-backed owner 树；Python audit 的迁移债/root owner violations 为 0，受管偏好 12/12、组件 1/1、结构 3/3 均 exit 0，独立复审 0/0/0。详见 [Editor07 fixed 回传记录](07/fixed-2026-07-17-ui-root-owner-boundary-migration-debt.md)。 |
| 2026-07-14 | Editor07 两项失败 current-source 第二轮上行门 | 未进入测试体 / 文本 owner 阻断 | 受管 Windows job `9cc782db74224c43887dfe73b46a4680` 在 focused-document exact 编译期产生 E0432 + E0063；本计划自有的 `EDITOR_MANAGER_NAME` 测试 import 已按唯一 `ui::host::module` owner 修正，不恢复 host-root re-export。剩余 E0063 是 retained paint-text fixture 构造 `ShapedGlyph` 时缺少已定稿 `font_instance_id`，已追加到 [EditorUI03 retained-text failure](../editor_ui/03/failure-2026-07-11-retained-text-family-and-subpixel-contracts.md)，日志 `.codex/tmp/editor07-focused-document-current-exact-r2-20260714.log`。Editor07 两个既有 failure 继续保持 open，禁止用未执行测试冒充通过。 |
| 2026-07-22 | Performance handoff：UI asset generation/delta projection | 待实现（PERF-MVP-082） | `zircon_editor/src/tests/editing` 40/40复核确认inspector/tree/style/preview/binding/theme的每个细粒度动作都能触发完整source/document replay、递归schema/表达式图/cascade projection，而夹具仅2–3 nodes/rules/imports。本轮只止损palette move完整catalog重建；本计划须以domain dirty generation和frame coalescing让同一generation的document→reflection→retained projection至多一次，并补1/100/10k nodes/rules/imports与125/500/1000 Hz typing/drag的build/visit/allocation/p95门。实现参考`dev/slint/internal/core/properties.rs`的`PropertyTracker::evaluate_if_dirty`，不得把cache分散到consumer私有authority。 |
- fixed 已修复：[irradiance-volume-shader-ide-validation-dependency](07/fixed-2026-07-15-irradiance-volume-shader-ide-validation-dependency.md)

## Code Review 同步结论 (2026-08-01)

### 已同步到主计划

- 动画编辑器现状、目标 4、M3.1 与模块布局已改为复用既有 session 子模块，不再以“78 行占位骨架”为前提。
- runtime 资产现状、目标 6、映射表与 M4.1 已改为复用既有 graph/sequence/state-machine authority，仅把 montage/BT 资产族和经 current-source 核实后仍缺的求值能力保留为待办。

### 实现风险 / 技术债

- `ui/graph/`、`ui/timeline/`、`ui/curve/` 与 `ui/preview_scene/` 已有共享基础。`GraphNodePaletteDescriptor` 已在 active catalog materialization 时绑定 package owner 与 schema version，catalog generation 替换即回收 inactive plugin palette；当前仍缺 animation/state-machine 的 concrete `GraphModel` adapter、per-node/pin descriptor registry 与 unknown-node migration、per-user graph layout document、06 descriptor-table materialization 与 retained-host canvas；`ui/absm_editor/`、`ui/behavior_tree_editor/` 仍未建立。既有 `session/graph.rs` 的语义 mutator 必须迁入统一 transaction/compiler 链，而不能继续作为并行图 authority。
- `ui/material_editor/` 仍为 `{mod, projection, renderer_data_projection}` 三文件占位，与「非目标：材质图编辑器（表可注册，待 shader/04 材质绑定契约稳定后立案）」一致，无需改动，仅提示映射表中材质图行继续标注为远期。
