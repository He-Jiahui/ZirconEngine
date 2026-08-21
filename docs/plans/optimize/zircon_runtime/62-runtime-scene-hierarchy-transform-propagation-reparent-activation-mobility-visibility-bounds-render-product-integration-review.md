---
title: Runtime Scene Hierarchy、Transform Propagation、Reparent、Activation、Mobility、Visibility、Bounds、Render 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime62
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/core/framework/scene/mobility.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/dirty_state.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/scene/world/typed_api/component_mutation_effects.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render_visibility.rs
  - zircon_runtime/src/scene/inspection/subscription.rs
  - zircon_runtime/src/scene/reflect/builtin_reflection
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/gateway/in_process.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
tests:
  - zircon_runtime/src/scene/tests/derived_state.rs
  - zircon_runtime/src/scene/tests/derived_state
  - zircon_runtime/src/scene/tests/ecs_hierarchy_structure.rs
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs
  - zircon_runtime/src/scene/inspection/subscription/tests.rs
  - zircon_editor/src/tests/editing/node_ops.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/gateway/in_process.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/optimize/zircon_runtime/29-terrain-landscape-heightfield-quadtree-lod-material-layer-foliage-world-partition-physics-navigation-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SceneComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/SceneComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/Actor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Actor.cpp
  - dev/bevy/crates/bevy_transform/src/commands.rs
  - dev/bevy/crates/bevy_transform/src/systems.rs
  - dev/bevy/crates/bevy_ecs/src/hierarchy.rs
  - dev/bevy/crates/bevy_camera/src/visibility/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/graph/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/base.rs
  - dev/Fyrox/editor/src/scene/commands/graph.rs
  - dev/godot/scene/main/node.h
  - dev/godot/scene/main/node.cpp
  - dev/godot/scene/3d/node_3d.h
  - dev/godot/scene/3d/node_3d.cpp
  - dev/godot/scene/3d/visual_instance_3d.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.Jobs.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/RenderWorld.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Culling/InstanceCuller.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 62 · Runtime Scene Hierarchy、Transform Propagation、Reparent、Activation、Mobility、Visibility、Bounds、Render 与 Product Integration 工程化差距

## 1. 结论

Zircon 当前并非没有 Scene hierarchy 基础。`set_parent_checked` 已检查 missing parent、自父与循环，`HierarchyMutationIndex` 已维护稳定 root/child 顺序，PostUpdate/RenderExtract 有显式派生阶段，`WorldMatrix` 与 `ActiveInHierarchy` 已进入组件存储，Editor 的多节点reparent/delete也有undo transaction。这些基础应保留，不能退回递归对象树或把Scene逻辑塞进Renderer。

但当前“权威”并未真正成立。公共 `World::insert/get_mut/remove<T>` 可以直接写入或删除 `Hierarchy`、`LocalTransform`、`Mobility`、`WorldMatrix` 和 `ActiveInHierarchy`；反射层还直接暴露 `Hierarchy`/`ActiveInHierarchy` remove。由此可绕过parent存在、cycle、Static变换、派生状态只读和old/new ancestry事实路由。伪造循环后，`is_descendant`等无visited-set的父链会无限前进；删除或伪造派生组件后，camera/light/mesh/sprite提取可立即读到错误参与状态。这两项是当前合法公开API即可触发的P0，不是未来规模优化。

性能模型同样仍是全域型。五个布尔dirty domain决定任一hierarchy变化重跑全World validity、activation、world matrix、wide `SceneNode` cache和render extract；单个transform变化也重算所有WorldMatrix并clone所有NodeCache行。deep hierarchy validity逐entity重复走parent chain，最坏为O(N²)。dirty期间的world/active读取又为每次调用分配`Vec`/`HashSet`并走祖先链。新增diagnostics测试已经把1K/100K全量重建行为固化为“当前baseline”，但尚未形成必须下降到affected subtree的验收门。

产品语义也不完整：reparent只有“改parent并保留local transform”，没有KeepWorld/KeepRelative/Snap/逐轴规则；普通remove把子节点孤立并改变其world pose，recursive remove则删除整棵子树；Active同时承担render参与和潜在runtime enable含义；Mobility只有Dynamic/Static且Static连Editor构建期修改也拒绝；Scene侧visibility input没有bounds、delta、generation、removal或多view空间事实。Editor viewport还先clone World刷新render packet，再从原World的stale `nodes()`构建gizmo，形成同一帧混代读取。

本轮登记 **2项P0、64项P1、16项P2和48项验收门禁**。目标是建立 `SceneGraphAuthority + SceneMutationTransaction + AttachmentTransformRule + SceneLifecycleQueue + IncrementalDerivedStatePlanner + SceneParticipationState + MobilityPolicy + SpatialRepresentationRegistry + SceneRenderDelta`，而不是继续给全域dirty布尔增加例外。本轮只做静态review和计划记录，没有修改production、tests、Cargo、ABI或参考源码；没有运行Cargo、Editor、100K benchmark或跨引擎性能实验，因此不能宣称性能达到或超过Unreal。

## 2. 审查边界、规模与currentness

### 2.1 物理冻结

| 冻结组 | 文件 | 行 | bytes |
|---|---:|---:|---:|
| Hierarchy authority / public contracts | 20 | 5,663 | 218,544 |
| Derived state / render handoff | 10 | 3,747 | 131,366 |
| Editor product consumers | 7 | 1,970 | 64,917 |
| Focused tests | 12 | 3,287 | 125,396 |
| 去重合计 | **49** | **14,667** | **540,223** |

Zircon冻结集fingerprint为SHA-256 `27d7f49d6ecaabe36cb1a075f658bc4b01609516a455d65c3054faa27129602c`。算法将49个相对路径转为`/`、排序去重，以`path|lowercase per-file SHA-256`逐行编码，LF连接且末尾不追加LF，再对UTF-8 payload计算SHA-256。参考集另含20文件、40,518行、1,599,782 bytes。

冻结时49个Zircon文件中有12个working-tree修改：`frame_extract.rs`、`scene_system_registry.rs`、`derived_state.rs`、`schedule.rs`、`transform_validation.rs`、`world.rs`、`derived_state.rs`测试入口、`hierarchy_behavior.rs`、`ecs_performance_acceptance.rs`、Editor `render_packet.rs`、`editor_state_viewport.rs`和viewport测试。本报告审查工作区实际内容，不把这些改动归因于本会话，也不回退它们；实现前必须重算fingerprint并复核P0。

### 2.2 本轮拥有与明确不拥有

- Runtime62拥有Scene hierarchy authority、attachment/reparent/destruction语义、transform/activation派生传播、Mobility约束、Scene侧visibility/bounds输入与Editor产品接线。
- Runtime05继续拥有通用World clone、全局Scene lifecycle、NodeCache/whole extract父架构；Runtime60拥有ECS storage/query/change-detection/schedule kernel。本篇只纵切hierarchy protected state和具体derived hot path。
- Runtime23拥有坐标精度、large world、rebase和transform数值条件；Runtime62只要求attachment规则、derived generation和affected-subtree传播。
- Runtime09B拥有persistent render scene、GPU/CPU culling、occlusion、indirect draw与GPU residency；Runtime62只拥有Scene向其提供stable instance、bounds和delta的上游合同。
- Runtime29拥有world partition/terrain spatial hierarchy；Runtime62只定义普通Scene spatial representation与partition adapter边界。
- Runtime61拥有World/Level/project I/O和serialization schema；本篇不重复NodeRecord/clone的字段丢失问题。
- 用户已要求暂停tooling优化；本篇不新增脚本、Python工具或tooling迁移里程碑。

### 2.3 当前真实调用链

```text
Editor SetParent
  -> UpdateNodeCommand::capture_parent           [只改parent]
  -> apply_node_state
  -> World::set_parent_checked
  -> WorldFact::Reparented before mutation       [旧ancestry失效]
  -> insert<Hierarchy>
  -> WorldFact::Reparented after mutation        [新ancestry失效]
  -> mark_hierarchy_dirty                        [全域dirty]
  -> PostUpdate/RenderExtract
  -> full hierarchy validity
  -> full ActiveInHierarchy
  -> full WorldMatrix
  -> full wide SceneNode cache

Editor viewport
  -> World::build_viewport_render_packet(&self)
  -> clone whole World
  -> clone.run RenderExtract and build geometry
  -> original World remains dirty
  -> build_scene_gizmos(original)
  -> original.nodes()                            [retained stale rows]
  -> original.active/world reads                 [projected current values]

Render visibility
  -> collect mesh/sprite/particle for one camera
  -> filter ActiveInHierarchy + camera layer
  -> VisibilityRenderableInput
       {entity, stable_instance_key, Mobility, RenderLayerSet}
  -> sort/dedup and rebuild static/dynamic lists
  -> no bounds/delta/removal/generation
```

## 3. 当前应保留的能力

1. `set_parent_checked`对missing parent、自父和常规cycle fail closed，并在无变化时不制造dirty。
2. `HierarchyMutationIndex`维护稳定root/child顺序，subtree与direct-child访问不再必然全表扫描。
3. reparent前后各做一次ancestry invalidation，意图覆盖old/new subtree watcher链；这个需求应保留，但应由transaction receipt表达。
4. derived系统以显式PostUpdate/RenderExtract阶段运行，避免任意getter隐式可变刷新World。
5. dirty期间的projected read至少返回当前parent/transform/active语义，而不是永远读旧cache。
6. `DetachedEntityBatch`具备preflight、stable order、row move和失败时归还batch，可成为undo/transaction基础。
7. render mesh已有stable primitive instance key与transform revision，Mobility也进入extract。
8. Editor多选reparent在cycle失败时保持整批不提交，delete可以恢复selection与active camera。
9. 新增derived diagnostics能观测validity parent steps、active/world propagation和NodeCache rebuilt rows，后续可直接变为budget gate。

## 4. 参考引擎事实与Zircon差异

| 参考 | 代码事实 | Zircon应吸收的合同 |
|---|---|---|
| Unreal `SceneComponent` | `AttachToComponent`/`DetachFromComponent`接收逐轴KeepRelative/KeepWorld/Snap规则；检查cycle、template、mobility与physics；更新attachment hooks、tick prerequisite、overlap、bounds、render、navigation。 | Reparent是带规则、验证、回调和下游影响的transaction，不是只写parent。 |
| Unreal visibility/mobility | Visible、HiddenInGame、Editor temporary hidden分离；Mobility包含Static/Stationary/Movable并触发reregister/child policy。 | 分离authoring visibility、game visibility、process participation与render state；Mobility必须有阶段和下游registration语义。 |
| Bevy hierarchy | `ChildOf`是source of truth，hook同步`Children`，despawn默认递归；`set_parent_in_place`/`remove_parent_in_place`显式保持GlobalTransform。 | protected relationship必须在所有mutation入口同步双向索引；keep-world必须是显式API。 |
| Bevy transform | Changed/Removed组件形成dirty roots；static optimization把dirty向祖先传播并跳过未变subtree，parallel path按root处理。 | dirty state应携entity frontier、subtree range和generation，不应只有World级bool。 |
| Bevy visibility | Visibility、InheritedVisibility、ViewVisibility分层；bounds、frustum、visibility class、range、CPU/GPU culling有明确schedule。 | Scene authored/inherited state、per-view结果和spatial bounds必须分层。 |
| Fyrox graph | Editor `LinkNodesCommand`调用`link_nodes_keep_global_transform`；Graph按消息合并最小dirty roots，只更新变化chain并分别传播transform/visibility/enabled。 | Editor默认reparent需明确world-pose合同；派生域按变化种类和最小root合并。 |
| Godot Node/Node3D | `reparent(..., keep_global_transform=true)`默认保持global；parent/unparent通知、tree enter/exit、process mode、deferred queue_free和top-level transform均为显式生命周期。 | parent、transform、processing、tree membership和destruction不得压成一个布尔或立即erase操作。 |
| Godot VisualInstance3D | Scene node持local/global visibility，VisualInstance把transform/visible/scenario同步到RenderingServer，并公开AABB/custom AABB/layer。 | Scene需拥有可查询bounds与render instance同步事实，而非每帧临时分类列表。 |
| Unity Graphics RenderWorld | 持久保存local/world AABB、move current/previous、visible previous、rendering enabled、LOD、GPU handle；transform更新只提交变更instance队列并同步bounding sphere。 | SceneRenderDelta需create/update/remove、bounds、motion/teleport、LOD/layer与generation，供persistent render scene消费。 |

## 5. P0：公共API可制造非法Scene事实

### RSH-P0-001：Hierarchy、LocalTransform与Mobility的protected mutation可被公共泛型API绕过

`typed_api.rs:117-216`、`:261-292`和`:294-356`公开任意`Component`的insert/get_mut/remove。`validate_fixed_component`只在insert时检查LocalTransform数值与Mobility，完全不验证Hierarchy；get_mut则在返回`&mut T`前只标dirty。`compiled_binding/tests.rs:155-169`甚至把`get_mut::<Hierarchy>`当作受支持escape hatch。反射`hierarchy.rs:148-150`还允许直接remove Hierarchy。

因此调用者可插入missing/self/cycle parent、删除Hierarchy、通过`get_mut<LocalTransform>`移动Static节点、通过`get_mut<Mobility>`制造Static-under-Dynamic，且不会产生`set_parent_checked`的old/new `WorldFact`。后续validity只会在stage中静默断边；在此之前`is_descendant`和detached-root normalization沿parent链没有visited set，预存cycle可令下一次checked reparent或batch detach不终止。目标必须把Parent、LocalTransform、Mobility定义为protected components：通用ECS mutation不能取得写权限，只能通过Scene command/capability和原子preflight/commit入口修改。

### RSH-P0-002：WorldMatrix与ActiveInHierarchy可被伪造或删除，Renderer会消费错误派生事实

`fixed_components.rs:23-51`把`WorldMatrix`和`ActiveInHierarchy`注册为普通`Component`，所以公共insert/get_mut/remove均可写。`active_in_hierarchy.rs:133-135`还把derived component的反射remove接到通用remove。移除后，mutation routing不会标记transform/active dirty；当相应domain为clean时，`project_active_in_hierarchy_for_read`和`project_world_transform`会直接返回None。render collector把`active_in_hierarchy != Some(true)`视为不可渲染，world transform缺失又会落入default；反向伪造true或任意matrix则可绕过真实父链。

readonly反射field不能构成权威边界。WorldMatrix/ActiveInHierarchy必须是系统拥有的immutable derived storage，或需要内部`DerivedWritePermit`才能写；外部typed/reflection/dynamic API只能读。移除、clone、restore、schema rebuild也必须由derived planner统一重建并携generation，不允许合法World进入“domain clean但derived row缺失/伪造”的状态。

## 6. P1：Hierarchy、Attachment与Destruction（RSH-P1-001至018）

| ID | 差距 | 重构要求 |
|---|---|---|
| RSH-P1-001 | `set_parent_checked(child,parent)`只替换parent并保留local transform，world pose会随新parent改变。 | 引入`AttachmentTransformRule`，至少支持KeepWorld、KeepRelative、SnapToTarget。 |
| RSH-P1-002 | 没有translation/rotation/scale逐轴规则、socket/bone attachment、absolute axis或top-level继承开关。 | 规则对象分轴，attachment target使用typed node/socket identity，并明确不可逆分解诊断。 |
| RSH-P1-003 | 返回值只有`bool`，未记录old/new parent、old/new local/world transform、affected subtree、rule或generation。 | 返回`ReparentReceipt`，供undo、replication、render、physics和inspection消费。 |
| RSH-P1-004 | 同一`Reparented` fact在mutation前后各记录一次，靠当前World ancestry的隐式时序完成双链失效。 | 由一次transaction同时携old/new ancestry，subscriber只接收一个committed fact。 |
| RSH-P1-005 | `WorldFact::Reparented`只有new_parent，没有old_parent、sibling order、attachment rule或transaction id。 | 扩展typed topology delta，禁止consumer反查一个已变化World来猜旧事实。 |
| RSH-P1-006 | runtime没有batch reparent/reorder的统一preflight/commit；Editor transaction只能在更高层组合单节点命令。 | 提供batch topology transaction，一次cycle/mobility/order验证后原子发布。 |
| RSH-P1-007 | child顺序来自stable entity order，没有authorable sibling order、insert-before/after或order-change fact。 | TopologyIndex同时维护parent和显式sibling ordinal，保存、undo和query稳定。 |
| RSH-P1-008 | `insert_node_record`允许missing parent和out-of-order link进入live World，等PostUpdate再修复。 | import先进入staging graph；commit前统一resolve，允许forward ref但不发布非法live topology。 |
| RSH-P1-009 | validity repair把invalid parent静默改为None，没有source location、reason、repair receipt或Reparented fact。 | fail closed或发布typed repair diagnostic；Editor不得无提示改变作者层级。 |
| RSH-P1-010 | `remove_entity`把direct children改为root且保留local transform，导致每个child world pose突变。 | 删除API必须显式选择DestroySubtree、OrphanKeepWorld或OrphanKeepRelative。 |
| RSH-P1-011 | orphan child只走generic Hierarchy insert，没有各自old/new Reparented fact；child-root subtree watcher可能不被标dirty。 | topology transaction为每个受影响root发布合并delta并验证subscription coverage。 |
| RSH-P1-012 | plain remove采用orphan，`remove_entity_recursive`采用subtree detach；同名“remove”家族没有统一生命周期policy。 | 收敛为一个destruction service和显式policy，旧入口硬切迁移。 |
| RSH-P1-013 | entity立即从storage erase，没有PendingDestroy、safe point、cancel、dependent reference或late callback阶段。 | 引入deferred destruction queue、tombstone generation、quiesce和terminal receipt。 |
| RSH-P1-014 | 缺OnParentChanged/ChildAttached/ChildDetached/EnterTree/ExitTree/Destroyed等有序hook合同。 | 定义prepare/commit/post-commit hook顺序、重入规则和panic disposition。 |
| RSH-P1-015 | hierarchy mutation没有main-thread/scene-phase/command-buffer约束；公开`&mut World`可在任意caller阶段改拓扑。 | mutation capability绑定Scene phase和owner；迭代期改动进入deferred command segment。 |
| RSH-P1-016 | attachment没有physics weld、simulated-body、constraint、tick prerequisite或overlap同步policy。 | physics adapter在transaction preflight/commit中参与，不允许Scene与physics双transform owner。 |
| RSH-P1-017 | 没有max depth、max children、batch size、cycle-check step或topology memory预算。 | project/profile提供结构预算，admission前估算，超限给typed terminal。 |
| RSH-P1-018 | 多个parent-chain walker假设树已合法且没有cycle guard，和允许raw mutation/import invalid state的现实冲突。 | 所有边界先保证不可构造非法图；诊断/恢复walker仍须bounded visited/depth保护。 |

## 7. P1：Derived Transform、Activation与Cache（RSH-P1-019至034）

| ID | 差距 | 重构要求 |
|---|---|---|
| RSH-P1-019 | `DerivedStateDirty`只有hierarchy/active/transforms/node_cache/render_extract五个World级bool。 | 记录changed entities、dirty roots、subtree intervals、component generation和reason。 |
| RSH-P1-020 | 已经由`set_parent_checked`验证过的单条边仍会触发全World `rebuild_hierarchy_validity`。 | checked commit直接维护拓扑不变量；全图validator只用于import/debug/repair lane。 |
| RSH-P1-021 | 每次hierarchy dirty都先复制所有entity parent到HashMap。 | TopologyIndex成为canonical read model，不为一次局部变更重建parent snapshot。 |
| RSH-P1-022 | validity对每个entity重新走parent chain，deep hierarchy最坏O(N²)。 | 用color/topological generation一次O(N+E)验证staging graph，live graph只验证变更边。 |
| RSH-P1-023 | 单个leaf transform变化会重算所有WorldMatrix；1K diagnostics测试已明确观测1K row。 | 从最小dirty roots更新affected subtree，未变branch必须0 visit/0 write。 |
| RSH-P1-024 | 单个ActiveSelf变化会重算所有root及其整棵World，而不是只处理该节点subtree。 | activation frontier从changed node向下传播，并在effective value未变时剪枝。 |
| RSH-P1-025 | propagation对每个节点无条件`replace_derived_component`，即使matrix/active值相同也推进change tick。 | 值/parent generation未变则跳过写入，避免下游虚假Changed和GPU更新。 |
| RSH-P1-026 | 任意transform/hierarchy/多数component变化都会清空并重建wide `Vec<SceneNode>`，clone name及多类组件。 | Inspection使用columnar artifact或row delta；不再复制整个SceneNode作为runtime cache。 |
| RSH-P1-027 | `project_node_for_read`与`refresh_node_cache`各维护一份宽字段列表，新增组件易产生双实现漂移。 | 一个schema-driven inspection projector同时生成snapshot和delta。 |
| RSH-P1-028 | dirty world-matrix read每次分配Vec+HashSet并O(depth)组合祖先。 | 通过generation-aware derived cache或bounded scratch arena提供O(1)常规读。 |
| RSH-P1-029 | dirty active read每次分配HashSet并O(depth)检查ActiveSelf链。 | effective participation由增量传播维护；同步查询不重复祖先扫描。 |
| RSH-P1-030 | `nodes()`返回stale retained slice，`node_records()`/`find_node()`返回fresh projected值，形成两个合法真相。 | API按`SceneSnapshotGeneration`返回sealed snapshot；禁止同名查询混用freshness模型。 |
| RSH-P1-031 | `nodes()`没有文档化generation、stage或staleness，consumer无法判断何时安全。 | 返回typed snapshot/view和generation，不直接暴露无上下文slice。 |
| RSH-P1-032 | PostUpdate可刷新NodeCache但RenderExtract仍pending；不同stage读取到的derived domain并非同一sealed generation。 | 一次derived commit原子发布关联domain generation，partial stage只能内部可见。 |
| RSH-P1-033 | 多次同subtree mutation没有明确root collapse/ancestor dominance算法，主要依赖World级coalescing。 | DirtyRootSet删除被已有ancestor覆盖的root，并合并相邻topology intervals。 |
| RSH-P1-034 | 没有static subtree skip、parallel root/chunk propagation或dynamic-scene策略切换。 | 参考Bevy按profile选择static optimization，使用依赖有序parallel chunks并记录work量。 |

## 8. P1：Activation、Mobility与Participation（RSH-P1-035至046）

| ID | 差距 | 重构要求 |
|---|---|---|
| RSH-P1-035 | `ActiveSelf/ActiveInHierarchy`同时被render提取使用，又没有定义它对script/tick/physics/audio/navigation的语义。 | 建立`SceneParticipationState`，分别定义process、render、physics、audio、navigation与editor participation。 |
| RSH-P1-036 | 可见性只有Active的布尔AND，没有Inherited/Hidden/Visible override三态。 | authored visibility与effective visibility分层，允许child显式override并定义root默认。 |
| RSH-P1-037 | 没有Visible、HiddenInGame、EditorTemporaryHidden、OwnerNoSee等独立域。 | authoring、game、editor和per-view visibility使用独立flag与组合规则。 |
| RSH-P1-038 | 没有pause/process mode、tick enabled、physics simulation enabled或script lifecycle enable的继承政策。 | 每个subsystem注册participation adapter，禁止用Active推断全部状态。 |
| RSH-P1-039 | 缺Hierarchy/LocalTransform/ActiveSelf时分别默认为root/identity/true，能把构造错误包装成合法状态。 | required-component schema在spawn/import/restore时补全或拒绝，并记录来源。 |
| RSH-P1-040 | leaf ActiveSelf切换仍触发全World active和wide cache重建。 | effective participation按subtree增量传播，分域通知真正受影响consumer。 |
| RSH-P1-041 | Mobility只有Dynamic/Static，没有Stationary、construction/editor state或runtime frozen state。 | 定义Mobility/UpdateFrequency/BuildState三轴，避免一个enum承担所有优化政策。 |
| RSH-P1-042 | Static节点在Editor构建期也禁止transform/reparent，无法区分authoring edit和runtime mutation。 | mutation context携Authoring/Construction/Runtime；Static只约束已注册runtime资源并触发重建。 |
| RSH-P1-043 | 把节点改为Dynamic时为找Static direct child扫描全部stable entities，已有child index却未使用。 | 使用TopologyIndex direct children，验证成本与direct child数相关。 |
| RSH-P1-044 | mobility transition只拒绝不合法关系，没有cascade、auto-promote、rebuild或明确用户选择。 | `MobilityTransitionPolicy`支持Reject/Cascade/Promote，并返回affected set与cost receipt。 |
| RSH-P1-045 | Static/Dynamic目前主要进入visibility bucket和`RendererCommon.is_static`，没有static spatial/bake registration。 | Static commit必须驱动spatial、lighting、physics、navigation、render registration generation。 |
| RSH-P1-046 | Scene没有teleport、moved-this/previous-frame或previous transform语义，render history只能自行猜测。 | transform receipt区分continuous/teleport/reparent，并向Runtime09H1/09B发布motion generation。 |

## 9. P1：Bounds、Spatial Visibility与Render Handoff（RSH-P1-047至056）

| ID | 差距 | 重构要求 |
|---|---|---|
| RSH-P1-047 | 普通mesh/sprite没有canonical local/world bounds component或provider；particle才有独立optional bounds列表。 | `SpatialRepresentationRegistry`按renderable/provider产生local bounds、world bounds和validity。 |
| RSH-P1-048 | `VisibilityRenderableInput`只有identity、Mobility和layer，无法做frustum/LOD/range/occlusion admission。 | 加入world bounds、visibility flags、LOD group/range、spatial cell和bounds generation。 |
| RSH-P1-049 | Scene先按单个camera active/layer过滤，再生成visibility input，render scene看不到其他view/shadow可能需要的实例。 | persistent SceneRenderDelta先发布全Scene实例；per-view culling在render owner执行。 |
| RSH-P1-050 | 每帧只有完整列表，没有created/updated/removed、old/new generation或tombstone。 | 发布bounded delta stream和periodic full resync receipt，remove不可由“本帧没出现”猜测。 |
| RSH-P1-051 | 没有Mesh3D/Mesh2D/Sprite/Particle/Light等visibility class，所有renderable挤在一个向量。 | typed visibility class支持多owner注册和per-class extraction/culling。 |
| RSH-P1-052 | 没有distance range、small-object、HLOD/LOD group、occluder、always-visible或no-frustum policy。 | Scene提供authorable policy，Runtime09B编译为view/GPU culling数据。 |
| RSH-P1-053 | static/dynamic/renderable三个列表每帧sort、BTreeSet去重并重复保存同一identity。 | persistent registry按handle维护分类bitset/index，delta只更新改变的instance。 |
| RSH-P1-054 | particle bounds与`VisibilityRenderableInput`没有stable一一对应，visibility entry只到emitter entity。 | emitter/particle batch使用typed stable instance key并直接关联bounds与draw range。 |
| RSH-P1-055 | 没有bounds invalid、culled reason、missing provider、stale generation或overflow诊断。 | 每阶段输出reasoned counts、budget、fallback与qualification receipt。 |
| RSH-P1-056 | Scene没有明确spatial owner，Renderer、world partition和各组件将被迫各建一套事实。 | Runtime62拥有canonical Scene spatial truth；Runtime29/09B通过adapter消费，不复制authority。 |

## 10. P1：Editor产品接线与测试证据（RSH-P1-057至064）

| ID | 差距 | 重构要求 |
|---|---|---|
| RSH-P1-057 | Editor SetParent默认改变world pose，UI/intent没有KeepWorld/KeepRelative选择。 | hierarchy drag/drop默认显式KeepWorld并允许用户选择；journal记录rule。 |
| RSH-P1-058 | `NodeEditState`只保存local transform和parent，没有attachment rule、sibling order、socket或world pose receipt。 | undo保存transaction inverse，不靠重新应用一组可能已失真的字段。 |
| RSH-P1-059 | viewport geometry在World clone上flush，gizmo却遍历原World stale `nodes()`并读取fresh active/world值。 | geometry和overlay消费同一sealed SceneSnapshotGeneration；移除clone刷新旁路。 |
| RSH-P1-060 | InProcessGateway `with_world_mut`只借用World，不返回commit generation、affected domains或freshness fence。 | Editor mutation通过command gateway获得`SceneMutationReceipt`和snapshot-ready notification。 |
| RSH-P1-061 | Create undo调用plain `remove_entity`，Delete调用recursive detach，命令语义依赖“新节点暂时没有child”的隐含前提。 | command journal使用统一destruction policy并验证新增child/extension后的inverse。 |
| RSH-P1-062 | `ecs_hierarchy_structure.rs`仍要求`self.entities.iter().copied()`，当前实现已改为`stable_entity_ids()`，文本条件不能通过。 | 删除source-shape断言，改为复杂度diagnostics与行为测试。 |
| RSH-P1-063 | `projected_reads.rs`要求旧的`entity`变量精确文本，当前实现使用`current`；大量测试锁定Option写法而非合同。 | 测试observable generation、allocation、visited/written rows和结果，不比较源码拼写。 |
| RSH-P1-064 | 缺raw-illegal-state拒绝、keep-world、orphan policy、fact coverage、bounds delta、mixed-generation viewport、deep-chain/fuzz/model测试。 | M62-0先建RED characterization矩阵，再允许production重构。 |

## 11. P2：可维护性、命名与诊断（16项）

| ID | 差距 | 收敛方向 |
|---|---|---|
| RSH-P2-001 | 公开`WorldTransform`类型没有生产使用，实际派生owner是`WorldMatrix`。 | 删除死类型或定义唯一用途，禁止同义公开合同。 |
| RSH-P2-002 | `node_records()`返回`Vec<SceneNode>`而非`NodeRecord`。 | 改为`projected_nodes`或typed inspection snapshot。 |
| RSH-P2-003 | 双次`Reparented`被fact coalescing吞并，但diagnostics仍计两次ancestry walk/coalesced fact。 | 一个transaction一次记录old/new链和成本。 |
| RSH-P2-004 | 多个Scene component字段公开，调用者容易把可读DTO误当可写authority。 | authorable与derived类型分module并收紧field visibility。 |
| RSH-P2-005 | `WorldMatrix`仍derive Serialize/Deserialize，和“runtime derived only”语义不一致。 | derived type不直接持久化，仅由snapshot schema声明可重建性。 |
| RSH-P2-006 | ActiveInHierarchy reflection `contains`在row缺失时仍因entity存在返回true。 | contains/read/remove对同一sealed generation保持一致。 |
| RSH-P2-007 | `VisibilityInput`重复保存renderable/static/dynamic identity向量与完整entries。 | persistent index+classification bitset，查询按需形成view。 |
| RSH-P2-008 | stable instance key仍是裸`u64`，entity和primitive key可被误混。 | 使用Runtime24 owner-qualified typed handle。 |
| RSH-P2-009 | Editor把SceneError压成外部字符串，attachment失败缺结构化字段。 | gateway传typed error domain与localizable diagnostics。 |
| RSH-P2-010 | `nodes()`调用点常立即clone/to_vec，放大wide cache成本。 | consumer使用borrowed column view或增量artifact。 |
| RSH-P2-011 | system enum名`WorldTransform`与组件`WorldTransform`同名，但系统实际写WorldMatrix。 | 重命名为`PropagateWorldMatrices`并删除歧义类型。 |
| RSH-P2-012 | hierarchy/derived diagnostics只有累计count，没有reason、max depth、dirty roots或per-stage latency分布。 | 输出结构化frame receipt和p50/p95/p99原始样本引用。 |
| RSH-P2-013 | 100K构造测试位于普通unit lane且未标large/benchmark profile。 | 分离deterministic unit、scale acceptance和benchmark lane。 |
| RSH-P2-014 | 默认root/identity/active行为散落在getter中，required schema不可发现。 | 集中到component requirement descriptor。 |
| RSH-P2-015 | `Reparented`、component invalidation、scene binding generation各自维护结构变更含义。 | 从一个TopologyDelta派生所有consumer notification。 |
| RSH-P2-016 | 当前测试把“全量重建数量正确”当baseline，却没有目标预算字段。 | diagnostics断言同时记录current与required ceiling，里程碑逐步收紧。 |

## 12. 目标架构

```text
Editor / Script / Runtime commands
  -> SceneMutationTransaction
       - authority + phase capability
       - topology/transform/mobility preflight
       - AttachmentTransformRule
       - destruction policy
       - batch atomic commit
  -> SceneGraphAuthority
       - Parent + Children + sibling order
       - topology generation + subtree intervals
       - lifecycle/tombstone queue
  -> IncrementalDerivedStatePlanner
       - minimal dirty roots
       - local/global transform generations
       - participation generations
       - static subtree skip + parallel chunks
  -> SpatialRepresentationRegistry
       - local/world bounds
       - visibility class/range/cell/LOD
  -> SceneRenderDelta
       - create/update/remove
       - stable instance + bounds + motion + layer
       - source generation + resync receipt
  -> Runtime09B Persistent Render Scene

SceneInspectionSnapshot
  <- same committed topology/derived generation
  -> hierarchy panel / viewport gizmo / selection / automation
```

### 12.1 必须冻结的不变量

1. live Scene永远是forest；missing/self/cycle parent不能进入committed generation。
2. Parent/Children/sibling order由一个authority原子维护，任意public path都不能只改其中一边。
3. WorldMatrix/effective participation只能由derived planner写，外部API没有写/删能力。
4. attachment rule明确决定local/world transform；失败不改变topology、transform或事实队列。
5. destruction policy显式，deferred destroy前后身份、引用和callback顺序可验证。
6. 单节点变换访问量与affected subtree相关，不与World总entity数相关。
7. Scene inspection、render delta和runtime query只能引用已提交且可比较的generation。
8. bounds缺失或provider失效必须有typed disposition，不能默认为“无限可见”或“静默消失”。
9. Static是可验证的下游registration承诺，不只是render bucket标签。
10. Editor和runtime mutation使用同一核心transaction，不复制第二套层级规则。

## 13. 分阶段重构里程碑

### M62-0：P0 characterization与合同冻结

- 为generic insert/get_mut/remove和reflection remove建立非法状态RED测试；
- 覆盖cycle导致parent walker不终止、Static绕过、derived row删除/伪造、render错误参与；
- 冻结所有Hierarchy/LocalTransform/Mobility/WorldMatrix/ActiveInHierarchy写入口和caller；
- 删除或改写当前不能匹配源码的source-shape测试。

### M62-1：Protected component authority硬切

- 引入internal write permit和Scene mutation capability；
- 通用ECS API拒绝protected/derived component write/remove/get_mut；
- reflection/property path全部路由structured mutator；
- 迁移caller后删除raw hierarchy escape，不保留compat shim。

### M62-2：Topology、Attachment与Destruction transaction

- 建Parent/Children/sibling order统一TopologyIndex；
- 实现KeepWorld/KeepRelative/Snap、batch reparent和typed receipt；
- 加入deferred destruction、orphan policy、lifecycle hooks与subscription delta；
- import/restore在staging graph完成O(N+E)验证后一次发布。

### M62-3：增量Derived State

- dirty roots、subtree intervals、generation和reason替代World级bool；
- transform/active按affected subtree更新并在值不变时剪枝；
- parallel root/chunk、static skip和bounded scratch；
- 删除wide runtime NodeCache，改为generation-bound inspection artifact。

### M62-4：Participation与Mobility policy

- 分离process/render/physics/audio/navigation/editor visibility；
- 建Visibility authored/inherited/view状态与pause/process mode；
- Mobility扩为policy+build state，authoring/runtime transition有receipt；
- 下游static registration/rebuild participant接线。

### M62-5：Bounds、Spatial Representation与Render Delta

- 内建Mesh/Sprite/Particle/Light bounds provider与custom bounds；
- world bounds随transform generation增量更新；
- SceneRenderDelta发布create/update/remove、motion、LOD/range/layer/cell；
- Runtime09B persistent render scene消费delta，Runtime29通过adapter接partition cell。

### M62-6：Editor产品迁移

- hierarchy drag/drop提供明确attachment rule和sibling placement；
- command journal存inverse receipt；
- viewport geometry/gizmo/selection消费同代inspection snapshot；
- 删除World clone刷新与stale `nodes()`产品旁路。

### M62-7：规模、故障与跨引擎资格

- deep/wide/random hierarchy model test、cycle fuzz和transaction fault injection；
- 1/1K/100K/1M entity的visited/written/allocated bytes与p50/p95/p99；
- multi-view、shadow、LOD、bounds、visibility delta和render removal E2E；
- 与Unreal/Bevy/Fyrox/Godot/Unity Graphics同场景、同硬件、同profile保存原始数据。

## 14. 验收门禁（48项）

### Authority与transaction（G01-G12）

- [ ] RSH-G01：外部generic/reflection/dynamic API无法write/remove/get_mut protected或derived component。
- [ ] RSH-G02：missing/self/cycle parent在commit前typed拒绝，live generation始终为forest。
- [ ] RSH-G03：预存损坏图上的所有诊断walker受depth/visited预算保护，不挂起。
- [ ] RSH-G04：Static transform/mobility约束不能被任一typed/property/script入口绕过。
- [ ] RSH-G05：derived row不存在或generation不匹配时fail closed并触发rebuild/quarantine，不返回伪clean。
- [ ] RSH-G06：single/batch reparent失败时topology、local/world transform、fact和generation均不变。
- [ ] RSH-G07：KeepWorld/KeepRelative/Snap及逐轴组合有matrix tolerance golden tests。
- [ ] RSH-G08：old/new parent、sibling order、attachment rule和affected set进入一个receipt。
- [ ] RSH-G09：Parent/Children/sibling order双向一致，random operation model test通过。
- [ ] RSH-G10：DestroySubtree/OrphanKeepWorld/OrphanKeepRelative三种policy行为明确。
- [ ] RSH-G11：deferred destruction hook顺序、重入、panic和reference disposition通过fault tests。
- [ ] RSH-G12：旧raw mutation/duplicate fact/implicit orphan入口被结构门禁止重新引入。

### Derived state与性能（G13-G24）

- [ ] RSH-G13：leaf transform只visit/write affected subtree，未变sibling branch为0。
- [ ] RSH-G14：leaf ActiveSelf只传播其subtree，effective值未变时提前停止。
- [ ] RSH-G15：checked reparent不运行全World validity snapshot。
- [ ] RSH-G16：staging graph validation在deep/wide输入上为O(N+E)且受预算约束。
- [ ] RSH-G17：dirty root collapse正确消除被ancestor覆盖的root。
- [ ] RSH-G18：unchanged WorldMatrix/participation不推进change tick或render delta。
- [ ] RSH-G19：普通world/active查询O(1)且0 heap allocation；诊断repair lane除外。
- [ ] RSH-G20：parallel propagation结果与serial oracle逐bit/容差一致且无alias/ordering race。
- [ ] RSH-G21：static-heavy和dynamic-heavy profile都能选择合适策略并记录选择原因。
- [ ] RSH-G22：inspection snapshot、runtime query和render delta携同一committed generation。
- [ ] RSH-G23：删除wide NodeCache后hierarchy/inspector/selection功能和稳定顺序不回退。
- [ ] RSH-G24：1K/100K/1M visited、written、allocated bytes和latency都有原始样本与ceiling。

### Participation、Mobility与Spatial（G25-G36）

- [ ] RSH-G25：process/render/physics/audio/navigation/editor participation状态可独立设置和继承。
- [ ] RSH-G26：Inherited/Hidden/Visible override及root默认有完整truth table。
- [ ] RSH-G27：HiddenInGame与EditorTemporaryHidden不互相污染。
- [ ] RSH-G28：required component缺失在spawn/import/restore边界补全或拒绝，不由getter静默猜测。
- [ ] RSH-G29：Mobility authoring/runtime transition按policy Reject/Cascade/Promote且可undo。
- [ ] RSH-G30：Static registration同步render/physics/navigation/lighting generation。
- [ ] RSH-G31：Mesh/Sprite/Particle内建local/world bounds与custom bounds有golden tests。
- [ ] RSH-G32：bounds仅在source asset或transform generation变化时更新。
- [ ] RSH-G33：SceneRenderDelta明确create/update/remove，丢帧后可用full resync恢复。
- [ ] RSH-G34：multi-view/shadow不会因主camera预过滤而丢实例。
- [ ] RSH-G35：visibility class、layer、range、LOD、cell和always/no-cull policy正确组合。
- [ ] RSH-G36：missing/invalid/stale bounds、provider unload和budget overflow均有typed disposition。

### 产品、测试与资格（G37-G48）

- [ ] RSH-G37：Editor hierarchy drag/drop默认语义明确，world pose与undo/redo守恒。
- [ ] RSH-G38：multi-node reparent/reorder为一个原子journal operation。
- [ ] RSH-G39：viewport geometry、gizmo、selection、picking消费同一Scene snapshot generation。
- [ ] RSH-G40：InProcess与dynamic gateway返回相同mutation receipt/freshness合同。
- [ ] RSH-G41：create/delete/undo在命令执行后新增child或extension component时仍守恒。
- [ ] RSH-G42：subtree watch对reparent、orphan、recursive destroy、restore均只产生正确合并失效。
- [ ] RSH-G43：测试不再依赖源码变量名、Option写法或旧循环文本。
- [ ] RSH-G44：cycle/depth/child-count、NaN、non-invertible scale、provider unload和panic fuzz通过。
- [ ] RSH-G45：Renderer持久instance在Scene create/update/remove后数量、identity、bounds守恒。
- [ ] RSH-G46：long-session reparent/destroy/spawn soak后entity、topology、subscription、render handle和memory守恒。
- [ ] RSH-G47：Windows/Linux动态验证和Miri/sanitizer适用lane通过；Linux专属要求才进入WSL。
- [ ] RSH-G48：跨引擎比较使用相同语义场景、硬件、build profile和原始数据，不以单次FPS宣称超越。

## 15. 禁止的临时修补

- 禁止只在`validate_fixed_component`里增加Hierarchy分支，却继续允许`get_mut/remove`绕过authority。
- 禁止保留raw Hierarchy mutation并用“下一帧validity会修复”作为合法性证明。
- 禁止给`DerivedStateDirty`再加更多World级bool来模拟增量更新。
- 禁止把KeepWorld只做成Editor侧先读后写两条松散命令；必须由runtime topology transaction原子完成。
- 禁止让Renderer反向扫描Scene组件计算bounds或猜测remove；Scene必须发布稳定spatial delta。
- 禁止把ActiveSelf继续扩展成更多含义不明的全局enable开关。
- 禁止用`World::clone`刷新stale cache或为每个consumer复制一份SceneNode快照。
- 禁止用source-shape test、API存在、compile-only或100K全量重建“能完成”代替复杂度和产品验收。
- 禁止为旧raw mutation/old hierarchy path保留compat re-export；迁移完成后硬切删除。

## 16. 当前状态

- review状态：complete；implementation状态：pending。
- 新增计数：P0=2、P1=64、P2=16、验收门禁=48。
- 本轮只新增审查文档并同步总账；没有修改production、tests、Cargo、ABI或reference source。
- MVP 00 baseline尚未accepted，本轮按政策没有运行Cargo、Editor、benchmark或动态产品验证。
- 当前源码已存在12个相关dirty文件，实施切片开始前必须按最新fingerprint重核finding和测试漂移。
- 首个实现切片必须从M62-0/M62-1关闭protected/derived authority P0开始；在authority硬切前不得先做局部缓存或GPU visibility扩展。
