---
title: Runtime Prefab、Archetype、Prototype、Class Default、Instance Override、Instantiation、Propagation、Hot Reload、Network、Save 与 Product Integration Current Source Review
category: zircon_runtime
report_id: Runtime128
review_date: 2026-08-23
baseline_head: 471bb732e3683fd7c12d7b69a9e85a22048efcba
observed_head: 471bb732e3683fd7c12d7b69a9e85a22048efcba
baseline_epoch: 382
supersedes:
  - docs/plans/optimize/zircon_runtime/39-prefab-archetype-prototype-class-default-instance-override-runtime-instantiation-propagation-hot-reload-network-save-scalability-product-integration-review.md
related_code:
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/project_document/scene.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/asset/artifact/chunk_residency.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_authoring_asset.rs
  - zircon_runtime/src/asset/facade/impls.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager
  - zircon_runtime/src/builtin/runtime_modules/assembly
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/scene/dynamic_scene/remap.rs
  - zircon_runtime/src/scene/dynamic_scene/entity
  - zircon_runtime/src/scene/dynamic_scene/scene
  - zircon_runtime/src/scene/dynamic_scene/scene_asset
  - zircon_runtime/src/scene/dynamic_scene/spawn_task
  - zircon_runtime/src/scene/dynamic_scene/asset_reload
  - zircon_plugins/prefab_tools
  - examples/vampire/zircon-project.toml
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/scripts/vampire_game/main.zr
  - examples/woc/zircon-project.toml
  - examples/woc/assets/scenes/bootstrap.scene.toml
  - examples/woc/native/plugins/woc_runtime/src/transaction.rs
  - examples/woc/scripts/woc_game/src/main.zr
tests:
  - zircon_plugins/prefab_tools/editor/src/tests.rs
  - zircon_plugins/prefab_tools/runtime/src/tests.rs
  - zircon_runtime/src/asset/tests
  - zircon_runtime/src/scene/tests/asset_scene
  - zircon_runtime/src/scene/tests/dynamic_scene
  - zircon_runtime/src/tests/plugin_extensions
plan_sources:
  - docs/plans/optimize/zircon_runtime/39-prefab-archetype-prototype-class-default-instance-override-runtime-instantiation-propagation-hot-reload-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99i-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99j-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99l-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99m-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99u-runtime-dynamic-scene-asset-reload-event-generation-reconciliation-stage-apply-instance-replacement-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99v-runtime-dynamic-scene-session-archive-slot-capture-restore-path-merge-retention-durability-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99w-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/41-level-variant-data-layer-level-instance-world-outliner-authoring-review.md
  - docs/plans/optimize/zircon_editor/44-archetype-class-defaults-instance-override-property-propagation-reset-to-default-authoring-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/UObjectArchetype.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Internal/UObject/UObjectArchetypeHelper.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Tests/ClassDefaultObjectTest.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/InheritableComponentHandler.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/InheritableComponentHandler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/ComponentInstanceDataCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ComponentInstanceDataCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/LevelInstance/LevelInstancePropertyOverrideAsset.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/LevelInstance/LevelInstancePropertyOverrideAsset.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/WorldPartition/WorldPartitionPropertyOverride.cpp
  - dev/godot/scene/property_utils.h
  - dev/godot/scene/property_utils.cpp
  - dev/godot/scene/resources/packed_scene.h
  - dev/godot/scene/resources/packed_scene.cpp
  - dev/godot/tests/scene/test_packed_scene.cpp
  - dev/Fyrox/fyrox-core/src/variable.rs
  - dev/Fyrox/fyrox-core/src/reflect/inherit.rs
  - dev/Fyrox/fyrox-impl/src/scene/graph/mod.rs
  - dev/Fyrox/fyrox-impl/src/resource/model/mod.rs
  - dev/bevy/crates/bevy_scene/src/scene_patch.rs
  - dev/bevy/crates/bevy_scene/src/resolved_scene.rs
  - dev/bevy/crates/bevy_scene/src/scene.rs
  - dev/bevy/crates/bevy_scene/src/spawn.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeComponent.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeParameter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99zc · Runtime Prefab Current Source Review

## 1. 结论

Runtime39 的主裁决仍然成立：Zircon 当前没有可用于工程级产品的 Prefab runtime。仓库里存在 `PrefabAsset`、`PrefabInstanceAsset`、Scene/Prefab cache payload、内建 `.prefab.toml` 解析器、`prefab_tools` package，以及工程质量明显更高的 Dynamic Scene compile/preflight/commit transaction；但它们没有组成 `versioned source -> validated graph -> deterministic compiled artifact -> generation-qualified instance -> stable provenance -> rebase/update -> unload/save/network` 闭环。

当前最严重的 authority 矛盾仍未修复。内建 importer `zircon.builtin.toml.prefab` 与插件 importer `prefab_tools.prefab` 同时声明 `.prefab.toml`、同为 priority 0；前者实际执行通用 TOML 反序列化，后者只是 `DiagnosticOnlyAssetImporter`，错误文本为 `prefab importer backend is not installed`。`AssetImporterRegistry` 会拒绝同 matcher、同 priority，但错误只包含 matcher 和 priority，不包含冲突双方 ID。启用插件时这不是可组合扩展，而是装配失败。

与 Runtime39 相比，有一项真实修复：active importer merge 不再用 `let _ = register_arc(...)` 吞错，assembly 会收集 `asset_importer_errors` 并扩展到 `RuntimeModuleLoadReport`。因此 `PRF-P1-003` 本轮判定 Closed，`PRF-GATE-03` 判定 Pass；这不会关闭双 owner、伪 capability 或缺 instancer 的问题。

Scene document codec 现在能通过 flatten 的 `_rest` 保留 Prefab 附加字段，artifact cache payload 也保留 `prefab_instance`；但 live `World` 边界仍然断裂：`World::from_scene_asset` 不读取 `entity.prefab_instance`，`World::to_scene_asset` 固定写 `prefab_instance: None`。合法 Scene 一旦进入 World 再保存，Prefab link、local transform 与 override 仍会被静默擦除。该可达数据损失继续由 Editor44/Editor41 的父 P0 拥有，本报告不重复创建 P0。

`prefab_tools` editor helper 也不能算 authoring 产品。`effective_prefab_overrides` 仅以 `(entity_path, property_path)` 字符串做 last-write-wins；validation 只检查 source boolean、空字符串和重复路径；apply 只返回 DTO 后清空 instance overrides；revert 只清空；break 只返回 transform 与 baked override DTO。它们没有修改 source、物化实体树、创建 transaction、保存文档、生成 undo receipt 或触发 runtime update。

Dynamic Scene 是必须保留的底层资产：它已有 payload byte limit、compile、隔离 preflight World、component/resource decode、World/Level generation admission、紧凑 mutation、失败不发布、bounded reload queue 与 typed apply reports。但 `EntityRemap` 仍只是临时 `BTreeMap<EntityId, EntityId>`；prepared spawn 不携 source/artifact revision 或 instance identity；reload 对 Modified/Renamed 仍是 append-spawn，对 Removed/ReloadFailed 仍是 skip。它不是 Prefab Instance Registry，也不是 rebase/replacement/hot reload。

本轮将 Runtime39 的账本按当前源码重判为：**0 项本地新增 P0；53 P1 Open、18 P1 Partial、1 P1 Closed；16 P2 Open；31 Gate Fail、8 Gate Partial、1 Gate Pass**。目标架构仍应是单一 `PrefabRuntimeService`，内部明确分离 compiler、artifact store、instance registry、instantiation transaction、update/rebase coordinator 以及 streaming/network/save/script adapters；不得再把 DTO、descriptor、菜单项或一次性 scene spawn 相加后宣称 Prefab 完成。

“性能和表现优于当前 Unreal”目前仍无可验证证据。仓库只有一个 ignored 的 8,192 override borrowed-key microbenchmark 和通用 Dynamic Scene 规模测试，没有同内容、同硬件、同质量、同网络/存档语义的 Prefab load/update/frame-spike/RSS 基线。先建立完整语义和可复现 receipts，再谈领先；空实现或少做功能造成的低开销不能算优势。

本轮只做静态源码 review 和文档记录，没有修改 production、tests、Cargo、ABI 或参考源码，没有运行真实 Editor、Vampire/WOC、save/reopen、network、streaming、fault/soak/profile 或跨引擎 benchmark。Tooling 按用户要求排除。MVP 仍未完成，`source_recheck_required` 保持 true。

## 2. 审查边界与物理冻结

### 2.1 Focused 集合

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations / ignored | fingerprint |
|---|---:|---|
| Prefab definition/import/plugin/World core | 38 / 7,924 / 7,359 / 309,920 / 17 / 3 | `e9fdeb7f758e26945c0426ef479b67c366eb6ce37ac9246f6b83c39576349a68` |
| Dynamic Scene transaction/reload substrate，含同目录测试 | 38 / 6,521 / 5,934 / 227,034 / 33 / 1 | `68de3ce680624a069144301f048d90087446a4b0edc300762e2b6f0a6a790295` |
| Prefab/Scene focused tests | 29 / 9,682 / 9,107 / 355,550 / 96 / 2 | `42f7d39dd0b3a3706545c325aff6303acea25692b6267bf2b8db141a50a42282` |
| Vampire/WOC product boundary | 8 / 4,057 / 3,519 / 138,426 / 0 / 0 | `a8cba174ba0784321849742599ad1638cd3ac55d2a60d3dd8ba9d8a0fa5f84b2` |
| Zircon deduplicated focused total | 113 / 28,184 / 25,919 / 1,030,930 / 146 / 6 | `dd174dd492d3d8892b4f79442425b72cd8636e359985fb7944d74b7e927a6447` |
| Selected five-engine evidence | 26 / 17,228 / 15,136 / 649,378 / 47 / 0 | `51296867757fc06fc15b40d896a5fe011e50ab78a04e3a3cbee80e5a5f61f11a` |

分组允许重叠，总计按规范化路径去重。fingerprint 算法与本系列 current-source review 一致：仓库相对路径转 `/` 并小写、ordinal 排序去重；每项编码为 `lowercase-path + NUL + lowercase per-file SHA-256`，以 LF 连接且末尾无 LF，再对 UTF-8 payload 计算 SHA-256。它冻结本轮读取集合，不是 runtime artifact、save、network 或 release identity。

### 2.2 Currentness 与搜索边界

- Session baseline 与 observed HEAD 均为 `471bb732e3683fd7c12d7b69a9e85a22048efcba`，baseline epoch 为 382。
- 共享工作树包含其他 Session 与用户的在途修改；本轮只租用本报告、Runtime 索引、根索引和 coverage 台账，不回退、不归因其他文件。
- 对 `zircon_runtime`、`zircon_plugins/prefab_tools`、Vampire 与 WOC 做了 `PrefabAsset|PrefabInstanceAsset|prefab_instance|.prefab.toml|prefab_tools|PrefabInstance` consumer 搜索，并在 net、Dynamic Scene Session、script 与产品资产中复核 provenance consumer。
- 除定义、cache、测试和 plugin descriptor 外，没有 production Prefab instantiation consumer；仓库没有一份真实产品 `.prefab.toml` 资产。
- 参考源码按 exact path 逐文件读取；结论只提取身份、默认值、继承、resolve/apply、重建、失败和规模合同，不以类名相似度判断完成度。

### 2.3 状态含义

- **Open**：当前源码没有满足 finding 的 Prefab 专属合同。
- **Partial**：存在可复用的通用底座或局部行为，但没有闭合该 finding 的 Prefab owner、identity、lifecycle 和产品证据。
- **Closed**：当前源码直接关闭原 finding；本轮只有 importer merge 错误传播一项。

## 3. 当前实现事实

### 3.1 Source、Importer 与 Artifact

1. `PrefabAsset` 仍只有 `uri + name + embedded SceneAsset + Vec<String> exposed_properties`；没有 source schema version、stable object/component IDs、revision、digest、nested DAG、typed parameter schema 或 migration contract。
2. `PrefabInstanceAsset` 仍只有 asset reference、local transform 和 `Vec<PrefabPropertyOverrideAsset>`；override address 是 `entity_path + property_path`，value 是 raw `serde_json::Value`。
3. 内建 importer 版本为 1、suffix 为 `.prefab.toml`、priority 为 0；`import_prefab` 只执行 `toml::from_str::<PrefabAsset>`，不做 Prefab graph、cycle、dependency、budget、target/provider 或 override validation。
4. artifact cache 对 Prefab 只镜像 authoring DTO；没有独立 resolved artifact、creation order、relocation table、compiled storage slots、compatibility key 或 estimated runtime cost。
5. `ProjectAssetManager::load_prefab_asset` 只是 generic typed load；全仓没有调用它完成 instantiate、query、update 或 despawn。

### 3.2 Plugin 与 Capability Truth

1. `prefab_tools` package description 宣称 Prefab component、importer 和 instancing services，runtime capability 却是 Partial；实际 register 只有 component descriptor 和 diagnostic-only importer。
2. component descriptor 的两个属性仍是 `prefab: asset_ref` 与 `overrides: json`。descriptor 不是 component storage、resolver、system、instance registry 或 lifecycle owner。
3. builtin 与 plugin duplicate matcher 会被 registry 拒绝，当前错误为 `duplicate importer matcher suffix:.prefab.toml at priority 0`，不能标识现有 owner 与请求 owner。
4. assembly 已把 importer registration errors 纳入 `RuntimeModuleLoadReport`；原来的静默吞错已消失，但装配冲突仍存在。

### 3.3 Document、World 与 Editor Helper

1. `ScenePrefabDocument<R>` 以 `prefab + #[serde(flatten)] _rest` 保存扩展字段，reference mapper 会映射 prefab reference 并保留其余内容；document codec 因而具备无损扩展基础。
2. `World::from_scene_asset` 的 entity construction 不消费 `prefab_instance`；`World::to_scene_asset` 为每个 entity 固定生成 `prefab_instance: None`。World roundtrip 仍会丢数据。
3. 当前没有 non-None Prefab Instance 的 World load-save-reopen test；现有 Scene fixtures 大多显式写 `None`，不能覆盖该断路。
4. editor apply/revert/break helper 都只操作 DTO，不触达 source asset、live World、document transaction、undo/redo、save、runtime update 或 reference remap。

### 3.4 Dynamic Scene 与 Reload

1. `PreparedDynamicSceneSpawn` 可限制 payload bytes，capture target snapshot、compile spawn、构造 isolated preflight World、validate mutation，再以 World 或 Level generation 条件提交。
2. `PreflightedSceneMutation` 和 transaction tests 证明 generic Scene spawn 能做到失败不发布；component/resource codec 与 internal entity references 有可复用处理。
3. `EntityRemap` 仍是 `BTreeMap<EntityId, EntityId>`，没有 source asset/revision、instance ID、generation-qualified handle、reverse index、ownership set 或长期查询入口。
4. prepared/staged spawn 只记录 entity/component/resource count、estimated bytes、target Level 与 expected World generation；没有 artifact/source/override/network/save currentness。
5. Runtime53/99u 已证明 reload queue 有 event/count/bytes/time budget、single-flight、supersede、gap reconciliation 与 target staging；但 applied result仍 append 新实体，不定位或 retire 旧实例。
6. `DynamicScene::from_scene_asset` 先经过 World serializer，因此 Prefab metadata 在 capture/spawn 之前就已丢失。

### 3.5 Network、Save、Script 与 Product

1. net 目录没有 Prefab source/artifact/instance identity、authoritative digest、replicated override delta 或 late-join instance snapshot。
2. Dynamic Scene Session archive 没有 Prefab provenance、instance registry、artifact admission、rebase 或 restore ordering；它只能保存自身 scene/session 语义。
3. script 没有 bounded typed Prefab request facade；现有 gameplay entity spawn 直接面向 World/entity/component mutation。
4. Vampire 与 WOC 没有 `.prefab.toml` source、nested instance、override、update、save/reopen 或 cook/runtime load 案例。WOC 中的 profession/archetype 名词是产品玩法概念，不是 engine Prefab。

## 4. 五套参考实现的可迁移合同

| 参考 | 本轮实际证据 | Zircon 应吸收 | 不应照抄 |
|---|---|---|---|
| Unreal Engine | UObject archetype/CDO resolution、serial cache invalidation、Inheritable Component override template、Component Instance Data phased apply、Level Instance GUID/container/subobject map、diff serialization 与 construction phase | prototype authority、stable object/component identity、reinstance 前后 reference map、分阶段 state capture/apply、rename/reconstruction invalidation | 不复制 UObject 宏、历史兼容层或默认对象成本；目标是等价合同而非类层次 |
| Godot | PropertyUtils 的 native/script/scene inheritance/default precedence；PackedScene/SceneState 保存 node/property/connection/owner/nested instance/edit state，并有 instantiate tests | 完整 compiled graph、owner/connection/nested state、明确 default stack、失败清理与 load/instantiate/save roundtrip | NodePath 可作显示或兼容输入，不能继续作为 Zircon 稳定身份 |
| Fyrox | `InheritableVariable` modified/need-sync bits；只继承未修改字段；Graph/Model 保留 model resource、original handle、instance identity 与 remap | clean/modified provenance、source-object map、instance identity、typed inheritance 与 reference remap | 不把所有 hot field 永久包装成高开销动态容器 |
| Bevy | ScenePatch dependency enumeration/load、独立 resolve 到 `ResolvedSceneRoot`、unresolved spawn fail、失败时 despawn 中间 root、typed entity references | resolve/apply 分层、dependency readiness、cached resolved artifact、原子失败清理 | ScenePatch 仍是演进中的底层 API，不能当作完整 Prefab authoring/rebase 答案 |
| Unity Graphics | VolumeParameter 明确 `overrideState`；VolumeManager 以稳定顺序 flatten parameter list、求 default、override/interpolate/reset stack | 高频 override 的 compact state、稳定顺序、默认层预解析和增量 reset 思路 | Volume 是渲染参数域类比，不证明通用对象拓扑、Prefab identity 或实例传播 |

组合后的最低合同是：Unreal 的 prototype/reinstance 身份，Godot 的完整 scene state，Fyrox 的 modified inheritance，Bevy 的 resolve/apply transaction，以及 Unity Graphics 的紧凑 override hot path。Zircon 必须在自己的 runtime/plugin/World/network/save/streaming 边界上完成这些合同。

## 5. 唯一 Owner 与目标架构

Editor44 继续唯一拥有 authoring defaults、typed override、apply/revert/reset/break、传播 UX 和 5 个父 P0；Editor41 拥有 Scene/Level Instance 无损持久化；Runtime04/99m/99w 拥有通用 asset/resource/registry；Runtime05/99i/99j 拥有 World/ECS；Runtime53/99u 拥有 Dynamic Scene reload；Runtime08E 拥有网络底座；Plugin01 拥有 package/capability/ABI。Runtime128 只拥有 Prefab runtime compiler、artifact、instance registry、transaction、rebase/update、lifecycle、streaming/network/save/script adapters 与资格门。

建议唯一 runtime owner 与产物：

```text
PrefabRuntimeService
  PrefabCompiler
  PrefabArtifactStore
  PrefabInstanceRegistry
  PrefabInstantiationTransaction
  PrefabUpdateCoordinator
  PrefabStreamingAdapter
  PrefabNetworkAdapter
  PrefabSaveAdapter
  PrefabScriptFacade

ResolvedPrefabArtifactV1
  source_id + source_revision + source_digest
  engine/schema/catalog/plugin/build/target compatibility key
  stable object/component graph + deterministic creation order
  compact typed component/resource payloads
  internal/external/soft/asset/entity relocation table
  typed parameter/default schema + compact override program
  nested dependency DAG + budget estimates

PrefabInstanceSnapshotV1
  instance_id + owner/world/level + generation
  artifact identity + installed revision
  source-object -> generation-qualified runtime entity map
  ownership set + root + lifecycle
  effective override digest + provenance/conflict state
  pending update + network/save epochs
```

Runtime hot path只消费 resolved artifact 和 compact typed delta，不解析 TOML/JSON/path，不遍历 authoring inheritance chain。Compiler/cook 负责 source validation 和 migration；runtime 只在 compatibility/currentness admission 后 instantiate 或原子替换。

## 6. P1 状态：Authority、Import、Compiler 与 Artifact

| Finding | 状态 | 当前源码裁决 | 目标合同 |
|---|---|---|---|
| PRF-P1-001 | Open | builtin 与 plugin 仍同时声明 `.prefab.toml` | 选择唯一 importer owner，删除另一条写入 authority |
| PRF-P1-002 | Open | 两者 priority 同为 0，冲突诊断不含双方 ID | catalog/admission 返回双方 owner、matcher、priority 与 resolution |
| PRF-P1-003 | Closed | assembly 收集 `asset_importer_errors` 并扩展 load report；相关吞错搜索为 0 | 保持错误传播为 mandatory contract |
| PRF-P1-004 | Open | plugin 无 instancer factory/system/health，却宣称 instancing services | capability 必须由真实 factory、system、health、E2E receipt 证明 |
| PRF-P1-005 | Open | Partial plugin 仍发布 importer/component 可见表面 | 分离 Declared/Registered/Operational/Qualified，入口 fail-close |
| PRF-P1-006 | Open | import 仅 generic TOML deserialize | version/header/unknown field/limit/span/migration 先行 |
| PRF-P1-007 | Partial | 通用 AssetId、URI、project/resource generation 可复用；Prefab source 无 revision/digest | 每次 compile 携 source ID/revision/digest/expected-before |
| PRF-P1-008 | Open | 无 root/parent/cycle/duplicate stable ID/component-owner validation | compiler 构建并验证 deterministic object graph |
| PRF-P1-009 | Open | 无 nested Prefab dependency DAG 或 cycle chain | import/cook/runtime 共享 versioned DAG admission |
| PRF-P1-010 | Partial | 通用 target mode、capability 与 component registry admission 存在；Prefab compile 未消费 | provider/codec/plugin/platform 在 mutation 前 fail-close |
| PRF-P1-011 | Open | 无 deterministic Prefab compiler 或 artifact byte contract | 同输入得到相同 bytes、diagnostics、digest |
| PRF-P1-012 | Partial | 通用 artifact store 与 staged failure non-publication 可复用；无 Prefab DDC/LKG generation | compile failure 保留同源 last-good 并发布 stale/rollback receipt |
| PRF-P1-013 | Open | cache 仍镜像 authoring DTO | 发布独立 resolved artifact，runtime 禁止重解析 source Scene |
| PRF-P1-014 | Open | path 仍承担 entity/property identity | stable namespaced object/component/property IDs，path 仅展示 |
| PRF-P1-015 | Open | remap 仍为裸 EntityId 对 | object ID 映射 generation-qualified runtime entity handle |
| PRF-P1-016 | Open | 无 Prefab instance stable ID | owner+slot+generation，并拒绝 stale/cross-world handle |
| PRF-P1-017 | Open | 无 source-to-instance reverse index | source/revision/world/level 分区、bounded、可恢复索引 |
| PRF-P1-018 | Open | 无 instance ownership set | 保存 root 与完整 entity/component/resource ownership |
| PRF-P1-019 | Partial | Dynamic Scene 能 remap 一部分 internal entity refs；无完整分类与长期 artifact table | commit 前解析 internal/external/soft/asset/entity refs |
| PRF-P1-020 | Open | 无 nested instance namespace/map | 每层保留 source/instance namespace、parent 与 local mapping |
| PRF-P1-021 | Open | exposed properties 仍是 `Vec<String>` | stable ID/type/unit/constraint/default/alias/codec version schema |
| PRF-P1-022 | Open | artifact 无 topology operations | 表达 add/remove/reparent/reorder/component replace 与 policy |
| PRF-P1-023 | Partial | Scene document 可 opaque 保留扩展字段，Dynamic Scene 会拒绝 unknown codec；恢复链未证明 | opaque 无损保留、禁止执行、安装 provider 后确定恢复 |
| PRF-P1-024 | Partial | Dynamic Scene 检查 World/component/schema/resource generations；无完整 artifact compatibility key | 校验 engine ABI、schema、plugin/build set、target、endianness |

## 7. P1 状态：Instantiation、Identity、Lifecycle 与 Failure Atomicity

| Finding | 状态 | 当前源码裁决 | 目标合同 |
|---|---|---|---|
| PRF-P1-025 | Open | 没有公开 Prefab instantiate request | request 含 artifact、owner/world/level、transform、parameters、policy、budget |
| PRF-P1-026 | Partial | Dynamic Scene 会预编译 target IDs 与 mutation；没有 instance reservation/registry | 先 reserve instance/entity identities，失败统一释放 |
| PRF-P1-027 | Open | remap 只作为调用返回值 | commit 原子安装 mapping，并随 instance generation 查询 |
| PRF-P1-028 | Partial | generic preflight/stage/commit 已分层；无 construction/activation/publication lifecycle | Allocate -> Construct -> ResolveRefs -> Validate -> Activate -> Publish |
| PRF-P1-029 | Open | Scene resource writes 无 Prefab ownership/share policy | 禁止实例隐式覆盖 World singleton |
| PRF-P1-030 | Open | 无 nested root/local/world transform 与 attach policy | transform/parent/keep-world golden matrix |
| PRF-P1-031 | Open | 无 request token、dedupe、retry、cancel | 每 request 唯一 terminal receipt，retry 幂等 |
| PRF-P1-032 | Open | 无 instance lifecycle | Requested/Resolving/Staging/Active/Updating/Retiring/Failed/Removed |
| PRF-P1-033 | Open | 无 despawn instance API | quiesce、detach refs、remove ownership、release leases、tombstone |
| PRF-P1-034 | Partial | generic Dynamic Scene transaction 有 failure non-publication tests；无 Prefab registry/resource fault matrix | 每阶段 fault 后 World/registry/leases/counters 逐项不变 |
| PRF-P1-035 | Partial | current commit 检查 Level/World 及多个 schema/registry generation；不检查 Prefab artifact/source/override/net/save epochs | CAS admission 覆盖所有输入 generation |
| PRF-P1-036 | Partial | Dynamic Scene 有 typed apply/failure/stale report 与 remap；无 instance completion event | receipt 含 mapping、cost、diagnostics、correlation/instance IDs |

## 8. P1 状态：Defaults、Override、Propagation、Rebase 与 Update

| Finding | 状态 | 当前源码裁决 | 目标合同 |
|---|---|---|---|
| PRF-P1-037 | Open | override 仍为 path/path/raw JSON | runtime 只接 typed、stable-addressed、versioned operations |
| PRF-P1-038 | Open | operation 无 base value/revision/hash | expected base 支持 stale/conflict 分类 |
| PRF-P1-039 | Open | 无 default layer digest | artifact 固化 native/script/class/prefab/variant 结果与 fingerprint |
| PRF-P1-040 | Open | helper 每次 BTreeMap 整理字符串 key | compiler 生成 sorted compact delta、bitset、relocation |
| PRF-P1-041 | Open | 无 Inherited/Local/Transient/Conflict/Orphan 状态 | per-field provenance 与 modified bit |
| PRF-P1-042 | Open | 无 Prefab source publication/update listener | typed change set 查询 instance reverse index |
| PRF-P1-043 | Open | Scene reload 仍是 append-spawn | Prefab-specific update coordinator，禁止复用 append 语义 |
| PRF-P1-044 | Open | 无 old base/new base/local delta 三方 rebase | 输出 clean/kept/conflict/orphan/type mismatch |
| PRF-P1-045 | Open | 无 stable-ID topology rebase | object/component add/remove/reparent 按 policy 合并 |
| PRF-P1-046 | Open | 无 live instance replacement transaction | stage generation、迁移状态、CAS publish、retire old set |
| PRF-P1-047 | Open | 无 physics/script/AI/network transient migration policy | component adapter 明确 copy/reset/recreate/reject |
| PRF-P1-048 | Partial | generic stage/commit failure不会发布；成功 reload 不替换旧实体，也无 Prefab LKG | 任一步失败保留旧 active instance/artifact 并记录 degraded state |

## 9. P1 状态：World、Streaming、Network、Save、Script 与 Product

| Finding | 状态 | 当前源码裁决 | 目标合同 |
|---|---|---|---|
| PRF-P1-049 | Open | World load 忽略、save 固定清空 `prefab_instance` | 消费 Editor41/44 无损 codec gate；不能保留时拒绝保存 |
| PRF-P1-050 | Open | instance 无 World/Level/Cell owner | stream out 自动 retire 并保存必要状态 |
| PRF-P1-051 | Open | 无 unloaded consumer artifact manifest | load 前升级 required generation 或阻断 |
| PRF-P1-052 | Open | 无 cross-level Prefab reference policy | hard/soft/streaming refs 有 lease、timeout、teardown order |
| PRF-P1-053 | Open | net 无 Prefab identity/digest | authoritative spawn 复制 source/artifact/instance identity |
| PRF-P1-054 | Open | client 无 artifact/plugin/schema admission | 不兼容时实体生成前拒绝 |
| PRF-P1-055 | Open | 无 replicated override delta | authority/visibility/order/reliability/late join/rollback 明确 |
| PRF-P1-056 | Open | save/session 无 Prefab provenance | 保存 instance/source revision、typed delta、runtime state、migration |
| PRF-P1-057 | Open | 无 Prefab restore ordering | resolve -> reserve IDs -> instantiate -> save delta -> activate |
| PRF-P1-058 | Open | script 无 Prefab facade | bounded typed request，禁止 raw DTO/registry mutation |
| PRF-P1-059 | Open | Vampire/WOC 均无真实 Prefab | 两产品建立 nested/override/update/save/reopen/cook 案例 |
| PRF-P1-060 | Open | server/headless 无 Prefab contract | 同 artifact 裁剪 render-only payload，保留 gameplay/net/save semantics |

## 10. P1 状态：Performance、Budget、Observability、Tests 与 Qualification

| Finding | 状态 | 当前源码裁决 | 目标合同 |
|---|---|---|---|
| PRF-P1-061 | Open | source DTO 内嵌完整 Scene，runtime 无 resolved compact payload | cook 后使用 SoA/component batches 与共享 immutable pages |
| PRF-P1-062 | Partial | Dynamic Scene 可生成 prepared mutation并缓存部分 codec/generation facts；每次 Prefab spawn仍不存在 | artifact 预解析 storage/codec slots，按 catalog generation 缓存 |
| PRF-P1-063 | Open | remap 仍为临时 BTreeMap | bounded contiguous mapping，另保留 debug projection |
| PRF-P1-064 | Partial | Dynamic Scene 有 payload/target/count/bytes/time limits；没有 per-Prefab owner/source quota | per-world/level/owner/source count/entity/component/memory/time limits |
| PRF-P1-065 | Partial | reload queue 已有 batching、supersede、cancel 与 stale coalescing；语义仍是 Scene append | Prefab change fan-out 按 priority/deadline/backpressure 预算 |
| PRF-P1-066 | Partial | 通用 artifact chunk residency 有 lease/high-water/eviction facts；Prefab artifact未接入 | refcount/lease/LRU/pin/prefetch 与 eviction telemetry |
| PRF-P1-067 | Open | 无 instance health snapshot | counts/lifecycle/age/generation/update/conflict/last failure |
| PRF-P1-068 | Open | generic reports有局部耗时/bytes，但 compile/load/instance/update/despawn 无共享 correlation | 全阶段 source/instance/correlation trace |
| PRF-P1-069 | Partial | 已有 DTO/plugin/editor helper 与 generic Dynamic Scene transaction tests；无 Prefab E2E | 补 contract/migration/fault/net/save/stream/product E2E |
| PRF-P1-070 | Partial | 有 ignored 8,192 override microbenchmark及通用 Scene 规模测试；无 10万实例/深嵌套/storm/churn Prefab 基准 | 建立 release benchmark matrix 与原始 receipt |
| PRF-P1-071 | Open | 无 Windows/Linux Prefab artifact determinism | 同输入产生相同 bytes/order/receipt/digest |
| PRF-P1-072 | Open | 无同语义跨引擎性能证据 | 同内容/硬件/质量测 CPU/RSS/load/update/frame spike |

P1 汇总：**Open 53、Partial 18、Closed 1**。Closed 只表示原 finding 本身关闭，不表示其所在链路完成。

## 11. P2 状态：完整性与长期演进

| Finding | 状态 | 后续要求 |
|---|---|---|
| PRF-P2-001 | Open | variant/derived Prefab chain 与 source DAG 可视化 |
| PRF-P2-002 | Open | parameter collection、preset、批量 instance binding |
| PRF-P2-003 | Open | instance pooling，保持 generation/reset/ownership 正确 |
| PRF-P2-004 | Open | component runtime-state migration adapter 与版本协商 |
| PRF-P2-005 | Open | 大型 Prefab subtree/cluster streaming 与独立 retire |
| PRF-P2-006 | Open | network predicted spawn、confirmation map、rejection rollback |
| PRF-P2-007 | Open | cross-project/package identity remap receipt |
| PRF-P2-008 | Open | source revision timeline、runtime provenance query、debug overlay |
| PRF-P2-009 | Open | content-addressed duplicate Prefab artifact dedupe |
| PRF-P2-010 | Open | plugin custom relocation、migration、rebase policy |
| PRF-P2-011 | Open | World Partition/HLOD/instancing 消费 cluster metadata |
| PRF-P2-012 | Open | server shard instance ownership transfer/handoff |
| PRF-P2-013 | Open | mod sandbox type/resource/budget admission |
| PRF-P2-014 | Open | machine-readable dependency/fan-out/conflict/performance report |
| PRF-P2-015 | Open | legacy path/JSON migration、loss report、read-only quarantine |
| PRF-P2-016 | Open | Unreal/Godot/Fyrox/Bevy 可比 Prefab corpus 与长期看板 |

P2 汇总：**16 Open、0 Partial、0 Closed**。

## 12. Hard Cutover 规则

1. `.prefab.toml` 只能有一个 authoritative importer；builtin 与 plugin 二选一，禁止用 priority 或注册顺序维持双轨。
2. 保留当前 importer error propagation；duplicate matcher 必须升级为包含双方 owner/ID 的 admission diagnostic。
3. 在 instancer factory、runtime system、health 与 product E2E 存在前，`prefab_tools` 不得宣称 instancing service，入口必须 Unavailable 或不可执行。
4. legacy `PrefabAsset`、path address 与 raw JSON override 只能作为迁移输入；resolved artifact、save、network payload 不得继续写这种 authority。
5. World roundtrip 父 P0 未关闭前，包含 Prefab instance 的 Scene save 必须 fail-close，禁止继续静默写 `None`。
6. Dynamic Scene 保留为底层 transaction primitive；Scene reload 不得改名包装为 Prefab hot reload。
7. Instance Registry 上线后，一次性 `EntityRemap` 只作 transaction receipt 视图；长期查询必须使用 instance ID + generation。
8. update 必须 replace/rebase 既有 generation 并 retire old ownership；禁止 append 新实体后留下旧实例。
9. network/save/script 只能走 typed Prefab runtime facade，不得构造 authoring DTO、display path 或直接改 registry。
10. product E2E、fault gates、cross-platform determinism 与同语义 benchmark 未通过前，maturity 不得升 Stable，也不得宣称优于 Unreal。

## 13. 重构里程碑

### M0 · Truth Freeze 与父 P0 封口

冻结唯一 importer/runtime owner；让 duplicate owner 诊断包含双方 ID；修复或阻断 World roundtrip 数据损失；按真实能力降级 plugin；建立最小真实 Prefab corpus。

### M1 · Stable Source Schema 与 Deterministic Compiler

定义 source/object/component/property identity、revision/digest、typed parameter/override、nested DAG、migration 与 deterministic diagnostics；legacy 进入只读 quarantine。

### M2 · Resolved Artifact、Compatibility 与 LKG

发布 creation order、compact payload、relocation、dependency、budget 与 compatibility key；实现 DDC/LKG、atomic publication 和 cross-platform digest。

### M3 · Instance Registry 与 Transaction

实现 generation-qualified instance identity、persistent mapping、ownership、deferred construction、fault cleanup、despawn 与 terminal receipt。

### M4 · Override、Rebase 与 Live Update

实现 compact typed delta、provenance/modified bits、三方 conflict/orphan 分类、topology merge、runtime state adapter、CAS replace 与 rollback。

### M5 · World、Streaming 与 Product Host

接入 Level/Cell lifecycle、unloaded manifest、cross-level refs、server/headless，并让 Vampire/WOC 使用真实 Prefab source。

### M6 · Network、Save 与 Script

完成 artifact digest admission、replicated spawn/update、late join、save/restore migration 和 bounded script facade。

### M7 · Scale、Observability 与 Failure

完成 quota、residency、health/trace、fault matrix、source storm、stream churn、10万实例与深嵌套 benchmark。

### M8 · Hard Cutover 与 Competitive Qualification

删除双 importer、伪 capability、legacy write authority 与 append-update；通过产品、跨平台和同场景跨引擎基准后再提升 maturity。

依赖顺序为 `M0 -> M1 -> M2 -> M3 -> M4 -> M5 -> M6 -> M7 -> M8`。M0 前不得开放 Prefab Scene 保存，M2 前不得发布 runtime instantiation，M3 前不得接 network/save，M4 前不得声称 hot reload，M7 前不得调整 Stable maturity。

## 14. 验收门当前状态

| Gate | 状态 | 当前裁决 |
|---|---|---|
| PRF-GATE-01 | Fail | catalog 装配后仍有 builtin/plugin 两个 `.prefab.toml` owner |
| PRF-GATE-02 | Partial | duplicate matcher 会拒绝并进入 load error，但诊断缺双方 ID，startup fatal 产品证据未冻结 |
| PRF-GATE-03 | Pass | importer merge errors 进入 load report，相关忽略结果路径搜索为 0 |
| PRF-GATE-04 | Fail | 缺 instancer 时 capability 仍为 Partial 且发布组件/导入器表面 |
| PRF-GATE-05 | Fail | 只有 TOML/基本路径校验，无完整 version/size/graph/dependency matrix |
| PRF-GATE-06 | Fail | 无 deterministic Prefab compiler/artifact |
| PRF-GATE-07 | Partial | generic staged failure不发布，Prefab same-source LKG/receipt 不存在 |
| PRF-GATE-08 | Fail | rename/reparent 后 path override 无 stable ID |
| PRF-GATE-09 | Fail | 无 nested Prefab DAG/cycle chain |
| PRF-GATE-10 | Partial | document opaque 保留与 unknown codec fail-close 可复用；provider install 后恢复未证明 |
| PRF-GATE-11 | Partial | Dynamic Scene 检查部分 generations；缺完整 artifact compatibility key |
| PRF-GATE-12 | Fail | instantiate 后无 instance ID/artifact revision/ownership query |
| PRF-GATE-13 | Partial | generic Dynamic Scene 有 failure atomicity tests；Prefab registry/lease fault matrix不存在 |
| PRF-GATE-14 | Partial | internal entity remap 有底座；完整 reference class relocation matrix不存在 |
| PRF-GATE-15 | Fail | 无 nested transform/attach/keep-world golden matrix |
| PRF-GATE-16 | Fail | 无 idempotent request/cancel terminal receipt |
| PRF-GATE-17 | Fail | 无 despawn/retire/stale instance handle |
| PRF-GATE-18 | Fail | Scene document可保留，World roundtrip仍清空 instance |
| PRF-GATE-19 | Fail | override 仍是 raw JSON best effort |
| PRF-GATE-20 | Fail | 无 clean-field propagation 与 provenance |
| PRF-GATE-21 | Fail | 无 deterministic three-way conflict artifact |
| PRF-GATE-22 | Fail | 无 orphan classification，path 可误命中新对象 |
| PRF-GATE-23 | Fail | 无 topology/nested rebase |
| PRF-GATE-24 | Fail | reload append 实体，不 CAS replace/retire |
| PRF-GATE-25 | Fail | 无 transient state adapter matrix |
| PRF-GATE-26 | Partial | generic reload queue满足部分 bytes/time/cancel/coalescing；Prefab fan-out与无半发布未证明 |
| PRF-GATE-27 | Fail | 无 stream out/in Prefab identity/state |
| PRF-GATE-28 | Fail | 无 cross-level Prefab reference lease/timeout |
| PRF-GATE-29 | Fail | network spawn 无 artifact digest admission |
| PRF-GATE-30 | Fail | late join 无 instance map/override/lifecycle snapshot |
| PRF-GATE-31 | Fail | SaveGame 无 Prefab restore transaction |
| PRF-GATE-32 | Fail | script 无 typed bounded Prefab request |
| PRF-GATE-33 | Fail | server/headless Prefab payload contract不存在 |
| PRF-GATE-34 | Fail | Vampire/WOC 均无真实 Prefab product E2E |
| PRF-GATE-35 | Fail | 无 10万实例 steady-state hot-path qualification |
| PRF-GATE-36 | Partial | generic reload/artifact metrics存在；instance/update跨阶段 health/trace不存在 |
| PRF-GATE-37 | Fail | 无 Windows/Linux artifact determinism receipt |
| PRF-GATE-38 | Fail | 无 process crash/provider unload generation recovery |
| PRF-GATE-39 | Fail | 无 legacy path/JSON migration receipt |
| PRF-GATE-40 | Fail | 无同内容、同硬件、同质量跨引擎 benchmark |

Gate 汇总：**31 Fail、8 Partial、1 Pass**。

## 15. 首个实施切片

首个切片只能做 M0，不能跳到 instancer demo：

1. 建立 catalog truth RED test：同一最终 runtime profile 对 `.prefab.toml` 只能解析出一个 owner；冲突诊断必须包含 existing/requested importer IDs、matcher、priority、target 与 plugin provenance。
2. 选择并冻结唯一 importer owner，硬删除另一条 duplicate registration；同步 package description、capability status 与 executable entry truth。
3. 为 non-None `prefab_instance` 建立 `SceneAsset -> World -> SceneAsset` RED roundtrip；在 codec 未完成前先 fail-close save，禁止继续静默写 `None`。
4. 建立最小 corpus：单 root、父子层级、typed component、internal ref、nested Prefab、local override、missing provider、cycle、oversize 与 legacy path/JSON migration fixture。
5. 输出 M1 source schema、deletion matrix 和 compiler contract，再开始实现；不得以 editor helper 或 Dynamic Scene append-spawn 冒充 M0 完成。

## 16. 产出记录

- 审查结论：**Runtime Prefab 产品仍未形成**；当前只有 authoring DTO、generic importer/cache、plugin descriptors/editor DTO helpers 与 Dynamic Scene transaction substrate。
- 新增本地 P0：**0**；继续消费 Editor44/41 的 Prefab authoring/World persistence 父 P0。
- P1 当前状态：**53 Open、18 Partial、1 Closed**；唯一 Closed 为 importer merge error propagation。
- P2 当前状态：**16 Open**。
- Gate 当前状态：**31 Fail、8 Partial、1 Pass**。
- 保留：project document extension preservation、asset generation/capability/error report、Dynamic Scene bounded compile/preflight/commit、reload queue budget、generic artifact residency。
- 收敛：唯一 Prefab authority、resolved artifact、Instance Registry、typed override/rebase、atomic replacement、network/save/streaming/script adapters。
- 删除或隔离：duplicate importer、伪 instancing capability、path/raw JSON write authority、World 静默清空、append-spawn 式伪 hot reload。
- 动态验证：本轮未执行；实施按 M0-M8 顺序补 RED contract、migration、fault、product、cross-platform 与 competitive benchmark evidence。
