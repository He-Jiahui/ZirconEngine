---
title: Editor Scene Viewport Object Visibility、Temporary Hide、Isolate/Local View、Selection Eligibility、Hierarchy Feedback、Persistence、Performance 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor70
review_date: 2026-08-22
baseline_head: a922089697e41e07fa29e3e42a5e4c9afc1ae31b
baseline_epoch: 341
related_code:
  - zircon_editor/src/ui/binding/viewport/command.rs
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_state.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_selection.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_reset_from_scene.rs
  - zircon_editor/src/scene/viewport/interaction_extract/key.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/renderable_candidates.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_sync.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_visible_spatial_query.rs
  - zircon_editor/src/scene/viewport/pointer/precision/renderer_visible_spatial_pick_source.rs
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes/hierarchy.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/visible_spatial_query.rs
  - zircon_runtime/src/scene/components/scene/activation.rs
  - zircon_runtime/src/scene/components/scene/node.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render/lights.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/world/render_visibility.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/capture.rs
tests:
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_selection.rs
  - zircon_editor/src/scene/viewport/interaction_extract/tests.rs
  - zircon_editor/src/scene/viewport/pointer/tests.rs
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/host/binding_dispatch/viewport.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/typed_command.rs
  - zircon_editor/src/tests/host/retained_viewport_toolbar_pointer/dispatch.rs
  - zircon_editor/src/tests/host/retained_viewport_toolbar_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/retained_hierarchy_template_body.rs
  - zircon_editor/src/tests/ui/binding/viewport.rs
  - zircon_editor/src/tests/workbench/chrome_snapshot/viewport_settings.rs
  - zircon_runtime/src/scene/tests/derived_state.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
  - zircon_runtime/src/core/framework/render/visible_spatial_query.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/61-editor-scene-document-authoring-world-open-new-reload-save-close-dirty-transition-autosave-recovery-multi-document-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/68-editor-scene-viewport-display-mode-lighting-skybox-show-flag-debug-visualization-overlay-composition-profile-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/94-runtime-visibility-spatial-index-bounds-frustum-occlusion-hzb-culling-batching-instancing-gpu-scene-indirect-submission-instance-lifecycle-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorActor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ActorEditor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/Actor.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/LevelEditorViewport.cpp
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/SceneOutlinerGutter.cpp
  - dev/godot/editor/scene/scene_tree_editor.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_plugin.cpp
  - dev/bevy/crates/bevy_camera/src/visibility/mod.rs
  - dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Culling/InstanceCuller.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/GPUDriven/GPUDrivenRenderingTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Decal/DecalProjector.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/VolumetricLighting/LocalVolumetricFog.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/RenderPass/CustomPass/CustomPassVolume.cs
doc_type: current_source_review
canonical_owner: docs/plans/optimize/zircon_editor/70-editor-scene-viewport-object-visibility-temporary-hide-isolate-local-view-selection-eligibility-hierarchy-feedback-persistence-performance-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Scene Viewport Object Visibility、Temporary Hide、Isolate/Local View、Selection Eligibility、Hierarchy Feedback、Persistence、Performance 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon已经具备对象可见性工程化所需的若干真实底座。Runtime把持久作者意图`ActiveSelf`、层级派生结果`ActiveInHierarchy`和`RenderLayerMask`分开；mesh、sprite、particle、light与post-process提取会消费active/layer事实。Renderer还能发布带world、viewport、frame generation和view身份的immutable visible-spatial query snapshot，Editor pointer route可以在该snapshot存在时用renderer实际可见owner替代静态renderable候选。这些基础必须保留并扩展，不能在Editor再扫描一遍World、改`ActiveSelf`或给每类renderer各塞一个临时HashSet。

但当前Scene Viewport完全没有“仅影响这个编辑视口”的对象可见性产品。`ViewportCommand`、`SceneViewportSettings`、`SceneViewportState`与`SceneViewportExtractRequest`都没有Hide Selected、Hide Unselected、Show All、Toggle、Isolate/Local View、hierarchy scope、filter revision或effective receipt。仓库里只有两枚未接线的isolate图标；`isolated_scene_mode`负责插件scene-mode故障隔离，并不是对象隔离。产品没有可达按钮或伪成功反馈，因此本轮不新增P0；缺失本身仍是Unreal级编辑体验之前必须关闭的P1产品链。

不能用`ActiveSelf`或`RenderLayerMask`冒充临时隐藏。两者是public、serializable的scene component，`NodeRecord`与Dynamic Scene capture都会保存；修改它们会污染dirty/undo/save、Play/Game语义与其他viewport。工程级实现必须引入Editor-owned、per-ViewInstance、generation-qualified transient visibility session，再把immutable compiled filter作为runtime-neutral输入送入唯一render extraction authority。`Show All`只能清除此transient filter，绝不能把作者明确关闭的对象、层或未加载内容重新激活。

当前一致性风险比“缺按钮”更深。`select_nodes`只检查node存在；interaction extract cache key不含visibility revision；scene gizmo只检查`active_in_hierarchy`；renderer-visible snapshot虽有完整identity，Editor接纳时却只比较world generation；snapshot缺失时pointer又回退到全部`renderable_candidates`。如果只在主mesh提取加过滤，隐藏对象仍可能通过旧snapshot、fallback picking、gizmo、selection anchor、light、particle、volume、shadow或selection outline重新出现或被选中。

参考实现说明这是一条跨产品数据流。Unreal区分temporary editor hide、layer/level/editable状态和per-view layer bit，并在Hide Selected时处理hierarchy与deselect；Bevy区分authoring Visibility、InheritedVisibility与per-frame ViewVisibility，并维护per-camera visible set；Unity Graphics把SceneView hidden-object过滤送入GPU-driven culling、picking、selection outline、decal、fog和custom pass。Godot的SceneTree visibility与`_edit_lock_`更适合证明“持久作者属性必须走UndoRedo且锁参与选择”，而不是临时隔离的完整范本；Fyrox本地Editor没有同等级对象隔离产品，是负参考而非目标上限。

本报告新增 **32项P1、8项P2**，登记 **48个全部Fail的资格门**。目标是建立Editor-owned `ViewportObjectVisibilitySessionRegistry + ViewportObjectVisibilityProfile + EffectiveViewportObjectVisibilityReceipt + SelectionEligibilitySnapshot + OutlinerEffectiveObjectStateProjection`，Runtime-owned `CompiledViewportObjectFilter + ObjectParticipationResolver`，并以Editor58的ViewInstance identity、Editor59的选择/拾取机制、Editor60的Outliner投影和Editor68的Show Flag resolver组合成单一effective visibility链。

本轮是review-only：未修改production Rust，未运行Cargo、真实Editor、GUI/GPU、render golden、save/reopen、multi-viewport、Play/prefab transition、fault/soak/profile或同语义跨引擎benchmark；tooling按用户要求排除。当前不能声称Scene Viewport对象隔离的功能、性能或表现达到或超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Zircon Editor visibility product/routes | **36 / 7,160 / 6,673 / 255,381 / 35** | viewport command/state/extract、selection、pointer、Outliner projection与lifecycle route | `dc6d7f28202d7caa1a9c21b77bbbd6db6e268c6ef7e8397a432fd4c25d29d850` |
| Zircon Runtime extract/visibility | **17 / 6,229 / 5,672 / 225,094 / 26** | activation/layer truth、各类render contribution、visible-spatial snapshot与capture | `dbd6417ff02aafae7df14b6810e146092fd3130991721acdaf227f3e2a771b0b` |
| Zircon focused tests | **15 / 4,498 / 4,121 / 157,347 / 97** | viewport state/route/pointer/hierarchy与Runtime active/layer/render tests | `a545c6054765211ee003ebcc4829e9c01bf12ba3a1e74bea5aa87a614efc10b6` |
| Zircon deduplicated focused set | **68 / 17,887 / 16,466 / 637,822 / 158** | 上述三组按规范化路径去重 | `4dc466756b821210370fd9724aa095426832abd9ca676d52e11304ab0291ff36` |
| Unreal selected set | **7 / 17,974 / 15,161 / 720,674 / 0** | temporary hide、hierarchy、deselect、per-view layer bits、Outliner cache/transaction | `64d8fbdaff67f53e53c3f05d3a0c4c5529ab64f7d51a2bbf9fb1cdade5d46fe7` |
| Godot selected set | **3 / 12,093 / 10,185 / 444,421 / 0** | authored visibility UndoRedo/drag与authoring lock selection gate | `f9162b042a379097083f5c662f15623aff087d37a39823c22b0efe992497bcc0` |
| Fyrox selected set | **8 / 5,172 / 4,739 / 193,616 / 2** | scene selection/menu/helper visibility；对象隔离产品负证据 | `562a111911c97eb933cdca716ae1eddd88dea0c4eaee4ed1d15548adba4fc4c4` |
| Bevy selected set | **2 / 1,816 / 1,617 / 67,327 / 13** | authored/inherited/per-view visibility、per-camera set与wide render layers | `75eb1f46c61d0f70a5ada5a3b3bdb054e64c628d3e31588cd3d55d3dcb890b86` |
| Unity Graphics selected set | **7 / 7,357 / 6,218 / 338,628 / 12** | SceneView GPU culling、picking/outline及decal/fog/custom-pass consumers | `7718961c9445187a217cb08ed8b3ba57f8077d6746252ddc8701b524a9de6ec4` |
| Five-engine deduplicated set | **27 / 44,412 / 37,920 / 1,764,666 / 27** | 五类本地参考按路径去重 | `8c1710ecce2246e8b8ef269b6698eee11b839951eb1e539fe574f75fc9b5965b` |

fingerprint按小写规范化相对路径排序，并将每个`path + NUL + file SHA-256 + LF`聚合后再做SHA-256；它只证明本轮读取的working-tree物理语料，不是ABI、artifact、动态结果或性能receipt。主仓与Unreal镜像基线为`a922089697e41e07fa29e3e42a5e4c9afc1ae31b`；Godot、Fyrox、Bevy与Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`。Unreal参考树不是独立Git仓，按主仓基线和文件fingerprint冻结。

### 2.2 在途修改隔离

冻结的68份Zircon focused文件中有18份包含非本轮working-tree修改，共 **5,238行 / 4,776非空行 / 184,882 bytes**，fingerprint为`7a0760a3d89423f685180ab75c0fac23e1709676795a7d0ba20dc22a001ce1a7`。它们覆盖viewport controller access/reset/selection/state construction、interaction cache/tests、pointer overlay sync、renderer-visible pick source、pointer tests、render packet、editing viewport tests、host viewport event、pane projection、dirty marking、render submission、scene entries、editor-state render与Runtime derived state。

本报告读取并接受这些working-tree current-source事实，但不拥有、不覆盖也不归因这些修改。尤其renderer-visible picking/currentness和Outliner projection仍在演进，实施前必须重取全部68份文件、父报告、目标backend与动态结果。新报告与三个共享索引由coordinator Session `optimize-editor70-viewport-object-visibility-isolation-review-r4-20260822`取得四个精确lease及maintenance授权，baseline epoch为341。

### 2.3 范围与非范围

本报告唯一拥有per-viewport transient object visibility：Hide Selected、Hide Unselected、Show All、Toggle、Isolate/Local View stack、exact/hierarchy scope、effective reason、selection-eligibility contribution、render/picking/overlay一致性、view/session lifecycle、可选偏好恢复、diagnostics与规模资格。

Editor60继续拥有Outliner UI、persistent authoring visibility/lock column、provider transaction与undo；Editor68拥有category Show Flag、display/debug mode及其profile；Editor58拥有ViewInstance/session/product identity与frame currentness；Editor59拥有point/ray/box picking、selection mutation、highlight与interaction机制；Runtime Scene报告拥有`ActiveSelf`、`ActiveInHierarchy`、`RenderLayerMask`和持久World truth。Editor70只定义这些owner共同消费的transient filter和visibility-derived eligibility，不复制父合同。

## 3. 当前实现拓扑与可保留基础

### 3.1 Viewport控制面没有对象可见性意图

`ViewportCommand`止于pointer、navigation、view settings、overlay provider和Frame Selection；`SceneViewportSettings`只有transform/projection/gizmo/display/grid/lighting/skybox；`SceneViewportState`只有settings、selection、mode、viewport、camera、orbit、hover和drag。仓库搜索没有Hide Selected/Unselected、Show All、Local View或temporary hidden产品，只有两枚isolate SVG未接线。

### 3.2 Runtime正确地区分 authored active、derived active与render layer

`ActiveSelf`和`RenderLayerMask`是public、serializable component，`ActiveInHierarchy`明确`serialization = none`并由derived-state重建。Node record、project IO与Dynamic Scene capture保存作者active/layer事实。这是真实且应保留的Runtime authority，也直接证明临时隔离不能通过修改这些字段实现。

### 3.3 Render extract没有对象filter输入

`SceneViewportExtractRequest`只有render settings、active camera override、camera、viewport size与virtual geometry debug；`build_render_packet`没有传object filter。Runtime mesh/sprite/light/particle/post-process路径各自检查active/layer，尚无统一`ObjectParticipationResolver`在资源查找与贡献展开前裁剪对象。

### 3.4 Overlay与gizmo没有消费统一effective visibility

scene gizmo只检查`active_in_hierarchy`，selection anchor甚至只验证selected node存在且无mesh。未来若主render被隐藏而overlay仍使用scene扫描，camera/light icon、pick shape、selection anchor、handle和highlight都会暴露被隐藏对象或制造不可解释的“空选中”。

### 3.5 Renderer-visible spatial query是真实基础，但接纳条件不完整

Runtime snapshot identity包含world、viewport、frame generation和view，query还给出visited/candidate/hit统计；这是实现同帧render/picking一致性的好基础。但Editor `sync_renderer_visible_spatial_snapshot`只比较`identity().world.raw()`，没有验证viewport、frame/filter generation。filter变更后旧visible set可能在新view状态下继续被采用。

### 3.6 Pointer fallback与selection admission会重新引入隐藏对象

当renderer snapshot不存在时，router从interaction extract全部render meshes构建`renderable_candidates`；`select_nodes`又只按`scene.find_node(id).is_some()`准入。没有visibility/lock/editable eligibility resolver，point/box/frame/select-all、Outliner选择和automation未来会形成不同规则。

### 3.7 Interaction cache与产品失效没有filter revision

`ViewportInteractionExtractKey`只包含world generation、selected、settings、camera和viewport。对象visibility变化既不会改变key，也没有typed invalidation reason、compiled-filter digest或presentation receipt；缓存可以继续返回旧gizmo/renderables，Host也无法只重建受影响的view。

### 3.8 Outliner、lifecycle、persistence与测试尚未形成闭环

Runtime inspection能提供active/kind/children，但最终pane `SceneNodeData`只保留id/name/depth/selected；Play session只捕获scene、selection、gizmo，scene reset也没有visibility retirement。现有测试覆盖active hierarchy、render layer、snapshot transport与pointer route，没有temporary hide/isolate/local-view、multi-view独立性、world replacement、save/reopen或所有render consumer一致性测试。

## 4. 五引擎参考证据与适用边界

### 4.1 Unreal：temporary hide、per-view layer与authoring状态是不同合同

`EditorActor.cpp`的`SetActorVisibility`调用`SetIsTemporarilyHiddenInEditor`；`Internal_HideSelected`有显式child hierarchy选项并取消隐藏actor的选择，Hide Unselected与UnHide All各有独立命令。`Actor.h`同时保留`HiddenEditorViews` transient位集，以及`bHiddenEdLayer`、`bHiddenEdLevel`、`bEditable`、`bLockLocation`和`bHiddenEdTemporary`；`ActorEditor.cpp::IsHiddenEd`组合多种原因，temporary setter触发render-state dirty。Level viewport创建/销毁时更新并移除per-view layer visibility，Scene Outliner gutter用递归cache、transaction、selected-item batch和child propagation维持反馈。

适用边界必须明确：Unreal的temporary actor hide本身是editor-transient actor状态，不是天然per-view；`HiddenEditorViews`主要服务per-view layer visibility。Zircon以ViewInstance私有对象隔离为目标是有意的产品提升，不应把Unreal两条机制混写成同一实现。

### 4.2 Godot：持久visibility/lock必须transactional并参与选择

SceneTree visibility按钮直接调用node `is_visible/set_visible`，在交互结束时组成UndoRedo；这是作者场景属性，不是Local View。3D Editor以`_edit_lock_` metadata跳过locked node，并用UndoRedo实现Lock/Unlock，tooltip明确其阻止选择与移动。Godot可用于约束Zircon的authoring visibility/lock边界，但不能作为temporary isolation的完整金标准；其`_reset_visibility_drag`只清理drag session，不能被解释为已回滚所有已切换节点。

### 4.3 Fyrox：本地Editor没有同等级对象隔离产品

focused Editor语料能看到selection、world menu、grid/gizmo/helper graph visibility和各工具内部show选项，但没有Hide Selected、Hide Unselected、Show All、Isolate或Local View对象产品。它证明Rust引擎可以选择省略这一工作流，却不能作为用户要求的Unreal级工程目标上限。

### 4.4 Bevy：authoring、hierarchy与per-view结果分层

Bevy把`Visibility`作为用户意图、`InheritedVisibility`作为层级派生、`ViewVisibility`作为每帧算法结果；`ViewVisibility`以current/previous位避免无意义change detection churn，并能服务多个view，camera拥有自己的visible entity集合。`RenderLayers`用SmallVec支持宽位集，空集合不可见、camera/entity按intersection判断，并有层级传播、parent change/removal、change detection与位运算测试。适用点是数据流和规模原语，不是Editor UX。

### 4.5 Unity Graphics：隐藏对象必须贯穿所有SceneView consumers

本地Graphics镜像不含Unity私有`SceneVisibilityManager`实现，只能采用可验证consumer。GPU-driven `InstanceCuller`为SceneView camera/light、Picking、SelectionOutline与Filtering安排hidden-object culling，并在没有hidden对象时走fast path；GPUDriven test验证hide后0 visible、show后1 visible，但该test被`Ignore`标为不稳定，不能当作强制green证据。HDRP Decal、Local Volumetric Fog与Custom Pass Volume订阅visibility change或读取scene culling mask，证明非mesh贡献也必须同步。

## 5. 差异矩阵

| 能力 | Zircon当前事实 | 工程级目标 | 参考证据 | 判定 |
|---|---|---|---|---|
| Temporary Hide | 无意图、状态或产品入口 | per-view typed intent与effective receipt | Unreal | Missing |
| Hide Unselected / Isolate | 无 | reversible isolation stack/local view | Unreal | Missing |
| Show All | 无 | 只清transient state，不改authoring truth | Unreal | Missing |
| Hierarchy scope | 只有Runtime active hierarchy | exact/hierarchy显式scope与partial state | Unreal/Bevy | Missing |
| Authored visibility boundary | ActiveSelf/RenderLayerMask会持久化 | transient filter完全不dirty scene | Unreal/Godot/Bevy | Partial |
| Per-view identity | snapshot有viewport；controller共享state | ViewInstance/document/world/filter generation | Unreal/Bevy | Partial |
| Effective resolver | 各render path分散active/layer判断 | 单一precedence与reason vector | Unreal/Bevy | Missing |
| Render extraction | request无object filter | immutable compiled filter早期裁剪 | Unity/Bevy | Missing |
| Non-mesh consumers | light/particle/volume各自提取 | 全consumer共享participation predicate | Unity | Missing |
| Overlay/gizmo | 只按active或selected扫描 | 与base render同filter generation | Unity/Unreal | Missing |
| Picking | renderer snapshot基础真实；fallback全候选 | visible+eligible同帧set，缺失时fail-closed | Unity | Partial |
| Selection eligibility | 只检查node存在 | visible/locked/editable/capability compositor | Godot/Unreal | Missing |
| Outliner feedback | row仅id/name/depth/selected | effective hidden reason、tri-state和恢复入口 | Unreal | Missing |
| Lifecycle | reset/Play/close不处理visibility | per-view retire/suspend/restore/stale reject | Unreal | Missing |
| Persistence | 无scope/schema | explicit ephemeral/session preference policy | Unreal | Missing |
| Qualification | 无专项测试或benchmark | correctness/fault/scale/backend evidence | 五引擎共同 | Missing |

## 6. 新增发现

### 6.1 P1：架构、正确性与产品闭环

#### ED70-P1-01：没有Viewport Object Visibility Session authority

controller没有以ViewInstance、DocumentSession、World epoch和owner generation限定的visibility authority。临时集合若挂在共享EditorState或Scene，会让多个viewport互相污染，也无法拒绝旧view事件。

#### ED70-P1-02：没有完整per-view identity与filter generation

当前render-visible snapshot具备viewport/frame identity，但visibility intent、compiled filter、interaction extract和Outliner projection没有共同revision。任何异步结果都无法证明属于当前visibility状态。

#### ED70-P1-03：没有与ActiveSelf/RenderLayerMask硬分离的transient合同

两类Runtime component均可序列化、可反射并进入record/capture；用它们做Local View会污染场景、undo、save、Play和其他camera。系统缺少“永不修改authoring truth”的类型与测试保证。

#### ED70-P1-04：没有单一effective visibility precedence resolver

authoring active/loaded/layer、Editor70 object filter、Editor68 Show Flag/category与stage/context policy没有确定组合顺序或reason vector。多个bool散落组合会造成UI、render和picking解释不同。

#### ED70-P1-05：没有qualified object/subobject handle与stale pruning

现有selection和candidate大量使用裸`u64` entity。对象删除、world replacement、prefab/stage切换或ID复用后，hidden/isolation set无法区分旧对象，也没有增量清理和拒绝回执。

#### ED70-P1-06：没有immutable compiled filter与effective receipt

产品没有把requested set编译成Runtime可消费的snapshot，也没有requested/effective/hidden/rejected/stale count、digest、capability、reason与source revision。UI只能猜测命令是否生效。

#### ED70-P1-07：Hide Selected完整工作流缺席

没有typed命令、菜单/快捷键、selected snapshot、hierarchy scope、hidden-selection policy、render invalidation或回执。不能以遍历selection并写`ActiveSelf(false)`临时补齐。

#### ED70-P1-08：Hide Unselected、Isolate与Local View缺席

系统没有基于selection roots生成保留集、处理ancestors/descendants、嵌套isolation或为不同viewport保留不同集合。两枚isolate图标没有command、binding或consumer。

#### ED70-P1-09：Show All、Toggle、Reveal与isolation pop缺席

没有只清transient hidden set的Show All，也没有single-object toggle、reveal selected、pop上一层isolation或restore receipt。未来实现必须确保这些动作绝不重启authoring-disabled对象。

#### ED70-P1-10：exact与hierarchy scope、partial/mixed状态未定义

隐藏parent是否自动隐藏children、隐藏child后parent如何投影mixed、isolation是否包含ancestors与dependency helpers均无政策。Unreal显式区分child hierarchy说明该语义不能藏在递归helper里。

#### ED70-P1-11：隐藏后selection保留、清除与恢复语义未定义

当前selection model与visibility无关联。产品必须决定Hide Selected是否deselect、是否保留hidden selection供Show All恢复、status如何显示hidden selected，以及gizmo/highlight是否抑制；不同入口不能各自决定。

#### ED70-P1-12：没有selection eligibility compositor

`select_nodes`只检查存在性。可选择性应组合transient visibility、Editor60拥有的authoring lock/editability、对象 capability、mode/tool policy和stage scope；Editor70只提供visibility contribution，不能夺取lock owner。

#### ED70-P1-13：Outliner没有effective hidden reason与tri-state投影

最终row只有id/name/depth/selected，无法区分authoring inactive、render layer不匹配、temporary hidden、isolation excluded、category hidden、unloaded或locked。用户也没有明确恢复当前view隐藏状态的入口。

#### ED70-P1-14：Command/menu/keymap/automation没有统一intent与admission

Viewport command codec、event dispatch和retained route都没有visibility vocabulary。若菜单直接改controller set，automation、remote command、undo policy、capability denial和错误反馈会形成旁路。

#### ED70-P1-15：SceneViewportExtractRequest缺少object filter

Editor `build_render_packet`无法把view-specific visibility送入Runtime。过滤若留在Editor post-process，只能删已展开的mesh，无法阻止resource lookup、light/volume/particle提取和下游GPU工作。

#### ED70-P1-16：所有render contribution没有共同participation predicate

mesh、sprite、particle、ambient/directional/punctual light、post-process volume及未来decal/fog/custom pass分别检查active/layer。没有统一resolver就会持续遗漏新类型，形成隐藏对象仍影响画面的回归。

#### ED70-P1-17：overlay、gizmo、anchor、handle与highlight忽略object filter

camera/light gizmo只按active hierarchy，selection anchor只按selection/node形态。base object隐藏后辅助视觉与pick shape仍可能存在，且highlight delivery也没有visibility filter revision。

#### ED70-P1-18：shadow、reflection、GI、GPU visibility与auxiliary view没有统一传播

当前合同没有规定隐藏对象是否进入shadow caster、reflection/probe、GI、depth/velocity、selection outline、picking、capture或debug pass。SceneView对象隐藏必须在编辑视图相关派生pass一致，而不能误改Game/Play camera。

#### ED70-P1-19：view-only filter与simulation副作用边界未定义

Temporary Hide默认只影响编辑视图的render/picking/feedback，不应停止physics、audio、script、navigation或animation simulation。若未来提供“mute/disable simulation”是其他owner的显式policy，不能从visibility隐式推导。

#### ED70-P1-20：visible-spatial currentness缺filter revision，Editor还忽略viewport/frame

Runtime snapshot identity是好基础，但Editor只校验world raw generation。即使未来renderer正确生成过滤结果，旧viewport、旧frame或旧filter snapshot仍可能被接纳并驱动选择。

#### ED70-P1-21：renderer snapshot缺失时fallback picking会重引全部renderables

router在没有snapshot时恢复interaction extract的全量候选。visibility变更到新render完成之间、backend不支持query或提交失败时，隐藏对象会重新可点；必须定义filtered fallback或fail-closed政策。

#### ED70-P1-22：point/box/frame/select-all与selection显示没有共享eligible set

Editor59拥有选择机制，但当前没有一个generation-qualified `SelectionEligibilitySnapshot`供所有入口消费。只修click picking会留下框选、全选、Frame Selection、Outliner和automation不一致。

#### ED70-P1-23：interaction cache与invalidation缺visibility revision

cache key没有filter generation，dirty marking也没有`ViewportObjectVisibilityChanged`或目标ViewInstance。改变hidden set可能继续复用旧renderables/gizmos，或粗暴触发全Workbench重建。

#### ED70-P1-24：没有GPU/static index增量visibility update政策

系统没有规定filter delta如何更新visible bitset、instance culling、spatial index和cached draw lists。每次toggle全量重建World/HashSet/packet会在大场景和多viewport下不可接受。

#### ED70-P1-25：multi-viewport独立性与view close清理缺席

Editor58已指出当前产品仍非完整per-view，但Editor70必须要求两个Scene view可有不同isolation stack，Game View不继承edit filter；关闭view要退休state、query snapshot、cache和诊断owner。

#### ED70-P1-26：scene/project/world replacement没有retirement与stale rejection

reset只清interaction/camera/hover/drag相关状态。Open/New/Reload/Close、world epoch变化、provider revoke或ID复用时，旧hidden set没有terminal choreography，可能错误隐藏新World同值entity。

#### ED70-P1-27：Play、prefab/stage、context与provider unload transition未定义

进入Play时edit local visibility应保留但不污染Game；退出后恢复同一edit view。Prefab/stage isolation、level/data-layer切换和plugin object provider unload也需要suspend/remap/reject政策，当前没有session转换合同。

#### ED70-P1-28：persistence scope、schema、unknown owner与crash restore缺席

临时隐藏默认可选择只活在view session，也可提供workspace/user恢复；无论哪种都要显式scope/version。当前没有atomic store、unknown provider preservation、stale object pruning或crash-recovery政策。

#### ED70-P1-29：100K/1M对象没有可扩展表示与hierarchy delta算法

没有dense qualified index、roaring/bitset/sorted sparse hybrid、hierarchy interval、ancestor exception或delta compiler设计。per-frame遍历全部Scene、递归HashSet查询和为每view复制全量set都不符合目标。

#### ED70-P1-30：没有visibility diagnostics与provenance

产品无法报告requested/effective hidden count、isolation depth、stale/rejected object、compile time、filter memory、extract rejects、fallback picks、hidden draw/instance savings和reason distribution，问题只能靠截图猜测。

#### ED70-P1-31：缺少行为、故障、生命周期与pixel测试

现有focused tests不覆盖hide/show/isolate、hierarchy mixed、selection policy、all render consumers、old snapshot、render failure、world replacement、multi-view、save/reopen或golden pixel。源码contains测试不能证明产品一致性。

#### ED70-P1-32：没有跨backend、规模与同语义性能资格

没有100K/1M对象、1/4/16 viewport、toggle latency、steady-state CPU/GPU、memory、hidden ratio、backend parity、device loss、long soak或同硬件同画质Unreal基线。当前不能支持性能优于Unreal的结论。

### 6.2 P2：质量、可维护性与资格表达

#### ED70-P2-01：没有named visibility set与preset

复杂关卡需要保存命名对象集合、快速切换并与临时isolation组合；该能力必须建立在stable qualified identity和明确scope上，不能先保存裸entity数组。

#### ED70-P2-02：没有private/shared与multi-user visibility overlay政策

个人Local View默认不应广播；团队review可能需要显式共享集合。应区分private presence、shared named set与authoring truth，避免协作系统复制每次hover/toggle。

#### ED70-P2-03：没有headless/automation query API

测试、capture与远程工具需要查询effective visibility、原因和generation，而不是读取UI文本或私有controller字段。

#### ED70-P2-04：plugin predicate与unknown provider preservation未定义

插件对象类型可能贡献额外selection/visibility policy。descriptor需要stable owner、generation、budget和fault boundary，未知provider设置只能保留为disabled，不能静默启用或丢失。

#### ED70-P2-05：accessibility、keyboard与reason projection无规范

Hide/Isolate/Show All、mixed hierarchy与hidden-selection提示需要一致shortcut、accessible name、focus order、checked/mixed/pending与disabled reason，不能只依赖淡化icon。

#### ED70-P2-06：isolation history、breadcrumb与compare视图缺席

嵌套Local View需要可检查stack、返回上一层和当前集合来源；复杂review还可比较两个visibility profile，但这些都应建立在核心state machine通过后。

#### ED70-P2-07：缺少可脱敏receipt导出与replay corpus

性能/故障分析需要导出filter digest、delta、reason统计和generation trace，同时避免泄露项目对象名称。当前没有schema或replay recipe。

#### ED70-P2-08：unloaded descriptor、World Partition与remote runtime view尚无扩展模型

本轮只拥有loaded edit-world对象；未加载cell descriptor、远程runtime entity和大世界streaming visibility应由各父owner提供qualified adapter，Editor70只组合其投影。

## 7. 目标架构与职责边界

### 7.1 Editor：per-view transient authority与typed intent

建立`ViewportObjectVisibilitySessionRegistry`，key至少包含`ViewInstanceId + DocumentSessionId + WorldId/Epoch + SessionGeneration`。`ViewportObjectVisibilityProfile`持有temporary hidden set、isolation stack、可选named set引用和monotonic revision；集合元素使用qualified object/subobject address，不保存裸`u64`。

统一`ViewportObjectVisibilityIntent`：`HideSelected`、`HideUnselected`、`ShowAll`、`Toggle`、`PushIsolation`、`PopIsolation`、`Reveal`，每个intent带`Exact | Hierarchy` scope、selection policy、expected revision和source owner。UI、menu、keymap、automation与tool必须走同一admission/state machine，并返回`EffectiveViewportObjectVisibilityReceipt`。

receipt至少包含view/document/world/filter identity、requested/effective hidden count、isolation depth、rejected/stale address、reason vector、capability/degraded state、source revision、compile cost和digest。Show All只重置Editor70 transient state；authoring active/layer、Editor68 Show Flag和stage policy保持不变。

### 7.2 Runtime：immutable compiled filter与唯一participation resolver

Editor把profile delta编译为runtime-neutral `CompiledViewportObjectFilter`，通过`SceneViewportExtractRequest`按immutable `Arc`传递。filter identity包含world generation、view/filter generation和digest；Runtime不理解Hide Selected等Editor操作，只回答qualified owner是否参与该view的编辑渲染。

`ObjectParticipationResolver`组合：`authored active/loaded/layer truth AND Editor70 object filter AND Editor68 category/show-flag result AND stage/context policy`，同时保留reason vector。它必须在material/resource lookup、primitive expansion和GPU work之前应用，并覆盖mesh、sprite、particle、light、decal、volume、shadow/reflection/GI、depth/velocity、picking、selection outline、capture和debug auxiliary view。

实现采用compile-on-change与hybrid sparse/dense qualified index；小集合走sorted sparse，大集合/高密度走bitset或等价压缩表示，hierarchy使用稳定拓扑interval/delta cache。steady frame只做bounded membership，不允许每帧构造HashSet、遍历全World或复制每view全量对象表。隐藏只抑制view contribution，资源默认保持resident，simulation不受影响。

### 7.3 Picking、selection与Outliner消费同一effective generation

Runtime renderer-visible query snapshot扩充filter generation；Editor58/59验证world、viewport、frame、view和filter全部current后才接纳。snapshot缺失时只能使用已应用同filter的fallback candidate，或在无法证明current时fail-closed，不能恢复全量renderables。

Editor70生成`SelectionEligibilitySnapshot`的visibility contribution；Editor59组合mode/tool/capability后供click、box、frame、select-all、highlight和Frame Selection统一消费。Editor60渲染`OutlinerEffectiveObjectStateProjection`，展示authoring/transient/category/stage原因、hierarchy mixed与hidden selection，但persistent visibility/lock的mutation和undo仍归Editor60/scene owner。

### 7.4 Lifecycle、persistence与硬边界

view close、document close、world replace、provider revoke会terminal retire session、cache、query和diagnostics；stale intent/receipt一律拒绝。进入Play/Game时不传edit transient filter，只suspend/retainScene View state；返回同一document/world generation才恢复，否则prune/remap并给receipt。Prefab/stage owner通过adapter贡献context policy，不直接改temporary set。

默认visibility profile只活在view session。若产品决定保存，则只能按versioned per-user/workspace/view scope原子写入，并保留unknown provider为disabled；绝不写入scene asset。禁止以`ActiveSelf`、`RenderLayerMask`、scene component、全局Editor bool、Editor post-extract mesh删除、per-frameWorld扫描或backend专属隐藏表作为兼容实现。

## 8. 分阶段重构计划

### ED70-M0：真实性、owner与RED基线

Goal：冻结Editor58/59/60/68及Runtime Scene父边界，证明当前对象visibility产品缺席且authoring truth不可复用。

Implementation slices：建立术语/precedence/owner表；添加RED contract tests覆盖无intent、无filter、selection只查存在和旧snapshot接纳；两枚isolate icon保持不可达，不制造假成功。

Testing stage：focused source/contract tests与owner guard；实施前重算全部corpus fingerprint和在途文件。

Exit evidence：边界审计通过，P0/P1不重复计数，禁止临时ActiveSelf方案写入实施计划。

### ED70-M1：Qualified identity、DTO、registry与receipt

Goal：建立per-view visibility session、qualified object address、typed intent和immutable receipt。

Implementation slices：定义View/Document/World/Session/Filter generation、exact/hierarchy scope、selection policy、stale pruning、schema与unknown owner policy。

Testing stage：property/negative tests覆盖ID复用、world replace、duplicate/stale intent、nested stack、invalid address和receipt immutability。

Exit evidence：任何请求/回执都能证明归属，无裸entity跨session存活。

### ED70-M2：Pure state machine、hierarchy与selection policy

Goal：在不接render前完成Hide/Show/Toggle/Push/Pop的确定性状态机。

Implementation slices：实现profile delta、hierarchy interval、ancestor/descendant政策、mixed state、hidden-selection策略和Show All hard boundary。

Testing stage：forest property tests、删除/重挂/partial hierarchy、nested isolation、empty/all selection、idempotence与inverse operation。

Exit evidence：相同source/revision/intent得到相同filter digest和receipt，authoring scene hash不变。

### ED70-M3：Compiled filter与Runtime participation resolver

Goal：把immutable object filter接入SceneViewportExtractRequest与所有Runtime贡献。

Implementation slices：实现sparse/dense compiler、resolver precedence/reason、早期resource裁剪和第一方consumer inventory；Runtime不接受Editor command语义。

Testing stage：mesh/sprite/particle/light/volume/decal-like fixture、authoring active/layer组合、filter generation、all-hidden/none-hidden和resource residency。

Exit evidence：所有view render contribution使用一个resolver，新增consumer必须通过inventory guard。

### ED70-M4：Overlay、gizmo、highlight与selection eligibility

Goal：base render与辅助视觉、选择准入使用同一effective generation。

Implementation slices：过滤gizmo/anchor/handle/highlight/pick shape；生成SelectionEligibilitySnapshot；明确hidden selection/status/reveal policy。

Testing stage：click/box/frame/select-all/Frame Selection、locked/editable组合、hidden selected、gizmo off/on、highlight failure和old filter rejection。

Exit evidence：被view隐藏对象不能通过任何viewport入口被意外命中或显示辅助视觉。

### ED70-M5：Renderer-visible currentness与filtered fallback

Goal：复用renderer visible-spatial基础，封闭旧snapshot与全候选fallback。

Implementation slices：identity加入filter generation；Editor校验world/viewport/frame/view/filter；fallback从compiled filter生成或fail-closed；记录fallback reason。

Testing stage：render latency、snapshot absent/stale、viewport resize/recreate、filter rapid toggle、backend no-query、submit/device failure与race tests。

Exit evidence：任何pick result可追溯到当前render/filter generation；旧对象不重新进入候选。

### ED70-M6：Outliner反馈、产品命令与view-scoped invalidation

Goal：用户可从toolbar/menu/keymap/Outliner理解并恢复当前view状态。

Implementation slices：接typed route、effective reason/mixed/breadcrumb、hidden-selection count、view-only dirty reason和bounded pane projection；persistent visibility/lock仍走Editor60 transaction。

Testing stage：route/automation parity、disabled reason、Outliner/viewport同步、multiple selection、accessibility、locale和无全Workbench重建。

Exit evidence：UI只投影receipt，不直接改set；每次变化只失效目标ViewInstance与必要Outliner rows。

### ED70-M7：Lifecycle、multi-view、Play/stage与可选持久化

Goal：跨view/document/world/Play/prefab/plugin lifecycle不泄漏状态。

Implementation slices：session suspend/retire、Game View exclusion、return restore、provider revoke、stale prune、可选workspace preference migration和crash recovery。

Testing stage：two/four viewport独立性、duplicate/close/reopen、Open/New/Reload、Play enter/exit、prefab/stage、plugin unload、save/reopen与unknown provider。

Exit evidence：其他view和scene asset不受影响；旧world state绝不作用于新world同值ID。

### ED70-M8：Scale、diagnostics、fault与backend parity

Goal：建立filter compile/update/query成本、节省与故障的bounded observability。

Implementation slices：qualified metrics、sampling budget、delta counters、memory census、fallback/stale/reject trace、fault injection和consumer coverage audit。

Testing stage：100K/1M objects、1/4/16 views、1%/50%/99% hidden、hierarchy bursts、rapid toggles、device loss、plugin panic、long soak和diagnostics disabled cost。

Exit evidence：超预算明确degrade/fail-closed并有receipt；steady frame没有全场扫描或无界allocation。

### ED70-M9：单一产品硬切与跨引擎资格

Goal：删除旁路，完成同语义正确性、表现与性能资格。

Implementation slices：所有Scene View、Outliner、picking和第一方consumer迁移共享resolver；删除直接authoring mutation、post-extract删除和backend私表；冻结benchmark recipe。

Testing stage：Windows优先真实Editor/GPU、pixel golden、cross-backend、fault/soak、save/reopen与同场景Unreal/Godot/Fyrox/Bevy/Unity Graphics可比证据；Cargo按里程碑统一批次执行。

Exit evidence：48项门禁全部Pass，性能结论含同硬件/同画质/同对象集合/统计置信与回归阈值，才能声称达到或超过参考引擎。

## 9. 资格门

| Gate | 要求 | 当前 |
|---|---|---|
| ED70-G01 | transient visibility与ActiveSelf/RenderLayerMask有类型级硬边界 | Fail |
| ED70-G02 | View/Document/World/Session/Filter generation限定所有请求与回执 | Fail |
| ED70-G03 | qualified object/subobject address拒绝ID复用与旧world | Fail |
| ED70-G04 | pure state machine相同输入产生相同digest/receipt | Fail |
| ED70-G05 | authoring scene hash在所有temporary操作后保持不变 | Fail |
| ED70-G06 | Hide Selected exact scope行为通过 | Fail |
| ED70-G07 | Hide Selected hierarchy scope行为通过 | Fail |
| ED70-G08 | Hide Unselected/Push Isolation行为通过 | Fail |
| ED70-G09 | nested isolation Pop精确恢复上一层 | Fail |
| ED70-G10 | Show All只清transient state且不激活authoring-hidden对象 | Fail |
| ED70-G11 | Toggle/Reveal/idempotence与stale address receipt通过 | Fail |
| ED70-G12 | hierarchy mixed/ancestor/descendant policy可检查 | Fail |
| ED70-G13 | Runtime只消费compiled filter，不理解Editor command | Fail |
| ED70-G14 | effective resolver有固定precedence与reason vector | Fail |
| ED70-G15 | mesh/sprite/particle共享participation predicate | Fail |
| ED70-G16 | ambient/directional/punctual light共享predicate | Fail |
| ED70-G17 | volume/decal/fog/custom-pass类贡献共享predicate | Fail |
| ED70-G18 | shadow/reflection/GI/depth/velocity辅助pass政策通过 | Fail |
| ED70-G19 | object在resource/primitive展开前被早期裁剪 | Fail |
| ED70-G20 | hidden object默认不影响physics/audio/script/nav/simulation | Fail |
| ED70-G21 | gizmo/icon/anchor/handle/pick shape与base render一致 | Fail |
| ED70-G22 | highlight/selection outline与current filter一致 | Fail |
| ED70-G23 | click picking消费current visible+eligible set | Fail |
| ED70-G24 | box/frame/select-all消费同一eligibility snapshot | Fail |
| ED70-G25 | authoring lock/editability由父owner贡献且被选择准入组合 | Fail |
| ED70-G26 | hidden-selection保留/清除/恢复政策在所有入口一致 | Fail |
| ED70-G27 | renderer snapshot校验world/viewport/frame/view/filter | Fail |
| ED70-G28 | snapshot absent/stale时filtered fallback或fail-closed | Fail |
| ED70-G29 | interaction cache key包含filter generation/digest | Fail |
| ED70-G30 | visibility变化只失效目标ViewInstance和必要projection | Fail |
| ED70-G31 | Outliner展示effective reason、mixed与isolation状态 | Fail |
| ED70-G32 | persistent visibility/lock仍走Editor60 provider transaction | Fail |
| ED70-G33 | toolbar/menu/keymap/automation走同一typed intent | Fail |
| ED70-G34 | UI展示requested/effective/rejected/stale/disabled reason | Fail |
| ED70-G35 | 两个Scene view可持有不同filter且互不污染 | Fail |
| ED70-G36 | Game/Play view不继承edit local visibility | Fail |
| ED70-G37 | view close/document close完整退休state/cache/query | Fail |
| ED70-G38 | world replacement拒绝旧set作用于新world同值ID | Fail |
| ED70-G39 | Play/prefab/stage/provider unload transition通过 | Fail |
| ED70-G40 | 可选profile有明确scope/version/atomic restore/migration | Fail |
| ED70-G41 | unknown provider设置保留为disabled且不静默启用 | Fail |
| ED70-G42 | 100K/1M对象compile与toggle latency满足预算 | Fail |
| ED70-G43 | 1/4/16 view steady frame无全场扫描/无界allocation | Fail |
| ED70-G44 | sparse/dense/hierarchy delta representation按阈值切换 | Fail |
| ED70-G45 | filter/selection/query diagnostics有qualified generation与上界 | Fail |
| ED70-G46 | device loss、render failure、plugin panic、long soak通过 | Fail |
| ED70-G47 | Windows真实Editor/GPU/pixel/cross-backend矩阵通过 | Fail |
| ED70-G48 | 同硬件同画质同对象集合跨引擎表现/性能证据达到目标 | Fail |

## 10. 测试与验证矩阵

### 10.1 State、identity与hierarchy unit/property/fuzz

覆盖View/Document/World/Session/Filter identity、qualified address、revision precondition、Hide/Show/Toggle/Push/Pop、exact/hierarchy、mixed state、ID复用、删除/重挂、nested isolation、stale pruning与receipt immutability。属性测试必须证明Show All不改变authoring truth、操作幂等、inverse stack可恢复且任意stale generation fail-closed。

### 10.2 Runtime participation与render consumer integration

以同一scene构造mesh、sprite、particle、五类light、volume、decal-like provider、shadow/reflection/GI/depth/velocity/capture贡献，验证统一resolver、reason、早期裁剪和resource residency。新增render consumer必须进入inventory test，禁止只测主mesh happy path。

### 10.3 Picking、selection、overlay与Outliner product integration

从toolbar/menu/keymap/automation/Outliner发送同一intent，验证click/box/frame/select-all、Frame Selection、hidden selection、lock/editable、gizmo/anchor/highlight、mixed reason、reveal与view-scoped invalidation。renderer snapshot currentness和filtered fallback必须在render延迟、backend无query与失败路径下验证。

### 10.4 Lifecycle、persistence与fault

覆盖multi-view、duplicate/close/reopen、Open/New/Reload/Close、world epoch、Play enter/exit、Game View、prefab/stage、plugin unload、unknown provider、save/reopen、crash restore、device loss、render failure和plugin panic。每个terminal路径都要证明scene不dirty、state不串view、资源/lease/cache无泄漏。

### 10.5 Performance、soak、pixel与跨引擎比较

固定scene、camera、visible set、hierarchy shape、quality、resolution、hardware/driver、warm-up和采样窗；记录compile/delta latency、steady CPU/GPU、draw/instance savings、query/pick、memory、allocation、fallback和power。使用100K/1M对象、1/4/16 view及多hidden ratio；与Unreal等比较必须同语义、同对象集合、同画质并报告分位数、置信区间和回归阈值。

## 11. Owner路由与非重复计数

| 范围 | Canonical owner | Editor70处理 |
|---|---|---|
| ViewInstance、surface、render product、frame currentness | Editor58 | 消费identity；新增filter generation关系，不重复Host缺口 |
| pointer/capture/picking/selection/highlight mechanics | Editor59 | 提供visibility-derived eligibility和current filter，不重复机制 |
| Outliner UI、persistent visibility/lock、transaction | Editor60 | 提供effective reason projection；不拥有authoring mutation |
| Scene document/world lifecycle | Editor61 | 注册visibility session suspend/retire adapter，不重复document问题 |
| Authoring transaction/history | Editor63 | transient state默认不进scene transaction；authoring列仍走父owner |
| Show Flag/display/debug/profile | Editor68 | object filter与category filter组合，不重复visualization registry |
| Realtime preview/activity visibility throttling | Editor69 | object visibility不同于pane/window activity，不重复计数 |
| ActiveSelf/ActiveInHierarchy/RenderLayerMask/World truth | Runtime24/109/110/111 | 只读组合，不以transient state修改父合同 |
| Editor70新增32项P1、8项P2 | 本报告 | 唯一计数per-view object visibility/isolation产品缺口 |

## 12. 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|

## 13. 最终判定

当前Zircon有可靠的authoring active/layer基础和正在成形的renderer-visible spatial query，但没有Scene Viewport对象temporary hide、isolate/local view或visibility-derived selection eligibility产品。它既不能表达用户意图，也不能保证render、picking、selection、overlay、Outliner和多view生命周期一致。

正确整改顺序必须从qualified per-view state、pure state machine和compiled filter开始，再接Runtime统一participation、renderer currentness/selection eligibility，最后开放UI、Outliner、persistence和规模资格。禁止先接两枚isolate图标、写`ActiveSelf`/`RenderLayerMask`、只删主mesh、保留全候选picking fallback、每帧扫描World，或让Game/Play继承编辑器Local View。

本报告完成current-source review，不代表实现完成。32项P1、8项P2和48个资格门保持Open/Fail，直到代码、产品、动态验证、故障/规模/pixel证据和同语义跨引擎基线逐项关闭。
