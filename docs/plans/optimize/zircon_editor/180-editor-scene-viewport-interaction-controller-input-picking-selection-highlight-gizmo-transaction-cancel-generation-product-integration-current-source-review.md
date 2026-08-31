---
related_code:
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/scene/modes
  - zircon_editor/src/scene/selection
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/core/gateway
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport
  - zircon_editor/src/ui/retained_host/app/native_keyboard_actions.rs
  - zircon_runtime_interface/src/ui/dispatch
  - zircon_runtime/src/ui/surface
  - zircon_runtime/src/core/framework/picking
  - zircon_runtime/src/core/framework/render/visible_spatial_query.rs
  - zircon_runtime/src/core/framework/render/viewport_highlight_store.rs
  - zircon_runtime/src/graphics/visibility/spatial_query.rs
  - zircon_runtime/src/dynamic_api/session
tests:
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/pointer_bridge.rs
  - zircon_editor/src/tests/host/retained_window/platform_input_translation.rs
  - zircon_editor/src/scene/viewport
  - zircon_runtime/src/core/framework/picking
  - zircon_runtime/src/dynamic_api/session/tests/highlight_set.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/174-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-current-source-review.md
  - docs/plans/optimize/zircon_editor/179-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-current-source-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/47-runtime-picking-pointer-ray-hit-hover-drag-drop-event-backend-product-integration-review.md
  - docs/plans/zircon_editor/editor/03/failure-2026-08-19-gizmo-world-space-interactive-transaction.md
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
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 180 · Editor Scene Viewport / Input / Picking / Selection / Highlight / Gizmo Transaction 当前源码复核

## 1. 结论与状态

Editor59发现的三条可达P0中，两条具体错误已经修复。次级鼠标按钮在活动Handle期间会被拒绝，不能再覆盖单一drag slot并触发隐式提交；Highlight gateway失败也只记录错误，base Scene extract继续生成。Pointer Cancel现在能穿过retained bridge并调用`cancel_gizmo_transaction()`恢复preview，但事件在桥接时丢失pointer、device、window、surface、viewport、sequence、capture generation和reason，也没有terminal receipt。因此旧P0-01只能从Open降为Partial，不能关闭。

Picking热路径也有实质进展。`ViewportInteractionExtractCache::resolve_for_pointer()`在miss时只返回`Stale/Preparing`，完整render packet、scene gizmo扫描和mesh payload复制留在render publication路径；renderer-visible snapshot已绑定world、viewport、frame generation和view，并有有界spatial index与visited/candidate/hit指标。这关闭了旧P1-06的具体同步重建问题。

其余工程化边界没有闭合。上层`EditorViewportEvent`和`ViewportInput`仍是无身份裸事件，`ViewportDragSession`仍是一个无owner的`Option`；Handle终止仍返回`()`且三个tool实现为空，workbench仍通过“Handle variant消失”猜测Commit。事务只捕获单node local transform，所有Move/Rotate/Scale command仍标记为`Move scene node`，Scale继续以正值下限破坏负缩放。Point只在可见球体broad phase后命中transform原点圆，Box遍历另一份全量代理圆，Frame只聚合节点位置；三者没有共同的Selectable Spatial Product。Highlight仍只存latest value，没有独立overlay revision、frame consumer receipt或teardown。

本轮不新增、不删除、不重排Editor59的canonical finding，只按当前磁盘重判其 **3项P0、8项P1、6项P2和36个资格门**：

| 等级 | Open / Fail | Partial | Closed / Pass |
|---|---:|---:|---:|
| P0 | 0 | 1 | 2 |
| P1 | 7 | 0 | 1 |
| P2 | 1 | 5 | 0 |
| Gate | 25 Fail | 8 Partial | 3 Pass |

这仍是review结论，不是实现完成证明。当前静态证据不能支持“性能或表现优于Unreal”；达到该目标必须先以相同正确性、可见性、画质、故障恢复和场景规模取得可复验benchmark receipt。

## 2. 冻结语料与currentness

### 2.1 物理选择集

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮证据 |
|---|---:|---|
| Zircon viewport/mode/selection/gateway、Workbench/host桥、Runtime UI metadata/capture、picking/highlight/spatial与聚焦测试 | **235 / 25,668 / 23,296 / 873,153 / 202 / 11** | 当前磁盘从OS事件到事务、spatial product与runtime store的依赖闭包；fingerprint `461d36ed17b7d8f5c0691725226bfaf375aa021b3509d494f214ae3114732da9` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics | **23 / 31,789 / 26,957 / 1,213,892 / 1 / 0** | capture owner、multi-object transform、真实geometry picking、per-view picking/selection过滤；fingerprint `27026b4f682be0d33b23cde03924df5fcff9237d31a02d26e15229d5e5a4a4bb` |

统计按normalized relative path的ordinal顺序，将`lowercase path + NUL + raw bytes + NUL`串联后计算SHA-256；tests/ignored为词法属性计数，不是执行receipt。冻结时Git HEAD为`e6bfb5c0240fb62434c4ba86a1dc2525c0434d96`。共享工作树包含其他在途修改，本报告以当前磁盘fingerprint为审查锚点；实施前必须重算选择集并重判状态。

235份Zircon选择集的源码检索结果为：`TODO/FIXME/unimplemented!/todo!`均为0，`capture_generation`、`TerminalDisposition`、`OwnerLost`、`SelectableSpatial`和`overlay_revision`均为0；与此同时`PointerId`出现94次。这证明底层已经存在per-pointer类型和部分capture设施，但Scene Viewport产品边界没有消费这些合同，不能用“底层有类型”替代端到端闭环。

### 2.2 当前产品链真实性

| 产品链 | 当前事实 | 判定 |
|---|---|---|
| Native/Runtime UI input | `UiInputEventMetadata`含timestamp、sequence、device、window、surface、pointer和source；metadata dispatch能按pointer id激活capture | 底层基础真实 |
| Retained viewport bridge | 接受不含metadata的`UiPointerEvent`，调用legacy `dispatch_pointer_event`，再映射成无身份`EditorViewportEvent` | 身份在产品adapter丢失 |
| Cancel | Pointer Cancel映射`CancelInteraction`，workbench恢复单对象preview并清controller drag | 单viewport happy path真实；无scope、代际、reason与receipt |
| Chord guard | Right/Middle press在任意drag存在时返回，release只清匹配Orbit/Pan variant | 旧隐式Commit bug关闭；仍不是capture-owned状态机 |
| Gizmo transaction | controller只发布preview，workbench写Scene并在release折叠command，错误会尝试rollback | 边界方向正确；单node/local/Move-label/推断Commit仍不合格 |
| Pointer extract | render path发布共享`Arc<ViewportInteractionExtract>`；pointer miss只返回Stale/Preparing | 旧同步全包构建关闭 |
| Point picking | generation-bound renderer-visible sphere query缩小owner，再投影owner原点/radius并用runtime排序 | broad phase真实；precision geometry不真实 |
| Box / Frame | Box遍历render extract代理圆和gizmo shape；Frame合并selected node世界位置 | 各自能用；不共享visibility、bounds或geometry product |
| Highlight | Editor提交全selection；Runtime按viewport/generation存latest set | producer/store真实；frame consumer、overlay currentness和present receipt不存在 |
| Base frame independence | highlight提交错误不再中止base extract，有fault test | 旧P0-03关闭；overlay degraded/retry仍无合同 |

### 2.3 当前源码修正证据

- `pointer_dispatch.rs:104`把Cancel映射为`EditorViewportEvent::CancelInteraction`；`pointer_bridge.rs:142`反转旧false-green，验证Cancel到达terminal command且后续move不再路由。
- `editor_state_viewport.rs:397-411`先调用`cancel_gizmo_transaction()`恢复preview，再调用controller cancel；`tests/editing/state/viewport.rs:410`验证单对象transform恢复。
- `scene_viewport_controller_handle_input.rs:86-103`拒绝活动drag期间的Right/Middle press，并只在variant匹配时release；`:429`的回归测试验证原Handle仍可取消。
- `editor_state_render.rs:37-49`在highlight submit失败后继续构造base extract；`:149`以CapabilityMissing fault验证base frame仍存在。
- `interaction_extract/cache.rs:67-88`的pointer miss只设置rebuild request并返回`Stale/Preparing`；构造scene gizmo和复制mesh payload只在render publication的`rebuild()`执行。
- `visible_spatial_query.rs`公开world/viewport/frame/view identity；`graphics/visibility/spatial_query.rs`使用static/dynamic index、有限cell预算和visible-entry fallback，并输出查询成本指标。
- `runtime_picking_adapter.rs:57-65`仍固定pointer id 1、camera id 0并只取`.first()`；`renderer_visible_spatial_pick_source.rs:85-119`只把sphere-query owner还原为屏幕代理候选。
- `selectable_owners_in_rect.rs:19`继续遍历`renderable_candidates`，以position和scale-derived radius做circle/rect相交；`selection_frame.rs:55`只合并world position。
- `scene_viewport_controller_selection.rs:24-33`只以`scene.find_node`作为selection admission，没有hidden/locked/context/editability policy。
- `viewport_highlight_store.rs`允许equal generation覆盖但没有store revision、consumer cursor、remove或tombstone；runtime frame extract对highlight store没有生产读取点。

### 2.4 相邻底座不能冒充本专项完成

Editor179确认Play/Simulate pick已绑定显示帧gateway identity，Runtime UI metadata path也已有per-pointer capture。这些能力应复用，但当前authoring Scene bridge仍走bare event和legacy surface dispatch；不能拿另一个入口的正确身份为本入口背书。Editor174继续拥有通用tool scheduler/capture lease/owner unload合同，Editor03开放failure继续拥有world-space多选Gizmo与batch transaction，Runtime47继续拥有完整picking event backend，Runtime10继续拥有highlight frame consumption。本报告只重判Scene Viewport adapter和这些owner之间的缺口，不重复登记。

## 3. P0：当前状态

### ED59-P0-01 · Partial · Pointer Cancel可回滚单对象preview，但仍无qualified owner与terminal receipt

旧“Cancel被bridge吞掉”已修复。当前Cancel能释放legacy UI capture、发布`CancelInteraction`并恢复活动gizmo preview。未关闭部分更关键：`UiPointerEvent`本身没有metadata，bridge没有走`UiInputEventMetadata`路径；进入Editor后事件也不携pointer/window/surface/viewport/sequence/capture generation/reason。Escape和focus loss同样发送无scope命令。旧capture的迟到Cancel无法与新capture区分，多window/viewport时也无法证明取消目标。

必须把bridge硬切到qualified input envelope，由唯一capture owner返回`Cancelled/OwnerLost/Rejected/Stale` terminal receipt。UI capture、controller interaction和interactive transaction必须以同一capture generation原子退休；旧generation的Cancel必须fail closed。

### ED59-P0-02 · Closed · 次级按键不再覆盖活动Handle并隐式Commit

Right/Middle press现在在`state.drag.is_some()`时直接拒绝，release也只清除匹配的Orbit/Pan variant。聚焦测试依次注入右键和中键press/release，确认Handle始终active，随后Cancel可恢复初始transform。旧可达序列已经被封住。

关闭只针对该具体错误。单一无owner drag slot、无pointer/button/capture generation和通过variant变化推断Commit的问题仍由P1-01/P1-02及G01-G09约束，不能把本条Closed解释为capture architecture完成。

### ED59-P0-03 · Closed · Highlight提交失败不再丢弃base Scene frame

`render_frame_submission()`现在记录highlight delivery error后继续访问world并构造base extract；CapabilityMissing fault test验证submission保留正确world generation。旧“可选overlay错误导致整个submission为None”的可达控制流已删除。

关闭不包括overlay自身恢复。当前仍没有typed degraded state、独立dirty/retry、overlay revision或Submitted/Consumed/Presented receipt，相关缺口继续由P1-07和G11-G16跟踪。

## 4. P1：当前状态与重构要求

| Finding | 状态 | 当前源码事实 | 必须重构为 |
|---|---|---|---|
| ED59-P1-01 Handle终止无Accept/Cancel/OwnerLost | Open | `HandleTool::end_drag(session) -> ()`，三个实现均为空；normal release与cancel共用无语义入口，workbench以`was_handle_drag && !is_handle_drag`猜Commit | typed terminal disposition + affected roots/preview generation/commit或rollback receipt |
| ED59-P1-02 Escape/focus loss无scope | Open | Escape、focus loss和pointer Cancel最终都是全局无参数`CancelInteraction` | 绑定window/viewport/pointer/capture generation；shutdown显式枚举全部session |
| ED59-P1-03 Selection admission仅检查node存在 | Open | `find_node`是唯一过滤；没有active、hidden、locked、document/context或tool policy | immutable `SelectionEligibilitySnapshot`和typed rejection reason |
| ED59-P1-04 重叠命中只消费top hit | Open | runtime picking内部能排序多个hit，Editor adapter固定`.first()` | 保留qualified ordered hit list，支持cycle/list/behind-object policy |
| ED59-P1-05 正值scale下限破坏负缩放 | Open | 三轴均执行`(initial + delta).max(0.05)` | negative/zero crossing、mirror、determinant、singular与TRS residual统一policy |
| ED59-P1-06 Pointer miss同步构造全render packet | Closed | pointer miss只返回Stale/Preparing；共享extract由render path发布，测试锁定不调用packet build | 保留当前边界，补deadline、publication scheduling与scale receipt |
| ED59-P1-07 Highlight无独立revision/receipt/teardown | Open | generation复用selection revision；equal generation可覆盖；store无overlay revision、consumer、remove/tombstone | per-view `ViewportHighlightProduct`、source revisions、consumer cursor与teardown |
| ED59-P1-08 Point/Box/Frame无共享spatial product | Open | Point用visible sphere+原点圆，Box用全量代理圆，Frame用位置点 | per-view/per-frame `SelectableSpatialProduct`统一visibility、eligibility、bounds、geometry和generation |

P1-06关闭并不意味着pointer路径已达到最终性能目标。当前event-time查询仍会分配`Vec<PrecisionCandidate>`和runtime hit records，owner table在generation adoption时复制候选；它只证明最危险的“miss时同步构造完整render packet并复制全部mesh payload”已移出事件处理器。

## 5. P2：测试、诊断与长期成熟度

| Finding | 状态 | 当前证据与剩余缺口 |
|---|---|---|
| ED59-P2-01 Pointer Cancel端到端negative regression | Partial | host测试已反转，state测试已验证rollback；没有携身份的host -> capture -> transaction -> terminal receipt闭环 |
| ED59-P2-02 chord、多pointer、乱序与stale cancel | Partial | 右/中键不能替换Handle已有测试；双pointer、乱序release、重复cancel、stale generation仍无测试 |
| ED59-P2-03 highlight fault independence | Partial | CapabilityMissing下base frame继续已有测试；overlay degraded、dirty/retry和consumer receipt仍无测试 |
| ED59-P2-04 真实geometry picking parity | Open | 没有offset/asymmetric/thin/alpha-tested/instanced/skinned mesh的precision parity |
| ED59-P2-05 Box/Frame policy矩阵 | Partial | circle/segment rect和多选position framing有测试；hidden/locked/occlusion/real bounds/near plane/extreme scale缺失 |
| ED59-P2-06 交互可观测性 | Partial | visible query和copy payload已有计数；input-to-receipt、wrong-target、cancel rollback、stale rejection、highlight consumed缺失 |

## 6. 本地参考源码对照

| 参考 | 本地源码事实 | 对Zircon的硬约束 | 不照搬的部分 |
|---|---|---|---|
| Unreal EditorViewportClient / InputRouter / TransformProxy | InputRouter保存keyboard/left/right capture behavior、owner和data，支持`ForceTerminateSource/All`，`DeregisterSource`先终止capture再移除；TransformProxy保存多个对象、初始与relative transform及begin/end edit序列 | capture必须有owner并在source卸载前terminal；多对象transform必须冻结相对状态，不能单node直写 | 不复制UObject、legacy左右capture API或全局viewport状态 |
| Godot Node3DEditorViewport/Gizmo | cancel恢复original transform/subgizmo；gizmo执行真实ray/frustum intersection；重叠候选进入selection result菜单 | Cancel必须恢复完整affected set；ray/frustum和overlap list是产品合同 | 不复制单例、SceneTree UI和快捷键模型 |
| Fyrox InteractionMode | Move保存多root初始状态并使用parent inverse，Move/Rotate/Scale释放生成CommandGroup | Rust实现同样可以工程化多对象snapshot与batch command | Fyrox自身的cancel和正值scale策略不是充分基线 |
| Bevy Picking | `PointerId`区分Mouse/Touch/Custom，`PointerInput`有稳定id和Cancel；mesh picking先AABB broad phase，再做真实mesh ray intersection并按距离排序 | Zircon已有类似底层DTO，但必须进入authoring frame authority；broad phase不能冒充precision | 不复制ECS schedule、Entity或Bevy UI冒泡模型 |
| Unity Graphics GPUDriven | Picking与SelectionOutline是独立view type，context携`viewID`，Editor culling应用include/exclude与hidden过滤 | Picking/Highlight必须per-view并共享visibility/filtering，consumer身份明确 | 本地Graphics不含闭源Editor transform/undo，不能从缺失源码外推 |

共同基线不是API长相，而是四条不变量：输入由稳定capture owner拥有；terminal有typed disposition和receipt；selection/picking/highlight绑定view与generation；broad phase之后必须有真实geometry验证或显式声明的fallback资格。低开销来自增量索引、稳定产品和有界查询，不能来自删除身份、错误与currentness合同。

## 7. 目标架构与唯一owner

### 7.1 核心产品

1. `ViewportInteractionSessionId`：绑定document/world、Editor179 viewport session、window、view generation和owner epoch。
2. `ViewportInputEnvelope`：保存user/device/pointer/source、window/surface/viewport、sequence/time、buttons/modifiers、position/delta、pressure/phase和capture generation。
3. `ViewportCaptureLease`：由Editor174 interactive tool authority签发，指定owner、pointer/channel、priority、generation、terminal policy与force-end入口。
4. `InteractiveEditSession`：由Editor03 transaction owner创建，冻结selection roots、world/local/parent matrices、pivot/space/snap和before state。
5. `SelectableSpatialProduct`：renderer/authoring extract按view/frame发布owner、instance/subobject、visibility、eligibility、world/screen bounds、geometry accelerator与source generation。
6. `ResolvedViewportHitList`：保留ordered hits、depth/point/normal/subobject、backend/completeness和product generation；single top route只是一个consumer。
7. `ViewportHighlightProduct`：独立overlay revision、selection/settings source revision、per-view lifetime、remove/tombstone与consumer receipt。
8. `ViewportInteractionReceipt`：记录Consumed/NoHit/Stale/Rejected/Accepted/Cancelled/OwnerLost及selection/edit结果和诊断身份。

### 7.2 Owner边界

| Owner | 唯一职责 | 禁止承担 |
|---|---|---|
| Runtime/host input | 产生qualified physical input与capture lifecycle | 决定Scene selection或事务Commit |
| Editor174 tool authority | capture lease、owner epoch、priority与force termination | 直接写Scene transform |
| Editor Scene Viewport adapter | 将qualified input路由到当前session并消费hit/terminal receipt | 丢弃身份后发全局命令 |
| Runtime47 picking authority | ray/backend/resolve/event和ordered hit list | Editor私有circle route冒充runtime product |
| Editor03 transaction owner | preview、batch commit、rollback、undo/redo和autosave currentness | controller以variant变化猜Commit |
| Runtime renderer | selectable/highlight per-view product与consumer receipt | 用store存在冒充已渲染/已呈现 |

## 8. 必须硬切的旧路径

- 删除bare `EditorViewportEvent`/`ViewportInput`作为authoring产品边界；内部枚举可保留，但只能从qualified envelope派生。
- 删除`drag: Option<ViewportDragSession>`作为capture authority，改为lease-owned session状态机；第二按键策略必须显式。
- 删除通过“Handle variant消失”推断Commit；Accept/Cancel只消费terminal disposition。
- 删除`HandleTool::end_drag() -> ()`及三个空实现，统一返回terminal receipt。
- 删除单node `GizmoTransactionCapture`与固定`Move scene node`标签，hard cut到affected-root batch command。
- 删除production scale的正值`.max(0.05)`，由统一transform policy决定negative/zero/singular语义。
- 删除固定pointer id 1、camera id 0和`.first()`作为Editor最终消费边界。
- 删除把origin/scale circle称为precision hit；只可作为显式qualified fallback并暴露completeness。
- 删除Box与Frame各自扫描另一套代理数据；统一消费same-generation selectable product。
- 删除只存不消费的Highlight完成性叙述；frame extract真实读取并返回Consumed/Presented receipt前保持Unavailable/Partial。

## 9. 分层重构里程碑

### M0 · 固化已修P0并补qualified Cancel

1. 保留secondary-button guard与base-frame-independent highlight fault test。
2. bridge改走metadata dispatch，建立pointer/window/surface/viewport/capture generation envelope。
3. Cancel/OwnerLost返回terminal receipt，原子退休UI capture、controller session和interactive edit。

### M1 · 收敛Capture与Interactive Transaction

1. 接入Editor174 lease/owner epoch，覆盖focus loss、window/viewport close、plugin unload和shutdown。
2. Handle终止迁移到Accept/Cancel/OwnerLost disposition，禁止variant推断。
3. 按Editor03 failure实现world-space frozen basis、parent inverse、selection-root去重、multi-object preview和一个typed batch command。

### M2 · 建立Selectable Spatial Product

1. Renderer发布per-view/per-frame selectable snapshot，统一visibility、eligibility、bounds、instance与geometry accelerator。
2. Runtime47接管pointer/ray/backend/resolve/event，Editor消费ordered hit list与completeness。
3. Point、Box、Frame统一消费该产品，完成hidden/locked/context/occlusion/near-plane策略。

### M3 · 闭合Highlight消费

1. Runtime10在frame extract读取per-view highlight product，并把overlay revision纳入currentness/cache。
2. 实现remove/tombstone、closed viewport teardown和独立dirty/retry/degraded状态。
3. 串联Submitted/Consumed/Presented generation receipt，禁止串viewport或沿用stale set。

### M4 · 性能、故障与对标资格

1. 在100k/1m selectable、1kHz pointer、large selection和4/16 viewport下测CPU、内存、allocation与p95/p99 latency。
2. 注入stale generation、target deletion、world replace、gateway/device/window/plugin/transaction故障并验证完整rollback。
3. 同机器、同scene、同画质、同正确性和同恢复资格对照Unreal/Godot/Fyrox；没有receipt不得宣称性能领先。

## 10. 36个资格门当前重判

| Gate | 状态 | 当前证据 / 缺口 |
|---|---|---|
| G01 Pointer Cancel到当前capture owner并有terminal receipt | Partial | 到达全局viewport owner；无qualified owner/receipt |
| G02 Cancel原子退休capture/controller/edit | Partial | legacy capture与单controller/edit会清理；无共同generation receipt |
| G03 Cancel/OwnerLost恢复所有preview且history不新增 | Partial | 单对象Cancel恢复；无multi-object/OwnerLost/batch证明 |
| G04 stale/duplicate Cancel不影响新capture | Fail | 无capture generation |
| G05 Handle期间第二按键不隐式Commit | Pass | guard与focused regression存在 |
| G06 release只终止同pointer/button/generation | Partial | variant/button匹配；无pointer/capture generation |
| G07 Escape/focus/window/viewport/shutdown reason明确 | Fail | 无scope、reason和receipt |
| G08 shutdown枚举全部session | Fail | 无session registry |
| G09 tool unload先终止capture/edit | Fail | Scene Viewport未接lease owner epoch |
| G10 Highlight失败时base frame继续 | Pass | CapabilityMissing fault test存在 |
| G11 Highlight失败独立dirty/retry/Degraded | Fail | 仅日志 |
| G12 base与overlay失败typed状态分离 | Fail | 无overlay状态产品 |
| G13 Highlight revision含selection/settings revision | Fail | 仅selection revision |
| G14 overlay revision使frame cache更新 | Fail | 无consumer/revision |
| G15 closed viewport highlight被remove/tombstone | Fail | store无remove |
| G16 Submitted/Consumed/Presented可追踪且隔离viewport | Fail | 只有Submitted/store latest |
| G17 broad phase后真实geometry或qualified fallback | Fail | 仍是原点圆final hit |
| G18 hit receipt含view/frame/backend/instance/subobject | Fail | Editor只返回route |
| G19 稳定ordered candidate list与选择策略 | Fail | `.first()`丢弃其余候选 |
| G20 alpha/instance/skinned/thin/offset命中策略 | Fail | 无precision parity |
| G21 Box消费same-generation visibility/spatial product | Fail | 遍历独立全量代理圆 |
| G22 Box方向/遮挡/hidden/locked/near-plane golden | Fail | 无完整policy/测试 |
| G23 Frame使用真实bounds并处理极端scale | Fail | 只用node位置 |
| G24 wrong-view/frame/stale snapshot fail closed | Partial | world与snapshot identity有拒旧；无完整receipt/diagnostic |
| G25 多选仅作用selection roots且父子不重复 | Fail | 事务只捕获primary |
| G26 world/local/parent在非均匀负父scale下定义 | Fail | local single-node行为 |
| G27 negative/zero/mirror/singular统一policy | Fail | 正值0.05钳制 |
| G28 preview/commit/cancel共享frozen basis/affected set | Partial | 单对象drag session冻结basis；无batch/parent currentness |
| G29 一个typed batch command和正确kind metadata | Partial | 单对象applied transform；标签固定Move、无batch/kind |
| G30 deletion/world/plugin/transaction fault完整rollback | Partial | target deletion/world/transaction有局部测试；无plugin与all-root guarantee |
| G31 pointer hot path不构造全packet/复制mesh | Pass | Stale/Preparing + render publication共享Arc |
| G32 100k/1m point/box/frame预算 | Fail | 无统一规模receipt |
| G33 高频motion合并不改变edge order | Fail | 无sequence/coalescing合同 |
| G34 multi-window/viewport/pointer无cross-cancel | Fail | authoring事件无identity |
| G35 soak后0 capture/edit/stale highlight/orphan product | Fail | 无soak与ledger |
| G36 性能比较满足同正确性/画质/恢复资格 | Fail | 无跨引擎benchmark receipt |

## 11. 本轮验证边界与后续入口

本轮只做当前源码静态review、参考源码对照、文档落盘和结构校验，没有修改Rust/Cargo/测试，也没有运行Cargo、Editor、GUI、真实GPU picking、touch/pen、多window、父子非均匀/负scale、fault、scale、soak或benchmark。202个Zircon test attributes与11个ignored只证明测试源码存在，不代表通过；参考切片中的1个test同理。

后续实现应从M0的qualified Cancel与terminal receipt开始，同时保留已经关闭的secondary chord guard、base frame independence和pointer miss Stale/Preparing。Editor59只有在G01-G36全部取得可复验receipt、Editor03/174/179与Runtime47/10 owner完成边界接线后，才能将`implementation_status`改为complete。
