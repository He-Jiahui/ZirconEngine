---
title: Editor Scene Viewport Interaction Controller、Input、Picking、Selection、Highlight、Gizmo Transaction、Cancel 与 Generation Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor59
review_date: 2026-08-21
baseline_head: be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1
baseline_epoch: 336
related_code:
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/scene/modes
  - zircon_editor/src/scene/selection
  - zircon_editor/src/ui/workbench/state/editor_state.rs
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport
  - zircon_editor/src/ui/retained_host/app/native_keyboard_actions.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell/runtime.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
  - zircon_editor/src/core/gateway
  - zircon_runtime/src/core/framework/render/viewport_highlight_store.rs
  - zircon_runtime/src/core/framework/render/visible_spatial_query.rs
  - zircon_runtime/src/graphics/visibility/spatial_query.rs
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime/src/scene/level_system.rs
tests:
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/pointer_bridge.rs
  - zircon_editor/src/tests/host/retained_window/platform_input_translation.rs
  - zircon_editor/src/tests/gateway/in_process.rs
  - zircon_runtime/src/dynamic_api/session/tests/highlight_set.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/53-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/47-runtime-picking-pointer-ray-hit-hover-drag-drop-event-backend-product-integration-review.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/05/failure-2026-08-19-gizmo-world-space-interactive-transaction.md
  - docs/plans/zircon_runtime/runtime/10/failure-2026-08-19-highlight-set-runtime-frame-consumption.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorViewportClient.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorViewportClient.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorModeManager.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorModeManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Public/InputRouter.h
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Private/InputRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Public/BaseGizmos/TransformProxy.h
  - dev/UnrealEngine/Engine/Source/Runtime/InteractiveToolsFramework/Private/BaseGizmos/TransformProxy.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.h
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_gizmos.h
  - dev/godot/editor/scene/3d/node_3d_editor_gizmos.cpp
  - dev/Fyrox/editor/src/interaction/mod.rs
  - dev/Fyrox/editor/src/interaction/move_mode.rs
  - dev/Fyrox/editor/src/interaction/rotate_mode.rs
  - dev/Fyrox/editor/src/interaction/scale_mode.rs
  - dev/bevy/crates/bevy_picking/src/pointer.rs
  - dev/bevy/crates/bevy_picking/src/events.rs
  - dev/bevy/crates/bevy_picking/src/mesh_picking/mod.rs
  - dev/bevy/crates/bevy_picking/src/mesh_picking/ray_cast/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Batching/InstanceCullingBatcher.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Culling/InstanceCuller.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Culling/InstanceOcclusionCuller.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Scene Viewport Interaction Controller、Input、Picking、Selection、Highlight、Gizmo Transaction、Cancel 与 Generation Product Integration 当前源码工程化差距

## 1. 结论

当前Scene Viewport交互并非空壳。Scene Mode stack能够先于builtin处理输入；selection模型支持Edit/Play双域与有序多选；Handle controller只返回preview，由workbench负责写Scene并在释放时折叠成一个undoable command；Escape、window focus loss和显式命令已经能到达`CancelInteraction`；point picking也已采用renderer-visible spatial query缩小owner集合。2026-08-21的当前源码还补上了全量active selection的`HighlightSet` DTO和多选位置聚合的Frame Selection。这些底座应保留。

但交互终止链现在存在三条可达P0。第一，底层UI明确产生Pointer Cancel并释放capture，viewport bridge却把Cancel映射成`None`；聚焦测试甚至要求“不生成viewport command”，因此触摸取消或capture loss不会回滚正在写Scene的Gizmo preview。第二，controller只有一个无owner的`drag: Option<_>`；Gizmo拖动期间按下右键或中键会覆盖Handle drag，workbench随后把“handle不再active”解释为正常结束并立即提交变换，尽管主键尚未释放。第三，highlight作为可选overlay在base scene extract之前提交；任何gateway/capability/protocol错误都会让`render_frame_submission()`返回`None`，host又消费`render_dirty`，于是一次高亮失败可以阻止整个Scene帧并冻结旧图。

Picking和Highlight的新接线也没有形成单一产品authority。renderer query只做可见实例包围球ray broad phase，Editor最终仍用mesh transform原点和scale推导屏幕圆；box selection遍历全量代理圆，Frame Selection只合并节点世界位置。三条用户操作分别依据“可见包围球”“全量原点圆”“节点位置点集”，不是同一代Selectable Spatial Product。Runtime picking adapter构造的`PointerInput`和debug feed仍无人消费；HighlightSet已经存入`LevelSystem`，dynamic frame extract却不读取它，cache key也没有overlay revision。存在DTO、store和测试不等于产品已经显示高亮或拥有真实Picking状态机。

本报告新增登记 **3项P0、8项P1、6项P2与36个资格门**。Editor03、Editor53、Editor58、Runtime47及两份开放failure中已经拥有的world-space多选Gizmo、通用capture、multi-viewport、Picking frame authority和runtime highlight consumption只做current-source重验，不重复计数。当前结论为`review complete / implementation not started`；没有运行Cargo、GUI、真实GPU picking、触摸/笔、多窗口、父子非均匀缩放、fault injection、soak或benchmark，不能据此宣称性能或表现优于Unreal。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 证据等级 | fingerprint |
|---|---:|---|---|
| `scene/viewport/**` | **116 / 6,926 / 6,314 / 239,804 / 38** | E3，逐文件读取controller、handle、projection、extract、pointer与render packet | `39a0740ad6ef6fbb71618b030cacef974343e67b2e180f2ef5578bdf3e6a031f` |
| Scene Mode + Selection | **24 / 2,341 / 2,054 / 70,747 / 24** | E3，registry/stack/isolation/context与selection domain/mutation | `7aae5ed6537f9af44180db25115edc815699e78c53f772967f180e544a3df7f6` |
| Workbench、host bridge与focused tests | **16 / 2,683 / 2,452 / 96,971 / 40** | E3，input translation、capture cancel、transaction、render dirty | `d396670dc2c371859a959eb346e675fd94c10d789f3c013611784543fa34cb34` |
| Gateway + Runtime integration | **14 / 2,927 / 2,546 / 99,442 / 27** | E3，highlight store/session、dynamic extract cache与visible query | `a832a90531256d52e35ff62ada6d0bae4ce35845ff81a8cca44c6c8dd8cb5fd7` |
| Zircon合计 | **170 / 14,877 / 13,366 / 506,964 / 129** | E3静态证据；未执行测试 | `b1ea75a64ffdc616bacdd1fc7a35f0a5566eeb444ea813e8aab5fb42d6807643` |
| 五引擎参考切片 | **23 / 31,789 / 26,957 / 1,213,892 / 0** | E2/E3，交互、transform、picking与per-view render product | `43a2b678b3e1f347c9c29613f4c27b18e86c7ad7488fdba18cb7c9b5ca65a567` |

fingerprint算法与Editor58一致：按normalized lowercase relative path排序，把`path + NUL + lowercase per-file SHA-256 + LF`串联后再取SHA-256。它只证明本轮读到的working-tree集合，不是ABI、cache key或验收receipt。

冻结Git基线为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator baseline epoch为336。Godot、Fyrox、Bevy、Unity Graphics参考revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal镜像随主仓基线和参考aggregate fingerprint冻结。

170份Zircon语料中有17份非本轮产生的dirty文件，包括controller/accessors/camera/frame selection、interaction extract、renderer-visible source、render packet、host focus/cancel bridge、workbench viewport/render state与`LevelSystem`。本报告审查当前working tree，不覆盖这些改动；实施前必须重算fingerprint，并重新检查P0事件序列是否仍可达。

### 2.2 当前产品链

```text
OS/window pointer + keyboard/focus
  -> retained UI input pump / UiPointerDispatcher capture
  -> EditorViewportEvent / ViewportCommand
  -> EditorState::handle_viewport_input
  -> SceneViewportController::handle_input
  -> SceneModeStack -> builtin selection/navigation/handle
  -> ViewportFeedback::transform_preview
  -> Scene::update_transform + GizmoTransactionCapture
  -> release/cancel -> history command or rollback

renderable pointer
  -> interaction extract + renderer-visible snapshot
  -> visibility-sphere query_ray
  -> editor origin/scale circle score
  -> private route; runtime PointerInput/debug payload not consumed

selection highlight
  -> build_runtime_highlight_set
  -> gateway -> LevelSystem::ViewportHighlightStore
  -X dynamic/editor frame extract consumption
```

### 2.3 已有基础必须保留

1. 保留Scene Mode stack的top-down dispatch、PassThrough checkpoint与plugin panic isolation。
2. 保留selection的domain、ordered items、primary和revision；修复应增加qualified scope，不回退单选。
3. 保留Handle controller只产生preview DTO、workbench拥有Scene mutation与history的边界。
4. 保留显式`CancelInteraction`命令及现有rollback测试，把Pointer Cancel/capture owner接入同一入口。
5. 保留renderer-visible immutable spatial snapshot和event-time broad phase；升级精确几何阶段，不退回每次全Scene扫描。
6. 保留Handle、Scene Gizmo、Renderable统一priority resolution的方向，但最终结果必须来自真实Picking authority。
7. 保留HighlightSet的per-viewport latest-value思路和generation拒旧，补独立overlay revision、consumer与teardown。
8. 保留interaction extract cache与profile counter；目标是稳定spatial product和预算，不是删除缓存后重新堆同步扫描。

## 3. 旧报告current-source校正与唯一owner

| 旧条目 / owner | 当前源码事实 | 剩余差距与裁决 |
|---|---|---|
| Editor03 P1-11，多选下游primary-only | `HighlightSet`已遍历`active_items()`；Frame Selection已遍历全部active items | 两项旧描述部分过时；Gizmo仍单node，highlight未被frame消费，frame只用位置点；开放failure与本报告分别继续跟踪 |
| Editor03 P1-16，Frame Selection只看primary | 当前`selection_frame`合并多选世界位置min/max | 不再是primary-only，但仍无mesh/component/spatial bounds，不能关闭P1-16 |
| Editor03 P1-17，输入无cancel | Escape、focus loss和command已有`CancelInteraction` | Pointer Cancel被host明确吞掉，输入仍无pointer/window/device/capture identity；本报告P0-01是当前产品adapter owner |
| Editor03 P1-20至P1-24 | origin-circle picking、硬编码identity、error吞没、全量box proxy、相邻去重仍存在 | 全部继承，不在本报告重复计数；Editor03继续拥有Scene picking UX与精确命中 |
| Editor03 P1-12/13/14/15 | local transform、preview直写、固定Move label、单node session仍存在 | `failure-...gizmo-world-space-interactive-transaction.md`保持fixing owner；本报告只要求输入终止receipt接入该事务 |
| Editor53 P1-19至P1-24 | ToolScheduler仍不是viewport capture authority | Editor53拥有通用tool/capture/terminal lifecycle；本报告拥有Scene Viewport具体input adapter与negative regression |
| Editor58 | viewport session、pane product、frame currentness与multi-viewport仍未建立 | identity与stale/presentation恢复归Editor58；本报告只消费qualified viewport/session identity |
| Runtime47 | runtime `PointerInput`、event state与backend没有产品caller | 本报告确认Editor adapter仍只读取private route，`runtime_input`/debug feed未进入frame authority；Runtime47保持唯一owner |
| Runtime10开放highlight failure | `ViewportHighlightStore`有producer和getter，dynamic extract无consumer，cache key无overlay | Runtime10负责runtime frame consumption；本报告负责Editor不能因overlay提交失败阻断base frame |

## 4. P0：必须先关闭的当前可达错误

### ED59-P0-01 · Pointer Cancel释放UI capture却被明确阻止进入viewport transaction

平台touch取消已被翻译成带`pointer_id`的`UiPointerEventKind::Cancel`，viewport `UiPointerDispatcher`也注册了Cancel并返回handled。可是在`map_pointer_route_to_viewport_event`中，Cancel固定返回`None`，`dispatch_viewport_pointer_event`因此直接返回空effects。更严重的是测试`shared_viewport_pointer_bridge_routes_cancel_to_capture_without_viewport_command`明确断言journal数量不变，把错误行为固化成false-green。

Gizmo preview在pointer move时已经执行`Scene::update_transform`。当OS取消触摸、设备断开或capture被强制终止时，UI capture消失，controller和`GizmoTransactionCapture`却收不到Cancel/OwnerLost，也没有primary release保证到达。必须让host route输出qualified `CancelInteraction { viewport, pointer, capture_generation, reason }`，由interactive edit owner同步rollback并返回terminal receipt；UI capture只有在receipt或强制隔离后才能退休。原测试必须反转为端到端rollback测试。

### ED59-P0-02 · 单一drag slot让第二按键覆盖Handle，并把未释放Gizmo误提交

`SceneViewportState`只有一个`Option<ViewportDragSession>`，session不含pointer、button、window、viewport、capture generation或owner。`RightPressed`和`MiddlePressed`无条件覆盖`state.drag`；`RightReleased`与`MiddleReleased`又无条件清空它。

可复现序列为：主键开始Handle并移动，workbench记录`was_handle_drag=true`；主键尚未释放时按右键，controller把Handle替换为Orbit；返回workbench后`is_handle_drag=false`，于是`finish_gizmo_transaction()`把当前preview提交为正式command。右键释放再清空Orbit，之后主键释放没有Handle可结束。用户没有执行正常accept动作，变换却已提交。中键、selection drag和camera drag之间也有同类覆盖。

必须用capture-owned interaction state machine替代无主drag slot。每条press/release/cancel按qualified pointer/button/capture generation匹配；第二按键由明确的chord policy决定拒绝、并行navigation或先cancel，绝不能依靠“variant变了”推断Accept。Commit只能来自owner的typed terminal disposition，Cancel/OwnerLost必须rollback。

### ED59-P0-03 · 可选Highlight提交失败会压掉base Scene帧并消费render dirty

`EditorState::render_frame_submission`在构造base extract前先调用gateway `submit_highlight_set`；任何错误只写一条log并返回`None`。Session gateway可能因session unavailable、capability missing、缺失API或protocol/status错误失败。Host的`submit_render_frame_if_dirty`仅在得到submission且runtime backend返回`Ok(false)`时保留dirty；当submission为`None`时`keep_render_dirty`保持false，最后执行`self.render_dirty = false`。

因此一次可选overlay失败会阻止base scene、HUD和visible spatial snapshot更新，不重试，同时Editor58已确认host可能继续绘制无stale标识的last-good image。必须将base frame与overlay admission拆开：base extract始终可提交；highlight失败进入typed degraded overlay state并保留独立dirty/retry。只有base source本身不qualified时才能阻止base frame，且必须显式呈现Unavailable/Stale而不是静默`None`。

## 5. P1：本轮新增的工程差距

### ED59-P1-01 · Handle终止接口没有Accept/Cancel/OwnerLost语义或receipt

`HandleTool::end_drag(session)`返回`()`，Move/Rotate/Scale三个实现全部为空。controller正常release和`cancel_interaction`都会调用同一个无语义入口；真实rollback另由workbench猜测`is_handle_drag_active`并操作私有capture。接口必须改为消费`InteractiveEditTerminalDisposition`，返回包含affected roots、preview generation、commit/rollback结果和failure stage的receipt，并由Editor03/53的共享interactive edit authority实现。

### ED59-P1-02 · Escape与focus loss是全局命令，不绑定产生交互的window/viewport/pointer

当前Escape和`on_native_window_focus_lost`直接调用同一个全局`cancel_viewport_interaction()`。这对单viewport happy path有价值，但未来多个window/viewport时，任意窗口失焦可能取消另一窗口工具；也无法验证过期focus event是否属于当前capture。命令必须携session/capture identity和reason；全局shutdown可显式枚举并终止全部owner，不能继续用无参数命令混合两种语义。

### ED59-P1-03 · Selection admission只检查node存在，不执行可编辑/可见/锁定策略

`select_nodes`只用`scene.find_node(id).is_some()`过滤。click、box和gizmo owner没有统一检查active-in-hierarchy、hidden、locked、editor-only、prefab/context ownership、current document domain或tool pickability。结果可能选择存在但当前不应可编辑的对象。需要`SelectionEligibilitySnapshot`和typed rejection reason；Godot的locked-selection分支与Unity Graphics Picking/SelectionOutline include-exclude filter证明该策略必须进入authoritative selection product，而不是散在UI快捷键里。

### ED59-P1-04 · 重叠命中只有单一top route，没有候选列表、循环选择或歧义UI

runtime adapter把resolved hover的第一项直接转成route。重叠、嵌套、薄片和相同priority对象没有candidate receipt、click cycling、list popup或“选择后面的对象”策略；approximate depth还会让错误top target稳定胜出。Godot会收集候选并提供selection result菜单。Zircon需要保留排序后的qualified hit list与selection policy，单击fast path只是其一个消费方式。

### ED59-P1-05 · Scale工具以正值下限破坏负缩放与跨零语义

Scale每个轴都执行`(initial + delta).max(0.05)`。已有负scale对象一旦拖动会立即跳到正0.05，镜像语义、handedness和child world transform可被破坏；非均匀父scale与奇异矩阵也没有typed拒绝。不能仅因Fyrox也做正值钳制就视为工程标准。必须由transform policy定义negative/zero crossing、minimum magnitude、mirror、determinant与non-representable shear，并让preview/commit/undo共享同一验证结果。

### ED59-P1-06 · Pointer cache miss同步构造完整render packet并复制全部mesh snapshot

`resolve_for_pointer`在cache miss时调用`build_render_packet`，随后`ViewportInteractionExtract::new`复制render meshes和morph weight payload；`build_scene_gizmos`还扫描全部nodes。该工作位于用户输入同步路径，只有profile counters，没有时间、items、bytes、deadline或fallback预算。应由render/extract阶段提前发布immutable selectable product，pointer path只查询同代索引；miss应返回typed Stale/Preparing并安排重建，不在event handler临时复制整Scene render payload。

### ED59-P1-07 · Highlight没有独立overlay revision、消费receipt或viewport teardown

Editor用selection revision作为highlight generation，display mode改变tint时selection revision不变。Store允许equal generation覆盖，但没有store revision、consumer cursor、remove/clear viewport或session teardown；dynamic extract cache key只有world change tick、visibility revision、camera和size。未来consumer既无法可靠感知同generation属性变化，也无法证明哪一帧消费了哪一份highlight。需要`HighlightProductRevision`、source selection/settings revisions、per-view lifetime、consumer receipt和remove/tombstone；Runtime10消费后Editor才可声明高亮可见。

### ED59-P1-08 · Point、Box与Frame没有共享Selectable Spatial Product

Point路径先查renderer可见包围球，再把返回owner映射成“transform原点 + scale推导半径”的屏幕圆；Box路径绕过renderer snapshot，遍历全部renderable代理圆；Frame路径只合并selected node世界位置。大型偏心mesh可能point broad phase命中后被原点圆拒绝，box可能选中不可见对象，frame又无法容纳同一对象的真实bounds。必须建立per-view/per-frame `SelectableSpatialProduct`，统一owner/instance/subobject、world bounds、visibility、pickability、geometry accelerator、screen bounds与source generation；ray/frustum/frame都消费同一receipt。

## 6. P2：诊断、测试与维护缺口

- **ED59-P2-01**：Pointer Cancel测试当前验证错误结果；缺touch cancel -> transaction rollback -> capture retirement的端到端negative regression。
- **ED59-P2-02**：没有主键Handle期间右/中键press/release、双pointer、乱序release、重复cancel和stale capture generation测试。
- **ED59-P2-03**：没有gateway capability/session/protocol故障下“base frame继续、overlay degraded、dirty重试”的测试。
- **ED59-P2-04**：没有offset/asymmetric/thin/overlap/alpha-tested/instanced/skinned mesh的真实geometry picking parity测试。
- **ED59-P2-05**：Box/Frame测试只覆盖代理happy path和多选位置，不覆盖隐藏/锁定/遮挡、真实bounds、near plane、极端scale和方向策略。
- **ED59-P2-06**：现有profile只计visited/candidate/hit/copy payload，没有input-to-receipt latency、stale rejection、cancel rollback、wrong-target或highlight-consumed generation指标；失败日志也缺viewport/request/capture身份。

## 7. 五引擎参考约束

| 参考 | 本轮源码事实 | 对Zircon的约束 | 不照搬的部分 |
|---|---|---|---|
| Unreal EditorViewportClient / InputRouter / TransformProxy | 输入携device、delta/time；mouse/keyboard/left/right capture分owner/data；支持force terminate与focus loss；tracking可abort；TransformProxy维护多对象相对变换和begin/end edit | 输入必须有qualified identity与capture owner；多键不能覆盖同一slot；transform edit必须有显式begin/accept/cancel和多对象relative state | 不复制UObject、legacy viewport全局状态或left/right历史API形状 |
| Godot Node3DEditorViewport/Gizmo | 每个selected item保存original local/global/last transform、subgizmo和children state；ray/frustum调用真实gizmo intersection；支持重叠候选菜单、bounds framing及cancel restore | 多选、父子、subobject、overlap、frustum、bounds和cancel必须是产品合同 | 不复制Godot单例、scene tree UI或其具体shortcut模型 |
| Fyrox InteractionMode | Mode拥有mouse/key/update/drop lifecycle；Move保存多root对象、gizmo-space offset与parent inverse；Move/Rotate/Scale释放时生成CommandGroup | Rust实现同样可以维护多对象snapshot和批量command，不需要单node capture换“简单” | Fyrox取消与负缩放策略不是充分基线，不能单独照搬 |
| Bevy Picking | `PointerId`区分mouse/touch/custom；`PointerInput`含稳定id、target location和显式Cancel；Cancel清pointer state；MeshPicking先AABB粗筛，再真实mesh intersection并按距离排序 | Zircon已有类似DTO但必须进入真实frame/event authority；broad phase不能冒充precision result | 不把ECS schedule、Entity或Bevy UI event模型原样搬入Editor |
| Unity Graphics GPUDriven | Camera、Picking、SelectionOutline、Filtering是独立`BatchCullingViewType`；context携`viewID`；Picking/Outline有include-exclude和Scene hidden过滤 | Picking与Highlight必须是per-view renderer product，并共享可见性/过滤与明确consumer | 本地Graphics镜像不含完整闭源Unity Editor transform/undo，不从缺失源码外推 |

共同约束是：用户交互由稳定owner/capture拥有，终止有typed disposition；selection/picking/highlight是per-view、per-generation renderer/authoring产品；精确命中必须在broad phase之后验证真实geometry或显式fallback资格。更低开销可以来自连续内存、增量索引和异步准备，不能来自删除身份、事务、错误或currentness合同。

## 8. 目标架构与hard cut

### 8.1 核心类型

1. `ViewportInteractionSessionId`：绑定document/world、viewport session、window、view generation与owner epoch。
2. `ViewportInputEnvelope`：包含pointer/device/window/viewport、sequence/time、buttons/modifiers、position/delta、pressure/phase和capture generation。
3. `ViewportCaptureLease`：由Editor53 authority签发，指定tool instance、pointer/channel、priority、terminal policy与force-end入口。
4. `InteractiveEditSession`：由Editor03 transaction owner创建，冻结affected roots、world/local/parent matrices、pivot/space/snap和before state。
5. `SelectableSpatialProduct`：由renderer/authoring extract按view/frame发布，统一visibility、bounds、hit proxy、geometry accelerator和selection eligibility。
6. `ResolvedViewportHitList`：保留ordered hits、depth/point/normal/subobject、backend/completeness和product generation。
7. `ViewportHighlightProduct`：独立overlay revision、selection/settings source revisions、per-view lifetime与consumer receipt。
8. `ViewportInteractionReceipt`：记录Consumed/NoHit/Stale/Rejected/Accepted/Cancelled/OwnerLost、selection/edit结果及diagnostic identity。

### 8.2 删除与迁移

- 删除`ViewportInput`作为无身份裸枚举的产品边界；内部shortcut可保留，但只能从qualified envelope派生。
- 删除`drag: Option<ViewportDragSession>`作为capture authority，改为session/capture表或严格单owner状态机。
- 删除通过“drag variant消失”推断Gizmo commit的逻辑；Commit/Cancel只消费terminal receipt。
- 删除Pointer Cancel到`None`的映射和对应false-green测试。
- 删除把runtime `PointerInput`仅附在debug返回对象中的假接线；hard cut到Runtime47 frame coordinator。
- 删除把origin/scale circle称为precision final hit；仅在明确fallback mode中保留并显示资格。
- 删除Highlight失败返回整个submission `None`的耦合；base和overlay各自admit/retry。
- 删除无consumer的“Runtime10会消费”完成性注释，直到frame extract真实读取并有测试receipt。

## 9. 分层实施顺序

### M0 · 先封闭三项P0

1. Pointer Cancel映射到typed CancelInteraction，反转host测试并覆盖touch/capture loss rollback。
2. 为现有controller增加严格button/capture ownership guard；在完整router落地前，第二按键不得隐式结束Handle。
3. 将highlight提交从base extract admission拆开；overlay失败保留独立dirty/retry和degraded状态。

### M1 · 收敛Input/Capture/Terminal

1. 由Editor53签发viewport-scoped capture lease，host只向published owner路由。
2. 引入qualified input envelope和sequence order；mouse/touch/pen/custom pointer分别测试。
3. Handle/Scene Mode迁移到Accept/Cancel/Completed/Aborted/OwnerLost disposition与receipt。

### M2 · 收敛Selectable Spatial与Picking

1. Renderer发布per-view selectable product，绑定same-frame visibility、bounds、instance与geometry accelerator。
2. Runtime47接管input/ray/backend/resolve/event pipeline；Editor消费resolved hit list，不再构造死`runtime_input`。
3. Point、Box、Frame统一消费产品；实现locked/hidden/context policy和overlap selection UX。

### M3 · 收敛Interactive Transform

1. 按开放Gizmo failure实现world-space frozen basis、parent inverse、multi-root dedupe和batch preview。
2. 支持pivot/space/plane/uniform/negative scale策略及non-representable transform拒绝。
3. Commit一个typed batch command；Cancel/OwnerLost恢复全部affected roots，autosave只读last committed generation。

### M4 · 闭合Highlight Runtime消费

1. Runtime10在frame extract读取per-view HighlightProduct并把overlay revision纳入cache/currentness。
2. Renderer建立SelectionOutline产品与include/exclude/visibility policy；无selection必须明确过滤全部而非沿用旧集合。
3. Editor等待或查询consumer receipt，区分Submitted、Consumed、Presented、Stale与Degraded。

### M5 · 性能、故障与产品资格

1. 在100k/1m selectable、1kHz pointer、多viewport和大selection下测CPU、内存、延迟与allocation预算。
2. 注入gateway、device、window、capture、plugin、transaction、stale generation和target deletion故障。
3. 同机器、同scene、同画质比较Unreal/Godot/Fyrox的selection/picking/transform latency与恢复，不得用较低正确性换取数字。

## 10. 资格门

| Gate | 必须满足的可观察结果 |
|---|---|
| G01 | Pointer Cancel必达当前capture owner并产生一个terminal receipt |
| G02 | Cancel后UI capture、controller interaction和interactive edit三者全部退休 |
| G03 | Cancel/OwnerLost恢复所有preview对象且history不新增command |
| G04 | stale/duplicate Cancel不能影响新capture generation |
| G05 | Handle期间第二按键不会隐式Commit、丢capture或清错drag |
| G06 | release只终止同pointer/button/capture generation的interaction |
| G07 | Escape、focus loss、window close、viewport close与shutdown各有明确reason |
| G08 | 全局shutdown枚举终止全部session，不借无scope命令误伤普通失焦 |
| G09 | tool owner unload先撤capture、终止edit，再卸载代码 |
| G10 | Highlight gateway失败时base Scene帧仍提交 |
| G11 | Highlight失败保留独立dirty/retry并显示Degraded原因 |
| G12 | base frame失败与overlay失败使用不同typed状态和恢复策略 |
| G13 | Highlight revision包含selection和visual settings source revision |
| G14 | runtime frame cache因新overlay revision重建或正确增量更新 |
| G15 | closed viewport/session的highlight被tombstone或移除 |
| G16 | Submitted/Consumed/Presented highlight generation可追踪且不串viewport |
| G17 | Point hit在broad phase后验证真实geometry或显式qualified fallback |
| G18 | hit receipt含view/frame/backend/owner/instance/subobject generation |
| G19 | 重叠命中保留稳定ordered candidate list并支持产品选择策略 |
| G20 | alpha-tested/instanced/skinned/thin/offset geometry按声明策略命中 |
| G21 | Box selection消费same-generation visibility与spatial product |
| G22 | Box方向、occlusion、hidden/locked和near-plane策略有golden测试 |
| G23 | Frame Selection使用合并真实bounds并处理无bounds与极端scale |
| G24 | wrong-view、wrong-frame或stale spatial snapshot fail closed并可诊断 |
| G25 | 多选Gizmo只作用于selection roots，父子同时选中不重复变换 |
| G26 | world/local/parent space在非均匀与负父scale下有明确定义 |
| G27 | negative scale、zero crossing、mirror与奇异矩阵遵循统一policy |
| G28 | transform preview/commit/cancel使用同一frozen basis和affected set |
| G29 | Commit只生成一个typed batch command和正确Move/Rotate/Scale metadata |
| G30 | target删除、world替换、plugin fault和transaction fault均完整rollback |
| G31 | pointer event hot path不构造完整render packet或复制全量mesh payload |
| G32 | 100k/1m selectable下point/box/frame均有CPU、内存与latency预算 |
| G33 | 125/500/1000Hz motion可合并但不跨press/release/cancel改变edge order |
| G34 | 多window、多viewport、多pointer运行无identity碰撞或cross-cancel |
| G35 | soak结束后0 capture、0 edit session、0 stale highlight与0 orphan product |
| G36 | 所有性能比较使用同正确性、同可见性、同画质和同故障恢复资格 |

## 11. 验证矩阵

| 层级 | 必须新增/执行的验证 |
|---|---|
| Unit/model | capture状态机、button chords、sequence/cancel、terminal disposition、highlight revision/store teardown、transform policy |
| Focused integration | retained host touch cancel、focus loss、right/middle chord、gateway failure、target deletion、world replace、plugin owner loss |
| Picking parity | real mesh ray、GPU ID/hit proxy、alpha/instancing/skinning、overlap list、same-frame visibility、stale snapshot |
| Transform | multi-root、parent/child、world/local/parent、pivot、snap、negative/non-uniform scale、cancel/undo/redo |
| Render product | highlight Submitted/Consumed/Presented、cache invalidation、multi-view isolation、base frame independence |
| Scale/performance | 100k/1m selectable、1kHz pointer、large selection、4/16 viewport、allocation与p95/p99 latency |
| Fault/soak | capture loss、window/device loss、gateway disconnect、renderer reset、plugin reload、transaction rollback failure、8h soak |

## 12. 审查限制与完成定义

本轮只做current-source review和文档落盘。没有修改Rust/Cargo/测试，没有执行Cargo、GUI、GPU capture或benchmark；因此129个test attributes只证明存在，不能当作通过证据。P0的可达性来自静态控制流和当前测试断言，实施时仍必须以动态negative regression复现。

Editor59只有在G01-G36全部有可复验receipt、开放Gizmo与Highlight handoff分别返回fixed记录、Editor03/53/58和Runtime47/10 owner接受硬切边界后，才能把`implementation_status`改为complete。在此之前，新增更多circle proxy、单nodeHandle variant或只存不消费的DTO都会增加能力假象，不属于用户要求的工程级引擎实现。
