---
title: Runtime Scene Hierarchy、Transform Propagation、Reparent、Activation、Mobility、Visibility、Bounds、Render 与 Product Integration 当前源码工程化差距复核
category: zircon_runtime
report_id: Runtime110
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - zircon_runtime/src/core/framework/scene/mobility.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/renderer_common.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
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
  - zircon_runtime/src/scene/ecs/frame_performance_diagnostics.rs
  - zircon_runtime/src/scene/inspection/subscription.rs
  - zircon_runtime/src/scene/reflect/builtin_reflection
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/gateway/contract.rs
  - zircon_editor/src/core/gateway/in_process.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/controller
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
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99i-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99j-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-current-source-review.md
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
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
doc_type: current-source-review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime110 · Scene Hierarchy / Transform / Visibility 当前源码工程化差距复核

## 1. 结论

Runtime62 的核心判断在当前源码中仍成立。Zircon 已有可保留的 scene forest 基础、稳定 entity 顺序、`HierarchyMutationIndex`、显式 PostUpdate/RenderExtract stage、`WorldMatrix`/`ActiveInHierarchy` 派生行、detached subtree batch、Editor undo command 和 stable render primitive key；它不是一个只有递归对象树的演示实现。但这些局部能力尚未形成受保护的 Scene graph authority、原子 attachment/destruction transaction、按 affected subtree 工作的 derived planner，或向 persistent render scene 发布 bounds/delta 的工程合同。

两项最高风险没有变化。公共 `World::insert/get_mut/remove<T>` 仍能写入或删除 `Hierarchy`、`LocalTransform`、`Mobility`、`WorldMatrix` 与 `ActiveInHierarchy`；`validate_fixed_component` 只校验 LocalTransform 和 Mobility，raw `get_mut` 又在返回引用前标 dirty，无法验证引用最终写入的值。Hierarchy 与 ActiveInHierarchy reflection 仍暴露 remove。合法公开 API 因而可以提交 missing/self/cycle parent、绕过 Static 约束，或伪造/删除 renderer 正在消费的 derived rows。**RSH-P0-001、RSH-P0-002 均保持 Open。**

从 Runtime62 到当前源码，真实进展集中在一个窄点：Mobility 的 Static direct-child 检查在 index current 时改为 `HierarchyMutationIndex::children_of`，并有 4,096 entity 行为测试、ignored release benchmark 和 deterministic visit-count 对照；但 index dirty 时仍退回 `stable_entity_ids()` 全扫描，`PERF-MVP-558` 的 managed validation 仍 pending，因此 RSH-P1-043 只能记为 Partial。旧的必然失配源码断言已经改写，但新测试仍检查函数体字符串而不是行为和复杂度，所以 RSH-P1-062 也只能记为 Partial。其余 canonical finding 没有关闭。

当前总账为：**P0 2 Open；P1 62 Open、2 Partial、0 Closed；P2 16 Open、0 Partial、0 Closed；48 项 RSH gate 全部 Fail。** 本文不新增重复 finding，继续以 Runtime62 的 RSH 编号作为唯一 owner。目标仍是 `SceneGraphAuthority + SceneMutationTransaction + AttachmentTransformRule + SceneLifecycleQueue + IncrementalDerivedStatePlanner + SceneParticipationState + MobilityPolicy + SpatialRepresentationRegistry + SceneRenderDelta`，不能继续在五个 World 级 dirty bool 和 whole-world clone/extract 上增加例外。

本轮只做 review 与文档维护，没有修改 production、tests、Cargo、ABI 或 `dev/` 参考源码；也没有运行 Cargo、Editor、RenderDoc、100K/1M benchmark、fault/fuzz/soak 或同语义跨引擎性能实验。因此本文不能证明 Zircon 的性能或表现达到、持平或超过 Unreal；它给出的是实现该目标前仍缺失的合同与证据。

## 2. 审查边界、currentness 与 ownership

### 2.1 Canonical owner 与去重规则

| 领域 | Canonical owner | Runtime110 的作用 | 不重复登记 |
|---|---|---|---|
| Hierarchy/transform/participation combined contract | Runtime62 | 逐项刷新 2/64/16 findings 与 48 gates | RSH-P0/P1/P2、RSH-G 编号 |
| Scene ECS kernel | Runtime60 / Runtime108 | 验证 generic component escape、schedule 与 change tick 后果 | storage/query/event/executor 内核 finding |
| World/Level/persistence | Runtime05/61/109 | 验证 clone、restore、destroy、NodeCache 与 generation 边界 | 通用 World lifecycle/schema finding |
| Render visibility/GPU scene | Runtime09B | Runtime110 只拥有 Scene 上游 instance/bounds/delta 合同 | GPU culling/residency/indirect draw finding |
| Coordinate/large world | Runtime23 | 只追踪 attachment、current/previous transform 和 derived generation | 精度/rebase/坐标空间 finding |
| Terrain/partition spatial hierarchy | Runtime29 | 普通 Scene spatial registry 只提供 adapter 边界 | world partition/terrain finding |
| Editor authoring/picking | Editor03/05/07 | 验证 hierarchy drag/drop、undo、viewport snapshot 接线 | Editor UI/prefab/process 父问题 |

固定 package 形态仍是 `zircon_app`、`zircon_runtime`、`zircon_editor` 三个 public root package，runtime 内部遵循 `core/{runtime,framework,manager,math,resource}` spine。Scene graph、derived state、participation、bounds 和 render delta 必须由 runtime 持有；Editor 只持有 authoring intent、selection、history、overlay 和 presentation，不得建立第二套 hierarchy authority。

### 2.2 当前源码物理冻结

算法：repo-relative path 转 `/` 并小写排序去重；逐文件计算 lowercase SHA-256；以 `path<TAB>hash` 按 LF 连接且末尾无 LF；再对 UTF-8 manifest 计算 SHA-256。

| 冻结组 | 文件 | 行 | 非空行 | bytes | test attrs | ignored | unsafe 行 | Fingerprint |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| Hierarchy authority / public contracts | 34 | 6,598 | 6,058 | 237,914 | 9 | 1 | 0 | `8d3a8e97a9d096faaaf4540c851ba46c3673223cd75214e21bd1044209b6a8e7` |
| Derived state / render handoff | 9 | 4,026 | 3,684 | 143,880 | 10 | 0 | 0 | `d6332badd5dd1d974357e3dbaf0adb060cc59a8156279717485ae9865adb74d4` |
| Editor product consumers | 7 | 1,598 | 1,437 | 51,834 | 1 | 0 | 0 | `f9a2dc95828b13587e66f4214e5def8437bbebe12db745aaca56adbc4ed19b4c` |
| Focused tests | 12 | 3,283 | 2,977 | 125,130 | 85 | 0 | 0 | `de0dd0dfe542c741f2d40014ffd9c09631f26d98fd5da10904fd551d2298b39f` |
| 去重 production | **50** | **12,222** | **11,179** | **433,628** | **20** | **1** | **0** | `b302d016f4baca63adbc028a18bd1b86ac84f26782c631ee3b3c654005659997` |
| 去重 focused set | **62** | **15,505** | **14,156** | **558,758** | **105** | **1** | **0** | `f67ed5c8e81429cd0490a316b997116cb0812c4a9b761f488ceda4b715608b5c` |
| 五引擎 20 个显式参考文件 | **20** | **40,518** | **34,525** | **1,599,782** | **49** | **0** | **46** | `1c7a756a3b74d93fc56ef936ccf42e7ec0024bd219372b54d8d4e402e0ae35f1` |

冻结对应 HEAD `bee4c707b714738346b49bba15c59468b8bd9b39`、baseline epoch 339。冻结时共享工作区有 239 个 status entries；本 focused set 有 5 个 working-tree 修改：`frame_performance_diagnostics.rs`、`derived_state.rs`、`hierarchy.rs`、`hierarchy_behavior.rs` 与 `ecs_performance_acceptance.rs`。本文绑定这些文件的实际 working-copy 内容，不归因、不回退，也不把并行会话的未集成内容冒充 accepted baseline；实施前必须重算 fingerprint。

相对 Runtime62 baseline `bea1acf91b909525ab1759e2c800858b0eda6528`，本冻结集有 11 个文件变化、570 additions、59 deletions。主要增量是 hierarchy direct-child index 使用、derived-state deterministic work counters、1K/100K full-rebuild characterization 和 viewport highlight owner 调整；这些变化没有建立 protected component capability、incremental dirty roots、bounds 或 render delta。

### 2.3 复核方法

1. 继承 Runtime62 对原 49 文件的逐文件结论，并按当前目录补入 frame diagnostics、transform validation、render DTO、Editor controller，focused set 扩展为 62 文件。
2. 对 Runtime62 baseline 到当前 working copy 的 11 个相关变化逐文件核对，区分真实语义改进、格式变化、characterization 和尚未验证的 performance candidate。
3. 沿 generic typed API、reflection、structured hierarchy API、deferred batch、Editor command、gateway、viewport clone/extract、visibility input 七条真实路径检查 authority、commit、generation 与 consumer freshness。
4. 对 activation/world matrix/NodeCache 的 visited/written rows、allocation、dirty granularity和fallback路径做正反证；不以 API 名称或 source-shape test 代替复杂度证明。
5. 展开 20 个参考文件，以 Unreal 为系统主参考，Bevy/Fyrox 为 Rust data-flow 校验，Godot为reparent/visibility语义校验，Unity Graphics仅为persistent instance/bounds/culling handoff校验。

## 3. 当前真实链路

```text
Editor / Script / Runtime caller
  -> World::insert/get_mut/remove<T>               [generic protected-state escape]
  -> set_parent_checked / update_transform         [structured but rule/receipt incomplete]
  -> DerivedStateDirty { 5 x world-level bool }
  -> HierarchyValidity                             [full parent snapshot + repeated chains]
  -> ActiveHierarchy / WorldTransform              [all roots, whole forest]
  -> NodeCache                                     [clear + clone all wide SceneNode rows]
  -> RenderExtractPrepare
  -> build_viewport_render_packet(&self)
       -> World::clone -> flush clone -> geometry
       -> original Scene::nodes/world_transform -> editor gizmos
  -> VisibilityInput
       -> full mesh/sprite/particle membership lists
       -> stable key + mobility + layer only
       -> no bounds/generation/delta/removal/multiview truth
```

`HierarchyMutationIndex` 目前维护 stable root/child order，并能让 subtree/direct-child lookup 在 index current 时避免重建 traversal；但 `DerivedStateDirty` 仍只有 hierarchy/active/transforms/node_cache/render_extract 五个 bool。任一 hierarchy 改动会把五域全部置 dirty；单叶 transform 改动仍遍历全部 roots、重写全部 WorldMatrix，再 clear/rebuild 全部 NodeCache。现有 deterministic tests 明确断言 1,000 节点下单叶 transform 写 1,000 个 world matrix 和 1,000 个 node-cache row，这只是 current baseline，不是目标预算。

## 4. P0 当前证据

### RSH-P0-001：Hierarchy / LocalTransform / Mobility 没有受保护写 authority

状态：**Open**。

- `World::insert<T>` 对任意 `Component` 公开；`validate_fixed_component` 只识别 LocalTransform 与 Mobility，不校验 Hierarchy parent existence/self/cycle，也没有 transaction capability。
- `World::get_mut<T>` 先标记 dirty/scene binding，再直接返回 `&mut T`；调用者可在返回后写入 cycle、non-finite transform 或非法 mobility 关系，框架无法 preflight/rollback。
- `World::remove<T>` 可以移除 required Hierarchy/LocalTransform/Mobility；fallback getter 又把缺失 LocalTransform 当 identity、缺失 ActiveSelf 当 true，掩盖 schema 损坏。
- Hierarchy reflection 的 field write 会路由 `set_parent_checked`，但 component remove 仍调用 generic `component_support::remove::<Hierarchy>`；受保护合同并未覆盖全部入口。
- `is_descendant` 沿 parent 链无 visited/depth budget；一旦 raw API 写入 cycle，checked reparent 的 cycle guard 本身也可能不终止。

关闭条件：generic/bundle/deferred/reflection/dynamic/script 所有入口在 commit 前拒绝 protected component raw write/remove/get_mut；只有 `SceneMutationTransaction` 持有内部 permit，并以 forest、mobility、transform 和 generation 守恒测试证明。

### RSH-P0-002：WorldMatrix / ActiveInHierarchy 可被伪造或删除

状态：**Open**。

- 两个 derived component 仍作为普通 registered component 存入 canonical ECS storage；public generic API 可 insert/get_mut/remove。
- `replace_derived_component` 只是 planner 内部方便函数，并没有阻止外部走公共 typed API。
- ActiveInHierarchy reflection 虽拒绝 field write，却仍暴露 remove；其 `contains` 在 row 缺失时因 entity 存在返回 true，contains/read/remove 语义不一致。
- renderer 的 mesh/sprite/camera/light路径直接读取 `active_in_hierarchy` 和 `world_transform`；伪造或删除 row 会改变参与状态或回退 default transform，而不是 fail closed。
- dirty-domain bool 只表达“是否计划跑 stage”，没有 source/derived generation pair；无法证明 consumer 读到同一 committed generation。

关闭条件：derived rows 对外只读且不可移除；缺失/陈旧 row 返回 typed stale disposition并触发受控 rebuild/quarantine；render、inspection与runtime query必须携同一 committed generation。

## 5. Runtime62 P1 状态逐项刷新

状态计数：**Partial 2（043、062）；Open 62；Closed 0**。

| ID | 状态 | 当前源码复核 |
|---|---|---|
| RSH-P1-001 | Open | `set_parent_checked(child,parent)` 仍只改 parent并保留local transform；无KeepWorld规则。 |
| RSH-P1-002 | Open | 无translation/rotation/scale逐轴rule、absolute axis、socket/bone attachment。 |
| RSH-P1-003 | Open | 返回值仍是`bool`，无old/new parent、rule、affected set、generation与cost receipt。 |
| RSH-P1-004 | Open | 成功reparent前后仍各记录一次相同`WorldFact::Reparented`。 |
| RSH-P1-005 | Open | fact只含entity/new_parent，缺old parent、old/new ancestry、sibling placement与rule。 |
| RSH-P1-006 | Open | 没有runtime atomic batch reparent/reorder；detached batch只解决subtree detach/restore。 |
| RSH-P1-007 | Open | topology index只有root/children稳定顺序，没有显式sibling order mutation合同。 |
| RSH-P1-008 | Open | generic insert/bundle仍可先发布非法parent，再依赖后续validity修复。 |
| RSH-P1-009 | Open | `rebuild_hierarchy_validity`仍把missing/self/cycle静默改为None，无typed quarantine/diagnostic receipt。 |
| RSH-P1-010 | Open | 普通`remove_entity`把direct children改成root并保留local transform，world pose改变。 |
| RSH-P1-011 | Open | orphan只写Hierarchy与dirty；没有old/new topology、pose与reason完整事实。 |
| RSH-P1-012 | Open | `remove_entity` orphan 与 `remove_entity_recursive` destroy-subtree是两套隐式策略。 |
| RSH-P1-013 | Open | remove仍立即擦除storage/identity，没有deferred destruction fence或tombstone。 |
| RSH-P1-014 | Open | lifecycle只有component remove/despawn触发，缺scene attachment/destruction pre/post hooks。 |
| RSH-P1-015 | Open | Static编辑、runtime mutation、restore/import没有phase capability区分。 |
| RSH-P1-016 | Open | attachment没有physics weld/joint ownership、teleport或collision update policy。 |
| RSH-P1-017 | Open | parent chain、depth、child count和batch size均无显式budget。 |
| RSH-P1-018 | Open | `is_descendant`等walker仍假设合法图；raw cycle可导致无界前进。 |
| RSH-P1-019 | Open | dirty state仍是五个World级bool，没有entity/subtree/reason/generation。 |
| RSH-P1-020 | Open | checked单边reparent仍触发full hierarchy validity与全部后续domain。 |
| RSH-P1-021 | Open | validity每次建立全entity `HashMap<EntityId, Option<EntityId>>` parent snapshot。 |
| RSH-P1-022 | Open | 每个entity重复走parent chain，deep chain最坏O(N²)。 |
| RSH-P1-023 | Open | 单叶transform仍从全部root传播并重写全WorldMatrix。 |
| RSH-P1-024 | Open | 单叶ActiveSelf仍从全部root传播并重写全ActiveInHierarchy。 |
| RSH-P1-025 | Open | propagation无value equality剪枝，unchanged derived row仍被replace。 |
| RSH-P1-026 | Open | `refresh_node_cache`仍clear/reserve并clone全World wide SceneNode字段。 |
| RSH-P1-027 | Open | `project_node_for_read`与`refresh_node_cache`继续维护重复字段名单。 |
| RSH-P1-028 | Open | dirty world transform read每次分配lineage Vec与seen HashSet并走祖先链。 |
| RSH-P1-029 | Open | dirty active read每次分配HashSet并走祖先链。 |
| RSH-P1-030 | Open | `nodes()`返回retained stale slice，`find_node/node_records`又可走fresh projection，读语义分裂。 |
| RSH-P1-031 | Open | NodeCache/inspection snapshot没有source generation/currentness token。 |
| RSH-P1-032 | Open | stage中间可出现hierarchy、active、matrix、node cache不同代，consumer无法比较。 |
| RSH-P1-033 | Open | 无dirty-root collapse；ancestor/descendant重复dirty不能合并成minimal frontier。 |
| RSH-P1-034 | Open | 无static subtree skip、parallel propagation、serial/parallel strategy与选择receipt。 |
| RSH-P1-035 | Open | ActiveSelf/InHierarchy仍被当成统一enable，未分process/render/physics/audio/navigation/editor participation。 |
| RSH-P1-036 | Open | visibility没有Inherited/Hidden/Visible authored override三态。 |
| RSH-P1-037 | Open | 无HiddenInGame、EditorHidden、TemporaryHidden等独立domain。 |
| RSH-P1-038 | Open | 无pause/process/tick mode及祖先继承规则。 |
| RSH-P1-039 | Open | required component缺失仍由getter默认identity/true/dynamic，未在spawn/import/restore边界补全或拒绝。 |
| RSH-P1-040 | Open | activation没有subtree frontier和unchanged short-circuit，仍是whole forest rebuild。 |
| RSH-P1-041 | Open | Mobility仍只有Dynamic/Static，没有Movable/Stationary或policy/build-state分层。 |
| RSH-P1-042 | Open | Static在Editor authoring期也拒绝transform/reparent，没有construction/runtime phase。 |
| RSH-P1-043 | Partial | index current时按direct children检查Static child；dirty时仍全World scan，managed benchmark ticket未完成。 |
| RSH-P1-044 | Open | mobility transition只有Reject，没有Cascade/Promote/Rebuild用户选择与affected receipt。 |
| RSH-P1-045 | Open | Static没有render/physics/navigation/lighting registration participant与generation。 |
| RSH-P1-046 | Open | transform mutation无teleport、previous transform、velocity/history invalidation语义。 |
| RSH-P1-047 | Open | Scene没有canonical local/world bounds component/provider registry。 |
| RSH-P1-048 | Open | `VisibilityRenderableInput`仍只有entity、stable key、mobility、layer；无bounds/generation/removal。 |
| RSH-P1-049 | Open | Scene extract先按一个camera layer过滤mesh/sprite/particle，不能作为multi-view/shadow共享scene truth。 |
| RSH-P1-050 | Open | 每帧输出完整Vec，没有create/update/remove delta、tombstone或resync receipt。 |
| RSH-P1-051 | Open | 无visibility class、always visible、no-cull、occluder/occludee policy。 |
| RSH-P1-052 | Open | 无range、LOD、cell、distance band与view family组合输入。 |
| RSH-P1-053 | Open | renderable/static/dynamic向量加entries重复identity，并sort/BTreeSet/dedup重建。 |
| RSH-P1-054 | Open | Particle bounds在`ParticleExtract`中另存，visibility entry未按stable instance绑定对应bounds。 |
| RSH-P1-055 | Open | 缺visibility/bounds reject、stale、overflow、provider-unload reason diagnostics。 |
| RSH-P1-056 | Open | 无Scene侧spatial representation owner；renderer仍只能消费临时全量列表。 |
| RSH-P1-057 | Open | Editor reparent只改`NodeEditState.parent`，默认改变world pose且UI不表达rule。 |
| RSH-P1-058 | Open | NodeEditState只有name/parent/local transform，无法保存sibling/rule/socket/absolute/mobility inverse。 |
| RSH-P1-059 | Open | viewport geometry仍在World clone上flush；gizmo继续遍历原Scene `nodes()`并另读fresh active/world。 |
| RSH-P1-060 | Open | InProcessGateway `with_world_mut`仍只返回`Result<()>`，无commit generation/affected domains/freshness fence。 |
| RSH-P1-061 | Open | create/delete undo仍依赖捕获的固定NodeRecord字段；扩展组件和执行后新增child的守恒未证明。 |
| RSH-P1-062 | Partial | 旧必然失配断言已改成indexed helper断言，但仍解析源码字符串，未转成行为/复杂度oracle。 |
| RSH-P1-063 | Open | `projected_reads.rs`仍大量锁定变量、Option分支和禁止字符串，而非generation/allocation/visited rows合同。 |
| RSH-P1-064 | Open | 仍缺raw-illegal-state、keep-world、orphan policy、fact coverage、bounds delta、mixed-generation、deep/fuzz/model矩阵。 |

## 6. Runtime62 P2 状态逐项刷新

状态计数：**Open 16；Partial 0；Closed 0**。

| ID | 状态 | 当前源码复核 |
|---|---|---|
| RSH-P2-001 | Open | 公开`WorldTransform`同义类型仍无清晰production owner，实际派生事实是WorldMatrix。 |
| RSH-P2-002 | Open | `node_records()`命名仍返回`Vec<SceneNode>`，inspection DTO边界含混。 |
| RSH-P2-003 | Open | 重复Reparented虽可被coalesce，仍产生重复fact/ancestry工作。 |
| RSH-P2-004 | Open | authorable/derived component公开字段与raw generic API仍让DTO和authority混同；index的`PartialEq`恒true也隐藏cache差异。 |
| RSH-P2-005 | Open | WorldMatrix仍可Serialize/Deserialize，与runtime-derived-only语义冲突。 |
| RSH-P2-006 | Open | ActiveInHierarchy reflection contains/read/remove仍不一致。 |
| RSH-P2-007 | Open | VisibilityInput仍重复四份identity/classification视图。 |
| RSH-P2-008 | Open | stable instance key仍是裸`u64`，无owner/type/generation资格。 |
| RSH-P2-009 | Open | Editor继续把SceneError压成字符串，attachment失败缺typed字段。 |
| RSH-P2-010 | Open | consumers仍围绕wide`nodes()`/SceneNode clone构建，不是column/delta artifact。 |
| RSH-P2-011 | Open | WorldTransform系统/类型命名仍不能清楚表达“传播WorldMatrix”。 |
| RSH-P2-012 | Open | 新增12个deterministic累计counter，但仍无reason、max depth、dirty roots、stage latency分布与sample引用。 |
| RSH-P2-013 | Open | 100K full-rebuild test仍在普通unit lane；mobility latency gate则是inline ignored test。 |
| RSH-P2-014 | Open | required component/default policy仍散落getter、bootstrap和fixed validation。 |
| RSH-P2-015 | Open | fact、component invalidation、inspection与scene-binding generation仍各自推导topology变化。 |
| RSH-P2-016 | Open | tests继续断言“单叶变化全量写1K rows”为current baseline，没有required ceiling字段。 |

## 7. 五套参考实现对照

| 参考 | 本轮读取到的工程做法 | Zircon 当前差距 | 采用边界 |
|---|---|---|---|
| Unreal SceneComponent / Actor | attachment有KeepRelative/KeepWorld/Snap与逐轴rule、socket、absolute transform axis；component-to-world、child propagation、mobility、visibility、bounds、render transform/state dirty和destroy lifecycle分层 | Zircon只有parent optional值和local transform；无rule/receipt/socket/absolute axis、bounds/render dirty/lifecycle transaction | 作为system-scale主参考；不照搬UObject/Actor ownership和宏体系 |
| Bevy hierarchy / transform / visibility | `ChildOf`/`Children`由relationship hooks维护；command提供`set_parent_in_place`；transform系统利用Changed/Removed和tree change；Visibility/InheritedVisibility/ViewVisibility及Aabb分责 | Zircon generic mutation可破坏双向关系；derived dirty不是entity-level change set；participation与bounds缺owner | 采用Rust ECS data-flow、hook与change-detection思想；不把Bevy API形状当最终产品合同 |
| Fyrox graph / node / editor commands | Graph集中link/unlink与`link_nodes_keep_global_transform`；node缓存local/global transform、visibility等层级数据；Editor command保存inverse并通过Graph authority执行 | Zircon Editor只保存parent/local transform，runtime无keep-global transaction与同代graph snapshot | 用于Rust graph authority和Editor/runtime分责校验；不恢复面向对象node继承树 |
| Godot Node / Node3D / VisualInstance3D | `reparent(..., keep_global_transform)`、parented/unparented notification、top-level/global transform dirty通知、visible-in-tree与VisualInstance AABB/scenario owner分层 | Zircon orphan/reparent默认改变world pose，无notification state machine；Active与render visibility混同；无AABB/scenario adapter | 用于产品语义和notification/bounds边界校验；不复制SceneTree全局singleton结构 |
| Unity Graphics GPUDriven | persistent instance handle/archetype allocator、CPU/GPU instance data、bounds、motion/history、allocation/free/update与culling job分层 | Zircon每帧重建full visibility Vec；没有stable lifetime、bounds update/remove、previous transform或resync | 仅作为SceneRenderDelta到persistent renderer的交接参考；不让Graphics package反向拥有Scene topology |

共同结论不是“把五套API都实现一遍”，而是冻结四个必须成立的结构事实：mutation只有一个 authority；attachment明确决定空间语义；derived/spatial工作量与 affected set 相关；renderer消费 generation-qualified persistent delta，而不是每帧从临时全量列表猜测scene变化。

## 8. 目标架构与不变量

```text
Editor / Script / Runtime intents
  -> SceneMutationTransaction
       authority + phase capability + preflight + atomic commit
       AttachmentTransformRule + DestructionPolicy + typed receipt
  -> SceneGraphAuthority
       Parent + Children + sibling order + topology generation
       lifecycle/tombstone queue + bounded diagnostic walker
  -> IncrementalDerivedStatePlanner
       minimal dirty roots + subtree intervals + reason/generation
       world transform + participation + previous transform
       equality pruning + static skip + parallel chunks
  -> SpatialRepresentationRegistry
       local/world bounds + visibility/range/LOD/cell policy
  -> SceneRenderDelta
       create/update/remove + stable instance + bounds + motion + layer
       source generation + loss/resync receipt
  -> Runtime09B Persistent Render Scene

SceneInspectionSnapshot
  <- same committed topology/derived generation
  -> hierarchy panel / viewport gizmo / selection / automation
```

必须冻结的不变量：

1. live Scene 永远是 forest；missing/self/cycle parent 不能进入 committed generation。
2. Parent/Children/sibling order 由一个 authority 原子维护，所有入口共享同一 preflight/commit。
3. WorldMatrix/effective participation 只能由 derived planner 写，外部 API 没有写、删或 raw mutable access。
4. attachment/destruction失败不改变 topology、transform、fact、generation 或 downstream registration。
5. 单节点 transform/active 工作量与 affected subtree 相关，不与 World 总 entity 数相关。
6. inspection、runtime query、spatial state 与 render delta引用可比较的 committed generation。
7. bounds 缺失、陈旧、provider unload或budget overflow必须有typed disposition。
8. Static是render/physics/navigation/lighting共同验证的registration承诺，不只是bucket标签。
9. Editor和runtime只共享核心transaction，不复制第二套层级、pose或inverse规则。
10. hard cutover后删除raw hierarchy/derived escape和clone-refresh旁路，不保留compat facade。

## 9. 重构里程碑

| Milestone | 必做内容 | 退出条件 |
|---|---|---|
| M62-0 Characterization | 建generic/reflection非法状态RED测试；覆盖cycle hang、Static绕过、derived伪造/删除、renderer错误参与；删除source-shape oracle | 两项P0都能被稳定测试复现，所有写入口有inventory |
| M62-1 Protected authority | internal write permit；generic/bundle/deferred/reflection拒绝protected/derived raw mutation；迁移caller后硬删escape | RSH-G01-G05通过，无compat shim |
| M62-2 Topology transaction | Parent/Children/sibling统一index；KeepWorld/Relative/Snap；batch reparent；orphan/destroy policy；deferred lifecycle与receipt | forest、pose、fact、inverse在成功/失败路径守恒 |
| M62-3 Incremental derived | dirty roots/subtree intervals/generation/reason；equality pruning；bounded scratch；static skip/parallel；删除wide runtime NodeCache | leaf mutation访问/写入只覆盖affected set，inspection另有sealed artifact |
| M62-4 Participation/mobility | 拆process/render/physics/audio/navigation/editor状态；visibility三态；authoring/runtime mobility policy；downstream participant | policy truth table、undo、registration generation通过 |
| M62-5 Spatial/render delta | 内建与provider bounds；增量world bounds；create/update/remove/motion/LOD/range/layer/cell delta；full resync | Runtime09B持久instance的identity/count/bounds守恒 |
| M62-6 Editor product | drag/drop显式rule/sibling；journal存inverse receipt；geometry/gizmo/selection/picking消费同代snapshot；删除World clone刷新 | Editor真实workflow与dynamic gateway同合同 |
| M62-7 Qualification | deep/wide/random model、cycle fuzz、fault、1/1K/100K/1M scale、multi-view/shadow/removal E2E、跨引擎同场景原始数据 | 48 gates全Pass且证据可复现 |

实现顺序不能调换：M62-0/1 的 authority P0 必须先于局部缓存、GPU visibility或Editor体验扩展；否则新优化只会加速不可信状态的传播。

## 10. 48 项门禁当前状态

| Gate组 | 当前状态 | 失败原因摘要 |
|---|---|---|
| RSH-G01-G12 Authority / transaction | **12 Fail** | raw generic/reflection mutation仍存在；无forest commit fence、rule/receipt、双向topology、explicit destruction与bounded corrupt-graph walker。 |
| RSH-G13-G24 Derived / performance | **12 Fail** | whole-world bool dirty、snapshot/ancestry/full propagation/full NodeCache仍存在；无minimal roots、O(1)/zero-allocation read、parallel oracle与1M ceiling。 |
| RSH-G25-G36 Participation / mobility / spatial | **12 Fail** | Active/Mobility语义过窄；无domain participation、三态visibility、bounds provider、delta/resync、multi-view shared truth与typed stale disposition。 |
| RSH-G37-G48 Product / tests / qualification | **12 Fail** | Editor无rule/inverse/generation snapshot；测试仍锁源码形状；缺fuzz/fault/soak、persistent renderer conservation、双平台和同语义跨引擎数据。 |

`docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md` 中 RSH-G01-G48 的逐项定义继续有效。本轮没有足够证据把任何 gate 标为 Partial 或 Pass。`docs/plans/optimize/zircon_runtime/62/2026-08-20-indexed-mobility-child-validation.md` 只证明 direct-child candidate visit 数从 524,288 降至 512；managed release latency验证仍 pending，且该局部结果不能关闭RSH-G24。

## 11. 禁止的临时修补

- 禁止只在`validate_fixed_component`增加Hierarchy分支，却继续允许`get_mut/remove`绕过authority。
- 禁止保留raw Hierarchy mutation并以“下一stage会静默修复”为合法性证明。
- 禁止继续给`DerivedStateDirty`增加World级bool模拟增量更新。
- 禁止把KeepWorld做成Editor侧松散的“读world、改parent、写local”三步命令。
- 禁止让Renderer反向扫描Scene组件计算bounds或猜remove；Scene必须发布stable spatial delta。
- 禁止把ActiveSelf扩展成更多含义不明的全局enable位。
- 禁止继续通过`World::clone`刷新derived state，或让geometry与gizmo消费不同World实例。
- 禁止以source-shape test、API存在、compile-only、ignored benchmark或100K全量重建完成代替复杂度与产品验收。
- 禁止为旧raw mutation/old hierarchy path保留compat re-export；迁移完成后必须硬切删除。

## 12. 当前状态与下一执行切片

- review状态：current-source refresh complete；implementation状态：pending。
- canonical总账：P0=2 Open；P1=62 Open/2 Partial/0 Closed；P2=16 Open/0 Partial/0 Closed；RSH gates=48 Fail。
- 本轮只新增review与索引记录，没有修改production、tests、Cargo、ABI或reference source。
- MVP 00 baseline尚未accepted，且本轮是docs-only review，因此没有运行Cargo、Editor、benchmark或动态产品验证。
- Runtime62 child validation仍为implementation complete / managed validation pending，不能宣称latency已达标。
- 首个实现切片固定为M62-0/M62-1：建立非法状态RED矩阵并关闭protected/derived authority两项P0；在此之前不得先做局部GPU Scene或Editor workaround。
