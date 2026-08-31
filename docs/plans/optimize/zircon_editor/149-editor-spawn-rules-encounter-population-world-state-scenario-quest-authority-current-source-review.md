---
title: Editor Spawn Rules、Encounter、Population、World State、Scenario、Quest、Authority 与 Simulation 当前源码复核
category: zircon_editor
report_id: Editor149
review_date: 2026-08-26
baseline_head: 166720dcb59c57fb4b33c34b859dc1a3f572b222
verification_head: 166720dcb59c57fb4b33c34b859dc1a3f572b222
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor28
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/28-spawn-rules-encounter-population-world-state-scenario-quest-flag-authority-simulation-authoring-review.md
  - docs/plans/optimize/zircon_editor/86-editor-spawn-rules-encounter-population-world-state-scenario-quest-flag-authority-simulation-authoring-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/102-editor-spawn-rules-encounter-population-world-state-scenario-quest-authority-current-source-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/gameplay/workbench_extension_spawn_rules_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/gameplay/workbench_extension_world_state_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/index/workbench_extension_module_workspaces.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
  - zircon_runtime/src/scene/dynamic_scene/remap.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn
  - zircon_runtime/src/scene/dynamic_scene/spawn_task
  - zircon_runtime/src/scene/dynamic_scene/asset_reload
  - zircon_runtime/src/scene/dynamic_scene/scene_asset/prepared_spawn.rs
  - zircon_runtime/src/scene/dynamic_scene/scene_asset/dynamic_scene.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host
  - zircon_runtime/src/core/runtime/state_machine
  - zircon_runtime/src/core/runtime/handle/states.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/99i-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99j-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99z-runtime-wide-state-next-state-transition-hook-history-schedule-scope-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zb-runtime-gameplay-framework-game-instance-world-context-level-game-mode-game-state-local-player-controller-pawn-possession-spawn-travel-network-save-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zc-runtime-prefab-archetype-prototype-class-default-instance-override-runtime-instantiation-propagation-hot-reload-network-save-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zd-runtime-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/138-editor-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/142-editor-ai-behavior-tree-blackboard-perception-eqs-state-tree-smart-object-debug-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/143-editor-gameplay-ability-effect-attribute-set-gameplay-tags-tag-query-cue-prediction-debug-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/146-editor-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/148-editor-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/World.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/GameStateBase.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/MassGameplay/Source/MassSpawner/Public/MassSpawnerTypes.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/MassGameplay/Source/MassSpawner/Public/MassSpawnerSubsystem.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/MassGameplay/Source/MassSpawner/Private/MassSpawnerSubsystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/WorldPartition/DataLayer/DataLayerSubsystem.h
  - dev/bevy/crates/bevy_scene/src/spawn.rs
  - dev/bevy/crates/bevy_scene/src/spawn_system.rs
  - dev/bevy/crates/bevy_state/src/state/resources.rs
  - dev/bevy/crates/bevy_state/src/state/transitions.rs
  - dev/bevy/crates/bevy_state/src/state/sub_states.rs
  - dev/bevy/crates/bevy_state/src/state/computed_states.rs
  - dev/godot/modules/multiplayer/multiplayer_spawner.h
  - dev/godot/modules/multiplayer/multiplayer_spawner.cpp
  - dev/godot/modules/multiplayer/tests/test_multiplayer_spawner.h
  - dev/Fyrox/fyrox-impl/src/resource/model/mod.rs
  - dev/Fyrox/editor/src/scene/commands/graph.rs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Data/VFXDataSpawner.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Expressions/VFXExpressionSpawnerState.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/NewCompiler/Tasks/SpawnerTask.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Models/Blocks/Implementations/Spawn/VFXSpawnerPeriodicBurst.cs
  - dev/Graphics/Tests/SRPTests/Packages/com.unity.testing.visualeffectgraph/Tests/Runtime/VFXSpawnerTests.cs
finding_status:
  p0_open: 5
  p1_open: 52
  p1_partial: 8
  p1_closed: 0
  p2_open: 12
gate_status:
  fail: 32
  partial: 0
  pass: 0
---

# Editor149 · Spawn Rules / World State / Scenario / Authority 当前源码复核

## 1. 结论

当前Editor仍没有Spawn Rules、Encounter/Population、World State、Scenario或Quest Flag产品authority。两张可达Gameplay Workspace继续固定显示`SpawnRules_Enemy`、`Zone_A`、`Condition_Night`、`Tag_Combat`、18 rules、12 zones、96 spawns、`Seed: 2026`、Server/Client Preview/Offline，以及`Scenario_NightRaid`、`Layer_Global`、`Alarm.Active=true`、Weather/AI/Quest、84 keys、6 layers、42 events与`Authority: Server`。Simulate/Validate没有创建document、compiler、job、isolated world、runtime request或receipt，只返回queued/fixed文本。

Runtime底层有真实进展，但仍不能改名为产品。DynamicScene已经具备World/schema/component registry/change-tick currentness fence、隔离preflight、fallible reflected-write validation、compact commit artifact、no-fail publication、bounded payload/target snapshot、异步cancel、asset reload revision/generation gap、ready-result bytes和resident queue预算。`World`/ECS也已有stable external entity ID、slot generation、typed/dynamic component、deferred structural command与change detection。这些是Spawn Runtime可以复用的执行substrate，不包含Spawn source、semantic compiler、instance registry、owner lease、whole-instance lifecycle或terminal receipt。

脚本旁路仍然危险。`zr.zircon.gameplay`用同一个`gameplay.entity` capability覆盖transform、component、combat、`spawn_empty`、`spawn_model`和`despawn`；后三者直接在mutable World上调用`spawn_node`/`remove_entity`，接受或返回裸`i64/u64`。它没有principal、self/owned/admin scope、authority lease、World generation、quota、cancel或typed outcome，不能成为未来Spawn Authority Service。

全production Rust对`SpawnDefinitionDocument`、`SpawnRuleDocument`、`CompiledSpawnPlanArtifact`、`SpawnAuthorityService`、`SpawnInstanceRecord`、`SpawnReceipt`、`WorldStateSchemaDocument`、`ScenarioDefinition`、`ScenarioInstanceId`、`WorldStateTransaction`、`WorldStateChangeSet`、`EncounterDirector`、`PopulationDirector`和`QuestFlagSchema`精确扫描仍为零命中。Editor28账本因此保持：5项P0 Open；60项P1为52 Open/8 Partial/0 Closed；12项P2 Open；32门全部Fail。没有同语义、同硬件、同规模的原始benchmark receipt，不能声称优于Unreal。

## 2. 冻结范围与方法

### 2.1 当前工作树选择集

本报告读取当前工作树，以`166720dcb59c57fb4b33c34b859dc1a3f572b222`标记提交基线。共享工作树存在其他会话修改，本轮不回退、不覆盖、不暂存。物理行按文件读取；Zircon tests只统计Rust `#[test]`，ignored统计`#[ignore...]`；fingerprint由排序后的相对路径与逐文件SHA-256聚合。

| 选择集 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---:|
| Editor Gameplay product surface | **9 / 2,946 / 2,733 / 158,519 / 0 / 0** | `48b4c65dcc71199665a630012c728c2f23e76803a70aa5fcb0547cf60f780864` |
| DynamicScene spawn/reload substrate | **30 / 6,835 / 6,226 / 241,280 / 39 / 4** | `5760d7a5e774810e513545661226e2105920501a940ed95f21373f4825bdc951` |
| Script gameplay host | **16 / 2,900 / 2,750 / 106,641 / 15 / 0** | `c0f63f60791e0d4658cf4f0856be215495d86582aa5accc8c20e908693669fd8` |
| Generic Runtime state contrast | **14 / 931 / 810 / 28,631 / 2 / 0** | `474606dfea341c344abe9b023c6eb306d67f2e6b7d857e41d2f1ec8a1fb4bd37` |
| Zircon selected union | **69 / 13,612 / 12,519 / 535,071 / 56 / 4** | `a8048a4949ce31ad5d302249359dcf54d68a36fc420685867a757a9fe9828e89` |

### 2.2 参考源码选择集

22个参考文件均存在，共11,820行、9,842非空行、499,904 bytes，fingerprint为`e9d0cba336f3be17723a931fe677e3291a778f7b8d08c6908988b360b3deb3ab`。其中有2个Rust `#[test]`、6个Godot `TEST_CASE`和23个Unity `[UnityTest]`，这里只计静态marker，不声明运行。Godot revision为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`，Bevy为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`，Fyrox为`8d815db36494f1badb347547dfc7094bf4fbbdf8`，Unity Graphics为`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal以vendored文件fingerprint冻结。

### 2.3 动态证据边界

本轮没有运行Cargo、Editor、Spawn/WorldState compiler、Preview/PIE、server/client、save/load、fault injection、100K/1M scale、soak、profile或跨引擎benchmark。现有56个Zircon test attribute只表示静态测试深度，不能声明通过。Tooling按用户要求排除。

## 3. 当前产品事实

### 3.1 Workbench仍是第二authority

| 事实 | 当前证据 | 产品后果 |
|---|---|---|
| 两张领域ZUI | 各230行、27个node、19条route、0 provider；合计460行/54 nodes/38 routes | 业务事实由模板默认值产生，不绑定source/provider generation |
| Binding | Spawn和World State各20条，共40条 | 只把control event映射为字符串route，不建立typed command |
| Navigation | 只切tab/row selection、popup与command | selected状态不是document/runtime truth |
| Field edit | 只写control `value`/`value_text`并refresh | 没有dirty revision、undo、save、conflict或validation |
| Feedback | 固定96 spawns、18 rules/1 conflict、42 events、84 keys/1 conflict | 没有job、artifact、world、authority、trace或terminal outcome |

这两张surface可以保留布局壳，但M0必须把业务rows与成功语义切为Unavailable。继续添加固定condition、authority、count或queued分支只会扩大迁移成本。

### 3.2 DynamicScene不是Spawn Instance Registry

`CompiledSceneSpawn`绑定expected World generation、schema catalog generation、component registry generation和target change tick，保存`EntityRemap`、`NodeRecord`、component/resource writes、descriptor与preview。preflight把fallible writes应用于隔离World，再提取`PreparedSceneSpawnCommit`；真正commit前复验currentness并发布。prepared/task和reload队列还提供payload/target bytes、worker cancel、revision stale/superseded、generation gap、count/time/bytes与resident result预算。

但公开成功结果仍是：

```text
EntityRemap { BTreeMap<EntityId, EntityId> }
```

它没有source/artifact revision、stable `SpawnInstanceId`、owner/instigator/authority、entity ownership set、lifecycle、lease generation、whole-instance despawn/reload、query cursor或`SpawnReceipt`。Asset reload仍以scene spawn/apply为中心，不具instance-aware keep/replace/patch/drain/reject语义。把`EntityRemap`或reload result重命名为Spawn Instance会掩盖这些合同缺失。

### 3.3 Generic state machine不是World State

原`core::framework::state`已经移动到`core::runtime::state_machine`，但语义仍是process-local `TypeId + Any` state registry和caller-driven facade。排除测试与facade后，App/World/Session/Editor/plugin/script/network/save没有product caller；没有World/Session/Player scope、stable schema/key、authority、replication/save policy、transaction CAS、bounded cursor journal或Scenario lifecycle。文件移动不改变Editor28的World State判定。

### 3.4 Script仍绕过authority

`spawn_empty`和`spawn_model`直接在`LevelSystem::with_world_mut`中创建Entity并逐项写transform/name/model/material/JSON script bindings；中途失败没有Spawn产品级instance/compensation receipt。`despawn`直接删除任意解析成功的raw EntityId。capability只证明调用者拥有宽泛`gameplay.entity`字符串，不证明其是实体owner、server authority或受预算的Spawn producer。

## 4. Owner边界与目标合同

| 领域 | 唯一owner | Editor149职责 |
|---|---|---|
| ECS/World identity与structural transaction | Runtime ECS/World owners | 只消费generation-safe request/receipt，不直接持有mutable World |
| Spawn source/compiler/runtime instance/authority | 新Runtime Gameplay Spawn owner | typed document、compile/simulate orchestration与projection |
| World State schema/runtime/Scenario transaction | 新Runtime Gameplay State owner | typed key/scope/transition authoring与observation |
| Prefab/archetype construction | Runtime99zc +对应Editor owner | Spawn artifact只持stable source reference与revision |
| Script capability | Runtime Script/Game framework owner | scoped request/query/observe adapter，不直接spawn/despawn |
| Network/late join | Runtime network + Editor148 | 消费stable instance/state change artifact，不复制schema |
| Save/restore | Runtime99zd + Editor146 | 通过participant保存schema/generation/linkage，不另建存储 |
| Partition/AI/Tags/Weather/Quest | 各自owner | 注册typed condition/action/key contributor |
| Document/job/diagnostic | Editor02/09/11/25 | transaction、currentness、cancel、journal和trace基础 |

目标必须分成两条唯一数据链：

```text
SpawnDefinitionDocument
  -> SpawnSemanticCompiler
  -> CompiledSpawnPlanArtifact
  -> SpawnAuthorityService
  -> DynamicScene/ECS commit substrate
  -> SpawnInstanceRecord + SpawnReceipt

WorldStateSchemaDocument + ScenarioDefinition
  -> WorldStateSemanticCompiler
  -> CompiledWorldStateProgram
  -> WorldStateRuntimeService
  -> WorldStateTransaction + ChangeSet + bounded observation
  -> Spawn/AI/Weather/Quest/Network/Save adapters
```

### 4.1 最低合同

| 合同 | 必须包含 |
|---|---|
| `SpawnDefinitionDocument` | document/schema/source revision、stable rule/region/condition/composition ID、typed reference、policy与unknown preservation |
| `CompiledSpawnPlanArtifact` | artifact/compiler/source/dependency revisions、deterministic digest、runtime payload、cost estimate、diagnostics与compatibility |
| `SpawnRequest` | request ID、artifact、qualified World/Level、authority context、owner lease、seed/clock、budget、deadline/cancel与expected generations |
| `SpawnInstanceRecord` | stable instance ID、artifact/source provenance、owner/authority、entity set、lifecycle、lease generation与network/save linkage |
| `SpawnReceipt` | admission、terminal outcome、instance/entity set、seed、cost、partial/deferred/rejected reason、correlation与single-terminal proof |
| `WorldStateSchemaDocument` | stable schema/key IDs、type、namespace/scope/default/validator、authority/replication/save/redaction/migration policy |
| `WorldStateTransaction/ChangeSet` | transaction/idempotency key、expected generation、ordered mutations、before/after、cause/authority、events、diagnostics与cursor sequence |
| `ScenarioDefinition/Instance` | stable state/transition/action IDs、guards、entry/exit/timeout/failure/cancel、clock/seed/source、authority与terminal receipt |

## 5. P0 currentness重判

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| P0-1 | **Open** | Spawn Rules继续固定rule/zone/conflict/count，Simulate/Validate无provider。 | M0降为Unavailable；M1-M3接document/compiler/job/runtime receipt后恢复。 |
| P0-2 | **Open** | World State继续固定key/layer/scenario/authority/events。 | 建typed schema/runtime provider；无provider时禁止写入、模拟和成功语义。 |
| P0-3 | **Open** | DynamicScene成功仍只返回`EntityRemap`，无instance lifecycle/owner/receipt。 | Runtime建立stable instance registry、owner lease、whole-instance lifecycle/query与terminal receipt。 |
| P0-4 | **Open** | `gameplay.entity`允许裸ID直接spawn/despawn任意World实体。 | 拆observe/self/owned/spawn/admin capability，统一typed service、generation、rate/bytes/entity/CPU budget。 |
| P0-5 | **Open** | Simulate/Validate没有shared compiler、isolated World、deterministic trace或server + N clients证明。 | Preview/PIE/Shipping消费同artifact，输出source-qualified trace、World generation与terminal receipt。 |

## 6. P1 currentness状态

Canonical语义与编号继续由Editor28/86拥有。当前源码只强化已有8个Partial，没有足够证据新增Partial或关闭finding。

| IDs | 状态 | 当前证据与目标 |
|---|---|---|
| P1-1..P1-17 | **Open** | Spawn source/rule/region/condition/composition、reference、compiler/artifact、seed/clock、budget、request、instance/owner identity均缺失。 |
| P1-18 | **Partial** | DynamicScene已有compile -> isolated preflight -> compact no-fail publish；仍缺Spawn construction/initialize/observer/ownership barrier。 |
| P1-19 | **Partial** | generic Scene batch具原子publication；仍无Spawn产品的bounded partial/streamed policy与逐项outcome。 |
| P1-20..P1-21 | **Open** | task/report不是SpawnReceipt，也没有whole-instance despawn request/receipt。 |
| P1-22..P1-23 | **Partial** | reload有bounded staging/stale/superseded/gap/result bytes，World还有generation/change-tick fence；仍无instance-aware reconcile、quiesce/drain/teardown barrier。 |
| P1-24..P1-25 | **Open** | pooling/reuse identity与按instance/rule/region/owner的分页snapshot/cursor缺失。 |
| P1-26 | **Partial** | count/time/payload/target/ready/resident bytes与profile counter可复用；无per-owner产品quota、完整fault/soak和同语义benchmark。 |
| P1-27..P1-50 | **Open** | World State schema/key/type/scope/layer/CAS/change journal/computed state、Scenario lifecycle、contributors、Save/Network、Editor document/preview/trace均缺失。 |
| P1-51 | **Partial** | DynamicScene task有status/cancel，reload有revision currentness；领域compile/simulate仍未接Editor09 job和single terminal receipt。 |
| P1-52..P1-57 | **Open** | LKG/hot reload、typed Tag/cross-domain contributor、partition/network/save与scoped script adapter缺失。 |
| P1-58..P1-59 | **Partial** | 56个selected Rust test与generic profile/budget/probe可复用；没有domain compiler/determinism/authority/product/scale matrix。 |
| P1-60 | **Open** | 两张ZUI、40条领域binding、route/fixed feedback仍在production；目标链闭合后必须零引用硬删除。 |

汇总：**52 Open / 8 Partial / 0 Closed**；Partial仅为**18、19、22、23、26、51、58、59**。

## 7. P2 currentness状态

| IDs | 状态 | 后续专项 |
|---|---|---|
| P2-1..P2-2 | **Open** | Encounter/Population Director、partition-aware population lease与streaming协调。 |
| P2-3..P2-4 | **Open** | 长期生态/人口模拟、deterministic generator/constraint solver/cache。 |
| P2-5..P2-6 | **Open** | predicted spawn/rollback与cross-server/shard authority handoff。 |
| P2-7..P2-8 | **Open** | Scenario record/replay/branch/diff与stable-ID semantic merge。 |
| P2-9..P2-10 | **Open** | platform/mode/difficulty/DLC overlay compilation与provenance-bound analytics。 |
| P2-11..P2-12 | **Open** | signed/budgeted mod contributor SDK与distributed deterministic simulation farm。 |

## 8. 参考引擎差异

| 参考 | 已验证的工程合同 | Zircon应吸收 | 适用限制 |
|---|---|---|---|
| Unreal World/GameState | Spawn parameters含collision/name/owner/remote ownership/deferred construction；World分离AuthorityGameMode和replicated GameState，GameState有server time/begun-play replication。 | qualified World/authority、deferred construction、server truth/client projection、typed lifecycle与time domain。 | 不照搬Actor/UObject类层次。 |
| Unreal MassSpawner | entity template、typed transform/spawn data generator、creation context、subsystem-owned batch spawn/destroy与initializer。 | compiled composition、batch creation context、subsystem owner、cost/budget和instance/provenance receipt。 | Mass entity handle本身不替代Spawn Rule document/Scenario。 |
| Unreal DataLayer | stable layer instance、runtime/effective state与state-changed event，World拥有DataLayer manager。 | World State scope/layer identity、effective provenance与owner-qualified change event。 | DataLayer不是通用Quest/Scenario store。 |
| Bevy SceneSpawner | `InstanceId`、queued dynamic/scene spawn、instance readiness/despawn和mapping；mutation通过command/system阶段。 | stable logical instance、waiting queue、deferred apply、instance lifecycle和World-bound identity。 | Bevy scene instance不包含Zircon所需authority/receipt全部语义。 |
| Bevy State | typed `State/NextState`、transition schedule、SubStates/ComputedStates和依赖关系。 | typed schema、deterministic transition order、computed dependency/cycle治理与scope cleanup。 | Zircon不复制silent last-writer或弱receipt语义。 |
| Godot MultiplayerSpawner | spawn path、spawnable scenes、custom spawn、tracked node、spawn limit与replication integration。 | declared source allowlist、tracked instance、network owner、explicit limit与despawn lifecycle。 | Godot API是网络spawner，不覆盖完整Encounter/World State authoring。 |
| Fyrox | model instance保存resource/provenance与mapping；Editor graph command明确execute/revert/finalize。 | stable source provenance、instance mapping与transactional Editor mutation/undo。 | 不是大规模population或authority模板。 |
| Unity Graphics VFX | spawner state/compiler task、loop/delay/burst、spawn count/delta/total time及23个runtime UnityTest。 | bounded state machine、compiled task、time/burst semantics和可重复测试模式。 | VFX spawner不是gameplay authority，不能冒充Spawn Rules。 |

## 9. Currentness资格门

| Gates | 状态 | 当前依据 |
|---|---|---|
| G01-G04 | **Fail** | 默认入口仍显示fixture；无provider/registry/conformance/Unavailable闭环。 |
| G05-G08 | **Fail** | stable source identity、shared compiler/artifact、reference currentness与deterministic simulation缺失。 |
| G09-G14 | **Fail** | Spawn admission、atomic construction、single terminal、instance despawn/reload与script authority safety缺失。 |
| G15-G24 | **Fail** | World State schema/CAS/layer/computed/Scenario/journal/security/network/save/partition lifecycle缺失。 |
| G25-G29 | **Fail** | isolated Preview、server + N clients、trace/currentness与bounded observation缺失。 |
| G30-G32 | **Fail** | 1/1K/100K产品矩阵、平台/fault lane和长期soak缺失。 |

汇总：**32 Fail / 0 Partial / 0 Pass**。DynamicScene primitive的unit test不提升任何端到端产品gate。

## 10. 分层重构路线

1. **M0 Truthfulness / Inventory**：两张Workspace删除固定业务rows/success，缺provider显式Unavailable；冻结40 binding、route、feedback和脚本旁路删除清单。
2. **M1 Source Contracts**：建立stable document/rule/region/schema/key/scenario/action identity、typed value/reference、scope/layer/authority与migration，先写round-trip/rename/reorder/delete RED tests。
3. **M2 Shared Compiler / Artifacts**：Runtime-owned compiler产canonical artifact/digest/dependency/cost/diagnostics/compatibility；Editor、Preview、PIE、cook、shipping只消费同一产物。
4. **M3 Spawn Authority / Instance**：建立typed admission、owner lease、deferred initialize、atomic publish、stable instance/entity ownership、single terminal receipt、whole-instance despawn/reload/query。
5. **M4 World State / Scenario Runtime**：建立scope/layer resolver、CAS/idempotent transaction、bounded cursor journal、computed state、security projection与Scenario clock/action/terminal lifecycle。
6. **M5 Transactional Editor**：接Editor document/history/save/conflict、schema inspector、graph/table、diff/navigation、LKG/currentness；领域字段禁止写control-local truth。
7. **M6 Isolated Simulation / PIE**：复用Preview World/Time和Editor148的server + N clients topology，输出deterministic trace、timeline、overlay、cancel/currentness receipt。
8. **M7 Script / Network / Save / Partition**：脚本改scoped request/query/observe并删除直接spawn/despawn；接stable wire/save artifact、late join、migration、cell unload/drain与cross-cell ownership。
9. **M8 Hard Cutover / Failure / Scale**：旧Workspace/binding/feedback/raw authority零引用删除，通过fault、100K、redaction、queue/journal bound、Windows与soak资格。
10. **M9 Encounter / Population / Competition**：建立Director、长期simulation、prediction/rollback、shard handoff与simulation farm，再做同语义Unreal对照。

## 11. 性能与规模资格

目标不是让缺功能的路径比Unreal耗时更低。所有性能结论必须绑定source/artifact/compiler/runtime/hardware/topology digest，并同时保留正确性与失败语义。

| 维度 | 必须报告 |
|---|---|
| Compile | rule/key/transition规模、dependency count、cold/warm latency、allocation、artifact bytes与diagnostic count |
| Spawn | 1/1K/100K entity、steady/burst、P50/P95/P99 admission/preflight/publish、CPU/RSS/alloc与rejection reason |
| State | 1/1K/100K keys/changes、CAS contention、computed fan-out、journal bytes/lag/gap与observer cost |
| Network/Save | baseline/delta bytes、late join、instance/state linkage、capture/restore latency与migration failure atomicity |
| Lifecycle | cancel/timeout/reload/unload/crash/restart、instance/queue/journal retention、24h+ soak与orphan count |
| Comparison | 同场景、同hardware、同角色/功能、同采样窗口、raw trace/artifact及tail latency，不只给平均值 |

## 12. 禁止的临时修补

- 禁止继续在ZUI写固定rule/zone/key/event/count、Server字符串、queued/succeeded feedback或editable authority。
- 禁止把DynamicScene、`EntityRemap`、generic StateRegistry、Session Archive、VFX spawner或Prefab placement改名成Spawn Instance/World State/Scenario。
- 禁止用`HashMap<String, Value>`、raw JSON、裸entity ID、全局bool、control value或registration order承担stable schema/authority/protocol。
- 禁止让Editor、script或Workbench直接`spawn_node`/`remove_entity`绕过compiler、lease、budget、transaction和journal。
- 禁止另写Preview/cook/shipping compiler，或长期保留old/new双轨与compatibility facade。
- 禁止以append-spawn冒充reload/reconcile，以EntityId list冒充instance ownership，以task ready冒充terminal receipt。
- 禁止复制Network、Save、Partition、Tag、AI、Weather或Quest owner；只允许typed adapter/contributor。
- 禁止无界queue/journal/history、silent partial success、late result覆盖新generation、cancel后发布success或teardown后写复用World。
- 禁止用无网络、无保存、无失败注入、低entity数量或静态Preview的低CPU/RSS声称优于Unreal。

## 13. 本轮完成定义

本轮完成Editor28/86/102 current-source刷新：冻结69个Zircon selected文件、13,612行、535,071 bytes、56个Rust test attributes和4个ignored declarations；冻结22个Unreal/Bevy/Godot/Fyrox/Unity Graphics参考文件、11,820行、499,904 bytes。5项P0保持Open；P1保持52 Open/8 Partial/0 Closed，Partial编号18、19、22、23、26、51、58、59；12项P2保持Open；32门全部Fail；canonical finding delta为零。

本轮只修改review与导航索引，不修改Runtime、Editor、App、plugin、ABI、测试或产品资源；没有运行Cargo或动态产品资格，也没有查询、轮询、等待或实时跟踪协调器。实现状态仍为pending，后续源码修正必须从M0 truthfulness和M1 source contract RED evidence开始。
