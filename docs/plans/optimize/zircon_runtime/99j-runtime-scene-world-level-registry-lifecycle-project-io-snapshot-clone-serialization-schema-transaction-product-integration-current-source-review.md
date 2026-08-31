---
title: Runtime Scene World、Level、Registry、Lifecycle、Project I/O、Snapshot/Clone、Serialization Schema、Transaction 与 Product Integration 当前源码工程化差距复核
category: zircon_runtime
report_id: Runtime109
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/level_system
  - zircon_runtime/src/scene/module
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/schedule.rs
  - zircon_runtime/src/scene/world/performance_diagnostics.rs
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/scene/dynamic_scene/document
  - zircon_runtime/src/scene/dynamic_scene/scene
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/assets/project_document
  - zircon_runtime/src/core/framework/scene
  - zircon_runtime/src/core/resource/io/atomic_file
  - zircon_runtime/src/core/resource/io/transaction
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_editor/src/core/editing/authoring_world.rs
  - zircon_editor/src/core/play/snapshot
  - zircon_editor/src/core/project/scene_document.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/workbench/project
  - zircon_editor/src/ui/workbench/state/editor_state_play_mode.rs
tests:
  - zircon_runtime/src/scene/tests/world_basics/world_state.rs
  - zircon_runtime/src/scene/tests/derived_state/hierarchy_behavior.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/tests.rs
  - zircon_runtime/src/asset/tests/assets/scene/management.rs
  - zircon_runtime/src/asset/tests/project/example_vampire/manifest_scene_imports.rs
  - zircon_editor/src/core/play/snapshot/tests.rs
  - zircon_editor/src/core/project/tests/scene_document.rs
  - examples/vampire/assets/scenes/main.scene.toml
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/40-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/52-runtime-dynamic-scene-session-archive-slot-capture-restore-path-merge-retention-durability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/53-runtime-dynamic-scene-asset-reload-event-generation-reconciliation-stage-apply-instance-replacement-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99i-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-current-source-review.md
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/Engine.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/World.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/Level.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/LevelStreaming.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/GameInstance.h
  - dev/bevy/crates/bevy_ecs/src/entity/map_entities.rs
  - dev/bevy/crates/bevy_ecs/src/reflect/map_entities.rs
  - dev/bevy/crates/bevy_world_serialization/src/components.rs
  - dev/bevy/crates/bevy_world_serialization/src/dynamic_world_builder.rs
  - dev/bevy/crates/bevy_world_serialization/src/dynamic_world.rs
  - dev/bevy/crates/bevy_world_serialization/src/lib.rs
  - dev/bevy/crates/bevy_world_serialization/src/reflect_utils.rs
  - dev/bevy/crates/bevy_world_serialization/src/serde.rs
  - dev/bevy/crates/bevy_world_serialization/src/world_asset_loader.rs
  - dev/bevy/crates/bevy_world_serialization/src/world_asset_spawner.rs
  - dev/bevy/crates/bevy_world_serialization/src/world_asset.rs
  - dev/bevy/crates/bevy_world_serialization/src/world_filter.rs
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/mod.rs
  - dev/Fyrox/fyrox-graph/src/lib.rs
  - dev/Fyrox/editor/src/scene/container.rs
  - dev/godot/scene/main/scene_tree.h
  - dev/godot/scene/main/scene_tree.cpp
  - dev/godot/scene/main/node.h
  - dev/godot/scene/main/node.cpp
  - dev/godot/scene/resources/packed_scene.h
  - dev/godot/scene/resources/packed_scene.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/ReloadAttribute.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/ReloadGroupAttribute.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/ResourceReloader.cs
doc_type: current-source-review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime109 · Scene World / Level / Project Persistence 当前源码工程化差距复核

## 1. 结论

Runtime61 的核心判断在当前源码中仍成立：Zircon 已有可复用的 `World`、typed/dynamic component 基础、`LevelSystem`、bounded scene artifact lane、single-file atomic writer、通用 durable multi-file transaction、DynamicScene preflight/commit 和 Editor Save/Play 接线，不能称为空壳；但这些局部能力尚未形成一个可以证明数据守恒、身份稳定、生命周期隔离、schema 演进和崩溃恢复的工程级 World/Level 产品合同。

本轮逐文件复核后，Runtime61 的 **5 项 P0 全部仍为 Open**。旧 P0-001 需要精确纠偏：当前 `World::clone` 已复制 dynamic JSON component map，不再是“所有 dynamic component 都丢失”；真正仍成立的是 `component_storage` 被清空后只重建七组硬编码 typed projection，任意已注册但未列入白名单的 typed component 仍会消失，而 canonical `.scene.toml` 又只持久化固定字段和 script bindings，不能保存开放组件集合。退出 Play 继续以该 clone 覆盖 authoring World；版本化 Play snapshot 继续 spawn 到含 Camera/DirectionalLight/Cube 的 `World::new()`；terrain/tilemap/prefab 与 Sprite2D/Mesh2D 的 canonical 双向缺口也未关闭。

状态总账保持 canonical 编号，不重复计数：**P0 5 Open；P1 56 Open、4 Partial、0 Closed；P2 13 Open、1 Partial、0 Closed；45 项门禁全部 Fail**。当前变化改善了稳定遍历、handle exhaustion、部分 JSON preflight、单文件 durability 和 subsystem read snapshot，但没有关闭任何 canonical gap。实现顺序必须先建立数据守恒 characterization，再 hard-cutover 到 `WorldContextRegistry + LevelInstanceRegistry + WorldLifecycleCoordinator + AuthoringSceneDocument + SceneSchemaRegistry + SnapshotCompiler + ScenePersistenceService + WorldReplacementTransaction`，不能继续扩展 clone/serializer 白名单。

本轮只做 review 与文档维护，没有修改 production、tests、Cargo、ABI 或 `dev/` 参考源码，也没有运行 Cargo、Editor、Vampire、fault injection、断电恢复或 benchmark。因此本文不能证明功能、性能或表现达到或超过 Unreal；它给出的是达到该目标前仍缺失的合同、实现和可重复证据。

## 2. 审查边界、currentness 与 ownership

### 2.1 Canonical owner 与去重规则

| 领域 | Canonical owner | Runtime109 的作用 | 不重复登记 |
|---|---|---|---|
| World/Level/persistence combined contract | Runtime61 | 逐项刷新 5/60/14 findings 与 45 gates | RWL-P0/P1/P2、RWL-G 编号 |
| Scene ECS kernel | Runtime60 / Runtime108 | 只追踪 clone、component conservation 与 schedule 重建的产品后果 | storage/query/schedule/event 内核 finding |
| World lifecycle 父架构 | Runtime05 | 验证 World truth、derived state、replace 边界 | 通用 Scene/World 生命周期父问题 |
| Stable identity | Runtime24 | 验证 WorldHandle、scene entity、operation generation | 全仓 owner/epoch/generation 规范 |
| SaveGame / archive / reload | Runtime40 / 52 / 53 | 识别可复用 participant、artifact 和 operation 基础 | 不把 authoring save 冒充 SaveGame/archive |
| Editor document/play | Editor02 / Editor07 | 追踪真实 Save、dirty、Enter/Exit Play 调用链 | Editor UI/进程体验父问题 |

固定架构仍是 `zircon_app`、`zircon_runtime`、`zircon_editor` 三个 public root package，runtime 内部遵循 `core/{runtime,framework,manager,math,resource}` spine。Scene/World 是 runtime truth，Editor 只能持有 authoring adapter、selection、history 与 presentation；不得创建第二个 Editor scene authority，也不得通过 compatibility facade、旧 JSON writer 或 re-export 长期保留迁移前合同。

### 2.2 当前源码物理冻结

算法：repo-relative path 转 `/` 并小写排序去重；逐文件计算 lowercase SHA-256；以 `path<TAB>hash` 按 LF 连接且末尾无 LF；再对 UTF-8 manifest 计算 SHA-256。

| 冻结组 | 文件 | 行 | 非空行 | bytes | test attrs | ignored | unsafe 行 | Fingerprint |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| Level/module lifecycle | 17 | 2,865 | 2,548 | 102,649 | 23 | 1 | 0 | `303e47af9f657d290c8bfe1a51c2aedca175fd42a81b74b8502bed1951d1b522` |
| World/document/persistence | 63 | 14,668 | 13,582 | 530,217 | 54 | 1 | 5 | `9a1aff396347e79dd3cae9c0f7c07a5055cf04814b526828b09c6665f2500e94` |
| Product consumers | 38 | 3,742 | 3,400 | 131,247 | 25 | 0 | 4 | `deb4d2faee813d76048c5cf20eab75600b16b709d1266c32e76bbabd0086a4ee` |
| Focused tests/fixtures | 9 | 5,822 | 5,119 | 203,475 | 58 | 1 | 0 | `9c2e44198c23def6095d123714d967bae812609f2ad07c2f961669129956d507` |
| 去重 production | **118** | **21,275** | **19,530** | **764,113** | **102** | **2** | **9** | `235d9764ca52d7edd47dfa799a5b97cb4d06830d52b21e003b0e48310fc5c129` |
| 去重 focused set | **126** | **27,045** | **24,604** | **965,721** | **158** | **3** | **9** | `3d6665edccce41597a9dabf6d8d7321cc8d4e5ebd979bf00c49e200e2f5d0e49` |
| 30 个显式参考文件 | **30** | **36,013** | **30,024** | **1,351,243** | **49** | **0** | **4** | `c6be811aeb3b54d5836611aaa5ffc262107782146118efd8bb5eecd4e6ec8c47` |

冻结对应 HEAD `bee4c707b714738346b49bba15c59468b8bd9b39`、baseline epoch 339。冻结时共享工作区有 2,507 个 status entries；本范围有 7 个 working-tree 修改：`level_system.rs`、`level_manager_lifecycle.rs`、`world_driver.rs`、`hierarchy_behavior.rs`、`derived_state.rs`、`hierarchy.rs`、`schedule.rs`。本文绑定这些文件的实际 working-copy 内容，不归因、不回退，也不把并行会话中的变化当成已合入基线；实现前必须重新计算 fingerprint。

### 2.3 复核方法

1. 继承 Runtime61 对 74 个旧冻结文件的逐文件结论，按当前目录迁移补齐 scene asset、atomic file、durable transaction、play snapshot、hierarchy/schedule/diagnostics；当前 focused set 扩展为 126 个文件。
2. 对 Runtime61 baseline `bea1acf91b909525ab1759e2c800858b0eda6528` 到当前 working copy 的相关变化逐文件复核；核心冻结范围有 10 个文件变化、497 additions、90 deletions。
3. 沿 Editor Save、Enter/Exit Play、runtime versioned snapshot、canonical `.scene.toml` load/save、LevelManager save/load 五条真实链路检查输入、artifact、commit、dirty、rollback 与 reopen，不把 isolated unit API 当产品接入。
4. 对 clone/serde/schema/entity reference/default injection/unknown field/CAS/directory sync/operation generation/lifecycle poison 做正反证搜索；对新增测试检查其断言是否覆盖真实产品 conservation。
5. 展开 30 个明确参考文件，以 Unreal WorldContext/LevelStreaming、Bevy reflection world serialization、Fyrox generational SceneContainer、Godot PackedScene/current-pending switch、Unity Graphics resource reload policy 对照职责，而不是比较 API 名称。

## 3. 当前真实产品链

```text
Editor Save
  -> capture SaveToken
  -> EditorState::project_scene
  -> EditorAuthoringWorld::try_snapshot
  -> World::clone                           [开放 typed component 不守恒]
  -> EditorProjectDocument::save_to_project
  -> World::to_scene_asset                  [固定字段白名单]
  -> workspace write + scene atomic_write   [非同一 durable transaction]
  -> success -> mark_saved_if_unchanged

Editor Enter / Exit Play
  -> try_snapshot -> EditorPlaySession.scene
  -> DynamicScene versioned JSON
  -> runtime: DynamicScene::spawn_into(World::new())
  -> stop: *authoring_world = session.scene

Level save / load
  -> save: level.snapshot before lane admission
  -> full SceneAsset + TOML bytes in memory
  -> keyed bounded SceneArtifactIo -> single-file atomic_write
  -> load: sync read/parse/reference resolve/build World
  -> replace/reset fixed known runtime states
```

`zircon_runtime/src/core/resource/io/transaction` 已经有 journal、multi-file stage/commit/rollback/recovery 与 fault tests，这是必须复用的基础。当前缺口是 Editor project save 仍先写 workspace、再写 scene，并以补偿式 rollback 修复失败；崩溃点不能保证二者作为一个 project generation 原子发布。Play snapshot store 也仍是 temp `sync_all` 后 `rename`，没有与 authoring atomic writer 相同的目录 durability/capability receipt。

## 4. 五项 P0 当前证据

### RWL-P0-001：Save 成功并清 dirty 后仍可能丢失开放组件

状态：**Open，旧措辞部分过时但风险未关闭**。

- `World::clone` 现在复制 `dynamic_components` 与 `dynamic_component_generations`，相关 clone/JSON 单测已有正证据。
- 同一实现仍令 `component_storage: Default::default()`，随后只重建 persistent entity core、scene render、runtime-only post-process、physics、lighting、render2D、animation runtime 七组硬编码 projection；任意已注册但不在组内的 typed component 消失。
- canonical `SceneAsset` 仍是固定 Rust 字段 schema，dynamic payload 主要只写 script bindings；不能把 dynamic clone 的改善等同于 project save 守恒。
- Editor Save 仍通过 `try_snapshot -> Clone::clone`，成功后 `mark_saved_if_unchanged`；现有 reopen test 只覆盖 Cube transform，没有“自定义 typed/dynamic component -> save -> clear dirty -> reopen”的产品回归。

关闭条件：任意 provider 注册 component 经真实 Editor Save/reopen 逐字段守恒；unsupported provider 必须 fail closed 且 dirty/undo 不清除。继续给 clone 加白名单不能关闭此项。

### RWL-P0-002：退出 Play 以不完整快照覆盖 authoring World

状态：**Open**。

`editor_state_play_mode.rs` 进入 Play 时捕获 `try_snapshot`，退出时执行 `*scene = session.scene`。现有测试覆盖 selection/history、transform 和 unloaded state，但没有任意 typed component、resource、provider payload、entity exact-set conservation。由于 P0-001 未关闭，这条覆盖链不能证明作者数据安全。

### RWL-P0-003：Play snapshot 被追加到带默认实体的 World

状态：**Open**。

`dynamic_api/session/project.rs` 的 `VersionedSnapshot` 路径先构造 `World::new()` 再 `scene.spawn_into`；`World::new` 经 bootstrap 创建 Camera、DirectionalLight、Cube。没有默认实体与 snapshot entity 集合的 conservation 证明，也没有 stable identity/remap 处理重复角色。

### RWL-P0-004：terrain/tilemap/prefab canonical 双向丢失

状态：**Open**。

`SceneEntityAsset` 声明 `terrain`、`tilemap`、`prefab_instance`，但 `World::to_scene_asset` 明确写入三个 `None`；loader 又缺少对应 runtime component/provider 落点。当前 tests 主要检查统计和 reference inventory，没有 non-None World load/save/reopen golden roundtrip。

### RWL-P0-005：Sprite2D/Mesh2D 无 canonical 工程文档合同

状态：**Open**。

runtime 已有 first-party `Sprite2D`/`Mesh2D` component，但 `from_scene_asset` 明确设置 `sprite_2d: None`、`mesh_2d: None`，`SceneEntityAsset` 本身也没有相应字段。当前没有真实 Editor/project roundtrip。

## 5. Runtime61 P1 状态逐项刷新

状态计数：**Partial 4（005、006、026、048）；Open 56；Closed 0**。

| ID | 状态 | 当前源码复核 |
|---|---|---|
| RWL-P1-001 | Open | manager contract 仍只有 create/exists/summary/load/save，无 enumerate/current/activate/unload/destroy/travel/streaming/replace。 |
| RWL-P1-002 | Open | lifecycle 仍只有 Loaded/Unloaded，无工程状态机与合法迁移表。 |
| RWL-P1-003 | Open | `LevelSystem::tick` 不读取 lifecycle，Unloaded 仍可执行。 |
| RWL-P1-004 | Open | `levels: HashMap` 仍无 remove/unload/destroy 回收路径。 |
| RWL-P1-005 | Partial | multi-world 操作现按 raw handle 排序；稳定顺序有改善，但无 registry slot order/owner identity/receipt。 |
| RWL-P1-006 | Partial | `fetch_update` 允许分配 `u64::MAX` 一次并以 `LevelHandleExhausted` terminal；仍无 owner-qualified generational handle。 |
| RWL-P1-007 | Open | `WorldHandle(pub u64)` 仍可 Default/serde，0、owner、epoch、generation 合同未解决。 |
| RWL-P1-008 | Open | `try_for_each_world` 仍顺序原地修改，后续失败无 conservation rollback。 |
| RWL-P1-009 | Open | schema atomic sync 仍同时持有多 World mutex，形成全局停顿。 |
| RWL-P1-010 | Open | rollback 仍依赖不完整 `World::clone`，不能作为原子性证明。 |
| RWL-P1-011 | Open | `LevelSystem: Clone` 仍别名共享 Arc/Mutex，未改为显式 lease。 |
| RWL-P1-012 | Open | World、metadata、lifecycle、frame、physics、animation、script、subscription 仍分散锁，缺统一 sealed generation。 |
| RWL-P1-013 | Open | poison 仍统一 `into_inner` 继续运行，测试还固化该语义。 |
| RWL-P1-014 | Open | replacement 仍只重置固定已知 runtime state，无 provider participant protocol。 |
| RWL-P1-015 | Open | replacement epoch 仍 unchecked `fetch_add`。 |
| RWL-P1-016 | Open | `snapshot() -> World` 仍没有 Included/Skipped/Unsupported receipt。 |
| RWL-P1-017 | Open | authoring save、runtime fork、checkpoint、render extract artifact 类型仍未完全隔离。 |
| RWL-P1-018 | Open | canonical `.scene.toml` root 仍无 format/schema/migration lineage。 |
| RWL-P1-019 | Open | `_rest` 只在 rewrite 中保留，decode 为 `SceneAsset` 后 future fields 仍丢失。 |
| RWL-P1-020 | Open | 无连续 N→N+1 migration registry、历史 corpus、downgrade policy。 |
| RWL-P1-021 | Open | `PROJECT_FORMAT_VERSION=2` JSON World writer 与 canonical SceneAsset 仍双轨。 |
| RWL-P1-022 | Open | JSON persistent state 仍以固定 component maps 表达 typed 数据。 |
| RWL-P1-023 | Open | canonical byte stability/digest golden 未覆盖全部 map/type/property 顺序。 |
| RWL-P1-024 | Open | JSON load 仍 `normalize_after_load(true)`，会注入默认 Camera/DirectionalLight。 |
| RWL-P1-025 | Open | JSON load 仍重建默认 Schedule，读同一文件不恢复运行行为 receipt。 |
| RWL-P1-026 | Partial | 新增 orphan component-map、local transform、next-ID exhaustion、future version preflight；完整 hierarchy/reference graph 仍未验证。 |
| RWL-P1-027 | Open | hierarchy rebuild 仍把 missing/self/cycle parent 静默修复为 None。 |
| RWL-P1-028 | Open | joint/skeleton 等 entity reference 仍可能 dangling，无统一 schema policy。 |
| RWL-P1-029 | Open | DynamicScene remap 仍依赖内建手写字段名单，未统一 `EntityMapper` metadata。 |
| RWL-P1-030 | Open | active camera 仍为隐式 live ID/首个 camera 语义，无稳定 document role。 |
| RWL-P1-031 | Open | canonical entity identity 仍使用 live `u64 EntityId`，无 `SceneEntityGuid`。 |
| RWL-P1-032 | Open | SceneAsset 扩展仍要求修改 core struct、codec、World converter。 |
| RWL-P1-033 | Open | terrain/tilemap/prefab 无 provider readiness 与 Applied/Opaque/Unsupported disposition。 |
| RWL-P1-034 | Open | asset strict resolution 未扩展成 entity/scene/provider typed reference graph。 |
| RWL-P1-035 | Open | open/save 无 source digest/revision/file identity CAS。 |
| RWL-P1-036 | Open | save 仍在 I/O admission 前同步 clone World。 |
| RWL-P1-037 | Open | save 同时物化 clone、SceneAsset、TOML String/bytes；64 MiB 只是末端上限。 |
| RWL-P1-038 | Open | load 仍同步 read/parse/resolve/build，无 staged operation/cancel/progress。 |
| RWL-P1-039 | Open | ticket 仍主要是 generation/terminal/wait，无统一 phase/cancel/deadline/wake contract。 |
| RWL-P1-040 | Open | terminal diagnostics 仍不足以保留稳定 kind、source location、cause chain、recovery disposition。 |
| RWL-P1-041 | Open | ticket 未绑定 project/scene/source digest/world/schema/published generation。 |
| RWL-P1-042 | Open | save 无 expected source digest/CAS，外部编辑可能被覆盖。 |
| RWL-P1-043 | Open | scene artifact generation 仍 `fetch_add().saturating_add(1)`，耗尽后会别名。 |
| RWL-P1-044 | Open | 8 pending/64 MiB 仍为进程硬编码，无 project/principal/priority/config 公平性。 |
| RWL-P1-045 | Open | same-key superseding 仍未定义 started serialize/publish 的 cancel/order disposition。 |
| RWL-P1-046 | Open | authoring save 的 single-file atomic write 仍不是 workspace+scene 原子事务。 |
| RWL-P1-047 | Open | Editor 仍先写 workspace 后写 scene，以补偿 rollback 处理失败，crash split state 未解决。 |
| RWL-P1-048 | Partial | authoring atomic writer 已有 file sync、Unix parent sync、Windows replace/write-through；Play store 仍 raw rename，跨平台目录 durability receipt 未统一。 |
| RWL-P1-049 | Open | generic recovery 基础存在，但 scene/project startup 未成为 transaction orphan owner。 |
| RWL-P1-050 | Open | scene create staging identity 仍未绑定 project owner epoch/checked transaction service。 |
| RWL-P1-051 | Open | Save 菜单链仍在 UI 控制流同步 clone World。 |
| RWL-P1-052 | Open | Enter Play 仍 clone 后再反射扫描/pretty JSON，未从 sealed authoring artifact 直接 fork。 |
| RWL-P1-053 | Open | DynamicScene record 与 reflected component 仍可能双写同一语义。 |
| RWL-P1-054 | Open | 无 Unreal 式 WorldContext/PIE instance/project/net mode 隔离。 |
| RWL-P1-055 | Open | `LevelSummary` 仍仅 handle/entity_count/active_camera。 |
| RWL-P1-056 | Open | subsystem snapshot 改为 `Arc<[String]>`，但注册本体仍是可重复字符串、无 owner/DAG/lifecycle/rollback，故 canonical gap 仍 Open。 |
| RWL-P1-057 | Open | replacement 无 old/new identity map 及 selection/network/script/asset consumer notification。 |
| RWL-P1-058 | Open | 无 terrain/tilemap/prefab non-None World canonical roundtrip。 |
| RWL-P1-059 | Open | 无自定义 typed/dynamic component 真实 Save/dirty/reopen conservation 回归。 |
| RWL-P1-060 | Open | 无 Play exact conservation、100K entity、multi-world、fault/soak/competitive benchmark。 |

## 6. Runtime61 P2 状态逐项刷新

状态计数：**Partial 1（004）；Open 13；Closed 0**。

| ID | 状态 | 当前源码复核 |
|---|---|---|
| RWL-P2-001 | Open | World、Scene alias、LevelSystem、ProjectSceneDocument 术语仍重叠。 |
| RWL-P2-002 | Open | contract default methods 仍隐藏 project lookup/parse 依赖。 |
| RWL-P2-003 | Open | 多处 scene/project error 仍为 String，缺 entity/property/source path。 |
| RWL-P2-004 | Partial | `registered_subsystems` 发布 `Arc<[String]>` immutable snapshot；typed registry generation 与 owner contract 仍缺。 |
| RWL-P2-005 | Open | LevelSystem Debug 仍可能跨多锁读取，不是同一 generation summary。 |
| RWL-P2-006 | Open | poison 后继续运行的测试语义尚未替换为 quarantine/repair。 |
| RWL-P2-007 | Open | scene/project writer ownership 与 source inventory guard 未集中。 |
| RWL-P2-008 | Open | `.json`、`.zrscene.json`、`.scene.toml` 类型/magic/API 仍不一一对应。 |
| RWL-P2-009 | Open | publication/ticket/staging identity 仍来自多个 AtomicU64。 |
| RWL-P2-010 | Open | focused tests 仍手工管理 temp 文件/目录，缺 fault-capable RAII workspace。 |
| RWL-P2-011 | Open | project_io 继续按 camera/mesh/physics 等中央手写 converter。 |
| RWL-P2-012 | Open | reference collection 仍重复 Vec，缺 stable edge table/source。 |
| RWL-P2-013 | Open | LevelDisplayName 仍从 path/string fallback 推导。 |
| RWL-P2-014 | Open | 新增 hierarchy/topology/propagation/node-cache 诊断是邻接基础；仍无 snapshot/save/load/replacement bytes/duration/phase/skip/rollback 指标。 |

## 7. 当前变化的真实价值与边界

| 当前变化 | 可保留价值 | 不能据此关闭 |
|---|---|---|
| multi-world level 按 raw handle 排序 | 结果顺序较稳定，便于复现 | owner-qualified registry、事务 conservation |
| handle `fetch_update` exhaustion | 不再 wrap/panic/产生 0 | owner/epoch/generation stale rejection |
| `Arc<[String]>` subsystem snapshot | 读路径不再每次 clone 字符串 Vec | typed participant registry/DAG/lifecycle |
| JSON orphan/transform/version/ID preflight | 更多 malformed input 可在 commit 前拒绝 | 全 entity/reference graph 与 typed source diagnostic |
| direct-child hierarchy index/诊断 | 减少部分扫描并可观察 derived work | 静默 repair、document preflight、100K 资格 |
| robust single-file atomic writer | authoring scene 单文件替换和 sync 更可靠 | workspace+scene project transaction、Play snapshot durability |
| generic durable transaction 模块 | 已有可复用 journal/recovery/fault 底座 | 产品调用链接入与 project generation authority |
| clone 复制 dynamic JSON maps | 修正旧“dynamic clone 全丢”描述 | 任意 typed component 与 canonical dynamic persistence |

## 8. 五个参考体系的结构差异

### 8.1 Unreal：WorldContext、World、Level、Streaming 是不同控制面

`FWorldContext` 显式持有 world type、context handle、current world external references、TravelURL、PendingNetGame、待载入 levels、map-change failure、GameViewport/GameInstance、PIE instance、net drivers 与 dedicated/primary 标志；Engine 还提供 context create/destroy/lookup 与 prepare/commit/cancel map change。`UWorld` 区分 PersistentLevel、Levels、streaming levels、current level、visibility；`ULevelStreaming` 有 Removed/Unloaded/FailedToLoad/Loading/LoadedNotVisible/MakingVisible/LoadedVisible/MakingInvisible 状态。Zircon 当前单一 HashMap + Loaded/Unloaded 无法表达这些控制面。

### 8.2 Bevy：反射序列化的关键是 type registry、filter 和 entity remap

`MapEntities`/`ReflectMapEntities` 要求所有 entity 字段通过 mapper；world serialization 先从 reflection registry 构建 dynamic world，按 `WorldFilter` 明确 allow/deny component/resource，目标侧先分配全部 entity，再映射并应用 component，unregistered type/component/resource 返回 error。Zircon 仍依赖固定 component maps、中央 converter 和手写 reference 字段名单。Bevy 的 whole-instance artifact 也不应直接复制成 Zircon 最终 hot-reload 方案；Zircon 仍需增量 compile、budget 与 participant receipt。

### 8.3 Fyrox：generational SceneContainer 与 authoring scene entry 分层

Fyrox `SerializationContext` 拥有 node/script constructor containers；Scene 显式拥有 graph/render options/skybox/enabled，clone 返回 `NodeHandleMap` 并重映射跨引用。runtime SceneContainer 使用 generational Pool 校验 handle，remove 后进入 destruction list 延迟 deinit；Editor SceneContainer 另持 current scene、UUID、dirty、path、command stack。Zircon 的 raw `WorldHandle`、aliasing `LevelSystem::Clone` 和无 remove 路径达不到相同的 identity/lifecycle 基线。

### 8.4 Godot：PackedScene schema 与 current/pending 切换都是显式状态

Godot PackedScene 有版本、node owner/properties/groups、connections、base scene、editable instances、node/id paths；Node duplicate 使用 flags 与 duplicate map/resource remap。SceneTree 区分 `current_scene` 和 `pending_new_scene_id`，在 flush 时发布并 queue 旧 scene。Zircon 当前 clone 覆盖与 append spawn 没有等价的 owner map、pending publication、previous destruction 语义。

### 8.5 Unity Graphics：可借鉴的是资源 reload policy，不是 World owner

`ReloadAttribute`/`ReloadGroupAttribute` 与 `ResourceReloader` 通过反射递归修复 group/array，只加载缺失资源，并显式编码 package/path policy。这可支持 Zircon scene provider resource reload 与 missing-resource repair，但 Unity Graphics 样本不是 Unity 核心 scene/world 源码，不能用它证明 WorldContext、authoring persistence 或 entity remap 设计。

### 8.6 差距矩阵

| 能力 | Zircon 当前 | 参考基线 | 目标 owner |
|---|---|---|---|
| multi-world purpose/PIE/travel | raw handle + HashMap | Unreal FWorldContext/map change | WorldContextRegistry |
| level state/streaming/destruction | Loaded/Unloaded，无 remove | Unreal streaming state；Fyrox delayed destruction | LevelInstanceRegistry + LifecycleCoordinator |
| snapshot conservation | broad `World::clone` + whitelist rebuild | Fyrox NodeHandleMap；Godot duplicate map | SnapshotCompiler + ParticipantRegistry |
| open component schema | fixed Rust fields/maps | Bevy type registry/filter/world builder | SceneSchemaRegistry |
| entity reference remap | 手写字段名单 | Bevy MapEntities；Godot/Fyrox remap | EntityReferenceSchema + SceneEntityMap |
| authoring vs runtime truth | clone/overwrite/append | Fyrox editor entry/runtime container；Godot pending publish | AuthoringSceneDocument + WorldForkArtifact |
| durable project save | 两次单文件写 + compensation | 本仓已有 generic journal transaction 基础 | ScenePersistenceService |
| async operation | terminal/wait | phased travel/load/cancel/progress | RuntimeOperationService adapter |

## 9. 目标架构

```text
AuthoringSceneDocument
  stable scene/entity GUIDs
  schema + migration lineage
  provider component/resource rows
  opaque/fail-closed future-data policy
  source digest + expected revision
             |
             v
SceneSchemaRegistry ---- EntityReferenceSchema
             |
             v
SnapshotCompiler / WorldForkCompiler
  explicit include policy + budgets
  SceneEntityMap + provider receipts
  no broad World::clone contract
             |
             v
WorldContextRegistry
  purpose / project / PIE / net mode / feature profile
             |
             v
LevelInstanceRegistry ---- WorldLifecycleCoordinator
  owner-qualified generational handles
  lifecycle state machine + execution permits
  participant DAG + quiesce/fence/destroy
             |
             v
WorldReplacementTransaction
  prepare -> quiesce -> preflight -> publish CAS
  remap/rebind -> resume | rollback/quarantine
             |
             v
ScenePersistenceService
  bounded phased operation
  multi-file journal transaction + recovery
  CAS/conflict artifact + durable receipt
```

关键约束：authoring document、runtime fork、checkpoint、render extract 必须是不可互换的 artifact type；live World handle 与 persistent scene identity 必须分离；provider 没有 schema/clone/serialize/remap participant 时必须 fail closed；load 不得注入模板默认实体；save 的 Durable terminal 必须晚于 file/directory/project-generation publication。

## 10. 依赖顺序重构计划

### M0：P0 characterization 与 writer freeze

- 增加 arbitrary typed/dynamic component Save/reopen、Play enter/exit exact conservation、empty snapshot no-default、terrain/tilemap/prefab、Sprite2D/Mesh2D 五组 RED 产品测试。
- 冻结 legacy JSON 与 canonical SceneAsset writer 扩展；只允许修复 diagnostic，不再加字段白名单。
- 给当前所有 component/resource/reference owner 生成 Included/Skipped/Unsupported inventory。

### M1：identity、context 与 lifecycle registry

- 建立 `WorldContextId`、owner-qualified generational `LevelHandle`、stable `SceneGuid/SceneEntityGuid`。
- 引入 context purpose/PIE/project/net profile 与 Level lifecycle state machine。
- hard-cutover create/resolve/enumerate/activate/unload/destroy；删除 raw-handle/facade 旧路径。

### M2：schema、reference graph 与 document migration

- 建立 `SceneSchemaRegistry`、provider type/schema generation、component/resource row envelope。
- 所有 entity/asset/scene/provider reference 通过统一 schema mapper 和 source-located diagnostics。
- 唯一 canonical authoring format 带 format/schema/migration lineage；legacy JSON 只读迁移后删除 writer。

### M3：artifact compiler 与 conservation

- 用 `AuthoringSaveArtifact`、`RuntimeForkArtifact`、`CheckpointArtifact`、`RenderExtractArtifact` 取代 broad World clone。
- compiler 先 admission，增量处理 entity/type/reference，记录 bytes/budget/unsupported participant receipt。
- provider 必须注册 capture/clone/serialize/remap/apply contract，缺失即 fail closed。

### M4：World replacement 与 multi-world transaction

- 建立 participant DAG 的 prepare/quiesce/fence/capture/replace/remap/rebind/resume/rollback/quarantine。
- multi-world mutation 先全量 preflight，再 generation CAS 发布，不跨所有 World 长期持锁。
- poison、panic、provider unload、epoch exhaustion 都进入 typed terminal/quarantine。

### M5：persistence operation 与 durable project transaction

- 将 scene load/save 接入统一 bounded operation：phase/progress/cancel/deadline/wake/disposition。
- 复用 `core/resource/io/transaction`，让 workspace+scene 以同一 project generation journal 原子发布并启动恢复。
- 加 expected revision/content digest CAS、external editor conflict artifact、temp/journal owner retention。

### M6：产品 hard-cutover

- Editor Save 只提交 immutable authoring generation；worker compile/persist；成功 receipt 才推进 dirty baseline。
- Enter Play 直接从 sealed authoring artifact fork empty runtime world；Exit Play 不覆盖 authoring truth，live edit 走显式 patch transaction。
- runtime project open、LevelManager、Vampire fixture 全部迁移；删除旧 clone/save JSON/implicit default contamination 入口。

### M7：资格与竞争性证据

- 1/1K/100K entity、1/10/100 World 的 save/load/fork/travel/unload p50/p95/p99、peak RSS、alloc、I/O bytes。
- Windows/Linux crash-point、disk full、permission、rename、rollback、external edit、dual writer fault matrix。
- long-session handle/epoch/ticket/temp/subscription/memory conservation soak。
- 与 Unreal/Fyrox/Godot 使用同语义场景、硬件、build profile 和原始数据对照；没有可复现实测不得宣称超越。

## 11. 45 项门禁当前状态

Runtime61 对 RWL-G01..G45 的定义继续唯一有效。本轮逐组复核如下，**45 项全部 Fail，0 项 Pass**。

| 门禁 | 数量 | 当前状态 | 主要阻断 |
|---|---:|---|---|
| G01-G05 数据守恒 P0 | 5 | Fail | arbitrary typed save、Play exact-set、default contamination、terrain/2D 均未通过 |
| G06-G12 schema/migration/reference | 7 | Fail | future-field roundtrip、stable schema/provider ID、migration corpus、统一 mapper、golden bytes、legacy writer freeze 均缺 |
| G13-G23 World/Level lifecycle | 11 | Fail | context/PIE、generational handle、状态机、unload、atomic multi-world、participant、quarantine、exhaustion 未闭环 |
| G24-G34 persistence/transaction | 11 | Fail | post-admission compile、cancel/progress、generation receipt、CAS、结构预算、multi-file atomicity、crash recovery 未闭环 |
| G35-G45 product/performance/evidence | 11 | Fail | 真实 Save/Play/travel、sealed diagnostics、100K/multi-world、soak、cross-platform fault、reference parity 均无资格证据 |

局部单元测试不能冒充 gate：handle exhaustion test 只支撑 P1-006 Partial；orphan/ID/future-version tests 只支撑 P1-026 Partial；atomic-file fault tests 只支撑 P1-048 Partial；subsystem Arc snapshot test 只支撑 P2-004 Partial。它们都没有覆盖真实 Save/Play/Level lifecycle 端到端守恒。

## 12. 禁止的临时修补

- 禁止继续把 component 字段塞进 `World::clone`、`SceneEntityAsset` 或 JSON maps 后关闭 P0。
- 禁止仅把 `World::new()` 换成 `World::empty()`，却不建立 identity remap、participant receipt 与 exact conservation。
- 禁止以 `_rest`/`serde(flatten)` 字段存在证明 future data 可 roundtrip；必须穿过 open/edit/save/reopen。
- 禁止给 LevelManager 追加一批 `*_async_with_progress` facade，而没有 context、lifecycle、operation owner。
- 禁止 poison 后 `into_inner` 继续发布半变更 World。
- 禁止把 authoring project、SaveGame、checkpoint、DynamicScene archive 合并成万能 JSON。
- 禁止新建 Editor scene authority、compat re-export、shim trait 或 bridge folder 保留旧 API；迁移完成后 hard cutover 删除。
- 禁止用 source-shape、compile-only、小 fixture、单次 FPS 或平均值替代 conservation、fault、tail latency 与规模证据。

## 13. 当前状态与 hand-off

- review：`current_source_refresh_complete`；implementation：`pending`。
- canonical 状态：P0 5 Open；P1 56 Open + 4 Partial；P2 13 Open + 1 Partial；45 gates Fail。
- 本篇新增 finding：0；关闭 finding：0；只纠正 P0-001 dynamic clone 的过宽旧措辞并记录当前局部进展。
- 首个实现切片只能从 M0 的五组 RED 产品测试与 writer freeze 开始；schema/identity/participant owner 未冻结前，不得扩展临时白名单。
- 当前 broad integration 会话仍在修改相关源码。任何实施或关闭判断都必须先重算 `235d...` production fingerprint 并复核 working-copy diff。
- 本轮未运行 Cargo、Editor、Vampire、fault、durability 或 benchmark；这些是实施后的资格工作，不属于本次 docs-only review。
