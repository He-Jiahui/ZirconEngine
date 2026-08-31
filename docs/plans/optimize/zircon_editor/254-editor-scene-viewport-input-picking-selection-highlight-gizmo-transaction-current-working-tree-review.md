---
related_code:
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/scene/modes
  - zircon_editor/src/scene/selection
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/core/editing/interactive_transform
  - zircon_editor/src/core/editing/command/batch_transform.rs
  - zircon_editor/src/core/gateway
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport
  - zircon_runtime_interface/src/ui/dispatch
  - zircon_runtime/src/ui/dispatch
  - zircon_runtime/src/ui/surface
  - zircon_runtime/src/core/framework/picking
  - zircon_runtime/src/core/framework/render/visible_spatial_query.rs
  - zircon_runtime/src/core/framework/render/viewport_highlight_store.rs
  - zircon_runtime/src/graphics/visibility/spatial_query.rs
  - zircon_runtime/src/scene/level_system.rs
tests:
  - zircon_editor/src/tests/editing/interactive_transform.rs
  - zircon_editor/src/tests/editing/history.rs
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/pointer_bridge.rs
  - zircon_editor/src/tests/host/retained_window/platform_input_translation.rs
  - zircon_runtime/src/dynamic_api/session/tests/highlight_set.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/174-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-current-source-review.md
  - docs/plans/optimize/zircon_editor/180-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/253-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/47-runtime-picking-pointer-ray-hit-hover-drag-drop-event-backend-product-integration-review.md
  - docs/plans/zircon_editor/editor/03/failure-2026-08-19-gizmo-world-space-interactive-transaction.md
  - docs/plans/zircon_runtime/runtime/10/failure-2026-08-19-highlight-set-runtime-frame-consumption.md
  - docs/plans/zircon_editor/editor/05/failure-2026-07-18-viewport-pointer-candidate-regeneration.md
  - docs/plans/zircon_runtime/render/04/failure-2026-07-18-viewport-picking-visible-spatial-query.md
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
  - docs/plans/optimize/zircon_editor/180-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
review_status: current_working_tree_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 254 · Editor Scene Viewport / Input / Picking / Selection / Highlight / Gizmo Transaction 当前工作树复核

## 1. 结论与状态

Editor180之后，Gizmo事务内核已经发生实质变化，不能继续描述为“单节点local transform + 固定Move标签”。当前唯一`InteractiveTransformSession`会冻结document、world generation、tool/axis/space/snap、pivot、selection roots、local/world/parent-inverse快照；preview先计算并验证全部目标，再执行`O(k)`写入，失败补偿已写前缀；finish生成一个带正确Move/Rotate/Scale标签的`BatchTransformCommand`。源码测试覆盖父子选择去重、共同质心、parent inverse、global rotate/scale、不可表示shear拒绝、cancel恢复以及100次preview只形成一个command/history record。因此G25、G28、G29从旧Fail/Partial提升为Pass，G26从Fail提升为Partial。

但产品外围仍没有同步工程化。Retained viewport bridge把带capture的UI pointer route降成无window/surface/viewport/pointer/sequence/capture generation的`EditorViewportEvent`；`ViewportInput`同样是裸枚举，controller仍以一个无owner的`Option<ViewportDragSession>`承载selection、camera与Handle。`HandleTool::end_drag()`仍返回`()`且三个tool实现为空，Workbench继续通过Handle variant消失推断commit。Scale工具继续使用正值`.max(0.05)`，阻断负缩放与镜像。Runtime picking虽保留多个有序hit，Editor adapter仍固定pointer id 1、camera id 0并只取`.first()`。Selection admission仍只检查node存在；Point、Box、Frame仍不共享同一可见性/geometry/bounds产品。Highlight仍止于latest-value store，没有frame consumer、独立overlay revision、teardown或Consumed/Presented receipt。

本轮不新增、不删除、不重排Editor59的canonical finding，只按当前磁盘重判其3项P0、8项P1、6项P2和36个资格门：

| 等级 | Open / Fail | Partial | Closed / Pass |
|---|---:|---:|---:|
| P0 | 0 | 1 | 2 |
| P1 | 7 | 0 | 1 |
| P2 | 1 | 5 | 0 |
| Gate | 23 Fail | 7 Partial | 6 Pass |

这仍是review结论，不是实现完成或性能领先证明。当前静态证据不能支持“优于Unreal”；先取得相同输入身份、命中正确性、可见性、故障恢复、画质和规模资格，才有可比较的性能数据。

## 2. 冻结语料与currentness

### 2.1 选择集

| 范围 | files | lines | non-empty | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Zircon viewport/mode/selection/interactive transform/gateway、Workbench/host bridge、Runtime UI dispatch/surface、picking/highlight/spatial及聚焦测试闭包 | **534** | **99,416** | **91,732** | **3,414,493** | **606 markers** | **56 markers** | `006e1f12754cf1c713426e612174c87fec3c2bc2cbd531841d1f79eea38bec47` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics参考切片 | **23** | **31,789** | **26,957** | **1,213,892** | **1 marker** | **0** | `f9c108ebb4e674e8bf2f8341035b30ce72751fb763ddcbf49c96ca17d87430bd` |

统计按normalized relative path排序，将`lowercase path + NUL + raw bytes + NUL`串联后计算SHA-256；tests/ignored是词法marker，不是执行receipt。冻结时Git HEAD为`cc5cadbd597c3707954ebd6109fad0fd5643a152`。共享工作树包含大量其他在途修改，本报告以当前磁盘指纹为审查锚点；实施前必须重算并重判。

### 2.2 当前产品链

| 链路 | 当前事实 | 判定 |
|---|---|---|
| Native/Runtime UI input | Runtime Interface已有timestamp、sequence、device、window、surface、pointer和source metadata，Runtime UI也有per-pointer capture设施 | 可复用底座真实 |
| Retained authoring bridge | `dispatch_viewport_pointer_event()`只接`UiPointerEvent + modifiers`，调用legacy dispatch后映射裸`EditorViewportEvent` | 身份在产品adapter丢失 |
| Controller interaction | selection/orbit/pan/handle共用一个`Option<ViewportDragSession>`，没有owner/capture generation/terminal disposition | 临时单槽状态机 |
| Interactive transform | Workbench创建唯一批量会话；preview、cancel、finish都经过它，controller不再直接写Scene | 内核工程化；外围terminal未闭环 |
| Point picking | same-generation visible spatial ray query缩小owner，再把owner还原为投影代理候选并只消费top hit | broad phase真实；precision/overlap policy不真实 |
| Box / Frame | Box扫描interaction extract的代理圆与gizmo shapes；Frame只聚合节点world position | 各自可用；无共同spatial product |
| Selection | domain、ordered set、primary和revision真实存在；admission只检查`find_node` | 模型底座真实；资格策略缺失 |
| Highlight | Editor提交全selection，LevelSystem按viewport/generation存latest set | producer/store真实；frame consumer不存在 |

### 2.3 Editor180之后的实质进展

- `InteractiveTransformSession::begin()`去重selection并过滤被选祖先覆盖的descendant，只保留top-level roots；Static目标在任何preview前typed reject。
- 每个root冻结local edit state、world matrix与optional parent inverse；共同pivot支持Primary与Centroid，Global Move/Rotate/Scale统一从冻结pivot求world delta。
- `preview()`先对全部目标完成finite、TRS分解、重组残差和Scene可写性检查，再执行写入；中途失败会逆序补偿前缀。
- `finish()`验证document/world generation和after state，生成单个`BatchTransformCommand`；journal type为`zircon.editor.scene.batch_transform`，历史标签按kind区分。
- 专门测试覆盖parent/child root过滤、world translation经parent inverse写回、非均匀父scale产生shear时typed拒绝、共享质心旋转与全局缩放。
- 真实viewport输入测试覆盖100次PointerMoved后只有一个command/history record，普通Scene编辑、target删除、world替换、mode切换和transaction fault会尝试取消或回滚。
- renderer-visible picking snapshot绑定world/viewport/frame/view，event-time只查询返回owners；stable generation复用projection和owner map，并记录visited/candidate/hit/projected count。

### 2.4 不能冒充本专项完成的相邻能力

Editor253已确认Play/Simulate displayed-frame identity与Runtime surface transport的局部正确性；Editor174拥有通用tool scheduler/capture lease设计；Runtime47拥有完整picking backend目标；Runtime10拥有highlight frame consumer修复责任。这些都不是authoring Scene Viewport当前入口已经接线的证据。尤其不能用Runtime UI底层存在metadata/capture，替代Retained viewport bridge实际丢失身份的事实。

## 3. P0当前状态

### ED59-P0-01 · Partial · Cancel能回滚批量事务内核，但没有qualified owner与terminal receipt

Pointer Cancel、Escape和focus loss最终都能触发`CancelInteraction`，Workbench会调用批量session的`cancel()`恢复affected roots，再清controller drag。相较Editor180，恢复内核已支持多root与补偿写入。

未关闭部分仍是产品级P0风险：bridge没有传pointer/window/surface/viewport/sequence/capture generation/reason，Escape/focus loss也发送同一个无scope命令。迟到Cancel不能与新capture区分，多window/viewport时不能证明取消对象；UI capture、controller drag和interactive session没有共同terminal receipt。必须硬切到qualified input envelope和唯一capture lease，由owner返回`Cancelled/OwnerLost/Rejected/Stale`，旧generation必须fail closed。

### ED59-P0-02 · Closed · 次级按键不再覆盖活动Handle并隐式Commit

Right/Middle press在任意drag存在时拒绝，release只清匹配的Orbit/Pan variant；聚焦回归确认活动Handle不被替换，随后仍能Cancel恢复。关闭只针对该具体可达错误，不代表capture architecture完成。

### ED59-P0-03 · Closed · Highlight提交失败不再丢弃base Scene frame

`render_frame_submission()`记录highlight delivery error后继续构造base extract，fault test覆盖CapabilityMissing。关闭不包括overlay恢复、retry、revision、consumer或present receipt。

## 4. P1当前状态与重构要求

| Finding | 状态 | 当前源码事实 | 必须重构为 |
|---|---|---|---|
| ED59-P1-01 Handle终止无Accept/Cancel/OwnerLost | Open | `HandleTool::end_drag(session) -> ()`，Move/Rotate/Scale实现均为空；normal release与cancel共用无语义入口，Workbench以drag variant变化猜commit | typed terminal disposition + session/capture identity + affected roots/preview generation/commit或rollback receipt |
| ED59-P1-02 Escape/focus loss无scope | Open | Pointer Cancel、Escape与focus loss都降成全局无参数`CancelInteraction` | 绑定window/viewport/pointer/capture generation/reason；shutdown显式枚举session |
| ED59-P1-03 Selection admission仅检查node存在 | Open | `select_nodes()`唯一过滤为`scene.find_node`；Static mobility只在transform begin下游拒绝 | immutable `SelectionEligibilitySnapshot`，统一active/hidden/locked/document/context/tool/editability与typed rejection |
| ED59-P1-04 重叠命中只消费top hit | Open | Runtime可排序多个hit；Editor adapter固定pointer 1/camera 0并调用`.first()` | 保留qualified ordered hit list，支持cycle/list/behind-object与稳定tie policy |
| ED59-P1-05 正值scale下限破坏负缩放 | Open | Scale三轴继续执行`(initial + delta).max(0.05)`；事务残差检查无法弥补输入层禁止负值 | negative/zero crossing、mirror、determinant、singular和TRS residual统一policy |
| ED59-P1-06 Pointer miss同步构造全render packet | Closed | miss只返回Stale/Preparing；render publication生成共享extract，event-time visible query只投影命中owner | 保留当前边界，补publication deadline、allocation与1/1k/10k receipt |
| ED59-P1-07 Highlight无独立revision/receipt/teardown | Open | Store只有submit/get；selection generation兼作overlay generation，equal generation可覆盖，无remove/tombstone/consumer | per-view `ViewportHighlightProduct`、独立source/overlay revisions、consumer cursor、teardown与Submitted/Consumed/Presented receipt |
| ED59-P1-08 Point/Box/Frame无共享spatial product | Open | Point用visible sphere/ray broad phase后投影代理，Box遍历另一份代理圆，Frame只用位置 | per-view/per-frame `SelectableSpatialProduct`统一visibility、eligibility、bounds、geometry、instance和generation |

事务内核进展不能关闭P1-01：它解决的是“写哪些对象、如何批量写、如何回滚”，不是“哪个输入owner以何种终态结束会话”。这两个边界必须显式连接，不能继续靠状态差分猜测。

## 5. P2测试、诊断与长期成熟度

| Finding | 状态 | 当前证据与剩余缺口 |
|---|---|---|
| ED59-P2-01 Pointer Cancel端到端negative regression | Partial | host Cancel与单对象state rollback已有测试，批量session cancel有内核测试；缺qualified host -> capture -> multi-root transaction -> terminal receipt |
| ED59-P2-02 chord、多pointer、乱序与stale cancel | Partial | secondary chord已有回归；双pointer、乱序release、重复Cancel、旧generation Cancel仍无测试 |
| ED59-P2-03 highlight fault independence | Partial | gateway失败保留base frame；overlay degraded/dirty/retry/consumer/teardown无测试 |
| ED59-P2-04 真实geometry picking parity | Open | offset/asymmetric/thin/alpha-tested/instanced/skinned mesh没有precision parity |
| ED59-P2-05 Box/Frame policy矩阵 | Partial | circle/segment rect与多选位置framing有测试；hidden/locked/occlusion/real bounds/near plane/extreme scale缺失 |
| ED59-P2-06 交互可观测性 | Partial | visible query与owner-copy已有计数，100-preview历史收束已有测试源码；input-to-receipt、wrong-target、cancel rollback、stale rejection、highlight consumed仍缺 |

## 6. 本地参考源码对照

| 参考 | 本地源码事实 | 对Zircon的硬约束 | 不照搬 |
|---|---|---|---|
| Unreal EditorViewportClient / InputRouter / TransformProxy | InputRouter保存capture behavior、owner与data；source注销先`ForceTerminateSource`，失焦`ForceTerminateAll`；TransformProxy保存多个object的initial/relative transform及begin/end edit | capture必须有owner且在source卸载/失焦前terminal；多对象transform冻结相对状态 | UObject、legacy left/right capture和全局viewport状态 |
| Godot Node3DEditorViewport/Gizmo | cancel恢复original transform/subgizmo；gizmo提供ray/frustum intersection；重叠候选进入selection result | Cancel恢复完整affected set；point/box/overlap是产品合同 | singleton/SceneTree UI与快捷键组织 |
| Fyrox InteractionMode | Move保存多root初始状态并使用parent inverse；Move/Rotate/Scale释放CommandGroup | Rust实现同样可提供批量snapshot/command；Zircon当前内核方向正确 | Fyrox自身cancel和正scale策略不是充分基线 |
| Bevy Picking | `PointerId`区分Mouse/Touch/Custom，PointerInput含稳定id与Cancel；mesh picking先AABB broad phase，再做mesh ray intersection并按距离排序 | 身份必须进入authoring authority；broad phase不能冒充precision，ordered hits不能在adapter丢弃 | ECS schedule、Entity和Bevy UI传播模型 |
| Unity Graphics GPUDriven | Picking/SelectionOutline为独立view type，context使用`viewID`，culling应用include/exclude和hidden过滤 | Picking/Highlight必须per-view并共享visibility/filtering，consumer身份明确 | 本地Graphics不含闭源Editor transform/undo，不从缺失源码外推 |

共同基线是四个不变量：稳定capture owner；typed terminal与receipt；selection/picking/highlight绑定view和generation；broad phase后有真实geometry验证或明确的qualified fallback。低开销来自增量索引、稳定产品和有界查询，不来自删除身份/currentness/error合同。

## 7. 目标架构与唯一owner

### 7.1 核心产品

1. `ViewportInteractionSessionId`：绑定document/world、viewport session、window、view generation、owner epoch。
2. `ViewportInputEnvelope`：保存user/device/pointer/source、window/surface/viewport、sequence/time、buttons/modifiers、position/delta、phase、reason与capture generation。
3. `ViewportCaptureLease`：由interactive tool authority签发，指定owner、pointer/channel、priority、generation、terminal policy与force-end入口。
4. `InteractiveTransformSession`：保留当前唯一批量owner，增加capture/session identity与terminal receipt接线，不复制第二套事务。
5. `SelectionEligibilitySnapshot`：同generation冻结active/hidden/locked/document/context/tool/editability规则与rejection reason。
6. `SelectableSpatialProduct`：renderer/authoring extract按view/frame发布owner、instance/subobject、visibility、eligibility、world/screen bounds、geometry accelerator与source generation。
7. `ResolvedViewportHitList`：保留ordered hits、depth/point/normal/subobject/backend/completeness/product generation；top route只是consumer策略。
8. `ViewportHighlightProduct`：独立overlay revision、selection/settings source revision、per-view lifetime、remove/tombstone与consumer receipt。
9. `ViewportInteractionReceipt`：记录Consumed/NoHit/Stale/Rejected/Accepted/Cancelled/OwnerLost及selection/edit结果和诊断identity。

### 7.2 Owner边界

| Owner | 唯一职责 | 禁止承担 |
|---|---|---|
| Runtime/host input | qualified physical input与capture lifecycle | 决定Scene selection或transaction commit |
| Interactive tool authority | capture lease、owner epoch、priority与force termination | 直接写Scene transform |
| Scene Viewport adapter | qualified route、selection policy与hit/terminal receipt消费 | 丢弃身份后发全局命令 |
| Runtime picking authority | ray/backend/resolve/event与ordered hit list | Editor私有circle route冒充runtime precision product |
| Editor transaction owner | preview、batch commit、rollback、undo/redo与autosave currentness | controller根据variant变化猜commit |
| Runtime renderer | selectable/highlight per-view product与consumer receipt | 用store存在冒充已渲染/已呈现 |

## 8. 必须硬切的旧路径

- 删除bare `EditorViewportEvent`/`ViewportInput`作为authoring产品边界；内部枚举只能从qualified envelope派生。
- 删除`drag: Option<ViewportDragSession>`作为capture authority，迁移为lease-owned session state machine；第二按键策略显式化。
- 删除通过Handle variant消失推断commit；Accept/Cancel/OwnerLost只消费typed terminal disposition。
- 删除`HandleTool::end_drag() -> ()`及三个空实现，统一返回terminal receipt。
- 保留唯一`InteractiveTransformSession`和`BatchTransformCommand`，禁止在controller/workbench另建capture/transaction；补managed behavior/profile，而不是重写已正确部分。
- 删除production Scale的正值`.max(0.05)`，由统一transform policy决定negative/zero/singular语义。
- 删除固定pointer id 1、camera id 0和`.first()`作为最终Editor消费边界。
- 删除把origin/scale circle称为precision hit；只能作为带completeness/qualification的fallback。
- 删除Box与Frame各自扫描另一套代理数据，统一消费same-generation selectable product。
- 删除只存不消费的Highlight完成性叙述；真实frame extract读取并返回Consumed/Presented前保持Partial/Unavailable。

## 9. 分层重构里程碑

### M0 · Qualified input与terminal hard cut

1. Retained bridge改走metadata dispatch，建立pointer/window/surface/viewport/capture generation envelope。
2. Capture lease成为唯一owner；Pointer Cancel、Escape、focus/window/viewport close、plugin unload和shutdown均带scope/reason。
3. Accept/Cancel/OwnerLost返回receipt，并原子退休UI capture、controller session和interactive transform；旧generation fail closed。

### M1 · 完成事务外围接线与数学策略

1. 保留现有selection-root、frozen basis、parent inverse、batch command和补偿回滚内核。
2. Handle只生成world target/delta；terminal disposition显式驱动现有session finish/cancel，禁止状态差分猜测。
3. 定义negative/zero/mirror/singular policy，覆盖负/非均匀父scale、跨零、不可表示shear与all-root rollback。
4. 执行现有Rust行为/journal replay，并补10k roots/deep hierarchy Windows profile、allocation、peak memory和frame budget。

### M2 · Selection eligibility与Selectable Spatial Product

1. 发布per-view/per-frame eligibility + selectable snapshot，统一visibility、lock/hidden/context、bounds、instance和geometry accelerator。
2. Runtime picking返回qualified ordered hit list与completeness；Editor实现top/cycle/list/behind-object策略。
3. Point、Box、Frame统一消费同一产品，完成occlusion、near plane、extreme scale和真实bounds策略。

### M3 · Highlight frame consumption

1. Runtime frame extract读取per-view highlight product，把独立overlay revision纳入cache/currentness。
2. 实现remove/tombstone、closed viewport teardown、独立dirty/retry/degraded状态。
3. 串联Submitted/Consumed/Presented receipt，禁止串viewport或沿用stale set。

### M4 · 产品资格与对标

1. 在100k/1m selectable、1kHz pointer、10k selection roots和4/16 viewport下测CPU、GPU、内存、allocation与p95/p99。
2. 注入stale generation、target deletion、world replace、gateway/device/window/plugin/transaction fault并证明完整rollback/retirement。
3. 同机器、同scene、同画质、同正确性和同恢复资格对照Unreal/Godot/Fyrox；没有receipt不得声明领先。

## 10. 36个资格门当前重判

| Gate | 状态 | 当前证据 / 缺口 |
|---|---|---|
| G01 Pointer Cancel到当前capture owner并有terminal receipt | Partial | 到达全局viewport命令；无qualified owner/receipt |
| G02 Cancel原子退休capture/controller/edit | Partial | 单controller与batch edit会清理；无共同generation receipt |
| G03 Cancel/OwnerLost恢复所有preview且history不新增 | Partial | batch session可恢复所有roots；无qualified OwnerLost与端到端multi-root receipt |
| G04 stale/duplicate Cancel不影响新capture | Fail | authoring链无capture generation |
| G05 Handle期间第二按键不隐式Commit | Pass | guard与聚焦回归源码存在 |
| G06 release只终止同pointer/button/generation | Partial | variant/button匹配；无pointer/capture generation |
| G07 Escape/focus/window/viewport/shutdown reason明确 | Fail | 无scope、reason和receipt |
| G08 shutdown枚举全部session | Fail | 无session registry |
| G09 tool unload先终止capture/edit | Fail | Scene Viewport未接lease owner epoch |
| G10 Highlight失败时base frame继续 | Pass | CapabilityMissing fault test源码存在 |
| G11 Highlight失败独立dirty/retry/Degraded | Fail | 仅日志 |
| G12 base与overlay失败typed状态分离 | Fail | 无overlay状态产品 |
| G13 Highlight revision含selection/settings revision | Fail | selection revision兼作generation |
| G14 overlay revision使frame cache更新 | Fail | 无frame consumer/revision |
| G15 closed viewport highlight被remove/tombstone | Fail | store无remove |
| G16 Submitted/Consumed/Presented可追踪且隔离viewport | Fail | 只有Submitted/store latest |
| G17 broad phase后真实geometry或qualified fallback | Fail | 投影代理仍是final hit |
| G18 hit receipt含view/frame/backend/instance/subobject | Fail | Editor只返回route |
| G19 稳定ordered candidate list与选择策略 | Fail | `.first()`丢弃其余候选 |
| G20 alpha/instance/skinned/thin/offset命中策略 | Fail | 无precision parity |
| G21 Box消费same-generation visibility/spatial product | Fail | 遍历独立代理圆/shape |
| G22 Box方向/遮挡/hidden/locked/near-plane golden | Fail | 无完整policy/测试 |
| G23 Frame使用真实bounds并处理极端scale | Fail | 只用node位置 |
| G24 wrong-view/frame/stale snapshot fail closed | Partial | visible snapshot有world/viewport/frame/view identity；无完整receipt |
| G25 多选仅作用selection roots且父子不重复 | Pass | root过滤与parent/child专门测试源码存在 |
| G26 world/local/parent在非均匀负父scale下定义 | Partial | parent inverse与非均匀shear typed reject已有；negative parent矩阵未完整覆盖 |
| G27 negative/zero/mirror/singular统一policy | Fail | Handle仍正值0.05钳制 |
| G28 preview/commit/cancel共享frozen basis/affected set | Pass | 唯一session冻结并复用同一roots/snapshots/pivot |
| G29 一个typed batch command和正确kind metadata | Pass | BatchTransformCommand、kind label、100-preview单command/history源码存在 |
| G30 deletion/world/plugin/transaction fault完整rollback | Partial | deletion/world/transaction有局部测试；无plugin/owner-loss与全链receipt |
| G31 pointer hot path不构造全packet/复制mesh | Pass | Stale/Preparing + render publication + visible query owner map |
| G32 100k/1m point/box/frame预算 | Fail | 无统一产品规模receipt |
| G33 高频motion合并不改变edge order | Fail | 无sequence/coalescing合同 |
| G34 multi-window/viewport/pointer无cross-cancel | Fail | authoring事件无identity |
| G35 soak后0 capture/edit/stale highlight/orphan product | Fail | 无soak与ledger |
| G36 性能比较满足同正确性/画质/恢复资格 | Fail | 无跨引擎benchmark receipt |

## 11. 验证边界与实施入口

本轮只做current-working-tree静态review、参考源码对照、状态重判与文档记录。没有修改Editor/Runtime/plugin/Cargo/ABI/ZUI/assets或测试；没有运行Cargo、Editor、真实GPU picking、touch/pen、多window、父子负scale、highlight render、fault、scale、soak或benchmark。606个Zircon test markers与56个ignored markers只证明源码存在，不代表本轮执行通过；报告也不宣称当前工作树可构建或性能优于Unreal。

后续实现必须从M0开始，把qualified input与terminal receipt接到现有批量transaction owner；不要重写已经正确的selection-root/frozen-basis/batch-command内核。M1随后关闭Scale数学策略并完成managed行为/profile，M2/M3分别由picking/renderer与highlight frame owner补齐产品消费。Tooling不在本轮范围；按用户要求，本轮没有查询、轮询、等待或实时跟踪协调器。

## 12. 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
