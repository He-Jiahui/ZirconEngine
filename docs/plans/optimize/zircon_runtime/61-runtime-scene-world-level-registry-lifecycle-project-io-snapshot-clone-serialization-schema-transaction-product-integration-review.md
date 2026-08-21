---
title: Runtime Scene World、Level、Registry、Lifecycle、Project I/O、Snapshot/Clone、Serialization Schema、Transaction 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime61
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/level_system
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/scene/runtime_level_traits.rs
  - zircon_runtime/src/scene/module
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/transaction.rs
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/scene/serializer/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/document/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/capture.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction.rs
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/assets/project_document/codec.rs
  - zircon_runtime/src/asset/assets/project_document/scene.rs
  - zircon_runtime/src/core/framework/scene
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_editor/src/core/document/scene_route.rs
  - zircon_editor/src/core/editing/authoring_world.rs
  - zircon_editor/src/core/play/snapshot/source.rs
  - zircon_editor/src/core/play/snapshot/store.rs
  - zircon_editor/src/core/project/scene_document.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/snapshot.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_load.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_save.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_project.rs
  - zircon_editor/src/ui/workbench/state/editor_state_play_mode.rs
tests:
  - zircon_runtime/src/scene/tests/world_basics/world_state.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/tests.rs
  - zircon_runtime/src/asset/tests/assets/scene/management.rs
  - zircon_runtime/src/asset/tests/project/example_vampire/manifest_scene_imports.rs
  - zircon_editor/src/core/play/snapshot/tests.rs
  - examples/vampire/assets/scenes/main.scene.toml
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/39-prefab-archetype-prototype-class-default-instance-override-runtime-instantiation-propagation-hot-reload-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/40-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/52-runtime-dynamic-scene-session-archive-slot-capture-restore-path-merge-retention-durability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/53-runtime-dynamic-scene-asset-reload-event-generation-reconciliation-stage-apply-instance-replacement-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/Engine.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/World.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/Level.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/LevelStreaming.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/GameInstance.h
  - dev/bevy/crates/bevy_ecs/src/entity/map_entities.rs
  - dev/bevy/crates/bevy_ecs/src/reflect/map_entities.rs
  - dev/bevy/crates/bevy_world_serialization/src/dynamic_world.rs
  - dev/bevy/crates/bevy_world_serialization/src/dynamic_world_builder.rs
  - dev/bevy/crates/bevy_world_serialization/src/serde.rs
  - dev/bevy/crates/bevy_world_serialization/src/world_asset.rs
  - dev/bevy/crates/bevy_world_serialization/src/world_asset_spawner.rs
  - dev/bevy/crates/bevy_world_serialization/src/world_filter.rs
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/graph/mod.rs
  - dev/Fyrox/editor/src/scene/mod.rs
  - dev/godot/scene/main/scene_tree.h
  - dev/godot/scene/main/scene_tree.cpp
  - dev/godot/scene/main/node.h
  - dev/godot/scene/main/node.cpp
  - dev/godot/scene/resources/packed_scene.h
  - dev/godot/scene/resources/packed_scene.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/ReloadAttribute.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/ResourceReloader.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 61 · Runtime Scene World、Level、Registry、Lifecycle、Project I/O、Snapshot/Clone、Serialization Schema、Transaction 与 Product Integration 工程化差距

## 1. 结论

当前 Zircon 已经存在可复用的 World、LevelSystem、LevelManager、SceneAsset、DynamicScene、项目引用解析、bounded keyed I/O lane、原子文件替换和 Editor Play snapshot 链路，不能把它描述成完全空壳。尤其应保留 World replacement epoch、DynamicScene compile/preflight/commit generation fencing、严格 asset reference resolution、physics/animation 的短锁 `Arc` frame snapshot、同 scene key 的 queued save superseding，以及 Editor 新场景 staging/catalog rollback。

但“有类型和 API”不等于形成工程级 World/Level 产品合同。本轮沿 Editor Save、Enter/Exit Play、runtime play snapshot load、canonical `.scene.toml` load/save 和 LevelManager save 五条真实调用链追踪后，确认五项直接破坏用户数据或运行世界事实的 P0：Editor Save 在成功返回并清除 dirty state 前先做有损 `World::clone`；退出 Play 用同一有损 clone 覆盖 authoring World；版本化 Play snapshot 被追加进带 Camera/DirectionalLight/Cube 的 `World::new()`；canonical SceneAsset loader/saver忽略真实产品 terrain 以及 schema 已声明的 tilemap/prefab；first-party Sprite2D/Mesh2D 没有进入 canonical document。它们不是“未来规模不足”，而是当前合法数据在 save/reopen 或 play/exit 后已经可能丢失、重复或改变。

World/Level 控制面也仍停留在最小包装：`DefaultLevelManager` 只有只增不减的 `HashMap`；公开 contract 没有 enumerate/current/activate/unload/travel/streaming/cancel/progress；`LevelLifecycleState` 只有 Loaded/Unloaded 且生产 tick 不读取；World replacement 只手工清理少数组件状态；多个独立 Mutex 不能构成一致生命周期快照；poison 被无条件恢复为“继续运行”。相比 Unreal 的 WorldContext/World/Level/LevelStreaming 分层、Fyrox 的 generational SceneContainer 与 delayed destruction、Godot 的 current/pending/previous scene切换，当前结构不能承载多 PIE、长时 server、异步 travel、插件 participant 和开放世界。

本轮登记 **5项P0、60项P1、14项P2和45项验收门禁**。目标不是继续给 `World::clone` 增加字段白名单，而是建立 `WorldContextRegistry + LevelInstanceRegistry + WorldLifecycleCoordinator + AuthoringSceneDocument + SceneSchemaRegistry + SnapshotCompiler + ScenePersistenceService + WorldReplacementTransaction`。本轮只做静态 review 与计划记录，没有修改 production、tests、Cargo、ABI 或参考源码；没有运行 Cargo、Editor、Vampire、fault injection、100K entity、多 World/PIE、断电恢复或 benchmark，因此不能据此宣称性能达到或超过 Unreal。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 冻结组 | 文件 | 行 | bytes |
|---|---:|---:|---:|
| Level/module lifecycle | 16 | 2,664 | 95,965 |
| World/project document | 31 | 7,483 | 286,154 |
| Product/dynamic consumers | 16 | 3,712 | 134,191 |
| Public scene contracts | 11 | 374 | 10,299 |
| 去重合计 | **74** | **14,233** | **526,609** |

production fingerprint 为 SHA-256 `23aa86d8421657bf4df5b4882743b5aab06d65b686ace6d45278782840d1ae34`。算法将74个相对路径转为`/`、排序去重，以`path|lowercase per-file SHA-256`逐行编码，LF连接且末尾不追加LF，再对UTF-8 payload计算SHA-256。

冻结时74个文件中有6个 working-tree 修改：`editor_project_document_save.rs`、`world_driver.rs`、`derived_state.rs`、`project_io/document.rs`、`project_io/scene_asset.rs`、`world.rs`。本报告审查的是这些文件在 2026-08-20 工作区的实际内容，不把它们归因于本会话，也不回退用户或其他会话改动。提交前必须重新计算 fingerprint 并逐条复核 P0。

### 2.2 本轮拥有与明确不拥有

- Runtime61拥有 World/Level container、registry、lifecycle、canonical scene project I/O、authoring snapshot/clone、scene schema/transaction和Save/Play产品接线。
- Runtime05继续拥有通用 World clone/serde primitive、hierarchy derived state、render extract和scene partition父架构；本篇新增的是可达产品链与具体document字段损失证据。
- Runtime24拥有全仓 identity/generation/owner/epoch规范；本篇只说明 WorldHandle/scene entity identity如何阻断World lifecycle与I/O。
- Runtime39拥有 prefab/archetype实例、override和传播；本篇只登记 canonical SceneAsset 声明 `prefab_instance` 却不加载/保存。
- Runtime40拥有 SaveGame/Checkpoint 产品；Runtime52拥有 session archive/durability；Runtime53拥有动态 scene asset reload。Runtime61不把 authoring project save 冒充 SaveGame，也不重建第二套 archive/reload服务。
- Runtime60拥有 ECS storage/query/schedule/event kernel。本篇不重复其清空 Clone、恒真 Eq、schedule panic等底层finding，只追踪 World snapshot和replacement对产品的影响。
- 用户已要求暂停 tooling 优化；本篇不新增 tooling milestone、脚本迁移或 Python 工具改造。

### 2.3 当前真实产品链

```text
Editor SaveProject
  -> EditorState::project_scene()
  -> EditorAuthoringWorld::try_snapshot()
  -> World::clone()                         [有损]
  -> EditorProjectDocument::save_to_project
  -> World::save_scene_to_project
  -> World::to_scene_asset                  [硬编码字段]
  -> project document TOML + atomic_write
  -> success -> clear dirty history

Editor Enter/Exit Play
  -> state.world.try_snapshot()             [有损]
  -> EditorPlaySession.scene
  -> PlaySceneSource::from_world
  -> DynamicScene JSON
  -> runtime load_play_scene_level
  -> create_level(World::new())              [预置3实体]
  -> DynamicScene::spawn_into                [append/remap]
  -> exit -> authoring world = session.scene [有损覆盖]

Runtime LevelManager save
  -> level.snapshot()                       [World::clone]
  -> whole SceneAsset/document materialize
  -> bounded keyed I/O admission
  -> atomic_write
```

## 3. 当前应保留的能力

1. `LevelSystem`在World replacement时具有epoch检查，DynamicScene commit还绑定World/schema/component generations。
2. physics/animation frame state通过短锁发布`Arc` snapshot，避免读者长期持有runtime state锁。
3. DynamicScene将compile、preflight、staging和commit分开，失败不会逐实体泄漏半发布World。
4. project resource reference同时使用stable id与locator，并对不存在的非builtin引用fail closed。
5. `SceneArtifactIo`有entry/byte admission、keyed superseding和terminal ticket，不是无界fire-and-forget线程。
6. `atomic_write`和Play snapshot临时文件+`sync_all`+rename提供了单文件局部原子基础。
7. Editor新场景先写staging，再catalog/install，失败时有显式rollback和cleanup错误传播。
8. Scene reference migration能先保留TOML中的`_rest`并重写旧引用形状；这可作为真正document migration的起点。

## 4. P0：当前合法工作流会破坏数据或World事实

### RWL-P0-001：SaveProject成功并清dirty，却丢失任意typed/dynamic component

`EditorAuthoringWorld::try_snapshot`在`authoring_world.rs:54-55`直接调用`Clone::clone`。`World::clone`在`world.rs:143-221`只重建硬编码NodeRecord组件组，`component_storage`先置空；canonical `to_scene_asset`在`scene_asset.rs:359-585`也只输出固定字段，dynamic components仅通过`script_bindings_for_record`读取`script.bindings`。菜单保存成功后会继续执行成功分支并清除dirty历史。

因此 gameplay、navigation、particle、HUD、插件或任何后来注册的typed/dynamic component可在Save成功后没有错误地消失，用户还会被告知文档已保存。Runtime05拥有删除`Clone for World`的底层重构；Runtime61拥有Save入口必须改为authoring snapshot compiler且禁止在unsupported participant存在时清dirty。

### RWL-P0-002：退出Play会以有损快照覆盖作者World

`enter_play_mode`把`state.world.try_snapshot()`存入`EditorPlaySession.scene`；`exit_play_mode`执行`*scene = session.scene`。这不是仅影响临时runtime fork：任何未进入硬编码Clone投影的作者组件会在一次“进入Play再退出”后从内存作者World消失，即使用户没有保存。必须用不可变authoring generation + runtime fork，退出时销毁fork，不得回写有损副本。

### RWL-P0-003：Play snapshot被追加到带默认实体的World，运行事实被污染

`World::new()`在`bootstrap.rs:72-81`创建Camera、DirectionalLight和Cube；`load_play_scene_level`在`dynamic_api/session/project.rs:261-275`先以`World::new()`创建Level，再`scene.spawn_into(world)`。DynamicScene的`first_available_entity_id`遇到ID冲突会顺延重映射，因此Play World同时保留默认三实体并改变快照实体ID。正确行为是对`World::empty()`或专用RuntimeWorldBuilder执行exact restore，并在commit前验证entity/resource conservation。

### RWL-P0-004：canonical SceneAsset声明terrain/tilemap/prefab，loader/saver却双向丢弃

`SceneEntityAsset`公开`terrain`、`tilemap`、`prefab_instance`，asset管理和引用统计也识别它们；但`from_scene_asset`从未消费这三项，`to_scene_asset`在`scene_asset.rs:581-583`无条件写`None`。这不是虚构fixture：`examples/vampire/assets/scenes/main.scene.toml:294-297`已有真实terrain引用，asset测试还明确要求它存在。Runtime load不会安装terrain语义，Editor reopen/save会永久删除该字段。三项能力必须在缺少runtime provider时fail closed或opaque-preserve，不能接受后静默丢弃。

### RWL-P0-005：first-party Sprite2D/Mesh2D无法进入canonical工程文档

`NodeRecord`和World read path支持`Sprite2dComponent`/`Mesh2dComponent`，但`from_scene_asset`在`scene_asset.rs:227-228`固定为`None`，`SceneEntityAsset`也没有对应字段，save端无输出。任何Editor/脚本创建的first-party 2D节点经canonical Save/reopen后丢失2D渲染组件。目标schema必须由component serializer registry驱动，并为内建2D提供golden roundtrip。

## 5. P1：World/Level registry、identity与lifecycle（RWL-P1-001至015）

| ID | 差距 | 重构要求 |
|---|---|---|
| RWL-P1-001 | `LevelManager`只公开create/exists/summary/load/save，没有enumerate、current、activate、unload、destroy、travel、streaming或replace。 | 建立typed `LevelInstanceRegistry`与`WorldContextService`，查询和mutation分开。 |
| RWL-P1-002 | `LevelLifecycleState`只有Loaded/Unloaded，生产代码没有状态迁移调用，测试是主要consumer。 | 定义Creating/Loading/Staged/Activating/Active/Quiescing/Unloading/Failed/Destroyed状态机和合法转移表。 |
| RWL-P1-003 | `LevelSystem::tick`不检查lifecycle，设为Unloaded仍可运行World和subsystem。 | execution permit必须由lifecycle generation签发，Inactive/Quiescing后拒绝新tick。 |
| RWL-P1-004 | manager没有remove/unload路径，`levels: HashMap`只增长。 | unload先quiesce participant、等待fence、发布tombstone，再回收slot。 |
| RWL-P1-005 | `HashMap`迭代决定multi-world同步顺序，顺序不稳定。 | registry维护稳定slot/order或显式并行plan，receipt记录处理顺序。 |
| RWL-P1-006 | handle生成使用`fetch_add(1)+1`，wrap时可panic、产生0或复用旧handle。 | 采用Runtime24 owner-qualified generational handle和checked exhaustion terminal。 |
| RWL-P1-007 | `WorldHandle(pub u64)`可Default/serde，无owner、epoch、generation，0也合法。 | live handle与persistent scene identity分离；每次resolve校验registry owner/epoch/generation。 |
| RWL-P1-008 | `try_for_each_world`先修改早期World，后续失败只返回error，无rollback。 | mutation先prepare全体，再按generation CAS一次发布；失败保证conservation。 |
| RWL-P1-009 | `sync_vm_types_atomically`同时持有所有World mutex，World数和数据量增大后形成全局停顿。 | schema candidate独立编译；每World短锁validate/publish，不跨World持锁。 |
| RWL-P1-010 | 上述rollback依赖有损`World::clone`，即使当前错误点较少也不能作为原子性证明。 | rollback artifact只能由显式participant capture产生，并列出unsupported owner。 |
| RWL-P1-011 | `LevelSystem: Clone`共享同一组`Arc<Mutex<_>>`，调用者看不出拿到的是alias而非独立Level。 | 改为显式`LevelHandle/LevelLease`，clone只复制handle且resolve受generation约束。 |
| RWL-P1-012 | World、metadata、lifecycle、physics、animation、script、frame、subscription各有独立Mutex。 | lifecycle transaction定义统一epoch和锁序；读者使用sealed state generation。 |
| RWL-P1-013 | 所有mutex poison都`into_inner`继续运行，可能发布panic留下的半变更World。 | poison触发Level quarantine/terminal failure；只有经过participant validation才能恢复。 |
| RWL-P1-014 | world replacement只手工重置已知runtime state，插件、任务、资产lease、observer和未来subsystem无参与协议。 | 建`WorldReplacementParticipant`的quiesce/capture/replace/rebind/resume/rollback阶段。 |
| RWL-P1-015 | `world_replacement_epoch.fetch_add`不处理wrap，旧异步operation最终可重新匹配。 | 使用checked epoch、owner incarnation和不可复用operation identity。 |

## 6. P1：Snapshot、document、schema与reference（RWL-P1-016至035）

| ID | 差距 | 重构要求 |
|---|---|---|
| RWL-P1-016 | `LevelSystem::snapshot() -> World`名称承诺完整副本，却没有Included/Skipped/Unsupported清单。 | 返回typed `SnapshotArtifact + ParticipantReceipt`，删除广义World snapshot。 |
| RWL-P1-017 | authoring save、runtime fork、checkpoint、render extract共用或容易误用同一World clone语义。 | 建四个不可互换的artifact类型和compile policy。 |
| RWL-P1-018 | `.scene.toml` root没有format/schema version。 | 写入engine format、document schema、build set与migration lineage。 |
| RWL-P1-019 | `SceneAuthoringDocument`的`_rest`只在reference rewrite阶段保留，decode为`SceneAsset`后未知字段丢失。 | unknown字段要么拒绝future schema，要么以opaque owner payload完整roundtrip。 |
| RWL-P1-020 | canonical scene没有连续N→N+1 migration chain和历史corpus。 | `SceneSchemaRegistry`登记deterministic migration、downgrade policy和provenance。 |
| RWL-P1-021 | 另有`PROJECT_FORMAT_VERSION=2` JSON World格式，与canonical SceneAsset形成双writer/双语义。 | 冻结唯一authoring格式；debug/checkpoint格式用不同magic/type，不再称project。 |
| RWL-P1-022 | JSON `WorldPersistentState`硬编码component maps，同样遗漏任意typed component。 | 旧格式只读迁移后删除writer，不能继续扩白名单。 |
| RWL-P1-023 | JSON中的HashMap输出顺序不保证稳定diff或content digest。 | canonical writer按stable entity/type/property key排序并做golden bytes。 |
| RWL-P1-024 | JSON load `normalize_after_load(true)`会注入Camera和DirectionalLight。 | load必须是纯decode/validate；创建模板默认值由显式NewScene命令负责。 |
| RWL-P1-025 | JSON load把schedule置Default，读取同一文件并不恢复同一运行行为。 | runtime schedule不是authoring document；由runtime profile重新构建并记录receipt。 |
| RWL-P1-026 | record batch validation不验证parent存在/自引用/环，只校验duplicate、transform和mobility。 | document preflight在commit前验证完整entity/reference graph。 |
| RWL-P1-027 | `rebuild_hierarchy_validity`把missing/self/cycle parent静默改成None。 | authoring load返回定位到entity/property的typed error；显式repair必须生成undo/diagnostic。 |
| RWL-P1-028 | joint `connected_entity`和skeleton binding可保留dangling ID。 | 所有entity reference由schema登记required/optional/external policy并统一验证。 |
| RWL-P1-029 | DynamicScene record remap只手工覆盖parent、joint和skeleton binding；新增内建entity字段容易漏。 | 内建与plugin字段都使用统一EntityMapper/type metadata，不保留手写名单。 |
| RWL-P1-030 | active camera通过插入顺序和“第一个camera”隐式选择，document无稳定role identity。 | SceneDocument保存stable entity GUID与explicit active-camera role。 |
| RWL-P1-031 | scene entity持久化使用live `u64 EntityId`，跨load/prefab/subscene无法稳定引用。 | 引入`SceneEntityGuid`，spawn时映射到owner-qualified live entity。 |
| RWL-P1-032 | SceneAsset按每个component增加Rust字段，扩展要求修改core document、codec和World转换。 | type catalog + component row schema由provider注册，core只拥有envelope。 |
| RWL-P1-033 | terrain/tilemap/prefab只有asset统计，没有runtime provider readiness或unsupported disposition。 | load receipt逐type记录Applied/Opaque/Unsupported/Migrated，产品不得静默成功。 |
| RWL-P1-034 | strict resource reference resolution没有覆盖entity refs、scene refs和provider refs的统一图。 | 建typed reference graph，边含kind、requiredness、expected schema和source location。 |
| RWL-P1-035 | document没有source digest/revision，打开后无法证明保存基于哪个版本。 | `OpenSceneReceipt`携file identity/content digest/schema revision，save使用expected revision CAS。 |

## 7. P1：Persistence operation、transaction与durability（RWL-P1-036至050）

| ID | 差距 | 重构要求 |
|---|---|---|
| RWL-P1-036 | `save_level/save_world`在I/O admission前同步`level.snapshot()`，大World先阻塞caller并分配。 | admission先取得预算/operation，再在worker上增量compile snapshot。 |
| RWL-P1-037 | save同时物化World clone、SceneAsset、TOML String/bytes，固定64MiB只是末端上限。 | streaming writer/chunk builder按entity/type预算，峰值内存纳入admission。 |
| RWL-P1-038 | load是同步read/parse/resolve/build World，没有ticket、cancel、progress或staging publication。 | `SceneLoadOperation`分header/schema/dependency/decode/preflight/commit阶段。 |
| RWL-P1-039 | `SceneArtifactTicket`只暴露generation/terminal/wait。 | 统一Runtime41 operation contract，含cancel、progress、phase、wake和structured error。 |
| RWL-P1-040 | failure terminal只保留固定code，底层`SceneProjectError`主要落可选日志。 | terminal保存stable error kind、source location、cause chain和recovery disposition。 |
| RWL-P1-041 | ticket不绑定project/scene path、source revision、content digest或world generation。 | receipt贯穿principal/project/scene/world/schema/source/attempt/published generations。 |
| RWL-P1-042 | save没有expected source digest/CAS，外部工具或第二进程修改可被覆盖。 | durable store提供compare-and-swap与conflict artifact。 |
| RWL-P1-043 | save generation使用`fetch_add().saturating_add(1)`，耗尽后多个operation同generation。 | checked non-reusable operation id；耗尽是terminal shutdown而非饱和。 |
| RWL-P1-044 | pending count和64MiB为进程硬编码，不区分项目、scene优先级、autosave/manual或设备。 | policy来自runtime config/store capability，并有per-principal公平性。 |
| RWL-P1-045 | same-key superseding只处理queued generation，未定义已开始serialize/publish时的取消与顺序。 | operation state machine定义before-start、pre-publish、post-publish disposition。 |
| RWL-P1-046 | `atomic_write`只覆盖单文件替换，workspace与scene不是一个原子事务。 | multi-file project transaction使用journal/manifest generation和startup recovery。 |
| RWL-P1-047 | Editor先写workspace再写scene，失败时补偿rollback；崩溃或rollback失败可留下split state。 | workspace引用新scene generation只在scene durable后一次publish。 |
| RWL-P1-048 | Play snapshot rename后未同步父目录；authoring atomic writer也未在本轮证明目录耐久。 | durable terminal前file sync + directory sync，并按平台实现能力报告。 |
| RWL-P1-049 | 没有temp/journal orphan发现和启动恢复owner。 | store启动扫描transaction manifest，完成/回滚/隔离并生成receipt。 |
| RWL-P1-050 | scene create staging sequence使用进程AtomicU64且可wrap，文件身份不含project generation。 | transaction id由store签发，包含project owner epoch并checked exhaustion。 |

## 8. P1：Product integration、performance与资格（RWL-P1-051至060）

| ID | 差距 | 重构要求 |
|---|---|---|
| RWL-P1-051 | Save菜单在UI控制链同步clone完整World，之后才提交I/O。 | UI只提交immutable authoring generation；snapshot compile在有预算worker执行。 |
| RWL-P1-052 | Enter Play先做有损World clone，又同步DynamicScene反射扫描和pretty JSON。 | runtime fork直接从sealed authoring snapshot构建，避免两次全量copy/encode。 |
| RWL-P1-053 | DynamicScene同时保存NodeRecord字段和可反射内建components，payload可能重复表达同一值。 | schema确定每类型唯一owner，禁止record与component row双写。 |
| RWL-P1-054 | manager没有Unreal式WorldContext/PIE instance identity，多Editor viewport、双PIE和server world无法隔离。 | `WorldContextId`绑定purpose、PIE instance、project、net mode、feature profile和current world。 |
| RWL-P1-055 | `LevelSummary`只有handle/entity_count/active_camera，无法观察lifecycle、revision、source、pending operation或memory。 | 发布generation-qualified bounded diagnostics snapshot。 |
| RWL-P1-056 | `subsystems: Vec<String>`仅保存名字，允许重复且没有owner、dependency、start/stop/rollback。 | 使用typed participant registry和dependency DAG；字符串仅用于display。 |
| RWL-P1-057 | world replacement没有pre-unload/post-load、entity remap、selection、network、script、asset通知合同。 | transaction输出old/new identity map并向registered consumers一次发布。 |
| RWL-P1-058 | 测试构造terrain/tilemap/prefab主要验证统计/引用，未测World load-save non-None roundtrip。 | 加入每个first-party field和组合场景golden roundtrip，P0先RED。 |
| RWL-P1-059 | 没有“自定义typed/dynamic component -> Save -> dirty clear -> reopen”产品回归。 | 用真实Editor/Project authority路径验证conservation和unsupported fail-closed。 |
| RWL-P1-060 | 没有“Enter/Exit Play保持作者组件且runtime实体集合exact”等验收，也无100K entity/multi-world/fault基准。 | 建产品E2E、fault matrix、soak和1/1K/100K基准；source-shape测试不能替代。 |

## 9. P2：一致性、可维护性与诊断（14项）

| ID | 差距 | 收敛方向 |
|---|---|---|
| RWL-P2-001 | `World`、`Scene` type alias、`LevelSystem`和`ProjectSceneDocument`语义重叠。 | 统一术语：document、world instance、level instance、context、snapshot。 |
| RWL-P2-002 | contract default method把project lookup和parse隐藏在trait默认实现中。 | service实现显式拥有依赖，contract只声明稳定request/result。 |
| RWL-P2-003 | 多处用String拼接SceneAsset错误，缺字段路径和entity identity。 | typed diagnostic path + error code + source span。 |
| RWL-P2-004 | `registered_subsystems()`每次clone完整字符串Vec。 | diagnostics snapshot共享immutable registry generation。 |
| RWL-P2-005 | LevelSystem Debug会跨多个锁读取状态，不能证明同一generation。 | Debug只读单一sealed summary，不触发重锁链。 |
| RWL-P2-006 | lifecycle/accessor测试刻意证明poison后继续运行，固化了错误安全语义。 | 改测quarantine、repair validation和terminal receipt。 |
| RWL-P2-007 | scene/project I/O没有集中source inventory与writer ownership guard。 | Runtime61实现期建立Rust-owned contract test；暂停tooling迁移不新增Python。 |
| RWL-P2-008 | `.json`、`.zrscene.json`、`.scene.toml`都可被称scene/project snapshot。 | magic、extension、MIME和API type一一对应。 |
| RWL-P2-009 | 多个全局AtomicU64分别生成publication/ticket/staging identity。 | 统一operation/transaction identity service。 |
| RWL-P2-010 | 测试直接写临时文件并手动remove，失败时易留孤儿。 | 使用RAII temp workspace与fault-capable store fixture。 |
| RWL-P2-011 | project_io按camera/mesh/physics等手写converter，扩展依赖core文件增长。 | per-type codec/provider模块由schema registry发现。 |
| RWL-P2-012 | direct reference collection重复构建Vec，缺稳定去重与边source。 | graph builder一次收集并输出stable edge table。 |
| RWL-P2-013 | `LevelDisplayName`由path/string fallback推导，不是稳定metadata identity。 | display与canonical scene identity分离。 |
| RWL-P2-014 | 没有snapshot/save/load/replacement的统一bytes、duration、phase、skip、rollback计数。 | `WorldLifecycleDiagnostics`发布有界generation snapshot和trace关联。 |

## 10. 参考引擎对照与采用边界

| 参考 | 本地源码事实 | Zircon应吸收 | 不应照搬 |
|---|---|---|---|
| Unreal WorldContext/World | `FWorldContext`拥有world type、context handle、PIE instance、travel、pending map和外部current-world引用；`UWorld`明确persistent level与streaming levels。 | 多World purpose/PIE identity、current world切换、travel状态和外部引用更新。 | UObject/GC和宏反射实现细节。 |
| Unreal Level/LevelStreaming | streaming state区分Removed/Unloaded/Failed/Loading/LoadedNotVisible/MakingVisible/Visible/MakingInvisible；Add/Remove可time-slice并有delegate。 | Level lifecycle、visibility/load分离、budgeted activation/unload和通知。 | 旧World Composition兼容包袱。 |
| Bevy world serialization | DynamicWorld按reflection registry抽取所有允许component/resource，WorldFilter显式allow/deny；SceneEntityMapper统一重映射所有登记entity字段。 | schema-driven participant、显式filter、统一entity mapper和instance registry。 | Bevy当前whole-instance heavy respawn不能直接作为Zircon热重载终态。 |
| Fyrox Scene/Graph | SceneContainer使用generational Pool，remove进入destruction list；Scene clone返回NodeHandleMap；Visitor显式visit Graph/render options/skybox，SerializationContext登记constructor。 | generational scene lease、延迟销毁、显式clone map、constructor/schema registry。 | visitor中panic和部分错误仅日志的历史行为。 |
| Godot PackedScene/SceneTree | PackedScene保存version、owner、properties、connections、base scene、editable instances；SceneTree管理current/pending/previous并延迟切换；duplicate按flags和映射处理。 | versioned authoring state、owner/instance/override、pending commit和精确duplicate policy。 | NodePath/name依赖及主线程全局树限制。 |
| Unity Graphics | 本地Graphics仓只有render pipeline/resource reload组件，不拥有Unity核心SceneManager或scene serializer。 | 仅借鉴render resource field reload与authoring/runtime profile分离。 | 禁止把Graphics包当完整Unity scene/world生命周期证据。 |

## 11. 目标架构

```text
ProjectAuthority
  -> AuthoringSceneDocument(schema version, SceneEntityGuid, type catalog, opaque rows)
  -> SceneSchemaRegistry(type/schema/provider/migration/reference policy)
  -> ScenePersistenceService(open receipt, expected revision, streaming writer, durable tx)

WorldContextRegistry
  -> WorldContext(purpose, project, PIE/server identity, current/pending world)
  -> LevelInstanceRegistry(generational handle, source, lifecycle, visibility, operations)
  -> WorldLifecycleCoordinator
       prepare -> dependency load -> stage -> quiesce -> commit -> rebind -> resume
       failure -> rollback/quarantine/terminal receipt

SnapshotCompiler
  -> AuthoringSaveSnapshot
  -> RuntimeWorldFork
  -> CheckpointSnapshot              [Runtime40 owner]
  -> FrameExtract                    [Runtime05/graphics owner]
  -> participant receipt(Included/Skipped/Unsupported/Migrated)
```

关键硬约束：

1. 删除“`Clone for World`等于完整snapshot”的假合同，不新增白名单补丁。
2. authoring save遇到未知或不支持participant必须fail closed或opaque preserve，不能成功后清dirty。
3. load、save、Play、travel和replacement都必须绑定project/world/schema/source generation。
4. live entity handle不能直接持久化；持久化只保存SceneEntityGuid和typed reference edge。
5. World replacement是participant transaction，不允许散落`reset()`调用成为唯一清理机制。
6. canonical writer只有一个；legacy JSON迁移完成后删除writer，不留compat shim。

## 12. 分阶段重构里程碑

### M61-0：P0 characterization与format冻结

- 为五项P0加入真实产品RED测试；
- 冻结现有`.scene.toml`、legacy JSON、DynamicScene JSON的writer/reader/caller矩阵；
- 对现存Vampire terrain执行load/save conservation审计；
- 禁止新增`World::clone`、`snapshot() -> World`或新的硬编码SceneAsset字段入口。

### M61-1：SceneSchemaRegistry与AuthoringSceneDocument

- 建立stable type/schema/provider/migration/reference catalog；
- 迁移内建3D、2D、terrain、tilemap、prefab和script bindings；
- unknown provider数据选择opaque preservation或明确unsupported；
- 引入SceneEntityGuid和统一EntityMapper。

### M61-2：SnapshotCompiler与World clone硬切

- 定义AuthoringSaveSnapshot、RuntimeWorldFork、Checkpoint、FrameExtract；
- 每个participant返回receipt和estimated/actual bytes；
- 迁移Editor Save与Enter/Exit Play；
- 删除`Clone for World`及伪装完整性的serde入口。

### M61-3：WorldContext与Level lifecycle

- 引入generational WorldContextId/LevelHandle；
- 建lifecycle transition table、current/pending world和multi-PIE隔离；
- 实现activate/quiesce/unload/destroy/travel；
- participant dependency DAG接管subsystems字符串Vec。

### M61-4：PersistenceService与durable transaction

- open receipt、source digest、expected revision CAS；
- streaming encode/decode、structure budgets、cancel/progress；
- multi-file journal、file+directory durability和startup recovery；
- legacy JSON只读迁移并删除旧writer。

### M61-5：World replacement transaction

- prepare/stage/quiesce/commit/rebind/resume与rollback/quarantine；
- old/new entity map、selection、script、physics、network、asset lease一次发布；
- poison、panic、provider unload、schema conflict和world churn fault injection。

### M61-6：产品、规模与发布资格

- Editor save/reopen、Play enter/exit、runtime load、multi-PIE、travel/unload E2E；
- 1/1K/100K entity、1/10/100 World和max-policy document基准；
- Windows/Linux文件事务、断电模拟、外部编辑冲突、long-session soak；
- 与Unreal/Fyrox/Godot相同语义场景做功能与性能对照，保留原始数据。

## 13. 验收门禁（45项）

### 数据完整性与schema（G01-G12）

- [ ] RWL-G01：任意registered typed/dynamic component经Save/reopen逐字段相等，unsupported时Save失败且dirty不清除。
- [ ] RWL-G02：Enter/Exit Play不改变authoring World的entity/component/resource集合与值。
- [ ] RWL-G03：Play runtime World不含snapshot外默认Camera/Light/Cube，entity conservation成立。
- [ ] RWL-G04：Vampire terrain及non-None terrain/tilemap/prefab完整load/save roundtrip。
- [ ] RWL-G05：Sprite2D/Mesh2D canonical roundtrip通过。
- [ ] RWL-G06：unknown future root/entity/component字段按策略拒绝或opaque-preserve，不静默删除。
- [ ] RWL-G07：每个component/resource row携stable type/schema/provider identity。
- [ ] RWL-G08：历史schema逐版本migration corpus和provenance通过。
- [ ] RWL-G09：所有entity refs经统一mapper，新增reference字段无需修改中央手写名单。
- [ ] RWL-G10：missing/self/cycle parent和dangling joint得到typed source diagnostic，不被静默改写。
- [ ] RWL-G11：canonical writer字节稳定，跨两次save无无意义diff。
- [ ] RWL-G12：legacy JSON只能迁移，不能继续产生新工程文档。

### World/Level lifecycle（G13-G23）

- [ ] RWL-G13：WorldContext区分Editor、PIE index、Game、Server、Tool和Preview。
- [ ] RWL-G14：LevelHandle跨owner/epoch/generation失效必拒绝。
- [ ] RWL-G15：lifecycle所有合法/非法转移有unit与model test。
- [ ] RWL-G16：Inactive/Quiescing/Unloaded Level不能tick或接受新operation。
- [ ] RWL-G17：unload/destroy回收registry slot且旧异步结果不能复活。
- [ ] RWL-G18：multi-world mutation失败不留下部分已修改World。
- [ ] RWL-G19：schema同步不跨所有World持锁且失败可回滚/隔离。
- [ ] RWL-G20：replacement participant按DAG quiesce/rebind/resume，缺participant时fail closed。
- [ ] RWL-G21：mutex poison/panic使Level quarantine，不继续发布半变更World。
- [ ] RWL-G22：operation/handle/epoch耗尽均typed terminal，无wrap、panic、0或饱和别名。
- [ ] RWL-G23：双PIE、双project、server+client World互不串扰。

### Persistence与transaction（G24-G34）

- [ ] RWL-G24：load/save在admission后执行，UI线程不做O(world bytes) clone/encode。
- [ ] RWL-G25：operation支持cancel/progress/wake/deadline和phase-specific disposition。
- [ ] RWL-G26：receipt贯穿project/scene/world/schema/source/attempt/published generations与digest。
- [ ] RWL-G27：external edit和双进程writer通过expected revision CAS产生conflict，不覆盖。
- [ ] RWL-G28：结构预算覆盖entity/type/field/reference/depth/string/decoded bytes。
- [ ] RWL-G29：max-policy document峰值内存、queue bytes和临时磁盘受预算约束。
- [ ] RWL-G30：workspace+scene以同一project generation原子发布。
- [ ] RWL-G31：file与directory durability在terminal Durable前完成。
- [ ] RWL-G32：每个crash point启动后可完成、回滚或隔离，不留下不可解释split state。
- [ ] RWL-G33：temp/journal orphan有owner、retention和可审计cleanup。
- [ ] RWL-G34：read-only/permission/disk-full/rename failure/rollback failure均给出准确disposition。

### 产品、性能与证据（G35-G45）

- [ ] RWL-G35：真实Editor Save成功后reopen保持全部数据；失败后dirty/undo仍在。
- [ ] RWL-G36：Enter/Exit Play、stop on error、runtime crash和hot reload都不修改authoring truth。
- [ ] RWL-G37：load/travel/unload有可见progress、cancel和terminal diagnostics。
- [ ] RWL-G38：LevelSummary/World diagnostics来自同一sealed generation。
- [ ] RWL-G39：1/1K/100K entity save/load/fork峰值内存与p50/p95/p99有原始样本。
- [ ] RWL-G40：1/10/100 World registry、schema sync、travel和unload无全局O(N×world bytes)停顿。
- [ ] RWL-G41：long-session create/unload/play/save soak后registry、ticket、temp、subscription和memory守恒。
- [ ] RWL-G42：Windows/Linux durable transaction与external editor冲突矩阵通过。
- [ ] RWL-G43：fault injection覆盖participant panic、provider unload、schema conflict、world churn和poison。
- [ ] RWL-G44：Unreal/Fyrox/Godot对照使用同一语义场景、硬件、构建profile和原始数据，不以单次FPS宣称超越。
- [ ] RWL-G45：所有旧World clone/project JSON writer/implicit default contamination入口被结构测试禁止。

## 14. 禁止的临时修补

- 禁止只把terrain/tilemap/prefab/Sprite2D/Mesh2D再塞进`World::clone`或`SceneEntityAsset`白名单就关闭P0。
- 禁止把`World::new()`改成`World::empty()`后继续把append spawn称为exact restore而不做conservation测试。
- 禁止用`serde(flatten)`字段存在证明unknown数据可roundtrip；必须经过decode/edit/save产品链验证。
- 禁止为LevelManager增加一组`unload_*_async_with_progress` facade而没有统一operation/lifecycle owner。
- 禁止在poison后继续`into_inner`并用日志宣称恢复成功。
- 禁止把authoring project save、SaveGame、DynamicScene archive和checkpoint合并成一个万能JSON格式。
- 禁止新增兼容re-export保留旧World clone/legacy writer；调用迁移完成后硬切删除。
- 禁止用source-shape测试、API数量、compile-only或小fixture代替数据守恒、fault和规模验收。

## 15. 当前状态

- review状态：complete；implementation状态：pending。
- 新增计数：P0=5、P1=60、P2=14、验收门禁=45。
- 本轮只新增审查文档并同步总账；没有修改production、tests、Cargo、ABI或reference source。
- 由于工作区存在并发修改且未运行Cargo/产品/性能验证，所有finding在实现切片开始前必须按最新source fingerprint重核。
- 首个实现切片必须从M61-0五项P0 characterization开始；在RED证据和schema/owner冻结前，不得继续扩展临时字段白名单。
