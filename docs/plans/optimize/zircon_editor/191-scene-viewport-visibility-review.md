---
title: Editor Scene Viewport Object Visibility、Temporary Hide、Isolate/Local View、Selection Eligibility、Hierarchy Feedback、Persistence、Performance 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor191
review_date: 2026-08-28
baseline_head: 681588f7a1cbfaae3147e8b93e1be6705d810f21
related_code:
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/ui/binding/viewport
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/hierarchy.rs
  - zircon_runtime/src/core/framework/render/camera/extract_request.rs
  - zircon_runtime/src/core/framework/render/visible_spatial_query.rs
  - zircon_runtime/src/scene/components/scene/activation.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/world/render_visibility.rs
tests:
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_selection.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_reset_from_scene.rs
  - zircon_editor/src/scene/viewport/interaction_extract
  - zircon_editor/src/scene/viewport/pointer
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/host/binding_dispatch/viewport.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport
  - zircon_editor/src/tests/host/retained_viewport_toolbar_pointer
  - zircon_editor/src/tests/host/retained_hierarchy_template_body.rs
  - zircon_editor/src/tests/ui/binding/viewport.rs
  - zircon_editor/src/tests/workbench/chrome_snapshot/viewport_settings.rs
  - zircon_runtime/src/scene/tests/derived_state.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/70-editor-scene-viewport-object-visibility-temporary-hide-isolate-local-view-selection-eligibility-hierarchy-feedback-persistence-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/179-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-current-source-review.md
  - docs/plans/optimize/zircon_editor/181-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/189-editor-scene-viewport-display-mode-lighting-skybox-show-flag-debug-visualization-overlay-composition-profile-persistence-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/05/failure-2026-07-18-viewport-pointer-candidate-regeneration.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorActor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ActorEditor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/Actor.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/LevelEditorViewport.cpp
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/SceneOutlinerGutter.cpp
  - dev/godot/editor/scene/scene_tree_editor.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_plugin.cpp
  - dev/Fyrox/editor/src/scene/mod.rs
  - dev/Fyrox/editor/src/scene/controller.rs
  - dev/Fyrox/editor/src/scene/selector.rs
  - dev/Fyrox/editor/src/scene/commands/graph.rs
  - dev/Fyrox/editor/src/menu/mod.rs
  - dev/Fyrox/editor/src/settings/navmesh.rs
  - dev/Fyrox/editor/src/world/selection.rs
  - dev/Fyrox/fyrox-impl/src/scene/graph/mod.rs
  - dev/bevy/crates/bevy_camera/src/visibility/mod.rs
  - dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Culling/InstanceCuller.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/GPUDriven/GPUDrivenRenderingTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Decal/DecalProjector.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/VolumetricLighting/LocalVolumetricFog.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/RenderPass/CustomPass/CustomPassVolume.cs
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/70-editor-scene-viewport-object-visibility-temporary-hide-isolate-local-view-selection-eligibility-hierarchy-feedback-persistence-performance-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/70-editor-scene-viewport-object-visibility-temporary-hide-isolate-local-view-selection-eligibility-hierarchy-feedback-persistence-performance-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Scene Viewport Object Visibility、Temporary Hide、Isolate/Local View、Selection Eligibility、Hierarchy Feedback、Persistence、Performance 与 Product Integration 当前源码复核

## 1. 结论

Editor70之后，Zircon的Scene Viewport交互提取与renderer-visible picking底座有实质进展。`ViewportInteractionExtractCache`按world generation、selection、settings、camera和viewport冻结同代render/pointer payload；pointer侧可接纳immutable `RenderVisibleSpatialQuerySnapshot`，以O(1) owner map查询renderer实际可见对象，并发布visited/candidate/hit/projected count。交互提取变化会清掉旧renderer snapshot，world replacement即使得到相同generation也会重建cache与pointer router。这些机制应保留并纳入对象可见性代际链。

Runtime authored/derived/render分层同样真实。`ActiveSelf`与`RenderLayerMask`是public、serializable作者事实，`ActiveInHierarchy`明确不序列化；Mesh、Sprite、Particle、Ambient/Directional/Point/Rect/Spot Light与Post Process Volume均在提取时检查active/layer。`VisibilityInput`还能按stable instance排序，并区分static/dynamic entity set。这个基础证明临时隐藏必须作为view-scoped输入组合到提取链，绝不能通过修改`ActiveSelf`或Layer伪装。

但对象临时隐藏产品本身仍为零。`ViewportCommand`、`SceneViewportSettings`、`SceneViewportState`、`SceneViewportExtractRequest`没有Hide Selected、Hide Unselected、Show All、Toggle、Isolate/Local View、filter generation或effective receipt。目标类型`ViewportObjectVisibilitySessionRegistry`、`ViewportObjectVisibilityProfile`、`EffectiveViewportObjectVisibilityReceipt`、`SelectionEligibilitySnapshot`、`OutlinerEffectiveObjectStateProjection`、`CompiledViewportObjectFilter`、`ObjectParticipationResolver`与`ViewportObjectVisibilityIntent`在当前tracked和untracked Rust语料均无实现。

当前一致性风险仍然存在。Renderer snapshot身份包含world、viewport、frame generation与view，但Editor接纳入口只检查world generation；snapshot缺失时point picking回退到全部renderable candidates，rectangle selection始终遍历全部interaction candidates。`select_nodes`只验证节点存在，Frame Selection只读取当前selection的位置。scene gizmo、selection anchor、handle与highlight没有visibility-derived eligibility，interaction key也没有filter revision。

Outliner产品合同没有进展。最终`SceneNodeData`仅有`id/name/depth/selected`，投影代码只复制这四项；没有requested/effective hidden、reason、partial/mixed、isolation membership、selection eligibility或恢复入口。`reset_from_scene`能清interaction cache、hover、drag和pointer router，却没有可退休、挂起或恢复的visibility session。

本轮不新增P0。Editor70的32项P1当前为 **20 Open / 12 Partial**，8项P2为 **8 Open**；48门为 **32 Fail / 16 Partial / 0 Pass**。Partial只表示通用render/picking/currentness基础可复用，不表示临时隐藏工作流已可达。

本轮只做review，未修改production Rust，未运行Cargo、Editor、GUI/GPU、render golden、save/reopen、multi-view、Play/prefab/stage、fault/scale/soak/profile或同硬件跨引擎benchmark。Tooling按用户要求排除；没有查询、轮询、等待或实时跟踪协调器。当前不能声称该域功能、表现或性能达到或超过Unreal。

## 2. 审查边界与冻结语料

### 2.1 Current working tree

主仓HEAD为`681588f7a1cbfaae3147e8b93e1be6705d810f21`。本报告以2026-08-28读取时当前磁盘为事实源；大量viewport/runtime文件包含其他会话的modified或untracked实现。本轮不回退、不格式化、不吸收这些生产代码，只按实际行为更新review。

MVP baseline recovery仍为`in_progress`。Editor05下与pointer candidate regeneration相关的failure记录说明共享projection/current-generation路径已建立，但spatial/BVH规模、1/1K/10K query命中与managed validation仍未闭合；它是本报告的可复用基础与资格缺口，不阻塞本轮只读审查。

### 2.2 冻结物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Editor viewport/product | **134 / 9,942 / 9,086 / 349,834 / 72** | viewport全树、binding、render submission与Hierarchy投影 | `e4e625bfdcdee2edf2ab835a6d2e674561cc45b490558e9a84a456a6156694ef` |
| Runtime visibility/extract | **8 / 2,665 / 2,464 / 99,361 / 16** | active/layer、全部一等贡献、VisibilityInput与visible-spatial snapshot | `26b008148d6e45da42282ce6a5b63587abbb802d3ea95bebba5691716df60326` |
| Focused tests | **18 / 3,538 / 3,216 / 124,654 / 80** | viewport route/pointer/hierarchy与Runtime active/layer/extract | `175a5974cc2754b97bb0399956080bb58b83c3fe20395ed6a5c8a6771d9cbc31` |
| Zircon deduplicated focused set | **160 / 16,145 / 14,766 / 573,849 / 168** | 上述集合按规范化路径去重 | `f1b7dd906382b3955c163c43778470bc0b1b1780c9cfdb1aa17f52ef65434c29` |
| Unreal selected set | **5 / 16,766 / 14,193 / 671,331 / 0** | temporary hide、hierarchy、selection、per-view bit与Outliner | `a67e0830ba271bcbfe935c981ca7e32558beaf863b2b12dfde1b314eb9ace08c` |
| Godot selected set | **2 / 6,821 / 5,761 / 257,886 / 0** | authored visibility UndoRedo与edit lock | `eac862f99b152994c19b35efef391efe09773436da367e0037462790eed57e20` |
| Fyrox selected set | **8 / 6,185 / 5,550 / 215,884 / 10** | selection/menu/helper visibility与对象隔离负证据 | `fc62be6529b7431fcc0210f3f1c5760a04dbf96c03f006365f425b3e98410ead` |
| Bevy selected set | **2 / 1,816 / 1,617 / 67,327 / 13** | authored/inherited/view visibility、per-view set与wide layers | `be10a812543c1b61597256e6b28b7b5ff1e8b68d89d4bb60a91ce3a48d5c41f3` |
| Unity Graphics selected set | **5 / 5,464 / 4,580 / 244,032 / 1** | GPU culling、picking/outline、decal/fog/custom pass consumers | `ae8d180495dd7c88a481c64c2f6a42c1e2f76ef475189d3c05481fdd7775a60c` |
| Five-engine deduplicated set | **22 / 37,052 / 31,701 / 1,456,460 / 24** | 五类本地参考按路径去重 | `3cd59ceb2375fddf2de3173730ad266329f483a1859f22b4165c733f4ec3bed3` |

fingerprint按小写规范化相对路径排序，将每个`path + newline + file SHA-256 + newline`聚合后再做SHA-256，只证明本轮working-tree选择集。Godot、Fyrox、Bevy与Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal跟随主workspace。

### 2.3 Owner边界

Editor191只刷新Editor70拥有的per-viewport transient object visibility。Editor60/181继续拥有Outliner本体和persistent authoring visibility/lock transaction；Editor58/179拥有ViewInstance/session/surface/currentness；Editor59拥有通用selection/picking/highlight/interaction；Editor68/189拥有Show Flag与overlay composition；Runtime Scene拥有`ActiveSelf`、`ActiveInHierarchy`、Layer、visibility/spatial与render extract。实施必须通过合同连接这些owner，不能在Editor191复制第二套World、selection或renderer authority。

## 3. 当前实现拓扑

### 3.1 控制面与产品状态为空

`ViewportCommand`止于navigation、transform/view settings、overlay provider与Frame Selection；settings/state也没有对象visibility字段。全仓目标类型与Hide/Isolate产品命令搜索为零，现存`ShowAllCategory`属于UI showcase，不是Scene对象显示命令。

### 3.2 作者态边界正确但没有transient类型

`ActiveSelf`和`RenderLayerMask`会序列化，`ActiveInHierarchy`是派生事实。它们适合Runtime truth，不适合view-local临时过滤。当前没有类型系统保证Show All不会改作者态，也没有scene hash/dirty/undo/save不变测试。

### 3.3 各贡献有active/layer检查但没有唯一resolver

Mesh与Sprite在构造snapshot前检查active/layer；Particle和五类Light各自重复同类判断；Post Process Volume还组合camera volume/render layers。`SceneViewportExtractRequest`没有compiled object filter，`VisibilityInput`在贡献全部收集后才生成。隐藏原因、show flag、stage/context和transient set没有单一precedence或receipt。

### 3.4 Point picking进步，box/frame/selection admission仍分叉

Point precision route可以查询renderer-visible spatial snapshot并复用owner map；但Editor adoption只核world，fallback仍为所有renderables。`selectable_owners_in_rect`遍历`renderable_candidates`与scene gizmos，不查询renderer snapshot。Frame Selection读取所有selected entity的world position；`select_nodes`只检查节点存在。因此同一隐藏对象可在不同入口得到不同结果。

### 3.5 Cache与world replacement有基础但没有filter generation

Interaction key含world generation、selected、settings、camera和viewport，cache miss有profiling；同步新extract会清旧renderer snapshot。`reset_from_scene`测试还覆盖“两个不同World具有相同generation”时必须重建。缺少的是ViewInstance/Document/World epoch/filter generation/digest统一身份，以及visibility-only精准失效。

### 3.6 Overlay、Hierarchy与lifecycle未消费effective visibility

Scene gizmo只看`active_in_hierarchy`，selection anchor主要看节点/mesh存在。最终Hierarchy row只有四字段，没有隐藏原因、partial/mixed或eligibility。Controller reset只处理交互状态，没有visibility profile、isolation stack、close/Play/prefab/provider unload的retirement choreography。

## 4. 五引擎参考与适用边界

### 4.1 Unreal

`EditorActor.cpp`把temporary visibility写入`SetIsTemporarilyHiddenInEditor`，Hide Selected支持显式child hierarchy并取消选择，Hide Unselected与UnHide All是独立路径。`ActorEditor.cpp::IsHiddenEd`组合layer、editable、temporary与level原因，setter会标脏render state。`Actor.h`把`bHiddenEdTemporary`和`HiddenEditorViews`标为Transient，并保留layer/level/editable等不同事实；Level viewport与Scene Outliner分别处理per-view layer bit和层级反馈。

边界：Unreal actor temporary hide本身不是天然per-view，`HiddenEditorViews`主要服务per-view layer。Zircon选择ViewInstance私有对象filter是更严格的产品合同，不能声称直接复制了Unreal语义。

### 4.2 Godot

SceneTree可见按钮调用节点`is_visible/set_visible`并以UndoRedo提交批量drag，lock使用`_edit_lock_`metadata并进入选择/操作约束。它证明作者visibility与lock必须transactional、可反馈；它不是Local View完整参考，也不能把其visibility drag临时过程当成view-local隔离。

### 4.3 Fyrox

所选Editor scene/controller/selector/menu/selection语料包含选择、grid/gizmo/helper与Navmesh show选项，却没有Hide Selected、Hide Unselected、Show All或Local View对象产品。Graph中的`isolate_node`是graph ownership/删除过程，不是viewport isolation。Fyrox在此只作为负证据，不能成为目标上限。

### 4.4 Bevy

Bevy明确分开`Visibility`、`InheritedVisibility`与算法产生的`ViewVisibility`，per-camera `VisibleEntities`按类型维护结果，current/previous bit避免无意义change detection。`RenderLayers`以SmallVec扩展宽位集，并定义empty/intersection/传播与测试。可借鉴的是事实分层、per-view结果和规模原语，不是Editor工作流。

### 4.5 Unity Graphics

本地Graphics镜像不包含私有`SceneVisibilityManager`实现，只能验证consumer。`InstanceCuller`在Scene culling mask、Picking、SelectionOutline和Filtering中执行early filter，并区分null与empty include list以避免outline错误地高亮全部对象。HDRP Decal、Local Volumetric Fog和Custom Pass Volume订阅visibility change或读取scene culling mask/prefab isolation，证明非mesh贡献必须同步。GPUDriven hide/show test被标记不稳定，不能作为强动态资格。

## 5. 差异矩阵

| 能力 | Zircon当前事实 | 工程级目标 | 判定 |
|---|---|---|---|
| Temporary Hide | 无意图、状态、入口 | qualified per-view intent/session/receipt | Missing |
| Hide Unselected / Local View | 无 | reversible nested isolation stack | Missing |
| 作者态边界 | active/derived/layer分层正确 | transient type保证不dirty/save | Partial |
| Render participation | 各贡献重复active/layer | immutable compiled filter + single resolver | Partial |
| Per-view currentness | visible snapshot身份完整，Editor只核world | filter generation贯穿extract/query/UI | Partial |
| Point picking | renderer-visible query与fallback并存 | current eligible set，缺失时fail-close | Partial |
| Box/Frame/Select All | 各自遍历不同输入 | 同一SelectionEligibilitySnapshot | Missing |
| Overlay/highlight | 共享extract有基础 | 全consumer同filter generation | Partial |
| Hierarchy feedback | row仅四字段 | effective reason、mixed、restore | Missing |
| Lifecycle/persistence | reset通用interaction状态 | session retire、profile schema、restore | Missing |
| Scale/diagnostics | query counters与static/dynamic set | incremental compile、100K/1M、receipt provenance | Partial |
| Qualification | 通用pointer/render tests | visibility产品/fault/backend/cross-engine matrix | Missing |

## 6. Canonical finding状态

### 6.1 P1

#### ED70-P1-01 [Open]：没有Viewport Object Visibility Session authority

没有以ViewInstance、Document、World epoch和owner generation限定的registry；多view隔离、stale reject和close retirement无处实现。

#### ED70-P1-02 [Partial]：身份底座存在，filter generation未贯通

Visible snapshot有world/viewport/frame/view，interaction key也有world/camera/viewport；visibility intent、compiled filter、Outliner和selection没有共同generation/digest。

#### ED70-P1-03 [Open]：没有与ActiveSelf/RenderLayerMask硬分离的transient类型

Runtime authored/derived分层可保留，但没有任何类型或测试保证临时命令不改scene、dirty、undo、save和Play事实。

#### ED70-P1-04 [Open]：没有单一effective visibility precedence resolver

authoring active/loaded/layer、object filter、Show Flag、stage/context与provider没有确定顺序、reason vector或冲突规则。

#### ED70-P1-05 [Open]：没有qualified object/subobject handle与stale pruning

selection/candidate仍大量使用裸`u64`，无法区分world替换、ID复用、prefab/stage或provider generation。

#### ED70-P1-06 [Open]：没有immutable compiled filter与effective receipt

缺requested/effective/rejected/stale count、digest、source revision、capability、fallback与reason。

#### ED70-P1-07 [Open]：Hide Selected工作流缺席

没有selection snapshot、exact/hierarchy plan、hidden-selection policy、命令回执或原子失败规则。

#### ED70-P1-08 [Open]：Hide Unselected / Isolate / Local View缺席

没有可逆allow set、nested stack、breadcrumb或空选择/全选择语义。

#### ED70-P1-09 [Open]：Show All、Toggle、Reveal与isolation pop缺席

没有只清transient state的恢复命令，也没有idempotence和stale receipt。

#### ED70-P1-10 [Open]：exact/hierarchy scope与partial/mixed未定义

Runtime active hierarchy不能替代Editor transient hierarchy plan；父子增删、重挂和部分隐藏没有政策。

#### ED70-P1-11 [Open]：selection retain/clear/restore语义未定义

Unreal hide-selected会deselect；Zircon没有明确保留、清除、ghost、restore或active-primary规则。

#### ED70-P1-12 [Open]：没有Selection Eligibility compositor

visibility、lock、editable、loaded、stage、tool capability和provider denial没有组合为immutable资格快照。

#### ED70-P1-13 [Open]：Outliner没有effective hidden reason/tri-state

最终row仍只有`id/name/depth/selected`，无法解释authoring hidden、parent hidden、local view exclusion或rejected state。

#### ED70-P1-14 [Open]：command/menu/keymap/automation没有统一intent与admission

对象visibility命令不存在，更没有权限、read-only、Play/stage和plugin capability准入。

#### ED70-P1-15 [Open]：SceneViewportExtractRequest没有object filter

Runtime-neutral request无法携带compiled filter identity、representation或policy。

#### ED70-P1-16 [Partial]：一等贡献已有active/layer过滤，缺统一transient predicate

Mesh、Sprite、Particle、五类Light和Post Process Volume均有基础判断；实现仍分散且完全不认识view-local filter。

#### ED70-P1-17 [Partial]：overlay共享extract基础存在，仍无effective visibility

Render和pointer能共享interaction payload，scene gizmo/anchor/handle/highlight仍未消费object filter generation。

#### ED70-P1-18 [Open]：shadow/reflection/GI/GPU visibility与辅助view传播未定义

没有证明hidden对象不会进入shadow、depth、velocity、reflection、GI、virtual geometry或GPU scene路径。

#### ED70-P1-19 [Open]：view-only filter与simulation副作用边界未定义

没有类型级保证physics、audio、script、navigation、animation与save不受临时隐藏影响。

#### ED70-P1-20 [Partial]：visible-spatial身份增强，Editor接纳仍只核world

Runtime identity已含viewport/frame/view；Editor缺filter generation并忽略其余身份，旧snapshot仍可能越代。

#### ED70-P1-21 [Partial]：extract变更会清snapshot，缺失时仍全候选fallback

同步新interaction extract时fail-closed一部分旧状态；没有snapshot时仍把全部renderables重新引入picking。

#### ED70-P1-22 [Partial]：point query改善，box/frame/select-all未共用eligible set

Rectangle遍历renderable/gizmo candidates，Frame读取selection位置；没有统一可见且可选集合。

#### ED70-P1-23 [Partial]：cache/currentness基础存在，key缺visibility revision

world/selection/settings/camera/viewport可触发cache miss并清旧snapshot；visibility-only改变无法精准失效。

#### ED70-P1-24 [Partial]：spatial/static-dynamic基础存在，缺filter增量更新

Runtime有sorted visibility input、static/dynamic set和renderer spatial query；没有sparse/dense filter编译、GPU index patch或层级delta政策。

#### ED70-P1-25 [Open]：multi-viewport与view close cleanup缺席

通用controller状态不能表达每view独立hidden set、isolation stack、shared world或close retirement。

#### ED70-P1-26 [Partial]：world replacement能清交互状态，visibility set无stale规则

当前测试证明相同generation的不同World也会重建interaction cache；尚无旧filter/session/receipt可拒绝。

#### ED70-P1-27 [Open]：Play、prefab、stage、context与provider unload转换未定义

缺suspend/retire/restore顺序、generation fence和unknown owner preservation。

#### ED70-P1-28 [Open]：persistence scope/schema/crash restore缺席

未决定ephemeral、user、workspace或view scope，也没有version、migration、atomic write和unknown provider策略。

#### ED70-P1-29 [Partial]：局部线性/去重纪律存在，100K/1M filter表示未设计

VisibilityInput排序去重，renderable candidates按输入分组避免常态HashSet并记录payload；没有大规模编译/toggle/层级delta证据。

#### ED70-P1-30 [Partial]：query diagnostics存在，visibility provenance缺失

有visited/candidate/hit/projected count与owner-map payload counter；没有view/filter/reason/hidden/rejected/stale receipt。

#### ED70-P1-31 [Partial]：pointer/currentness回归测试增强，产品测试为零

共享generation、owner query、world replacement和fallback行为有测试；temporary hide/isolate/selection/Hierarchy/aux-consumer没有测试。

#### ED70-P1-32 [Open]：没有跨backend、scale与跨引擎资格

未执行真实Windows Editor、GPU backend、pixel/golden、100K/1M、multi-view、fault/soak或同语义基准。

### 6.2 P2

#### ED70-P2-01 [Open]：没有named visibility set/preset

缺稳定身份、版本、scope、冲突和分享政策。

#### ED70-P2-02 [Open]：没有private/shared/multi-user overlay policy

缺本地私有过滤与协作共享状态边界。

#### ED70-P2-03 [Open]：没有headless/automation visibility query API

Visible-spatial query不是产品profile/receipt API，automation无法执行并核验Hide/Isolate。

#### ED70-P2-04 [Open]：没有plugin predicate与unknown provider preservation

Overlay provider registry不能贡献对象participation predicate，卸载/未知配置也没有保留规则。

#### ED70-P2-05 [Open]：没有accessibility/keyboard/reason projection

缺可访问名称、焦点顺序、disabled reason和非颜色状态表达。

#### ED70-P2-06 [Open]：没有isolation history/breadcrumb/compare

Nested stack、跳转、差异预览与精确回滚均不存在。

#### ED70-P2-07 [Open]：没有redactable receipt export/replay

无法导出、脱敏、重放visibility intent与effective result。

#### ED70-P2-08 [Open]：没有unloaded descriptor/World Partition/remote extension

当前合同只覆盖loaded裸entity，无法表示未加载对象、远端runtime或分区描述符。

## 7. 目标架构

### 7.1 Editor authority

建立`ViewportObjectVisibilitySessionRegistry`，以ViewInstance + DocumentSession + World epoch + owner generation寻址；`ViewportObjectVisibilityProfile`保存明确scope与schema。所有入口只发`ViewportObjectVisibilityIntent`，resolver生成immutable `EffectiveViewportObjectVisibilityReceipt`。

### 7.2 Runtime-neutral filter

Editor把qualified object plan编译为`CompiledViewportObjectFilter`，其中包含generation、digest、representation、world identity与stale policy。Runtime `ObjectParticipationResolver`只消费filter、authoring active/layer、show flag和stage/context，产出统一decision/reason；它不回调Editor、不修改Scene。

### 7.3 Selection与Hierarchy

同一代resolver投影`SelectionEligibilitySnapshot`和`OutlinerEffectiveObjectStateProjection`。Point/box/frame/select-all、Hierarchy click、automation、gizmo/handle/highlight必须只消费该快照或等价generation；UI展示requested/effective/rejected/stale和mixed reason。

### 7.4 性能与currentness

先以qualified handles构建sparse set；达到阈值后切dense bitset/range/hierarchy plan，阈值由benchmark决定。filter revision进入render extract、interaction key、visible-spatial identity、GPU/static index patch和Hierarchy projection。steady pointer event禁止全World扫描、重编译filter或重建owner table。

## 8. 重构里程碑

### ED70-M1：类型、身份与纯状态机

定义intent/profile/session/filter/receipt/qualified handle、scope、reason与transition sequence；以property tests冻结Show All不改作者态、nested isolation精确恢复和stale generation拒绝。

### ED70-M2：Runtime participation resolver

把active/layer/show flag/stage/object filter收敛为单一纯resolver，在资源/primitive展开前应用；迁移Mesh、Sprite、Particle、Light、Volume及所有辅助贡献，删除旁路bool组合。

### ED70-M3：render、visible-spatial与GPU currentness

filter generation进入extract request、frame/view identity、visibility input、spatial snapshot与GPU scene patch；定义shadow/reflection/GI/depth/velocity政策并拒绝旧产品。

### ED70-M4：Selection eligibility与overlay

构建immutable eligibility snapshot，迁移point/box/frame/select-all、Hierarchy、automation、gizmo/anchor/handle/highlight；snapshot缺失或stale时fail-close并请求重建。

### ED70-M5：产品命令与Outliner反馈

实现Hide Selected/Unselected、Toggle、Show All、Push/Pop Isolation、Reveal和exact/hierarchy scope；toolbar/menu/keymap/automation共用typed intent，Outliner投影原因、mixed与恢复入口。

### ED70-M6：Lifecycle、multi-view与persistence

实现view close、world replace、reload、Play、prefab/stage、provider unload的quiesce/retire/restore；验证两个view独立。可选profile按明确user/workspace/view scope版本化、原子保存和迁移。

### ED70-M7：规模表示与增量更新

以benchmark选择sparse/dense/hierarchy阈值，增量处理spawn/despawn/reparent/load/unload和static/GPU index；记录compile/toggle/p95、allocation和steady-state reuse。

### ED70-M8：Diagnostics、fault与产品测试

增加qualified receipt/provenance、filter compile和consumer counters；覆盖stale、OOM、plugin panic、device loss、world churn、rapid toggle、save/reopen和crash restore。

### ED70-M9：硬切与跨引擎资格

删除所有按调用点临时过滤、`ActiveSelf`/Layer冒充、全候选fallback和双重selection规则。仅在同scene/camera/visibility trace/quality/hardware下比较Zircon与参考引擎。

## 9. 资格门

| Gate | 要求与当前证据 | 当前 |
|---|---|---|
| ED70-G01 | transient与ActiveSelf/RenderLayerMask具有硬类型边界 | Fail |
| ED70-G02 | View/Document/World/Session/Filter generation共同限定状态 | Partial |
| ED70-G03 | qualified address拒绝删除、ID复用和旧world | Fail |
| ED70-G04 | 纯状态机给出稳定digest与immutable receipt | Fail |
| ED70-G05 | Hide/Isolate/Show All前后authoring scene hash不变 | Fail |
| ED70-G06 | Hide Selected exact scope端到端通过 | Fail |
| ED70-G07 | Hide Selected hierarchy scope端到端通过 | Fail |
| ED70-G08 | Hide Unselected/Push Isolation可逆 | Fail |
| ED70-G09 | nested isolation Pop精确恢复 | Fail |
| ED70-G10 | Show All只清transient state | Fail |
| ED70-G11 | Toggle/Reveal幂等并拒绝stale receipt | Fail |
| ED70-G12 | hierarchy mixed/partial政策稳定 | Fail |
| ED70-G13 | Runtime只消费compiled filter，不回调Editor | Fail |
| ED70-G14 | 单一resolver输出precedence与reason | Fail |
| ED70-G15 | Mesh/Sprite/Particle已有active/layer，transient predicate缺失 | Partial |
| ED70-G16 | 五类Light已有active/layer，transient predicate缺失 | Partial |
| ED70-G17 | Volume有active/layer；Decal/Fog/CustomPass统一对象政策未闭合 | Partial |
| ED70-G18 | shadow/reflection/GI/depth/velocity政策通过 | Fail |
| ED70-G19 | active/layer可在snapshot前裁剪；object filter early-cull缺失 | Partial |
| ED70-G20 | 隐藏不影响physics/audio/script/nav/simulation | Fail |
| ED70-G21 | gizmo/icon/anchor/handle/pick消费同一generation | Partial |
| ED70-G22 | highlight/outline与base render一致 | Partial |
| ED70-G23 | click picking只消费current visible+eligible set | Partial |
| ED70-G24 | box/frame/select-all消费同一eligibility | Partial |
| ED70-G25 | lock/editability/parent contribution有确定组合 | Fail |
| ED70-G26 | hidden-selection retain/clear/restore一致 | Fail |
| ED70-G27 | snapshot已有world/viewport/frame/view，filter与Editor全身份校验缺失 | Partial |
| ED70-G28 | extract变化会清snapshot；absent fallback仍全候选 | Partial |
| ED70-G29 | cache有world/settings/camera key，filter generation缺失 | Partial |
| ED70-G30 | view-scoped cache/失效有基础，visibility-only原因缺失 | Partial |
| ED70-G31 | Outliner展示原因、mixed与isolation membership | Fail |
| ED70-G32 | persistent visibility/lock经Editor60 transaction且与transient分离 | Fail |
| ED70-G33 | toolbar/menu/keymap/automation使用同一intent | Fail |
| ED70-G34 | UI展示requested/effective/rejected/stale/disabled | Fail |
| ED70-G35 | 两个view维护独立filter | Fail |
| ED70-G36 | Game/Play排除edit local visibility | Fail |
| ED70-G37 | view close退休filter/snapshot/lease | Fail |
| ED70-G38 | world reset已清interaction；旧visibility set拒绝尚无合同 | Partial |
| ED70-G39 | Play/prefab/stage/provider unload转换通过 | Fail |
| ED70-G40 | profile scope/version/atomic save/migration通过 | Fail |
| ED70-G41 | unknown provider配置保留且disabled | Fail |
| ED70-G42 | 100K/1M compile/toggle latency达预算 | Fail |
| ED70-G43 | query counters/reuse存在；1/4/16 view steady scale未证明 | Partial |
| ED70-G44 | sparse/dense/hierarchy阈值有实测依据 | Fail |
| ED70-G45 | visible-spatial有qualified stats；visibility receipt/provenance缺失 | Partial |
| ED70-G46 | rapid toggle/world churn/OOM/plugin/device fault与soak通过 | Fail |
| ED70-G47 | Windows真实Editor/GPU/pixel/backend矩阵通过 | Fail |
| ED70-G48 | 同语义同硬件跨引擎证据达到目标 | Fail |

Partial不是产品通过；本轮没有动态执行，因此没有任何Pass门。

## 10. 测试与验证矩阵

### 10.1 纯状态机与序列化边界

覆盖exact/hierarchy、push/pop、toggle/reveal/show-all、重复命令、空/删除/复用对象、world/filter generation、receipt digest和unknown provider。每条用scene hash/dirty/save snapshot证明transient操作不改变作者态。

### 10.2 Runtime consumer一致性

同一filter分别验证Mesh、Sprite、Particle、所有Light、Volume、Decal/Fog/CustomPass、shadow/reflection/GI/depth/velocity、virtual geometry、GPU scene和辅助view；对每个consumer记录相同decision/reason/generation。

### 10.3 Selection、overlay与Hierarchy产品测试

Point/box/frame/select-all、Hierarchy click、automation、gizmo、anchor、handle、highlight和outline必须共享eligibility。验证hidden-selection政策、mixed row、恢复入口、stale snapshot fail-close和disabled reason。

### 10.4 Lifecycle、fault与持久化

覆盖多view独立、view close、world replace/reload、Play enter/exit、prefab/stage、plugin unload/panic、device loss、OOM、crash restore与save/reopen。终态不得泄漏filter、snapshot、owner map或provider lease。

### 10.5 Scale与跨引擎

运行10K/100K/1M对象、深/宽hierarchy、1/4/16 view、sparse/dense toggle、spawn/despawn/reparent/load/unload churn；记录compile/toggle/query p50/p95/p99、allocation、GPU patch与steady reuse。跨引擎必须冻结scene、camera、visibility trace、quality、resolution、hardware/driver、warm-up和采样窗。

## 11. 最终判定

当前Zircon已经拥有值得保留的工程底座：作者态/派生态/Layer分层、各一等render contribution的active/layer检查、sorted static/dynamic visibility input、immutable renderer-visible spatial snapshot、generation-aware interaction cache、event-time spatial query counters，以及能防止相同generation不同World复用旧交互数据的reset测试。

这些进展没有形成对象临时隐藏产品。核心缺口仍是qualified per-view session、immutable compiled filter、single participation resolver、filter currentness、selection eligibility、Outliner effective projection、完整consumer传播、生命周期、持久化、规模表示与资格证据。任何先加一个hidden `HashSet<u64>`、修改`ActiveSelf`/Layer、只过滤主Mesh、或允许snapshot缺失时继续全候选拾取的实现都会复制新的临时系统。

整改顺序必须从身份/纯状态机和Runtime-neutral filter开始，再硬切全部render/selection/overlay consumers，之后接产品命令、Hierarchy、lifecycle和profile，最后完成fault/scale/backend/cross-engine资格。本报告完成current-source refresh，不代表实现完成；20项Open P1、12项Partial P1、8项Open P2以及32 Fail/16 Partial/0 Pass资格门仍需代码与动态证据逐项关闭。
