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

- fixed 已修复：[command-eval-scene-mode-selection-projection](08/fixed-2026-07-26-command-eval-scene-mode-selection-projection.md)
- 向 Editor07 移交（`open / viewport 单选兼容消费者硬切`）：[`07/failure-2026-07-16-viewport-selection-model-consumer-hard-cut.md`](07/failure-2026-07-16-viewport-selection-model-consumer-hard-cut.md)
- 向 Render04 移交（`open / viewport picking 缺少 runtime visible spatial query`）：[`../../zircon_runtime/render/04/failure-2026-07-18-viewport-picking-visible-spatial-query.md`](../../zircon_runtime/render/04/failure-2026-07-18-viewport-picking-visible-spatial-query.md)
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

**修正二：吸附与坐标系设置已存在**（v2「无吸附」失实）。当前 `SceneViewportSettings` 只持有持久化视口参数：`transform_space`、`grid_mode`、`projection_mode`、`view_orientation`、`gizmos_enabled` 与 `display_mode`；激活模式由 `SceneModeStack` 持有，transform handle 种类由 `SceneModeActivation::Transform(TransformHandleKind)` 表达，不再写入 settings。

**修正三：输入枚举已定型**。`ViewportInput` 9 变体（`viewport_input.rs:4-14`）：`PointerMoved/Left|Right|MiddlePressed|Released/Scrolled/Resized`——模式栈的输入词汇表现成。

### 真实缺口（重新定界）

1. **模式生命周期硬切已完成**：`SceneModeStack` 负责 enter/exit/update/input，`EditorExtensionRegistry` 只接受 descriptor + factory 的可执行 `SceneModeRegistration`；Select/Transform 与第三方模式走同一激活边界。
2. **单选模型**：选中是 `Option<NodeId>` 单值（03 命令族携带 + runtime session `selected_node`，01 已裁决删除）——无多选/框选/主选中。
3. **拖拽撤销走私有机制**：`GizmoDragState`（03 已裁决删除）而非事务。
4. **拾取仅射线**（projection.rs），无 id 缓冲精筛。
5. **hierarchy 视图只读**：无拖拽重排/就地重命名（`ui/layouts/views/hierarchy.rs`）。

## 目标

1. **模式栈**：`EditorSceneMode` trait + `SceneModeStack`（UE 栈式）；分派接通可执行 `scene_mode_registrations` 注册表；以 `SceneModeActivation` 统一 Select / Transform / 扩展模式激活，Move/Rotate/Scale 只作为 transform 私有 `TransformHandleKind`，拖拽状态由模式输入 effect 与 Editor03 事务共同闭环。
2. **`SelectionModel`**：顶层持有 Edit / Play 两个 `DomainSelection { items: IndexSet<EntityId>, primary, generation }`，并维护 `active_domain` 与跨域 `revision`；三视图（viewport/hierarchy/inspector）经 01 `FocusMessage` 同源；框选 + Ctrl/Shift 组合语义。
3. **事务化拖拽与多选操纵**：拖拽 = 03 `MergeMode::Ends` 长事务；多选质心操纵（`PivotMode`）。空间与 grid mode 仍由 `SceneViewportSettings` 持有；translate/rotate/scale snap 步长已迁入 17 `SettingsRegistry` 的 Project 域，并以 `SceneViewportSnapSteps` typed value projection 提供，不恢复 viewport settings 双重存储。
4. **Hierarchy 编辑闭环**：拖拽重排（`set_parent` 事务）、就地重命名、多选删除、搜索过滤、02 diff 刷新。
5. **id 缓冲拾取 + 高亮通道**：runtime extract 增 `PickIdExtract` 中性附件；`HighlightSet` 推送替换 01 的过渡通道。

## 非目标

- 地形/雕刻/顶点编辑（模式栈留位）；2D 专属模式；prefab 嵌套编辑（依赖 10）；相机书签/视口多联（editor_layout 辖）。

## 架构设计

### 模块布局

```
zircon_editor/src/scene/
  selection/             # SelectionModel + Edit/Play DomainSelection
  modes/
    editor_scene_mode.rs # EditorSceneMode trait
    scene_mode_stack.rs  # base + overlay stack
    scene_mode_ctx.rs    # 受限 selection/settings/effect 上下文
    scene_mode_activation.rs
    builtin_scene_mode.rs
  viewport/              # 八子模块保留；handles/ 全族保留为 transform_mode 的实现层
```

设计决策：**不拆三个变换模式**——Move/Rotate/Scale 共享同一交互生命周期（拾取轴→拖拽→提交），差异全在 `HandleTool` 实现内，已有 `HandleToolRegistry` 三字段即其选择器。`TransformSceneMode` 是一个模式，内部按 `TransformHandleKind` 选取手柄；模式栈层面的注册粒度是「Select / Transform / 第三方扩展」。当前模式仅由 `SceneModeStack` 持有，`SceneViewportSettings` 不保存 mode/tool 第二事实源。

### 关键类型

```rust
// modes/contract.rs
pub trait EditorSceneMode: Send {
    fn id(&self) -> &str;                          // SceneModeRegistry registration key
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

`SceneModeCtx` 只提供 active-domain `SelectionModel`、只读 `SceneViewportSettings`、单值 typed input effect 与 overlay invalidation signal；不暴露完整 controller/world。切换经 `SceneModeActivation`，处理者查 `SceneModeRegistry` 的可执行 registration 实例化；host 扩展入口必须同时提交 descriptor 与 factory。

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
| `SceneViewportTool` 枚举开关分派 | 删除；`SceneModeStack` 是 current mode 唯一事实源，`SceneModeActivation` 负责切换，`TransformHandleKind` 仅供 transform handle 实现层使用 |
| `HandleToolRegistry` 硬编码三字段 | 保留为 transform_mode 实现层（不对外）；对外扩展走 `SceneModeRegistration` |
| `dragging_gizmo`（01 批 3 暂存） | `transform_mode` 内 `DragTxn` 存在性即拖拽态 |
| `GizmoDragState/begin_drag/end_drag`（history.rs） | 03 M2 删除；本计划 M2 接管行为等价测试 |
| 命令族选中字段（03 摘除） | `SelectionModel` + `TransactionRecord.selection_*` |

### 深度测试

夹具模式（记录 enter/exit/input 序列）经 `SceneModeRegistration` 注入即可运行，`SceneModeStack`/分派零改动——注册制的直接验收。

## 里程碑

### M1 模式栈与 SelectionModel

- 切片 1.1：mode contract + 栈 + 分派接 `SceneModeRegistry`（首个消费者）；Select/Transform 两内建模式注册；旧枚举开关分派删除。
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
- `HandleTool` 族现为 `pub(crate)`/`pub(in ...)` 封闭可见性——transform_mode 若与 handles 同 crate 不同模块，需放宽到 `pub(crate)` 一级；不对 crate 外开放（第三方 gizmo 走 `SceneModeRegistration` 整模式粒度）。
- 多选 inspector 批量改值依赖 03 多 push——本计划只保证 SelectionModel 暴露多选集，批量路径归 06。
- 框选遮挡语义：默认选全部（无遮挡查询），设置项留位。

## 产出记录与时间

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

| 日期 | 里程碑/切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-01 | M1 SceneMode hard cut + M2 preview adapter | 实现完成 / 二次审查与 accepted open | 旧 viewport-tool enum/command/settings 已从 scoped source 删除；active/base mode 统一由 stack/activation 管理，custom registration 在 host 原子校验并安装可执行 factory，完整 id 贯穿 toolbar。Shift/Ctrl 点击与 screen-rect 框选已接 active-domain `SelectionModel`；mode/provider gizmo 进入 render/pointer 共用 interaction extract。handle 仅发布 transform preview，由 Editor03 transaction lane apply/record/finish，切换模式会先取消活动 transaction。前两批复审发现均已前向整改；二次审查、managed Cargo/UI、多选 pivot、Esc cancel 仍由 [open failure](05/failure-2026-07-31-scene-mode-input-ownership-hardcut.md) 跟踪。 |
| 2026-07-30 | Performance01 settings/viewport交接 | 静态9/9完成 / dynamic open | PERF-MVP-590确认snap-step单key命令仍clone完整registry并同步整层/整文件write+fsync；PERF-MVP-591确认stable `chrome_settings()`每次为三个静态key分配并做definition+三层BTree lookup，且viewport持有独立registry/load。Editor05只消费Editor17 immutable generation的typed resolved snap slots，stable frame parse/lookup=0；写入提交typed key/value+generation后立即read-your-write，UI filesystem wall=0，持久化由Runtime11有界ticket完成。keys 1/1K/100K、same-key 1/1M、60/120Hz与F4 snap交互仍待；证据见[settings current review](../../performance/01/2026-07-30-editor-core-settings-static-review.md)。 |
| 2026-07-30 | current-source editing/selection/gizmo/authoring复核 | 静态24/24完成 / dynamic open | typed `Arc` selection已让capture/restore为O(1)，PERF-MVP-549旧JSON/full-history结论失效；PERF-MVP-063仍确认transform/reparent/rename统一捕获`NodeEditState{name,parent,transform}`，连续transform复制稳定name且merge clone完整after。新增PERF-MVP-600记录hierarchy/inspector/render稳定读仍经gateway进入单`World` mutex、长DTO projection在锁内及transaction逐命令重锁；Editor05与Editor01/03/Runtime07共享immutable authoring generation，stable presentation lock=0、changed projection≤1 lock/generation、batch mutation≤1 lock/top-level operation。1/1k/100k drag仍需证明稳定name clone=0、setter=1、record=1，并完成undo/rollback与F4 gizmo交互。证据见[Performance01 current review](../../performance/01/2026-07-30-editor-core-editing-current-review.md)。 |
| 2026-07-23 | 性能移交：dynamic Inspector reflect generation/slot projection | 静态审查完成 / API迁移与F4门open | Interface reflect 15/15与合同测试1/1确认全owned schema/fields、缺generation/page/budget、field-name String；当前workbench snapshot按component clone schema+全fields并以嵌套find形成O(F²)。Editor05按PERF-MVP-567缓存Runtime13 catalog generation与compiled drawer field slots，只对affected fields投影；stable schema/full-field build=0，并与PERF-MVP-456共享selection/object generation，不保留第二Inspector schema truth。 |
| 2026-07-22 | 性能移交：World inspection generation projection | 局部中间工作止损 / F4生产启用门open | Runtime inspection 5/5已删除focus二次全表、parent BTreeMap与reflected field-name临时clone，但每次仍全量node project/sort、多容器subtree hash、全TypeRegistry fields；editor test-only projection再复制DTO并二次stats全扫。Editor05联动Editor02/Runtime07发布shared generation artifact+row/field delta，stable build/scan/clone=0，解除`cfg(test)`前完成规模/Cargo/F4门；见PERF-MVP-456与[open failure](05/failure-2026-07-22-world-inspection-generation-projection.md)。 |
| 2026-07-18 | 性能移交：viewport render packet禁止clone World | 局部hash止损完成 / generation extract待实现 | editor `build_render_snapshot`调用的兼容packet路径会先`Scene/World::clone()`整份场景以运行RenderExtract；多primitive共享transform hash已提到entity级。Editor05联动Runtime07让viewport复用generation-owned render-world artifact，idle/camera/selection分别精确失效，禁止保留clone-World双权威；World clone bytes=0、stable full extract=0，见PERF-MVP-349及[静态证据](../../performance/01/2026-07-18-runtime-core-framework-render-root-contracts-static-review.md)。 |
| 2026-07-18 | 性能修复：shared interaction extract + runtime render-mesh candidate source | 源码完成 / spatial broad phase 仍 open | `SceneViewportController` 已持有world/selection/settings/camera/viewport generation cache，render与pointer复用同一`Arc` handle/gizmo/render-mesh extract；旧pointer scene key、pointer-only gizmo adapter与`Scene::nodes()` renderable全扫已硬删除，runtime多primitive按owner折叠。Python契约TDD为RED 4/4→GREEN 4/4，Rust行为测试已落盘且rustfmt通过；camera/layer过滤extract并非BVH，Cargo/review/p95未完成，详见[本子计划](05/2026-07-18-viewport-shared-interaction-extract.md)与[open failure](05/failure-2026-07-18-viewport-pointer-candidate-regeneration.md)。 |
| 2026-07-18 | 性能移交：viewport pointer candidate generation | 局部止损完成 / 架构项待修复（open） | 性能审查已把增量SelectionModel全量clone删除，并让stable pointer generation在候选构建前early-out、route/debug单pass、ring hit-frame无临时Vec；剩余changed-generation的per-point matrix rebuild、全node candidate scan、render/pointer双份gizmo extract必须由本计划发布shared projection context与visible/BVH candidate generation。验收与禁止临时方案见[failure交接](05/failure-2026-07-18-viewport-pointer-candidate-regeneration.md)。 |
| 2026-07-18 | 性能移交：runtime picking resolved-frame/workspace | framework止损完成 / 产品验收待执行 | `core/framework/picking` 23/23静态审查已将per-pointer重复扫描/排序、owned hit二次clone、previous/release state深clone收敛；Editor05接入Runtime07/12后不得为route与debug各解析一次，也不得从workspace外另建hover/report。changed-generation primitive broad phase继续由既有Render04 visible spatial query failure负责；1/1k/100k candidates、stable move与debug on/off预算见PERF-MVP-332及[性能证据](../../performance/01/2026-07-18-runtime-core-framework-picking-static-review.md)。 |
| 2026-07-18 | 性能移交：camera scalar read与gizmo generation storage | framework止损完成 / 唯一owner与产品验收待执行 | dynamic camera action已从owned SceneNode projection硬切到`World::local_transform`；framework gizmo的per-endpoint matrix和circle temp Vec已止损，但该framework目前无生产caller。Editor05与Runtime07/Render17需收敛唯一overlay owner、generation-compiled retained geometry与clear/swap frame buffers，删除editor/runtime双实现后再用F4/RenderDoc验收；预算见PERF-MVP-333及[性能证据](../../performance/01/2026-07-18-runtime-core-framework-camera-gizmos-static-review.md)。 |
| 2026-07-16 | M1.1 基础切片：双域 SelectionModel、SceneModeStack 与 factory registry | 进行中 | 已新增 Edit/Play 双域有序选中权威、主选中/generation/revision 不变量、模式栈生命周期/栈顶输入分派，以及 descriptor-backed `SceneModeFactory/Registration/Registry`；viewport 旧存储和 controller 单选兼容 API 均已硬切删除，28 处 Editor07 生产 consumer 已直接迁移到 active-domain 模型，最终独立复审 `P0/P1/P2=0/0/0`，状态仍由 [Editor07 failure handoff](07/failure-2026-07-16-viewport-selection-model-consumer-hard-cut.md) 跟踪。旧 10/10 证据仍有效，但新增多选/PIE/history/源码守卫尚因 stale foreign reservation 未获受管 current-source gate；该 Coordinator01 failure 已移交，`Cargo.lock` 也仍含 foreign 依赖变化。全量 3217-test 超时不声明全绿；内建 Select/Transform 注册、生产 CommandEval 投影与插件 overlay provider 生命周期仍未完成，故不提升 M1.1 或父计划状态。详见 [本子计划记录](05/2026-07-16-m1-selection-mode-stack-output-records.md)。 |
| 2026-07-12 | Editor08 M1.2 失败移交：`SceneModeId` / `SelectionModel` 权威投影 | 待修复（open） | Editor08 已落地 `SceneModeActive` 与 `SelectionNonEmpty` when 谓词；当前宿主没有向 `CommandEvalCtx.scene_mode` 写入本计划模式栈权威值，`selection_count` 仍只是 inspector 是否存在的 0/1 临时投影。修复要求与静态复现证据见 [fixed 已修复：command-eval-scene-mode-selection-projection](08/fixed-2026-07-26-command-eval-scene-mode-selection-projection.md)。本行仅登记待修复，不声明本计划完成。 |
| 2026-07-13 | Navigation M6 插件 viewport overlay provider 宿主接线 | 待修复（open） | tool-mode provider registry/factory、每帧 extract 合并及 lifecycle/toggle 验收见 [failure 交接](05/failure-2026-07-13-plugin-viewport-overlay-provider-runtime-wiring.md)。 |
| 2026-07-18 | 产品overlay primitives性能交接 | 待执行 | Shaded wireframe全mesh展开已止损；selection仍O(S×M)，wire/gizmo/handle/icon每帧重建CPU vertices与GPU buffers。Editor05联动Render10/17以selection/overlay/model generation发布dense entity index、retained geometry与persistent arena，stable rebuild/upload/create=0；见PERF-MVP-333及primitives静态证据。 |
| 2026-07-18 | 产品overlay整目录补充交接 | 局部止损 / retained owner待执行 | Disabled sky与WireOnly LoadStore空工作已止损；selection/wire/grid/gizmo/handle仍最多5个pass且每frame重建buffer。Editor05只发布selection/overlay generation与dense entity/model index，Render10负责单pass/persistent arena；稳定frame不得因editor轮询重建geometry或GPU buffer。见PERF-MVP-333及overlay静态证据。 |
| 2026-07-22 | gizmo transaction delta性能交接 | operation-group key止损 / typed delta待执行 | PERF-MVP-063确认UpdateNode虽只写changed field，连续transform仍每update捕获并clone`NodeEditState{name,parent,transform}`，merge又clone完整after；operation-group稳定key clone已删除。Editor05只提交Transform before/current typed delta，最终commit一个record，稳定name String clone=0；1k/100k drag counter、undo/rollback与F4交互待验收。 |
| 2026-07-22 | viewport changed-generation gizmo scan增量止损 | 源码合同5/5 / spatial owner仍open | 当前128/128静态复核让gizmo scan先按kind过滤，非Camera/DirectionalLight不查询active，builder复用循环`&SceneNode`并删除重复lookup；PERF-MVP-222仍要求runtime visible/BVH query与generation-owned shared mesh snapshot，不能把kind-first线性scan当作完成。证据见[性能记录](../../performance/01/2026-07-22-editor-scene-current-incremental-review.md)与[open failure](05/failure-2026-07-18-viewport-pointer-candidate-regeneration.md)。 |

## Code Review 状态

2026-07-30 审查提出的 folder-backed 模块布局、双域 `SelectionModel` 和
SettingsRegistry snap projection 漂移已同步到本计划。2026-08-01 的
SceneMode hard-cut 删除了 `SceneViewportTool`、`SetTool` 和 controller
枚举分派；Select/Transform 已消费 primary pointer 并发布 inline effect。

直接 world transform 写入问题也已前向修复：handle tool 只计算
`ViewportTransformPreview`，workbench 在 Editor03 transaction lane 内 apply、
record 和 finish。当前剩余实现风险是 modifier-aware 框选、多选 pivot、Esc
cancel，以及 Editor03/Editor05 的 accepted managed gate；不再把旧 enum 或
direct-world 行为列为当前事实。
