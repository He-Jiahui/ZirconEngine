---
title: Editor Spawn Rules、Encounter、Population、World State、Scenario、Quest、Authority 与 Simulation 当前源码复核
category: zircon_editor
report_id: Editor102
review_date: 2026-08-26
baseline_head: 3282dfad2a3a0dce246dfa8f300d7d30d70ed9a9
baseline_epoch: 524
canonical_owner: Editor28
refreshes:
  - docs/plans/optimize/zircon_editor/28-spawn-rules-encounter-population-world-state-scenario-quest-flag-authority-simulation-authoring-review.md
  - docs/plans/optimize/zircon_editor/86-editor-spawn-rules-encounter-population-world-state-scenario-quest-flag-authority-simulation-authoring-product-integration-current-source-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/gameplay/workbench_extension_spawn_rules_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/gameplay/workbench_extension_world_state_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/index/workbench_extension_module_workspaces.zui
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
  - zircon_runtime/src/scene/dynamic_scene
  - zircon_runtime/src/script/vm/gameplay_host
tests:
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_scene.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_scene/asset_reload_selection_status.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/spawn_transform.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08g-gameplay-ability-effect-attribute-tag-cue-prediction-runtime-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_editor/21-gameplay-ability-effect-attribute-tag-cue-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/24-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-review.md
  - docs/plans/optimize/zircon_editor/26-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/World.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/GameStateBase.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/MassGameplay/Source/MassSpawner/Public/MassSpawnerTypes.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/MassGameplay/Source/MassSpawner/Public/MassSpawnerSubsystem.h
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
  - dev/Graphics/Tests/SRPTests/Packages/com.unity.testing.visualeffectgraph/Tests/Runtime/VFXSpawnerTests.cs
doc_type: current_source_refresh
review_status: complete
implementation_status: pending
source_recheck_required: true
finding_status:
  p0: 5 open
  p1: 52 open
  p1_partial: 8
  p2: 12 open
gate_status:
  fail: 32
  partial: 0
  pass: 0
---

# Editor28/102 · Spawn Rules、Encounter、Population、World State、Scenario、Quest 与 Authority 当前源码复核

## 1. 结论

当前 Editor 仍不存在 Spawn Rules、Encounter/Population、World State 或 Scenario 产品 authority。两张可达 Gameplay Workspace 固定显示 `SpawnRules_Enemy`、`Zone_A`、`Condition_Night`、`Tag_Combat`、18 rules、12 zones、96 spawns、`Seed: 2026`、Server/Client Preview/Offline，以及 `Scenario_NightRaid`、`Layer_Global`、`Alarm.Active=true`、Weather/AI/Quest、84 keys、6 layers、42 events、`Authority: Server`。Simulate/Validate 不创建 document/compiler/job/preview world/runtime request，只写 queued/fixed feedback。

Runtime 不是空壳。`World` 有稳定外部 `EntityId`、slot/generation、typed component/resource registry、deferred structural commands、events/observers 和 world generation；`DynamicScene` 有 capture/compile/preview、generation/schema/resource stale fence、bounded isolated preflight、fallible reflected write validation、一次性 publication、异步 prepared spawn、cancel、asset reload queue、revision stale/superseded 与限额。这些只能作为通用 Scene mutation substrate，不能被改名为 Spawn Rule 或 World State。

真正的产品合同仍为零：selected source 对 `SpawnDefinitionDocument`、`SpawnRuleDocument`、`CompiledSpawnPlanArtifact`、`SpawnAuthorityService`、`SpawnInstanceRecord`、`SpawnReceipt`、`WorldStateSchemaDocument`、`ScenarioDefinition`、`ScenarioInstanceId`、`WorldStateTransaction`、`WorldStateChangeSet`、`EncounterDirector`、`PopulationDirector`、`QuestFlagSchema` 没有实现。DynamicScene 公开结果仍是 `EntityRemap`；它没有 source/artifact revision、stable instance ID、owner/instigator/authority、whole-instance despawn、lifecycle、lease、cursor 或 receipt。

脚本侧更危险：`gameplay.entity` capability 同时覆盖 transform、component、combat、`spawn_empty`、`spawn_model` 和 `despawn`；spawn 直接调用 `World::spawn_node`，despawn 直接 `World::remove_entity`，接口接收/返回裸 `i64/u64` entity。没有 self/owned/admin 分层、authority lease、generation、rate/budget 或 typed request/receipt，不能作为 Spawn Runtime service。

因此 Editor86 的账本只做 current-source 重判，不关闭任何根问题：5 项 P0 全 Open；60 项 P1 中 52 Open、8 Partial、0 Closed；12 项 P2 全 Open；32 个 Gate 全 Fail。8 个 Partial 只归因于 DynamicScene 的通用事务、generation fence、bounded queue、异步 cancel、reload/revision 和局部性能探针，不能证明 Gameplay domain 已完成。当前没有同场景/同硬件 benchmark receipt，不能声称优于 Unreal。

## 2. 物理冻结与证据

统计递归展开本报告 `related_code`/`tests`；路径按正斜杠排序，逐文件 SHA-256 后计算集合 fingerprint。test count 是静态声明，不是执行通过数。

| 范围 | 文件 | 行 | 非空行 | bytes | tests | fingerprint |
|---|---:|---:|---:|---:|---:|---|
| Zircon selected union | 42 | 10,671 | 9,841 | 447,961 | 80 | `626266c287feb757b3f25cd3e6a0d899e8b68842449d01c988ec4f12a477b2a1` |
| Unreal/Bevy/Godot/Fyrox/Unity reference | 22 | 11,820 | 9,842 | 499,904 | 10 | `65a5c7d812096afcc2d6d29e2f36281f1015ba719096da582851eb8055bfb4f7` |

两张 Workspace 各 230 行、19 条 route、22 个 event；gameplay binding 为 235 行，navigation spec 为 310 行，feedback 为 87 行。固定 token 命中集中在这五个 Editor 文件；`module_field_edit.rs` 仅把输入写回控件 `value`/`value_text` 并刷新模板。selected DynamicScene/runtime 文件中，`CompiledSceneSpawn` 的 target generation 只保护 World/schema/component registry/change tick，不能替代 source artifact identity；`PreparedSceneSpawnCommit` 只携 remap/records/component descriptors。

动态边界：本轮没有运行 Cargo、Editor、Spawn/WorldState compiler、PIE、多客户端、save/load、fault、soak 或 100K probe；报告只声明静态源码事实。共享工作树 dirty，实施前必须重取 scope/fingerprint。

## 3. 参考引擎对照

- Unreal `World`/`GameStateBase` 将 world lifecycle、server authority、replicated client state、time 与 actor spawn 分开；MassSpawner 有 entity config、transform/data generator、批量 creation context 和可治理的 spawn subsystem；DataLayer 有 stable layer identity、effective state 与 state-changed event。
- Godot `MultiplayerSpawner` 至少定义 spawn path、spawnable scene、custom spawn、tracked node 与 spawn limit；Zircon 当前 `spawn_model` 尚无网络/ownership/limit contract。
- Bevy `Commands`/`SceneSpawner` 将 deferred mutation、scene asset/instance、waiting queue 分开；State/NextState/SubStates/ComputedStates 定义 typed transition，而不是任意字符串表。
- Fyrox 的 model instance provenance/stable ID 与 graph command/undo 是实例追踪和编辑事务参考；Unity Graphics VFX spawner 只作为 bounded spawn-state/compiler/task 的局部参考，不能替代 gameplay authority。

## 4. Owner 边界与目标架构

| 领域 | owner | Editor28 只能提供/消费 |
|---|---|---|
| ECS identity、DynamicScene transaction | Runtime Scene/ECS | 编译 artifact 的执行 substrate，不向 UI 暴露 mutable World |
| Spawn definition/compiler/runtime instance/authority | 新 Runtime Gameplay Spawn owner | typed document、artifact、request、instance、receipt |
| World State schema/runtime/scenario transaction | 新 Runtime Gameplay State owner | typed key/scope/transition/observation |
| Script authority/capability | Runtime07/08G | scoped spawn/despawn request、owner lease、rate/budget |
| Replication/late join | Runtime08E + Editor26 | 消费 Spawn/WorldState change set，不拥有 schema |
| AI/Tag/Weather/Quest contributors | 各 Runtime/Editor owner | 注册 typed condition/action/key adapter |
| Document/save/job/diagnostic | Editor02/04/09/10/11 | transactional authoring、compile/simulate job、receipt |
| World Partition/PIE/SaveGame | Editor16/26/24 + Runtime owners | 提供 isolated world、network/save participant，不复制 state authority |

目标链必须分成两条：

```text
SpawnDefinitionDocument -> SpawnSemanticCompiler -> CompiledSpawnPlanArtifact
  -> SpawnAuthorityService -> DynamicScene/ECS commit -> SpawnInstanceRecord + Receipt

WorldStateSchemaDocument + ScenarioDefinition -> WorldStateCompiler
  -> WorldStateRuntime -> WorldStateTransaction/ChangeSet
  -> Spawn/AI/Weather/Quest/Network/Save adapters
```

核心合同至少要有：`SpawnDefinitionDocument { document_id, schema_version, source_revision, rule_sets, regions, conditions, compositions, policies }`；`CompiledSpawnPlanArtifact { artifact_id, compiler_version, source_digest, dependency_revisions, deterministic_digest, runtime_payload, diagnostics }`；`SpawnRequest { request_id, artifact_id, target_world, authority_context, seed, budget, expected_world_generation, owner_lease }`；`SpawnInstanceRecord { instance_id, artifact_id, owner, authority, entity_set, lifecycle, lease_generation }`；`WorldStateSchemaDocument { schema_id, version, typed_keys, scopes, defaults, validators, replication_policy, save_policy }`；`WorldStateTransaction/ChangeSet { transaction_id, expected_generation, ordered_mutations, cause, before, after, events, diagnostics }`。

## 5. P0：必须先停掉假产品与危险旁路

| ID | 现状 | 必须重构 |
|---|---|---|
| P0-1 | Spawn Rules 固定 rule/zone/conflict/96 spawns，Simulate/Validate 无 provider | 立即显示 Unavailable，删除固定业务结果，接入 document/compiler/job 前禁止成功语义 |
| P0-2 | World State 固定 key/layer/scenario/authority/42 events | 禁用写入/模拟/验证；typed schema/runtime provider 未就绪时 fail closed |
| P0-3 | DynamicScene 只有 EntityRemap，没有 instance lifecycle/owner/receipt | 增加 stable instance、source artifact、lease、whole-instance despawn/reload、terminal receipt |
| P0-4 | `gameplay.entity` 可用裸 ID 直接 spawn/despawn 任意 World entity | 拆 observe/self/owned/admin capability，加入 authority/generation/rate/budget，统一 typed service |
| P0-5 | Simulate/Validate 没有 isolated world、共享 compiler、deterministic trace 或 server/client 证明 | 统一 preview/PIE/Runtime artifact，输出可复现 trace、artifact、world generation 和 terminal receipt |

## 6. P1：编译、实例、状态与授权差异

| ID | 差异与重构要求 | ID | 差异与重构要求 |
|---|---|---|---|
| P1-1 | Spawn document/schema/version/source revision 缺失 | P1-2 | typed rule/region/condition/composition graph 缺失 |
| P1-3 | asset/reference dependency identity 缺失 | P1-4 | semantic compiler/diagnostic span 缺失 |
| P1-5 | immutable compiled artifact/digest 缺失 | P1-6 | deterministic seed/clock/compiler version 缺失 |
| P1-7 | condition/action registry 无 capability/version | P1-8 | cross-domain Tag/AI/Weather/Quest adapter 缺失 |
| P1-9 | spawn budget/count/area/density/rate limits 缺失 | P1-10 | collision/nav/occupancy reservation 缺失 |
| P1-11 | placement transform/formation/sampling contract 缺失 | P1-12 | asset/model/material/script binding typed resolution 缺失 |
| P1-13 | batch streaming/chunk/queue backpressure 缺失 | P1-14 | failure atomicity/partial result/compensation 缺失 |
| P1-15 | source/artifact/world/schema generation fence 不完整 | P1-16 | SpawnRequest expected generation/lease 缺失 |
| P1-17 | stable SpawnInstanceId/owner/instigator/authority 缺失 | P1-18 | entity set/query cursor/whole-instance lifecycle 缺失 |
| P1-19 | despawn/respawn/reload/migrate state machine 缺失 | P1-20 | ownership transfer/lease expiry 缺失 |
| P1-21 | spawn receipt/audit/journal correlation 缺失 | P1-22 | runtime observation/debug snapshot 缺失 |
| P1-23 | schema typed key/namespace/scope 缺失 | P1-24 | default/validator/constraint/migration contract 缺失 |
| P1-25 | WorldState authority/replication/save policy 缺失 | P1-26 | ordered transaction/precondition/conflict resolution 缺失 |
| P1-27 | generation-aware ChangeSet/event cursor 缺失 | P1-28 | Scenario definition/instance/transition graph 缺失 |
| P1-29 | clock/seed/deadline/failure policy 缺失 | P1-30 | quest flag typed contributor/ownership 缺失 |

## 7. P1：Editor 产品、Runtime 集成与质量门

| ID | 差异与重构要求 | ID | 差异与重构要求 |
|---|---|---|---|
| P1-31 | authoring document/history/undo/save/recovery 缺失 | P1-32 | asset catalog/factory/toolkit/provider/catalog closure 缺失 |
| P1-33 | Simulate job admission/progress/cancel/shutdown 缺失 | P1-34 | isolated preview World/clock/subsystem/session 缺失 |
| P1-35 | PIE server+N clients/network profile 缺失 | P1-36 | Runtime apply/acknowledgement/receipt projection 缺失 |
| P1-37 | dynamic scene instance result 不能回投 Editor | P1-38 | world partition/data layer activation adapter 缺失 |
| P1-39 | save participant/checkpoint/migration adapter 缺失 | P1-40 | AI/Tag/Weather/Quest typed contributor registry 缺失 |
| P1-41 | script capability self/owned/admin boundary 缺失 | P1-42 | raw entity ID 改为 generation-safe handle/instance reference |
| P1-43 | component/resource mutation 与 spawn authority 未分权 | P1-44 | rate/bytes/entity/queue/CPU budget telemetry 缺失 |
| P1-45 | large population spatial index/query pagination 缺失 | P1-46 | deterministic replay/trace/failure fixture 缺失 |
| P1-47 | late join/replication snapshot/change set 缺失 | P1-48 | rollback/prediction/authority handoff 缺失 |
| P1-49 | diagnostics/error severity/span/correlation 缺失 | P1-50 | offline/unknown/stale/degraded UI 状态缺失 |
| P1-51 | semantic diff/merge/conflict UI 缺失 | P1-52 | validation result/artifact provenance 缺失 |
| P1-53 | performance baseline/throughput/tail/RSS/alloc qualification 缺失 | P1-54 | 100K/1M population scale fixture 缺失 |
| P1-55 | soak/long-lived scenario/respawn leak qualification 缺失 | P1-56 | crash/restart/idempotency recovery 缺失 |
| P1-57 | security/permission/secret boundary 缺失 | P1-58 | plugin/mod schema compatibility/migration 缺失 |
| P1-59 | unique authority/navigation hard-cutover 缺失 | P1-60 | fixture zero-reference/schema deletion gate 缺失 |

其中 P1-1..P1-30 为 8 项 Partial 之外的核心缺口；8 个 Partial 仅是通用 DynamicScene/runtime substrate 的局部可复用能力，不能上调为 domain closure。

## 8. P2：规模与高级能力

| ID | 缺口 | 方向 |
|---|---|---|
| P2-1 | 多 provider/mixed authority | capability adapter composition |
| P2-2 | world cell/streaming population | partition-aware instance lease |
| P2-3 | 长期生态/人口模拟 | bounded director/retention/LOD |
| P2-4 | procedural constraint solver | deterministic compiled sampling |
| P2-5 | prediction/rollback spawn | network authority integration |
| P2-6 | cross-server handoff | instance transfer receipt |
| P2-7 | Scenario record/replay/branch | immutable trace and comparison |
| P2-8 | semantic merge/team collaboration | typed change set/lock |
| P2-9 | authoring variants/platform/difficulty | overlay compilation/policy |
| P2-10 | analytics/tuning loop | provenance-bound telemetry |
| P2-11 | mod/plugin extension | versioned contributor SDK |
| P2-12 | distributed simulation farm | deterministic remote attempt/artifact |

## 9. 32 个 Gate 当前状态

Spawn schema/compiler/artifact、instance lifecycle、authority/lease、WorldState typed schema、transaction/change set、Scenario transition、Editor document/toolkit/catalog、job/cancel、isolated preview、PIE topology、replication/save adapter、script capability safety、diagnostic/currentness、scale/soak、security、migration 和 fixture hard-cutover 共 32 个 gate 全部 Fail。没有 Partial 或 Pass；DynamicScene 的局部 tests 只证明底层机制，不能越过产品 gate。

## 10. 分层重构顺序

1. **Truthfulness cutover**：两张 Gameplay Workspace 的业务 rows、fixed feedback、authority field、Simulate/Validate 先切为 Unavailable；保留布局壳，不再扩展字符串分支。
2. **Runtime contracts**：在 `zircon_runtime` 建 SpawnDefinition/CompiledArtifact/SpawnInstance/Receipt 与 WorldStateSchema/Scenario/Transaction/ChangeSet；补 source/artifact/world/schema generation 和 deterministic seed/clock。
3. **DynamicScene bridge**：让 compiled artifact 通过现有 preflight/commit substrate 执行，增加 instance registry、owner/lease、whole-instance despawn/reload、query cursor、receipt；不要复制第二套 ECS transaction。
4. **World State authority**：建立 typed namespace/scope/default/validator/replication/save policy，按 ordered transaction 产生 generation-qualified ChangeSet；AI/Tag/Weather/Quest 通过 contributor adapter 注册。
5. **Script security**：拆 `gameplay.entity` capability，所有 spawn/despawn 走 typed service，拒绝任意裸 ID、stale handle、越权 owner、超额 count/bytes/CPU/rate。
6. **Editor authoring**：实现 transactional document/toolkit/factory/catalog/provider，Simulate/Validate 进入 Editor09 job，投影 diagnostics/progress/cancel/terminal receipt；Editor 不直接写 runtime World。
7. **Preview/PIE/Save/Network**：安装同一 compiled artifact 到 isolated preview、server+N clients、World Partition 和 Save participant，验证 late join、rollback、reload、crash/restart。
8. **Qualification**：以固定 scene/rules/keys、同 hardware、同 compiler/runtime build 记录 compile latency、spawn throughput/tail、state transaction throughput、CPU/RSS/alloc、queue bound、replication delta、save/restore 和 soak receipt；没有 receipt 就不宣称性能优于 Unreal。

## 11. 禁止的临时修补

- 不得继续给 ZUI 添加固定 rule/zone/key/event/count、Server 字符串、queued/succeeded feedback 或 editable authority。
- 不得用 `HashMap<String, Value>`、JSON 字段、裸 entity `i64`、全局 bool 或 DynamicScene `EntityRemap` 冒充 typed schema、instance、authority 或 receipt。
- 不得让 Editor、脚本或 Workbench 直接 `World::spawn_node`/`remove_entity` 绕过 compiler、lease、budget、transaction 和 journal。
- 不得把 DynamicScene generic transaction、VFX spawner、Prefab placement 或 World Partition layer 改名当作 Spawn Rule/Scenario/World State 完成。
- 不得以 fewer entities、无网络、无保存、无失败注入或静态 preview 的低耗时声称优于 Unreal；所有性能结论必须有同语义可复现 receipt。

## 12. 验证边界

已完成：当前工作树递归枚举、固定事实扫描、Gameplay Host 逐函数检查、DynamicScene spawn/preflight/commit/reload 分层检查、参考路径存在性与 42/22 文件 fingerprint。未运行 Cargo 或动态产品 lane；`source_recheck_required: true` 反映共享 dirty worktree，后续实现前必须重算 selected manifest。Editor102 仅刷新 Editor28/86 的 currentness，不实施生产代码。
