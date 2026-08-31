---
title: Editor Spawn Rules、Encounter、Population、World State、Scenario、Quest、Authority 与 Simulation 当前源码复核
category: zircon_editor
report_id: Editor205
review_date: 2026-08-28
baseline_head: 11cac2d08a891ee92dcc206fd84a2d15f9e1a3f4
verification_head: 11cac2d08a891ee92dcc206fd84a2d15f9e1a3f4
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor28
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/28-spawn-rules-encounter-population-world-state-scenario-quest-flag-authority-simulation-authoring-review.md
  - docs/plans/optimize/zircon_editor/86-editor-spawn-rules-encounter-population-world-state-scenario-quest-flag-authority-simulation-authoring-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/102-editor-spawn-rules-encounter-population-world-state-scenario-quest-authority-current-source-review.md
  - docs/plans/optimize/zircon_editor/149-editor-spawn-rules-encounter-population-world-state-scenario-quest-authority-current-source-review.md
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
  - docs/plans/mvp/index.md
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

# Editor205 · Spawn Rules / World State / Scenario / Authority 当前源码复核

## 1. 结论

当前Editor仍没有Spawn Rules、Encounter/Population、World State、Scenario或Quest Flag产品authority。两张可达Gameplay Workspace继续固定展示`SpawnRules_Enemy`、`Zone_A`、`Condition_Night`、`Tag_Combat`、18 rules、12 zones、96 spawns、`Seed: 2026`、Server/Client Preview/Offline，以及`Scenario_NightRaid`、`Layer_Global`、`Alarm.Active=true`、Weather/AI/Quest、84 keys、6 layers、42 events与`Authority: Server`。Simulate/Validate仍未创建document、compiler、job、isolated world、runtime request或terminal receipt，只返回queued与固定结果文本。

Editor149之后的变化没有建立产品闭环。两张ZUI只迁移间距token、响应式tier和默认选中外观；`spawn_empty`/`spawn_model`开始传播`World::spawn_node`及后续写入错误，但仍直接持有mutable World并返回裸`i64`实体；generic state machine从保存transition列表改为只保留`latest_event`。DynamicScene新增大批量EntityId successor probe、hash-based validation、reload result-size cache、report capacity reuse、task-graph scope cancel等底层改进，但公开成功语义仍是`EntityRemap`，没有stable instance、owner lease、whole-instance lifecycle或Spawn receipt。

本轮还纠正Editor28/149的一项参考表述：当前vendored Bevy `bevy_scene`已经没有旧`SceneSpawner/InstanceId/instance_is_ready/despawn_instance`接口，所列`spawn.rs`与`spawn_system.rs`直接通过`WorldSceneExt`应用Scene或把该调用包装成一次性system。Bevy typed State仍可作为transition schedule参考，但当前源码不能再被引用为stable scene instance lifecycle证据；Zircon所需instance合同应由自身产品需求以及Unreal/Godot/Fyrox的owner、tracked lifecycle和provenance证据支撑。

tracked生产源码与2,293个未跟踪生产Rust/ZUI/配置文件对`SpawnDefinitionDocument`、`SpawnRuleDocument`、`CompiledSpawnPlanArtifact`、`SpawnAuthorityService`、`SpawnInstanceRecord`、`SpawnReceipt`、`WorldStateSchemaDocument`、`ScenarioDefinition`、`ScenarioInstanceId`、`WorldStateTransaction`、`WorldStateChangeSet`、`EncounterDirector`、`PopulationDirector`和`QuestFlagSchema`精确扫描仍为零。Editor28账本保持：5项P0 Open；60项P1为52 Open/8 Partial/0 Closed；12项P2 Open；32门全部Fail。没有同语义、同硬件、同规模、同时保留正确性与失败语义的原始benchmark receipt，不能声称优于Unreal。

## 2. 冻结范围与方法

### 2.1 当前工作树选择集

本报告读取当前共享工作树，以`11cac2d08a891ee92dcc206fd84a2d15f9e1a3f4`标记提交基线。范围内含其他会话在途修改，本轮不回退、不覆盖、不暂存。目录按frontmatter递归展开并按物理路径去重；物理行逐文件读取；tests只统计Rust `#[test]`，ignored统计`#[ignore...]`。fingerprint使用小写正斜杠workspace-relative路径与逐文件SHA-256，组成`path + NUL + hash + LF`清单后再次SHA-256。

| 选择集 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---:|
| Editor Gameplay product surface | **9 / 2,952 / 2,739 / 158,965 / 0 / 0** | `3aefbe9727550b1b2e90a911d988f53f44b5009eb93458bf00fcbe97e16b96cf` |
| DynamicScene spawn/reload substrate | **30 / 6,836 / 6,226 / 241,386 / 39 / 4** | `d7939a7eb58e8b0cd77fe44d4482ce1dba39b144c05942af1489104d4eb33cdf` |
| Script gameplay host | **16 / 2,900 / 2,750 / 106,641 / 15 / 0** | `57b16b36181f4f13629d26f971ace676e9e0f9b3596e8017d09531857c603cee` |
| Generic Runtime state contrast | **14 / 929 / 808 / 28,536 / 2 / 0** | `41738b127c1165d20a69de11bc8df70ede6091bfed53f054b08d0828056f8fd7` |
| Zircon selected union | **69 / 13,617 / 12,523 / 535,528 / 56 / 4** | `a4052e6b6096afa15f8c64ee125083547753013add06d1f57f636499bd089b93` |

与Editor149的物理冻结相比，选择集文件数仍为69，增加5行与457 bytes；大部分1,600行新增/502行删除发生在同一选择集内的重构，另有Runtime Session Archive改动不属于上述Spawn/World State owner，已阅读但不计入产品资格。

### 2.2 参考源码选择集

22个参考文件全部存在，共11,820行、9,842非空行、499,904 bytes，fingerprint为`bf7859e385fe269b1b6605e28220ac8ef49ce42637015641b5fc879cca7e627a`。静态marker包括2个Rust `#[test]`、6个Godot `TEST_CASE`与23个Unity `[UnityTest]`；它们不是运行通过数。Godot revision为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`，Bevy为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`，Fyrox为`8d815db36494f1badb347547dfc7094bf4fbbdf8`，Unity Graphics为`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal以vendored文件内容冻结。

### 2.3 动态证据与MVP边界

本轮没有运行Cargo、Editor、Spawn/World State compiler、Preview/PIE、server/client、save/load、fault injection、100K/1M scale、soak、profile或跨引擎benchmark。56个Zircon test attribute只表示静态测试深度，4个ignored项仍需受管性能lane。`docs/plans/mvp/index.md`当前仍为in progress，因此本轮仅做advanced read-only audit，不提前实现M0-M9。Tooling按用户要求排除。

## 3. Editor149之后的源码变化

| 变化 | 当前事实 | 对Editor28 finding的影响 |
|---|---|---|
| Gameplay ZUI | 两份233行Workspace只增加density token、responsive tier并清理默认checked/selected外观；固定业务rows、authority options与command仍在。 | 不提高P0-1/P0-2；只改善布局壳。 |
| Script spawn error传播 | `spawn_node`、transform、Name、MeshRenderer、dynamic component写入现在使用`?`并返回host error。 | 降低silent write failure，但仍无compensation、request identity、owner、budget或receipt；P0-4保持Open。 |
| Script despawn | `despawn`仍把任意非负`i64`转`u64`并直接`remove_entity`。 | authority旁路未变化。 |
| Runtime state machine | `state_transition_events()`被`latest_state_transition()`替代，machine只存一个`latest_event`。 | 更明确是process-local FSM；不能承担bounded World State journal、cursor或restore。 |
| Entity remap | 16实体以上使用`EntityIdReservationProbe`与successor cache，避免冲突ID线性重复探测。 | 强化P1-26/P1-59共享性能底座；不产生SpawnInstanceId。 |
| Scene validation | source entity和component descriptor唯一性改为hash membership，并加入65,536规模ignored perf证据。 | 强化P1-58/P1-59；不建立domain compiler或artifact。 |
| Reload result/queue | result size缓存、oversize转bounded failure、report Vec容量复用、resident/result/metadata bytes诊断增强。 | 强化P1-22/P1-23/P1-26底座；仍无instance-aware keep/replace/patch/drain/reject。 |
| Spawn task | 接TaskGraph scope、admission error、queued-before-start cancel acknowledgement和取消后禁止publication。 | 强化P1-51共享job/cancel底座；不是SpawnRequest/Receipt。 |
| Session Archive | 路径写入reservation/generation、lineage publication与atomic staging增强。 | 属于Save/Session owner；不能替代World State transaction或Spawn lifecycle。 |

这些变化证明底层工程化仍在推进，也证明必须严格保持owner边界。以“文件名含spawn”“已有generation”“已有archive transaction”为理由把它们改名成Spawn Rules、World State或Scenario，只会形成第二套不兼容authority。

## 4. 当前产品事实

### 4.1 两张Workspace仍是第二authority

| 事实 | 当前证据 | 产品后果 |
|---|---|---|
| 默认可达 | Extension总索引挂载两张Workspace，Assets页提供两个打开按钮。 | 用户会把fixture理解为已安装产品。 |
| 领域ZUI | 两份文件各233行、27个node、19条route、0 provider。 | 业务事实来自模板默认值，不绑定source/provider generation。 |
| Binding | Spawn Rules与World State各20条，共40条。 | 只把control event映射成字符串action。 |
| Navigation | 只切tab/row selection、popup与command control。 | selected状态不是document/runtime truth。 |
| Field edit | 通用handler只改control `value`与`value_text`并refresh。 | 没有document revision、undo、dirty、save、conflict或validation。 |
| Feedback | 固定96 spawns、18 rules/1 conflict、42 events、84 keys/1 conflict。 | 没有job、artifact、world、authority、trace或terminal outcome。 |

M0必须保留布局可复用性，同时把业务rows与成功语义改为明确Unavailable。不能继续向模板添加更多rule、condition、authority、count或queued分支。

### 4.2 没有实施owner

`docs/plans/zircon_editor/editor`当前没有Spawn Rules、World State、Scenario、Quest、Encounter或Population独立实施计划；命中仅来自其他计划中普通“scenario”措辞。Editor02/09、Runtime ECS/DynamicScene、Network、Save、Partition与Script计划都是依赖owner，不自动承担Editor28交付。实现前必须为Runtime Gameplay Spawn与Runtime Gameplay State各建立唯一owner，再由Editor28消费其合同。

## 5. Runtime事实与边界

### 5.1 DynamicScene不是Spawn Instance Registry

`CompiledSceneSpawn`绑定expected World generation、schema catalog generation、component registry generation和target change tick，保存`EntityRemap`、records、component/resource writes、descriptor与preview。preflight在隔离World中执行全部fallible reflected writes，再提取compact `PreparedSceneSpawnCommit`并在发布前复验currentness。task/reload链提供bytes limit、scope cancel、revision stale/superseded/gap、ready/apply time与resident result预算。

公开成功结果仍是：

```text
EntityRemap { BTreeMap<EntityId, EntityId> }
```

它没有source/artifact revision、stable `SpawnInstanceId`、owner/instigator/authority、entity ownership set、lifecycle、lease generation、whole-instance despawn/reload、query cursor或`SpawnReceipt`。新的successor probe优化“怎样选择空EntityId”，不会回答“这些实体属于哪个产品实例、谁有权管理、如何终止”。

### 5.2 Generic state machine不是World State

`core::runtime::state_machine`以`TypeId + Any` registry承载typed process-local state，caller设置`NextState`后显式apply，并按exit/transition/enter顺序调用hook。每类state只保存current、next与一个latest event；事件只有`exited`、`entered`和same-state flag。它没有World/Session/Player scope、stable schema/key ID、authority、replication/save policy、transaction/CAS/idempotency、bounded journal/cursor/gap或Scenario instance lifecycle。

生产代码没有Game/App/World/Session/Editor/plugin/script/network/save消费者把它提升为产品World State。文件从`core::framework::state`移动到`core::runtime::state_machine`、以及删除transition history，不改变Editor28判断。

### 5.3 Script仍绕过未来authority

`zr.zircon.gameplay`仍用一个`gameplay.entity` capability覆盖transform、component、combat、`spawn_empty`、`spawn_model`与`despawn`。spawn直接在active Level的mutable World创建节点并逐项写入，返回`entity as i64`；despawn接受裸ID并直接删除。错误传播虽改善，但中途写入失败仍没有产品级instance compensation、single-terminal receipt或orphan census。

未来脚本adapter必须区分observe、self、owned、spawn与admin scope，并携principal、qualified World generation、owner lease、quota、deadline/cancel与typed outcome。不能在当前helper外再包一层字符串route就宣称完成authority。

### 5.4 全生产语义扫描

14个目标合同类型在tracked生产源码与2,082个未跟踪生产文件均为0。宽语义扫描中：

- `spawn rules`只命中两张Editor surface、route/binding/feedback、Scatter普通文本和结构约束测试；没有Runtime model；
- `spawn_instance`、`ScenarioDefinition/Instance`、`EncounterDirector`与`PopulationDirector`为0；
- `world_state`的Runtime命中只是“ECS world state”说明、测试函数名或变量，不是领域schema/service；
- `quest_flag`只命中World State fixture和一个无关截图请求flag测试。

因此不存在“只是命名不同”的隐藏产品实现。

## 6. 参考引擎对照与纠偏

| 参考 | 当前源码确认的合同 | Zircon应吸收 | 限制/纠偏 |
|---|---|---|---|
| Unreal World/GameState | `FActorSpawnParameters`携owner、instigator、level、collision、name mode、remote-owned与deferred construction；GameState在server/client存在并复制begin-play和server time。 | qualified World/authority、deferred construct/finalize、server truth/client projection、typed lifecycle与time domain。 | 不复制Actor/UObject类层次。 |
| Unreal MassSpawner | entity template、比例/data generator、creation context、batch spawn/destroy与initializer owner明确。 | compiled composition、batch context、subsystem owner、cost/budget与provenance receipt。 | Mass handle不等于Spawn Rules document或Scenario。 |
| Unreal DataLayer | stable DataLayer instance、runtime/effective state、递归set与state-changed event。 | layer identity、effective provenance、World-owned resolver与change event。 | DataLayer不是通用Quest/Scenario store。 |
| Godot MultiplayerSpawner | spawn path、spawnable scene allowlist、custom spawn、tracked node、spawn limit、multiplayer authority检查与despawn signal。 | declared source、tracked lifecycle、network owner、explicit limit与失败拒绝。 | 只覆盖network spawner，不覆盖完整World State authoring。 |
| Bevy Scene current | `WorldSceneExt`直接应用Scene；`SpawnSystem`只把调用包装成schedule system。当前仓无`InstanceId/instance_is_ready/despawn_instance`。 | 借鉴Rust typed World apply与schedule-bound structural mutation。 | 撤回旧报告stable scene instance证据；不能以不存在的旧API支撑结论。 |
| Bevy State | typed `State/NextState`、明确transition schedule、SubStates/ComputedStates与exit/transition/enter顺序。 | typed schema、确定性transition order、computed dependency/cycle治理与scope cleanup。 | 仍不提供Zircon所需authority、journal、save/network receipt。 |
| Fyrox | model instance保留resource provenance与node mapping；Editor graph command有execute/revert/finalize。 | source provenance、instance mapping与transactional Editor mutation/undo。 | 不是大规模population或authority模板。 |
| Unity Graphics VFX | spawner state/compiler task、loop/delay/burst与spawn count/delta/total time，并有23个runtime UnityTest。 | bounded state machine、compiled task、time/burst语义和可重复测试结构。 | VFX spawner不是Gameplay authority，不能冒充Spawn Rules。 |

参考结论必须绑定当前vendored源码。尤其不能因为旧版Bevy曾有`InstanceId`，就在当前报告继续声称该合同已被本地参考文件验证。

## 7. 唯一Owner与目标合同

| 领域 | 唯一owner | Editor205职责 |
|---|---|---|
| ECS/World identity与structural transaction | Runtime ECS/World owners | 只消费generation-safe request/receipt，不直接持有mutable World。 |
| Spawn source/compiler/runtime instance/authority | 新Runtime Gameplay Spawn owner | typed document、compile/simulate orchestration与projection。 |
| World State schema/runtime/Scenario transaction | 新Runtime Gameplay State owner | typed key/scope/transition authoring与observation。 |
| Prefab/archetype construction | Runtime99zc与对应Editor owner | Spawn artifact只持stable source reference与revision。 |
| Script capability | Runtime Script/Game Framework owner | scoped request/query/observe adapter，删除直接spawn/despawn。 |
| Network/late join | Runtime Network与Editor148 | 消费stable instance/state change artifact，不复制schema。 |
| Save/restore | Runtime99zd与Editor146 | 通过participant保存schema/generation/linkage，不另建存储。 |
| Partition/AI/Tags/Weather/Quest | 各自owner | 注册typed condition/action/key contributor。 |
| Document/job/diagnostic | Editor02/09/11/25 | transaction、currentness、cancel、journal与trace基础。 |

目标数据链必须保持唯一：

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

### 7.1 最低合同

| 合同 | 必须包含 |
|---|---|
| `SpawnDefinitionDocument` | document/schema/source revision、stable rule/region/condition/composition ID、typed reference、policy与unknown preservation。 |
| `CompiledSpawnPlanArtifact` | artifact/compiler/source/dependency revision、deterministic digest、runtime payload、cost estimate、diagnostics与compatibility。 |
| `SpawnRequest` | request ID、artifact、qualified World/Level、authority context、owner lease、seed/clock、budget、deadline/cancel与expected generations。 |
| `SpawnInstanceRecord` | stable instance ID、artifact/source provenance、owner/authority、entity set、lifecycle、lease generation与network/save linkage。 |
| `SpawnReceipt` | admission、terminal outcome、instance/entity set、seed、cost、partial/deferred/rejected reason、correlation与single-terminal proof。 |
| `WorldStateSchemaDocument` | stable schema/key ID、type、namespace/scope/default/validator、authority/replication/save/redaction/migration policy。 |
| `WorldStateTransaction/ChangeSet` | transaction/idempotency key、expected generation、ordered mutation、before/after、cause/authority、event、diagnostic与cursor sequence。 |
| `ScenarioDefinition/Instance` | stable state/transition/action ID、guard、entry/exit/timeout/failure/cancel、clock/seed/source、authority与terminal receipt。 |

## 8. P0 currentness重判

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| P0-1 | **Open** | Spawn Rules仍固定rule/zone/conflict/count，Simulate/Validate无provider。 | M0降为Unavailable；M1-M3接document/compiler/job/runtime receipt后恢复。 |
| P0-2 | **Open** | World State仍固定key/layer/scenario/authority/events。 | 建typed schema/runtime provider；无provider时禁止写入、模拟与成功语义。 |
| P0-3 | **Open** | DynamicScene成功仍只返回`EntityRemap`，新remap算法也无instance lifecycle/owner/receipt。 | 建stable instance registry、owner lease、whole-instance lifecycle/query与terminal receipt。 |
| P0-4 | **Open** | `gameplay.entity`虽开始传播写入错误，仍允许裸ID直接spawn/despawn任意World实体。 | 拆observe/self/owned/spawn/admin capability，统一typed service、generation与quota。 |
| P0-5 | **Open** | Simulate/Validate没有shared compiler、isolated World、deterministic trace或server + N clients证明。 | Preview/PIE/Shipping消费同artifact，输出source-qualified trace与terminal receipt。 |

## 9. P1 currentness状态

canonical语义与编号继续由Editor28拥有。当前源码增强既有8个Partial，没有足够证据新增Partial或关闭finding。

| IDs | 状态 | 当前证据与目标 |
|---|---|---|
| P1-1..P1-17 | **Open** | Spawn source/rule/region/condition/composition、reference、compiler/artifact、seed/clock、budget、request、instance/owner identity均缺失。 |
| P1-18 | **Partial** | DynamicScene已有compile -> isolated preflight -> compact no-fail publish；仍缺Spawn construction/initialize/observer/ownership barrier。 |
| P1-19 | **Partial** | generic Scene batch具原子publication；仍无Spawn产品的bounded partial/streamed policy与逐项outcome。 |
| P1-20..P1-21 | **Open** | task/report不是SpawnReceipt，也没有whole-instance despawn request/receipt。 |
| P1-22..P1-23 | **Partial** | reload已有bounded staging、stale/superseded/gap/result bytes与scope cancel；仍无instance-aware reconcile、quiesce/drain/teardown barrier。 |
| P1-24..P1-25 | **Open** | pooling/reuse identity与按instance/rule/region/owner分页snapshot/cursor缺失。 |
| P1-26 | **Partial** | remap successor cache、hash validation、count/time/payload/target/ready/resident bytes可复用；无per-owner产品quota、完整fault/soak或同语义benchmark。 |
| P1-27..P1-50 | **Open** | World State schema/key/type/scope/layer/CAS/change journal/computed state、Scenario lifecycle、contributors、Save/Network、Editor document/preview/trace均缺失。 |
| P1-51 | **Partial** | DynamicScene task有TaskGraph scope、admission error与cancel acknowledgement；领域compile/simulate仍未接Editor09 job与single terminal receipt。 |
| P1-52..P1-57 | **Open** | LKG/hot reload、typed Tag/cross-domain contributor、partition/network/save与scoped script adapter缺失。 |
| P1-58..P1-59 | **Partial** | 56个selected Rust test、4个ignored perf gate与generic profile/budget/probe可复用；没有domain compiler/determinism/authority/product/scale矩阵。 |
| P1-60 | **Open** | 两张ZUI、40条领域binding、route/fixed feedback仍在production；目标链闭合后必须零引用硬删除。 |

汇总：**52 Open / 8 Partial / 0 Closed**；Partial仅为**18、19、22、23、26、51、58、59**。

## 10. P2 currentness状态

| IDs | 状态 | 后续专项 |
|---|---|---|
| P2-1..P2-2 | **Open** | Encounter/Population Director、partition-aware population lease与streaming协调。 |
| P2-3..P2-4 | **Open** | 长期生态/人口模拟、deterministic generator/constraint solver/cache。 |
| P2-5..P2-6 | **Open** | predicted spawn/rollback与cross-server/shard authority handoff。 |
| P2-7..P2-8 | **Open** | Scenario record/replay/branch/diff与stable-ID semantic merge。 |
| P2-9..P2-10 | **Open** | platform/mode/difficulty/DLC overlay compilation与provenance-bound analytics。 |
| P2-11..P2-12 | **Open** | signed/budgeted mod contributor SDK与distributed deterministic simulation farm。 |

## 11. Currentness资格门

| Gates | 状态 | 当前依据 |
|---|---|---|
| G01-G04 | **Fail** | 默认入口仍显示fixture；无provider/registry/conformance/Unavailable闭环。 |
| G05-G08 | **Fail** | stable source identity、shared compiler/artifact、reference currentness与deterministic simulation缺失。 |
| G09-G14 | **Fail** | Spawn admission、atomic construction、single terminal、instance despawn/reload与script authority safety缺失。 |
| G15-G24 | **Fail** | World State schema/CAS/layer/computed/Scenario/journal/security/network/save/partition lifecycle缺失。 |
| G25-G29 | **Fail** | isolated Preview、server + N clients、trace/currentness与bounded observation缺失。 |
| G30-G32 | **Fail** | 1/1K/100K产品矩阵、平台/fault lane与长期soak缺失。 |

汇总：**32 Fail / 0 Partial / 0 Pass**。DynamicScene primitive的局部测试、缓存优化与task cancel不提升任何端到端产品gate。

## 12. 分层重构路线

1. **M0 Truthfulness / Inventory**：两张Workspace删除固定业务rows/success，缺provider显式Unavailable；冻结40 binding、route、feedback与脚本旁路删除清单。
2. **M1 Source Contracts**：建立stable document/rule/region/schema/key/scenario/action identity、typed value/reference、scope/layer/authority与migration；先写round-trip/rename/reorder/delete RED tests。
3. **M2 Shared Compiler / Artifacts**：Runtime-owned compiler产canonical artifact/digest/dependency/cost/diagnostics/compatibility；Editor、Preview、PIE、cook、shipping只消费同一产物。
4. **M3 Spawn Authority / Instance**：建立typed admission、owner lease、deferred initialize、atomic publish、stable instance/entity ownership、single terminal receipt、whole-instance despawn/reload/query。
5. **M4 World State / Scenario Runtime**：建立scope/layer resolver、CAS/idempotent transaction、bounded cursor journal、computed state、security projection与Scenario clock/action/terminal lifecycle。
6. **M5 Transactional Editor**：接Editor document/history/save/conflict、schema inspector、graph/table、diff/navigation、LKG/currentness；领域字段禁止写control-local truth。
7. **M6 Isolated Simulation / PIE**：复用Preview World/Time和Editor148的server + N clients topology，输出deterministic trace、timeline、overlay、cancel/currentness receipt。
8. **M7 Script / Network / Save / Partition**：脚本改scoped request/query/observe并删除直接spawn/despawn；接stable wire/save artifact、late join、migration、cell unload/drain与cross-cell ownership。
9. **M8 Hard Cutover / Failure / Scale**：旧Workspace/binding/feedback/raw authority零引用删除，通过fault、100K、redaction、queue/journal bound、Windows与soak资格。
10. **M9 Encounter / Population / Competition**：建立Director、长期simulation、prediction/rollback、shard handoff与simulation farm，再做同语义Unreal对照。

当前MVP基线未闭合，实施不得跳过M0/M1直接做Encounter、Population或分布式simulation。

## 13. 性能与规模资格

目标不是让缺功能路径比Unreal耗时更低。所有性能结论必须绑定source/artifact/compiler/runtime/hardware/topology digest，并同时保留正确性与失败语义。

| 维度 | 必须报告 |
|---|---|
| Compile | rule/key/transition规模、dependency count、cold/warm latency、allocation、artifact bytes与diagnostic count。 |
| Spawn | 1/1K/100K entity、steady/burst、P50/P95/P99 admission/preflight/publish、CPU/RSS/alloc与rejection reason。 |
| State | 1/1K/100K keys/changes、CAS contention、computed fan-out、journal bytes/lag/gap与observer cost。 |
| Network/Save | baseline/delta bytes、late join、instance/state linkage、capture/restore latency与migration failure atomicity。 |
| Lifecycle | cancel/timeout/reload/unload/crash/restart、instance/queue/journal retention、24h+ soak与orphan count。 |
| Comparison | 同场景、同hardware、同角色/功能、同采样窗口、raw trace/artifact及tail latency，不能只给平均值。 |

本轮新增的successor cache、hash validation和result-size cache只可作为局部优化候选；在领域source、authority、instance和receipt存在前，不得把微基准结果升级为产品性能声明。

## 14. 禁止的临时修补

- 禁止继续在ZUI写固定rule/zone/key/event/count、Server字符串、queued/succeeded feedback或editable authority。
- 禁止把DynamicScene、`EntityRemap`、generic StateRegistry、Session Archive、VFX spawner或Prefab placement改名成Spawn Instance/World State/Scenario。
- 禁止引用当前Bevy不存在的旧`InstanceId/SceneSpawner` API支撑设计或验收。
- 禁止用`HashMap<String, Value>`、raw JSON、裸entity ID、全局bool、control value或registration order承担stable schema/authority/protocol。
- 禁止让Editor、script或Workbench直接`spawn_node`/`remove_entity`绕过compiler、lease、budget、transaction与journal。
- 禁止另写Preview/cook/shipping compiler，或长期保留old/new双轨与compatibility facade。
- 禁止以append-spawn冒充reload/reconcile，以EntityId list冒充instance ownership，以task ready冒充terminal receipt。
- 禁止复制Network、Save、Partition、Tag、AI、Weather或Quest owner；只允许typed adapter/contributor。
- 禁止无界queue/journal/history、silent partial success、late result覆盖新generation、cancel后发布success或teardown后写复用World。
- 禁止用无网络、无保存、无失败注入、低entity数量或静态Preview的低CPU/RSS声称优于Unreal。

## 15. 本轮完成定义

本轮完成Editor28/86/102/149 current-source刷新：冻结69个Zircon selected文件、13,617行、535,528 bytes、56个Rust test attributes与4个ignored declarations；冻结22个Unreal/Bevy/Godot/Fyrox/Unity Graphics参考文件、11,820行、499,904 bytes。纠正当前Bevy不再提供旧stable scene instance API的陈旧表述；确认Editor产品fixture不变、script仅增加错误传播、state machine只保留latest transition、DynamicScene增加局部性能/预算/cancel底座。5项P0保持Open；P1保持52 Open/8 Partial/0 Closed，Partial编号18、19、22、23、26、51、58、59；12项P2保持Open；32门全部Fail；canonical finding delta为零。

本轮只修改review与导航索引，不修改Runtime、Editor、App、plugin、ABI、测试或产品资源；没有运行Cargo或动态产品资格，也没有查询、轮询、等待或实时跟踪协调器。实现状态仍为pending，MVP门闭合后的源码修正必须从M0 truthfulness与M1 source contract RED evidence开始。
