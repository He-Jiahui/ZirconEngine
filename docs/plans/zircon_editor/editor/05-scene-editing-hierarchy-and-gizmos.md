---
related_code:
  - zircon_editor/src/scene/viewport/mod.rs
  - zircon_editor/src/scene/viewport/handles/handle_tool.rs
  - zircon_editor/src/scene/viewport/handles/handle_tool_registry.rs
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_input.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/ui/layouts/views/hierarchy.rs
  - zircon_runtime/src/scene/tests/authoring_boundary.rs
reference_sources:
  - dev/Fyrox/editor/src/interaction
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EdMode.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorModeManager.h
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md
status: in_progress
---

# 05 编辑场景 / Hierarchy / Gizmos

- 来自 Editor08 的失败交接（`open / SceneModeId 与 SelectionModel 权威投影`）：[`05/failure-2026-07-12-command-eval-scene-mode-selection-projection.md`](05/failure-2026-07-12-command-eval-scene-mode-selection-projection.md)
- 向 Editor07 移交（`open / viewport 单选兼容消费者硬切`）：[`07/failure-2026-07-16-viewport-selection-model-consumer-hard-cut.md`](07/failure-2026-07-16-viewport-selection-model-consumer-hard-cut.md)
- 向 Coordinator01 移交（`open / lifecycle orphan recovery 被 maintenance hold 阻断`）：[`../../zircon_tooling/session_coordinator/01/failure-2026-07-16-lifecycle-orphan-recovery-maintenance-hold-integrity-deadlock.md`](../../zircon_tooling/session_coordinator/01/failure-2026-07-16-lifecycle-orphan-recovery-maintenance-hold-integrity-deadlock.md)

本计划落地 00 §6 的「选中集」权威 `SelectionModel` 与场景交互层。

## 参照证据（dev/）

**Fyrox 交互模式家族**（`dev/Fyrox/editor/src/interaction/`）：select/move/rotate/scale/terrain 模式 + `gizmo/` 子模块。`InteractionMode` trait 以指针事件为轴（`on_left_mouse_button_down/up、on_mouse_move、update、deactivate`），模式注册进容器，经 `Message::SetInteractionMode(Uuid)` 切换。gizmo 是模式的视觉+拾取代理。

**UE 模式栈与工具框架**（`EdMode.h`/`EditorModeManager.h`）：`FEdMode` 钩子族（MouseMove/InputKey/InputAxis/InputDelta/Render），`FEditorModeTools` 持**活跃模式栈**路由视口输入；HitProxy id 缓冲拾取；新 ITF 中 gizmo 操纵**自动产生事务**。吸收：模式是栈不是单值；gizmo 与撤销自动挂钩。

**边界约束**（继承 runtime/05）：`SERIALIZED_AUTHORING_TOKENS`（`authoring_boundary.rs:3-13`）含 `gizmo/grid_mode/overlay/display_mode/...`——选中、gizmo、overlay、编辑相机不得进 runtime 序列化，双 token 守卫在案。

## 现状与证据（zircon，2026-07-05 实读）

### 视口交互栈比 v2 记载更完整（三处修正）

`viewport/` 八子模块（`mod.rs`）：`controller / edit_mode_projection(cfg(test)) / handles / interaction / pointer / projection / render_packet / settings`。

**修正一：三个变换工具已完整实现**。`handles/` 共 21 文件：`HandleTool` trait（`handle_tool.rs:10-20`，实读签名如下）+ `MoveHandleTool/RotateHandleTool/ScaleHandleTool` 三实现（各带 `*_behavior.rs`）+ `HandleToolRegistry`（`handle_tool_registry.rs:7-10`，**硬编码三字段** `move_tool/rotate_tool/scale_tool`，非开放注册表）+ `TransformHandleDragSession/HandleDragSession` 拖拽会话族：

```rust
pub(crate) trait HandleTool {
    fn build_overlay(&self, ctx: &HandleBuildContext<'_>) -> Option<HandleOverlayExtract>;
    fn begin_drag(&self, ctx: &HandlePickContext<'_>, axis: GizmoAxis) -> Option<HandleDragSession>;
    fn update_drag(&self, session: &mut HandleDragSession, ctx: &HandleDragContext<'_>) -> Option<Transform>;
    fn end_drag(&self, session: HandleDragSession);
}
```

**修正二：吸附与坐标系设置已存在**（v2「无吸附」失实）。`SceneViewportSettings`（`settings.rs:41-51`）：`tool: SceneViewportTool{Drag,Move,Rotate,Scale}`、`transform_space: TransformSpace{Local,Global}`、`grid_mode: GridMode{Hidden,VisibleNoSnap,VisibleAndSnap}`、`translate_step/rotate_step_deg/scale_step: Real`、`projection_mode/view_orientation/gizmos_enabled/display_mode`。

**修正三：输入枚举已定型**。`ViewportInput` 9 变体（`viewport_input.rs:4-14`）：`PointerMoved/Left|Right|MiddlePressed|Released/Scrolled/Resized`——模式栈的输入词汇表现成。

### 真实缺口（重新定界）

1. **工具切换是枚举开关，非模式生命周期**：`SceneViewportTool` 四值 + `HandleToolRegistry` 硬编码——无 enter/exit、无栈、无第三方扩展路径；`EditorExtensionRegistry.viewport_tool_modes` 注册表（`editor_extension.rs:16-31`）存在但**无消费者**。
2. **单选模型**：选中是 `Option<NodeId>` 单值（03 命令族携带 + runtime session `selected_node`，01 已裁决删除）——无多选/框选/主选中。
3. **拖拽撤销走私有机制**：`GizmoDragState`（03 已裁决删除）而非事务。
4. **拾取仅射线**（projection.rs），无 id 缓冲精筛。
5. **hierarchy 视图只读**：无拖拽重排/就地重命名（`ui/layouts/views/hierarchy.rs`）。

## 目标

1. **模式栈**：`EditorSceneMode` trait + `SceneModeStack`（UE 栈式）；分派接通 `viewport_tool_modes` 注册表（首个消费者）；`SceneViewportTool` 枚举开关与 `HandleToolRegistry` 硬编码结构**收编为注册条目**（Drag→Select 模式，Move/Rotate/Scale→变换模式包装既有三工具）；`dragging_gizmo` 收编为模式内部状态。
2. **`SelectionModel`**：`{ items: IndexSet<EntityId>, primary: Option<EntityId>, generation: u64 }` 双域（Edit/Pie）；三视图（viewport/hierarchy/inspector）经 01 `FocusMessage` 同源；框选 + Ctrl/Shift 组合语义。
3. **事务化拖拽与多选操纵**：拖拽 = 03 `MergeMode::Ends` 长事务；多选质心操纵（`PivotMode`）；**吸附/空间设置不新建**——`SceneViewportSettings` 既有字段保留为事实源，17 落地后其持久化迁 SettingsRegistry（Project 域）。
4. **Hierarchy 编辑闭环**：拖拽重排（`set_parent` 事务）、就地重命名、多选删除、搜索过滤、02 diff 刷新。
5. **id 缓冲拾取 + 高亮通道**：runtime extract 增 `PickIdExtract` 中性附件；`HighlightSet` 推送替换 01 的过渡通道。

## 非目标

- 地形/雕刻/顶点编辑（模式栈留位）；2D 专属模式；prefab 嵌套编辑（依赖 10）；相机书签/视口多联（editor_layout 辖）。

## 架构设计

### 模块布局

```
zircon_editor/src/scene/
  selection.rs           # SelectionModel（双域）
  modes/
    mod.rs
    contract.rs          # EditorSceneMode trait + SceneModeStack + SceneModeCtx
    select_mode.rs       # 点击/框选/组合键（Drag 工具语义并入）
    transform_mode.rs    # 单模式包装三 HandleTool（按 settings.tool 选取），非三个模式
  viewport/              # 八子模块保留；handles/ 全族保留为 transform_mode 的实现层
```

设计决策：**不拆三个变换模式**——Move/Rotate/Scale 共享同一交互生命周期（拾取轴→拖拽→提交），差异全在 `HandleTool` 实现内，已有 `HandleToolRegistry` 三字段即其选择器。`transform_mode.rs` 是一个模式，内部按 `SceneViewportTool` 切工具；模式栈层面的注册粒度是「Select / Transform / 第三方扩展」。

### 关键类型

```rust
// modes/contract.rs
pub trait EditorSceneMode: Send {
    fn id(&self) -> &str;                          // viewport_tool_modes 注册 key
    fn enter(&mut self, ctx: &mut SceneModeCtx);
    fn exit(&mut self, ctx: &mut SceneModeCtx);
    fn handle_input(&mut self, input: &ViewportInput, ctx: &mut SceneModeCtx) -> InputOutcome;
    fn update(&mut self, ctx: &mut SceneModeCtx);
    fn build_overlay(&self, out: &mut ViewportOverlayBuilder);  // HandleOverlayExtract 通道复用
}
pub enum InputOutcome { Consumed, PassThrough }

pub struct SceneModeStack {
    base: Box<dyn EditorSceneMode>,                // 常驻（默认 Select）
    overlays: Vec<Box<dyn EditorSceneMode>>,       // 临时压栈
}
// 输入自栈顶向下，Consumed 即止（FEditorModeTools 语义）
```

`SceneModeCtx` 提供：`SelectionModel` 可变引用、03 事务引擎、01 gateway、projection 工具、`SceneViewportSettings` 引用（吸附/空间事实源）。切换经 `ModeMessage::SceneModeChanged`，处理者查 `viewport_tool_modes` 实例化。

```rust
// transform_mode.rs 拖拽会话（替换 TransformHandleDragSession 的提交半段，拾取半段复用）
struct DragTxn {
    txn: ActiveTransaction,        // 03 MergeMode::Ends
    pivot: PivotMode,              // Primary | Centroid
}
// begin_drag: HandleTool::begin_drag 成功 → engine.begin("Transform", doc) 
// update_drag: HandleTool::update_drag 产 Transform → 对多选每实体 push set_transform（try_merge 吸收）
// end_drag: HandleTool::end_drag → txn.commit()；Esc → txn.cancel()（新增：拖拽中断=撤销到拖前）
```

### 拾取双通道

- 粗筛：projection.rs 射线（保留，CPU 零延迟）。
- 精筛：runtime extract `PickIdExtract`（实体 id → R32Uint 附件，命名不含 authoring token）；点击 → 像素 id 请求 → 一帧后读回修正选中（点击语义可容忍）。
- 高亮：编辑器每帧 `HighlightSet(Vec<EntityId>)` 经 gateway `push_editor_overlay`（01 定义的通道正式化），runtime 只见 id 集不见「选中」概念。

### Hierarchy 闭环

数据源=02 `WatchKey::Subtree` + `subtree_hash` diff。行操作映射：拖放→`set_parent` 事务（跨层拖放多实体=同事务多 push，循环父子由 `set_parent_checked` 既有校验拒绝）；F2/双击→`rename_node`；Del→多选同事务 `delete_node`（最后相机守卫在命令内，事务整体 cancel 并 toast）；过滤为视图态不入序列化。

### 现物迁移映射

| 现物 | 去向 |
| --- | --- |
| `SceneViewportTool` 枚举开关分派 | `SceneModeStack` 分派；枚举保留为 `transform_mode` 内部工具选择器与 settings 字段 |
| `HandleToolRegistry` 硬编码三字段 | 保留为 transform_mode 实现层（不对外）；对外扩展走 `viewport_tool_modes` |
| `dragging_gizmo`（01 批 3 暂存） | `transform_mode` 内 `DragTxn` 存在性即拖拽态 |
| `GizmoDragState/begin_drag/end_drag`（history.rs） | 03 M2 删除；本计划 M2 接管行为等价测试 |
| 命令族选中字段（03 摘除） | `SelectionModel` + `TransactionRecord.selection_*` |

### 深度测试

夹具模式（记录 enter/exit/input 序列）经 `viewport_tool_modes` 注入即可运行，`SceneModeStack`/分派零改动——注册制的直接验收。

## 里程碑

### M1 模式栈与 SelectionModel

- 切片 1.1：`modes/contract.rs` + 栈 + 分派接 `viewport_tool_modes`（首个消费者）；Select/Transform 两内建模式注册；旧枚举开关分派删除。
- 切片 1.2：`SelectionModel` 落地接 `FocusMessage`；三视图订阅同源；框选（屏幕矩形投影求交）+ Ctrl/Shift 语义。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（viewport/handles 既有测试须过；栈路由矩阵：Consumed/PassThrough/压退栈；三视图选中一致性；框选命中集合断言）。更新 `docs/zircon_editor/scene/modes.md`。

### M2 事务化拖拽与多选操纵

- 切片 2.1：`DragTxn` 接 03 长事务（依赖 03 M2）；Esc 中断=cancel；多选质心 `PivotMode`；`GizmoDragState` 行为等价测试接管。
- 测试阶段：拖拽 100 帧→历史 1 条（03 共用断言）；Esc 中断→世界回拖前态；质心计算与步进吸附边界值单测（projection 既有测试扩展）；手验四工具切换手感。

### M3 Hierarchy 闭环

- 切片 3.1：拖拽重排/就地重命名/多选删除（全走事务）+ 搜索过滤。
- 切片 3.2：02 diff 刷新接线（依赖 02 M2）。
- 测试阶段：行操作→事务→撤销往返矩阵；循环父子拒绝；多选删除含最后相机→整体 cancel；5k 节点过滤延迟基线记状态节。

### M4 id 缓冲拾取与高亮通道

- 切片 4.1：runtime `PickIdExtract` + 读回接口（render 计划集 owner 会签；`cargo test -p zircon_runtime --lib --locked`，extract 中性守卫走 token 白名单裁决）。
- 切片 4.2：点击精筛接线 + `HighlightSet` 正式化（替换 01 过渡推送）。
- 测试阶段：射线 vs id 缓冲同点击对账测试；证据记状态节。

## 风险与开放问题

- `PickIdExtract` 触碰渲染框架，会签被否则 M4 降级 CPU BVH 精筛（projection.rs 扩展）。
- `HandleTool` 族现为 `pub(crate)`/`pub(in ...)` 封闭可见性——transform_mode 若与 handles 同 crate 不同模块，需放宽到 `pub(crate)` 一级；不对 crate 外开放（第三方 gizmo 走 `viewport_tool_modes` 整模式粒度）。
- 多选 inspector 批量改值依赖 03 多 push——本计划只保证 SelectionModel 暴露多选集，批量路径归 06。
- 框选遮挡语义：默认选全部（无遮挡查询），设置项留位。

## 产出记录与时间

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

| 日期 | 里程碑/切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-16 | M1.1 基础切片：双域 SelectionModel、SceneModeStack 与 factory registry | 进行中 | 已新增 Edit/Play 双域有序选中权威、主选中/generation/revision 不变量、模式栈生命周期/栈顶输入分派，以及 descriptor-backed `SceneModeFactory/Registration/Registry`；viewport 旧存储和 controller 单选兼容 API 均已硬切删除，28 处 Editor07 生产 consumer 已直接迁移到 active-domain 模型，最终独立复审 `P0/P1/P2=0/0/0`，状态仍由 [Editor07 failure handoff](07/failure-2026-07-16-viewport-selection-model-consumer-hard-cut.md) 跟踪。旧 10/10 证据仍有效，但新增多选/PIE/history/源码守卫尚因 stale foreign reservation 未获受管 current-source gate；该 Coordinator01 failure 已移交，`Cargo.lock` 也仍含 foreign 依赖变化。全量 3217-test 超时不声明全绿；内建 Select/Transform 注册、生产 CommandEval 投影与插件 overlay provider 生命周期仍未完成，故不提升 M1.1 或父计划状态。详见 [本子计划记录](05/2026-07-16-m1-selection-mode-stack-output-records.md)。 |
| 2026-07-12 | Editor08 M1.2 失败移交：`SceneModeId` / `SelectionModel` 权威投影 | 待修复（open） | Editor08 已落地 `SceneModeActive` 与 `SelectionNonEmpty` when 谓词；当前宿主没有向 `CommandEvalCtx.scene_mode` 写入本计划模式栈权威值，`selection_count` 仍只是 inspector 是否存在的 0/1 临时投影。修复要求与静态复现证据见 [failure 交接](05/failure-2026-07-12-command-eval-scene-mode-selection-projection.md)。本行仅登记待修复，不声明本计划完成。 |
| 2026-07-13 | Navigation M6 插件 viewport overlay provider 宿主接线 | 待修复（open） | tool-mode provider registry/factory、每帧 extract 合并及 lifecycle/toggle 验收见 [failure 交接](05/failure-2026-07-13-plugin-viewport-overlay-provider-runtime-wiring.md)。 |
