---
title: Editor Spawn Rules、Encounter、Population、World State、Scenario、Quest Flag、Authority、Simulation Authoring 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor86
review_date: 2026-08-24
baseline_head: ad677990bd85466771a90096b646526a3daf0837
baseline_epoch: 406
final_recheck_head: 7fe97290fd3b0350c2c0f404fd00ad2d18f1335d
final_recheck_epoch: 407
canonical_owner: Editor28
refreshes:
  - docs/plans/optimize/zircon_editor/28-spawn-rules-encounter-population-world-state-scenario-quest-flag-authority-simulation-authoring-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/gameplay/workbench_extension_spawn_rules_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/gameplay/workbench_extension_world_state_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/index/workbench_extension_module_workspaces.zui
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn
  - zircon_runtime/src/scene/dynamic_scene/spawn_task
  - zircon_runtime/src/scene/dynamic_scene/asset_reload
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/lifecycle.rs
tests:
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_scene.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_scene/asset_reload_selection_status.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/spawn_transform.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_editor/21-gameplay-ability-effect-attribute-tag-cue-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/24-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-review.md
  - docs/plans/optimize/zircon_editor/26-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-review.md
  - docs/plans/optimize/zircon_editor/28-spawn-rules-encounter-population-world-state-scenario-quest-flag-authority-simulation-authoring-review.md
  - docs/plans/optimize/zircon_editor/65-editor-scene-object-creation-placement-palette-factory-asset-drag-drop-template-favorites-preview-transform-transaction-plugin-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99z-runtime-wide-state-next-state-transition-hook-history-schedule-scope-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zb-runtime-gameplay-framework-game-instance-world-context-level-game-mode-game-state-local-player-controller-pawn-possession-spawn-travel-network-save-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zc-runtime-prefab-archetype-prototype-class-default-instance-override-runtime-instantiation-propagation-hot-reload-network-save-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zd-runtime-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-product-integration-current-source-review.md
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
doc_type: current_source_refresh
review_status: complete
implementation_status: not_started
source_recheck_required: true
canonical_finding_delta:
  p0: 0
  p1: 0
  p2: 0
finding_status:
  open: 69
  partial: 8
  closed: 0
gate_status:
  fail: 32
  partial: 0
  pass: 0
---

# Editor Spawn Rules、Encounter、Population、World State、Scenario、Quest Flag、Authority、Simulation Authoring 与 Product Integration 当前源码复核

## 1. 结论

Editor28 的五项根问题在当前源码中全部成立。两张可达 Workbench 仍以固定业务字符串构造“产品”：Spawn Rules 显示 `SpawnRules_Enemy`、`Zone_A`、`Condition_Night`、`Tag_Combat`、18 rules、12 zones、1 conflict、`Seed: 2026` 和 Server/Client Preview/Offline；World State 显示 `Scenario_NightRaid`、`Layer_Global`、`Alarm.Active`、Weather/AI/Quest、84 keys、6 layers、1 conflict 与 `Authority: Server`。Simulate/Validate 不创建 compiler、job、Preview World 或 runtime request，只无条件返回“queued”、96 spawns 或 42 events。两张 ZUI 被总 workspace index 和 Assets Workspace 收录，40 个领域 binding、navigation spec、preview action allowlist、control-local field mutation 和 fixed feedback 使其成为真实可达的第二 authority，而不是孤立设计稿。

Runtime 底层并非全无进展。`DynamicScene` 已有 target-bound compile、World/schema/component-registry generation fence、隔离 preflight World、fallible reflected write validation、紧凑 mutation、提交前复验和 no-fail publication；prepared/staged spawn 有 payload/target byte limit，异步 task 有 pending/running/ready、cancel 与 terminal error，asset reload 有 bounded queue、revision stale/superseded、reconciliation、count/bytes/time budget 和 typed apply report。这些是应保留的通用 Scene mutation 底座。

但这些底座没有组成 Spawn 产品。公开结果仍只是 `EntityRemap { BTreeMap<EntityId, EntityId> }`；没有 source/artifact revision、stable `SpawnInstanceId`、owner/instigator/authority、ownership set、whole-instance despawn、lifecycle state、query cursor 或 `SpawnReceipt`。全产品生产范围对 `SpawnDefinitionDocument`、`SpawnRuleDocument`、`CompiledSpawnPlanArtifact`、`SpawnAuthorityService`、`SpawnInstanceRecord`、`SpawnReceipt`、`WorldStateSchemaDocument`、`ScenarioDefinition`、`ScenarioInstanceId`、`WorldStateTransaction`、`WorldStateChangeSet`、`EncounterDirector`、`PopulationDirector` 和 `QuestFlagSchema` 的精确词检索均为 0。

脚本旁路也没有收敛。`zr.zircon.gameplay` 用同一个 `gameplay.entity` capability 授权 transform、component、combat、`spawn_empty`、`spawn_model` 与 `despawn`；后三者直接调用 `World::spawn_node` / `remove_entity`，接受或返回裸 `u64`/`i64` entity。它没有 self/owned/admin 分层、authority lease、generation、rate、budget 或 typed receipt，不能作为未来 Spawn Rules 的执行服务。

因此本轮不新增 finding，只对 Editor28 的原账本做 current-source 重判：**5 P0 Open；60 P1 中 52 Open、8 Partial、0 Closed；12 P2 Open；32 Gate 全部 Fail**。8 个 Partial 只来自通用 DynamicScene 的隔离事务、批量原子性、reload/revision、generation fence、bounded queue、异步取消、profile counter 和底层测试；它们不证明 Spawn Rules、World State 或 Scenario 产品存在。

目标不是复制 Unreal 的 UObject/Actor 类层次，而是建立同等级且更可测、更紧凑的合同：

```text
SpawnDefinitionDocument
  -> SpawnSemanticCompiler
  -> CompiledSpawnPlanArtifact
  -> SpawnAuthorityService
  -> DynamicScene / ECS commit substrate
  -> SpawnInstanceRecord + SpawnReceipt

WorldStateSchemaDocument + ScenarioDefinition
  -> WorldStateSemanticCompiler
  -> CompiledWorldStateProgram
  -> WorldStateRuntimeService
  -> WorldStateTransaction + ChangeSet + bounded observation
  -> Spawn / AI / Weather / Quest / Network / Save adapters
```

“性能和表现优于当前 Unreal”目前仍无证据。仓库没有同场景、同硬件、同画质、同网络角色和同功能语义的原始 benchmark receipt。先关闭 identity、authority、lifecycle、failure atomicity 与产品可达性，再以 compile latency、spawn throughput/tail latency、state transaction throughput、CPU/RSS、queue/journal bound、network delta、save/restore 和长期 soak 建立可复现资格；不能用少做功能或静态反馈得到的低开销冒充性能优势。

## 2. Owner、currentness 与物理冻结

### 2.1 唯一 owner 与非重复计数

| 主题 | 唯一 owner | Editor28 / Editor86 只拥有的纵切面 |
|---|---|---|
| Runtime Gameplay Framework、SpawnService、GameRule authority | Runtime99zb | Editor只提交 compiled artifact/request 并投影 receipt，不复制 runtime owner |
| Prefab source/artifact/instance/rebase | Runtime99zc + Editor44 | Spawn rule只持 typed source reference，不把 `DynamicScene` 或 Prefab 政名为规则实例 |
| SaveGame participant、slot、migration、platform/cloud | Runtime99zd + Editor24 | World State/Scenario 暴露 typed participant projection，不另建存储系统 |
| 通用 Runtime state schedule/hook | Runtime99z | World State 是带 schema/scope/authority 的 gameplay domain，不改名复用全局 `StateRegistry` |
| Scene object placement/factory | Editor65 | authored placement 与 runtime population spawn 分离，二者共用 typed construction plan/receipt 原语 |
| Preview time/world/subsystem session | Editor69 + Runtime clock | 本报告编排 isolated scenario simulation，不复制 Preview World/Time owner |
| Document/history/save/conflict | Editor02 | Spawn/State document 贡献 typed command/path，不直接改 control property |
| Job/cancel/currentness | Editor09 | compiler/simulation 接入统一 job，不创建第二线程池或状态机 |
| World Partition、Tag、Network/PIE | Editor16/21/26 | 只定义 adapter 输入输出与 qualification，不重写各域实现 |

Editor28 继续是 Spawn Rules / Encounter / Population / World State / Scenario / Quest Flag authoring、semantic projection、simulation orchestration 和 diagnostics 的唯一 canonical owner；Editor86 只是 current-source refresh，不建立第二账本。Runtime 的 source schema、compiler artifact、authority/runtime service 与 ECS mutation 位于 `zircon_runtime`；Editor 不得拥有 mutable runtime truth。

### 2.2 Currentness 与共享工作树

- 协调 session 为 `optimize-editor86-spawn-world-state-current-review-r1-20260824`，注册基线为 `ad677990bd85466771a90096b646526a3daf0837`、epoch `406`；本轮初次收口检查为 `7fe97290fd3b0350c2c0f404fd00ad2d18f1335d`、epoch `407`。
- 共享工作树不是 clean HEAD。两张 gameplay ZUI 各有一处非本轮修改，仅删除 Validate 按钮的 `selected=true, checked=true`；Assets Workspace 也只删除 Import 按钮同类视觉状态；固定业务事实、可达 route 和 queued feedback 均未改变。通用 `control.rs` 只有相邻错误类型的 `to_string()` 适配；DynamicScene reload queue/staging 与 transaction test 也有其他 session 的在途修改。本文按冻结时 working-tree snapshot 读取，不回退、不覆盖、不把它们当已提交进展。
- 基线到初次收口 HEAD 的推进没有改变本报告对零领域 owner、静态产品路径、`EntityRemap` 结果形状或脚本直接 spawn/despawn 的裁决。最终交付前仍需复算 selected fingerprint；任何 source drift 都要求重新判定相关 finding。
- 本轮没有开放的 Editor86 failure handoff；四个文档路径由当前 session 领取 coordinator lease。MVP `00-current-source-baseline-recovery` 仍为 `in_progress`，review-only 合法，但不能宣称实施完成。

### 2.3 冻结语料

指纹算法为：按 normalized relative path 排序，对每个文件计算 lowercase SHA-256，以 `path|hash` 和 LF 连接且无末尾 LF，再对整体取 SHA-256。test declaration 是静态词法计数，不表示执行或通过。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 证据与 fingerprint |
|---|---:|---|
| Editor product surface | **13 / 4,632 / 4,387 / 242,175 / 1 / 0** | 两张 ZUI、index/Assets、40 个领域 binding、navigation、field、feedback、preview action；`fb336361fb727c02b2c17a96af3732f661da5e499cbab507265e38d6e083b44f` |
| DynamicScene spawn substrate | **31 / 5,357 / 4,901 / 185,784 / 11 / 0** | compile/preflight/commit、prepared/task、reload queue/report；`aebef63a8d3a212ad87fe5fabfaa5cd81553b3def6cb724787e1dba96f6ef481` |
| Script gameplay mutation surface | **11 / 1,799 / 1,692 / 70,457 / 5 / 0** | host catalog、lifecycle、transform/component/combat facade；`ea1e593e61441227a933a988020cc6942a552e5ada3f468c416a57172f10ca35` |
| Focused Zircon tests | **12 / 2,636 / 2,473 / 107,170 / 34 / 0** | generic DynamicScene transaction/reload 与 gameplay host tests；`4454cf8918d5b30f60ef50d805e10a5cad9b852712bb9502a07f22ce264510b1` |
| Unreal selected | **6 / 5,597 / 4,447 / 254,960 / 0 / 0** | World/GameState/DataLayer、MassSpawner types/subsystem/implementation；`6cd0f108e3abf03f150f63dfe1b5acf926f86f6095902dbaf55dfbe043a188d9` |
| Bevy selected | **6 / 1,814 / 1,682 / 68,735 / 2 / 0** | Scene spawn/queue 与 State/transition/sub/computed state；`330f09a204bb36f705976e5ab300b686ec93726edd3563ce4a469cc5957fbc3d` |
| Godot selected | **3 / 889 / 742 / 34,679 / 6 / 0** | MultiplayerSpawner definition/implementation/tests；`46694316b9af6466b78659f6e9c69e529892b2c7a1a6dbd0f977c42760045576` |
| Fyrox selected | **2 / 1,454 / 1,289 / 50,658 / 0 / 0** | Model instance provenance/stable IDs 与 AddNode command/undo；`eb367002e7dd24721d414ebe384257f94261adc6e81ce5eb967f811a9a9f28cc` |
| Unity Graphics selected | **5 / 2,066 / 1,682 / 90,872 / 26 / 0** | VFX spawn state/compiler task/migration/runtime tests；`8b9ee0093dd00ff057508806e267dd78f1d18c6d0eed59a938e9355fcedd2344` |
| all selected | **89 / 26,244 / 23,295 / 1,105,490 / 85 / 0** | 上述路径去重；`9f6b3b15a396a11167a13f9916a62ad5651f970f35c64d1da3c3aa3792404c23` |

## 3. 当前实现逐层事实

### 3.1 Editor 产品链仍是静态 control 投影

1. Spawn Rules 与 World State ZUI 分别把虚构 rule/region/key/scenario/authority/统计直接写进 `props`；它们没有 asset locator、document session、source revision、provider generation 或 runtime handle。
2. `gameplay_state.rs` 为 Spawn 与 World State 各登记 20 个 binding。navigation spec 只切换 tab/row 的 selected state、打开 popup 或接受 command；preview action allowlist 只证明 action ID 可被调用。
3. field edit 最终只执行 `mutate_control_property(control_id, "value", ...)` 和 `value_text`，没有 Editor02 document command、dirty generation、validation、undo、save 或 merge。
4. feedback 对 simulate/validate 无条件返回 `Simulation queued Zone_A 96 spawns`、`Validation queued 18 rules 1 conflict`、`Simulation queued Night Raid 42 events` 与 `Validation queued 84 keys 1 conflict`，没有 provider/job/receipt 查询。
5. 当前 13 文件聚焦集中只有通用词法 test，未发现创建 Spawn/World State document、编译 artifact、启动隔离 simulation 或消费 runtime receipt 的产品测试。

### 3.2 DynamicScene 是事务底座，不是 Spawn Instance Registry

1. `CompiledSceneSpawn` 固定 target World generation、schema catalog generation、component registry generation 与 change tick；compile 构造 entity remap、records、component/resource writes 与 preview。
2. apply 先捕获隔离 preflight World，执行所有 fallible reflected writes 和 validation，再提取紧凑 `PreflightedSceneMutation`；commit 再复验 target currentness，并走 no-fail publication。
3. prepared/staged spawn 对 payload 和 target snapshot 有 byte limit，可绑定 target Level 与 expected generation；异步 task 只做 decode/prepare，World mutation 留在调用线程，支持取消与 terminal error。
4. asset reload queue 有 event/schedule/apply count、bytes、time budget，能标记 stale revision、superseded task、generation gap、skip/failure 和 reconciliation；它仍把更新后的 Scene append-spawn，Removed/ReloadFailed 只 skip，没有 instance-aware replace/drain。
5. 所有成功入口最终只返回 `EntityRemap`。它不保存 source locator/revision、逻辑 instance、owner、authority、entity ownership set、lifecycle、despawn 或长期查询。将其改名为 `SpawnInstance` 会制造更危险的伪合同。

### 3.3 World State 与 Scenario production owner 为零

`zircon_runtime/src` 没有 gameplay 目录，也没有 World State schema/store/transaction/change set、Scenario definition/instance/clock、Encounter 或 Population director。Runtime99z 的全局 typed `StateRegistry` 仅是手工驱动的 process-local `TypeId + Any` current/next/hook/history 原语，且没有产品 caller、scope identity、authority、schema/ABI/save/network 或 bounded journal；它不能被改名吸收 World State。

### 3.4 脚本 facade 仍绕过未来 authority

`gameplay_host.rs` 的多数 entity 函数接受任意 raw entity ID，却统一要求 `gameplay.entity`。`spawn_empty` / `spawn_model` 直接构造 `NodeKind` 并写 transform/name/component，返回裸 ID；`despawn` 直接 `remove_entity`。现有 tests 证明 capability gating 和局部行为，但没有 principal、self/owned scope、Spawn source/artifact admission、authority、generation、quota、cancel 或 receipt。

## 4. 五套参考实现的可迁移合同

| 参考 | 已验证的工程合同 | Zircon 应迁移 | 不照抄的部分 |
|---|---|---|---|
| Unreal Engine | `FActorSpawnParameters` 明确 Template/Owner/Instigator/Level/collision/name/object flags；deferred spawn 将构造与完成分开；GameMode authority 仅在 server；MassSpawner 用 entity config、proportion/generator、template registry、batch creation context、initializer pipeline 与 destroy；GameState/DataLayer 区分 authoritative replicated/effective state | owner/instigator/authority、deferred construction、template/artifact identity、batch initialization、world subsystem lifecycle、actual/effective state 与 server admission | 不复制 UObject/Actor 层次、宏、反射成本和历史兼容 facade |
| Bevy | Scene spawn 区分 immediate/queued、dependency readiness、resolved patch/instance 与失败清理；State/Previous/Next、dependent/exit/transition/enter 顺序、sub/computed state 与 transition events 有明确 schedule | compiler/apply 分层、dependency currentness、失败回滚、ordered transition、computed dependency graph 和 state-scoped cleanup | 不把所有 gameplay authority 压成 ECS Resource；不接受静默 last-writer-wins 作为目标 |
| Godot | MultiplayerSpawner 管理 spawnable scene cache、spawn path、tracked node map、spawn limit、custom spawn、spawned/despawned lifecycle；tests 覆盖无效路径、配置 warning、limit、track/exit、custom spawn 和失败 | source catalog、tracked ownership、limit/admission、tree exit cleanup、observable lifecycle 与产品失败测试 | NodePath/ObjectID 不能作为 Zircon 跨保存/网络稳定 identity；不复制单 SceneTree global owner |
| Fyrox | Model instantiation保留 resource instance provenance，可选择 stable IDs 以维持多人一致；Editor AddNodeCommand 保留 handle/ticket、parent、selection 并支持 undo | source provenance、stable ID、attach/parent/selection transaction 与 undo 守恒 | 不把 command ticket 或 graph handle 直接当 runtime spawn instance |
| Unity Graphics | VFX Spawner 把 loop state、spawn count、delta/total time、delay/index/count、CPU/GPU expression限制和 compiler task usage 显式化；旧 PeriodicBurst 有 versioned sanitize migration；runtime tests覆盖 burst/state/time/loop/custom callback | 只迁移明确的 execution state、compiler usage、version migration、runtime trace 与测试深度 | Graphics 镜像是 VFX，不证明 Unity gameplay/world-state authority；不能用它替代 gameplay 参考 |

五套参考共同证明：definition、compiled program、runtime instance、owner、state、failure、budget 和 observation 必须分离。Zircon 可以用 data-oriented table、arena、SoA batch、immutable snapshot 与 bounded journal 获得更低开销，但不能删掉 identity、receipt 和 lifecycle 后宣称更高性能。

## 5. 目标 owner 与发布合同

### 5.1 Spawn

- `SpawnDefinitionDocument`：Editor document identity、revision、stable rule/region IDs、typed references、condition/action graph、seed/budget/lifetime policy。
- `SpawnSemanticCompiler`：Runtime-owned deterministic compiler；Editor validate、preview、PIE、cook 和 shipping 共用，输出 immutable `CompiledSpawnPlanArtifact`、dependency manifest、digest、compatibility 与 cost estimate。
- `SpawnAuthorityService`：per World/GameInstance 注册，持 owner lease；typed admission 校验 artifact/world/level/source/authority/budget/deadline/cancel。
- `SpawnConstructionTransaction`：复用 DynamicScene compile/preflight/commit，但补齐 deferred initialize、observer ordering、ownership 与 fault rollback。
- `SpawnInstanceRegistry`：generation-qualified instance ID、source provenance、owner/authority、entity set、lifecycle、whole-instance despawn/reload/query。
- `SpawnReceipt`：request/instance/entity set、seed、cost、diagnostics、cancel acknowledgement 和唯一 terminal outcome。

### 5.2 World State 与 Scenario

- `WorldStateSchemaDocument`：stable schema/key ID、typed value、scope、default/validator、layer policy、security/redaction、migration。
- `WorldStateSemanticCompiler`：校验 key/reference、computed dependency/cycle、scenario state/guard/action、wire/save compatibility，输出 immutable program。
- `WorldStateRuntimeService`：per World/Session scope，所有写入走 expected generation/CAS/idempotency 的 transaction，发布唯一 `WorldStateChangeSet`。
- `WorldStateObservationJournal`：bounded entries/bytes/age，cursor/gap/full-resync、visibility projection 与 correlation。
- `ScenarioInstanceRegistry`：stable instance、authority、clock/seed/source revision、start/pause/resume/complete/fail/cancel、timer/action cleanup 与 terminal receipt。
- AI、Weather、Quest、Tags、Spawn 只通过 owner-qualified contributor/adapter 接入；World State 不吸收它们的算法和数据 owner。

### 5.3 Editor 产品

Editor 只拥有 transactional document、schema-driven inspector、graph/table projection、diagnostic navigation、job orchestration、isolated simulation session、trace/diff/timeline/overlay 与 LKG UX。UI 必须投影 provider capability 和 immutable receipt；无 provider 时明确 Unavailable，不能生成 queued/success 文本。

## 6. P0 current-source 状态

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| ED28-P0-01 | Open | Spawn ZUI/route/feedback仍固定18 rules、12 zones、96 spawns、Server与queued | M0移除业务成功事实；无provider时Unavailable并禁用Simulate/Validate |
| ED28-P0-02 | Open | World State仍固定84 keys、6 layers、42 events、Weather/AI/Quest与Server | 停止把control字符串当状态；接真实schema/document/provider前fail closed |
| ED28-P0-03 | Open | DynamicScene仍只返回短期`EntityRemap`，无instance/source/owner/lifecycle/receipt | 建立Runtime99zb/99zc协同的Instance Registry、ownership、despawn、reload和query |
| ED28-P0-04 | Open | `gameplay.entity`仍可直接spawn/despawn任意裸ID实体 | 拆分observe/self/owned/spawn/despawn/admin；所有产品spawn走typed service admission |
| ED28-P0-05 | Open | Simulate/Validate不创建compiler、PreviewWorld、trace或server/client topology | 同一artifact在isolated Preview/PIE执行并返回source-qualified receipt后才恢复成功语义 |

## 7. P1：Spawn source、compiler 与 artifact

| ID | 状态 | 当前证据与重构要求 |
|---|---|---|
| ED28-P1-01 | Open | 无Spawn Definition asset/document/stable identity；由Editor04登记真实asset/factory/toolkit/source revision |
| ED28-P1-02 | Open | rule仅为固定显示字符串；建立stable rule/set ID、enabled、priority/order、namespace与schema version |
| ED28-P1-03 | Open | 无typed source template/config；引用DynamicScene/Prefab/entity config时携revision、missing/stale/cycle诊断 |
| ED28-P1-04 | Open | `Zone_A`无几何或owner；定义volume/surface/point/spline/cell、transform、bounds revision、streaming ownership |
| ED28-P1-05 | Open | `Condition_Night`无AST/依赖；编译typed state/tag/time/player/distance依赖与pure/deterministic/authority属性 |
| ED28-P1-06 | Open | 96 spawns为feedback常量；定义proportion、weight、min/max/density、rounding和zero-weight policy |
| ED28-P1-07 | Open | 无placement/collision；建立generator、orientation/scale、ground/nav projection、overlap与bounded retry |
| ED28-P1-08 | Open | 无per-rule/region/world/owner quota；加入count/rate/CPU/memory预算及typed deferred/rejected |
| ED28-P1-09 | Open | 无despawn/respawn/lifetime policy；枚举distance/time/state/unload/owner loss/death/manual/shutdown与backoff |
| ED28-P1-10 | Open | `Seed: 2026`只是字符串；seed绑定artifact/source/region/tick并记录算法版本 |
| ED28-P1-11 | Open | 无共享semantic compiler；Editor/cook/PIE/shipping消费同一compiler、artifact和source-range diagnostic |
| ED28-P1-12 | Open | 无immutable artifact/cook/reference；加入digest、compatibility、dependency manifest、cost与canonical serialization |

## 8. P1：Spawn runtime、authority 与 instance lifecycle

| ID | 状态 | 当前证据与重构要求 |
|---|---|---|
| ED28-P1-13 | Open | 无`SpawnAuthorityService`或owner lease；per World/GameInstance注册并在unload/reload撤销旧generation |
| ED28-P1-14 | Open | 无typed SpawnRequest；校验artifact/world/level/authority/owner/budget/deadline/cancel并返回machine-readable rejection |
| ED28-P1-15 | Open | 无stable SpawnInstanceId；identity必须独立于EntityId并用于query/net/save/reload/despawn |
| ED28-P1-16 | Open | remap不保存source/revision provenance；instance record必须解释历史artifact与dependency revision |
| ED28-P1-17 | Open | 无owner/instigator/authority context；区分server/client-predicted/editor-preview/offline/system |
| ED28-P1-18 | Partial | DynamicScene已有compile -> isolated preflight -> no-fail publish；仍缺construction hook、required validation、observer/ownership barrier |
| ED28-P1-19 | Partial | generic Scene batch为all-or-nothing；仍无Spawn产品的bounded-partial/streamed policy与逐项结果 |
| ED28-P1-20 | Open | task/apply report不是SpawnReceipt；缺request/instance/entity set/seed/cost/cancel与唯一terminal outcome |
| ED28-P1-21 | Open | 无whole-instance DespawnRequest/Receipt；不得缓存remap后手工循环remove |
| ED28-P1-22 | Partial | reload已有bounded staging、stale/superseded/reconciliation；仍是append-spawn，无instance-aware keep/replace/patch/drain/reject |
| ED28-P1-23 | Partial | World/Level generation fence可拒绝旧target；仍无quiesce、取消在途、drain实例与teardown barrier |
| ED28-P1-24 | Open | 无pooling/reuse identity/reset合同；未来reuse不得复用instance/authority/network/save identity |
| ED28-P1-25 | Open | 无按instance/rule/region/owner/state的分页snapshot/cursor；Editor不得遍历World推导truth |
| ED28-P1-26 | Partial | 有count/bytes/time limit、diagnostics/profile与100K底层probe；无per-owner产品quota、完整fault/soak或同语义benchmark |

## 9. P1：World State、Scenario 与跨域集成

| ID | 状态 | 当前证据与重构要求 |
|---|---|---|
| ED28-P1-27 | Open | 无schema asset/stable schema ID；加入version/namespace/owner/typed keys/default/validator/migration |
| ED28-P1-28 | Open | 无stable key ID；display rename与runtime/save/network identity分离，治理delete/deprecated/redirect |
| ED28-P1-29 | Open | 无canonical typed value；定义bool/int/float/string/name/tag/enum/entity/resource/registered struct语义 |
| ED28-P1-30 | Open | Global/Region/System/Scenario只是文本；建立scope key、lifetime、parent/world/session与合法key set |
| ED28-P1-31 | Open | 无layer resolver；定义authored/server/scenario/debug precedence、merge、conflict与effective provenance |
| ED28-P1-32 | Open | 无authoritative transaction；所有写入走generation、ordered mutation、validation与唯一change set |
| ED28-P1-33 | Open | 无CAS/idempotency/concurrent writer；加入transaction key、retry、actor/owner audit和确定冲突策略 |
| ED28-P1-34 | Open | 无bounded change journal；事件含before/after/generation/cause/authority/correlation及gap/full-resync |
| ED28-P1-35 | Open | 无computed state；编译pure dependency graph、cycle rejection、incremental recompute与ordered transition |
| ED28-P1-36 | Open | 无ScenarioDefinition；建立stable states/transitions、typed guards/actions、entry/exit/timeout/failure/cancel |
| ED28-P1-37 | Open | 无Scenario instance；实现start/pause/resume/complete/fail/cancel、authority/clock/seed/source/receipt |
| ED28-P1-38 | Open | 无deterministic clock/timer/order；绑定time domain、timer budget与same-tick total order |
| ED28-P1-39 | Open | 无condition/action contributor registry；AI/Weather/Quest/Tags/Spawn以owner lease贡献typed node/key |
| ED28-P1-40 | Open | 无Save participant；保存schema/source/generation/authoritative keys/scenario clock/spawn linkage并原子restore |
| ED28-P1-41 | Open | 无replication/interest/late-join artifact；编译stable wire ID、visibility、delta/snapshot与baseline |
| ED28-P1-42 | Open | 无security/redaction/untrusted write；显式server/owner/client-request/debug/secret policy与投影 |

## 10. P1：Editor authoring、simulation、diagnostics 与治理

| ID | 状态 | 当前证据与重构要求 |
|---|---|---|
| ED28-P1-43 | Open | field edit只改control value；接Editor02 typed document command、dirty/history/save/conflict/recovery |
| ED28-P1-44 | Open | 无schema-driven inspector/multi-select；支持mixed/per-target、collection、reference picker与invalid raw preservation |
| ED28-P1-45 | Open | 无rule/scenario graph/table/reference nav；所有projection共享stable ID与source-qualified diagnostics |
| ED28-P1-46 | Open | 无isolated PreviewWorld；session绑定document/artifact/world generation、budget和teardown barrier |
| ED28-P1-47 | Open | 无deterministic trace；记录seed/clock/input/result/placement rejection/spawn receipt/state before-after |
| ED28-P1-48 | Open | Client Preview只是下拉文本；复用Editor07/26启动server + N clients并显示真实authority/lag |
| ED28-P1-49 | Open | 无overlay/heatmap；只消费reader-gated bounded observation snapshot，不在render loop求值rule |
| ED28-P1-50 | Open | 无source/artifact diff、timeline或debug override隔离；override需owner/expiry/audit且不回写source |
| ED28-P1-51 | Partial | DynamicScene async task有status/cancel，reload有revision stale；领域compile/simulate仍未接Editor09 job/currentness/terminal receipt |
| ED28-P1-52 | Open | 无LKG与hot-reload UX；compile失败时标示旧artifact revision并允许keep/replace/drain |
| ED28-P1-53 | Open | `Tag_Combat`为字符串；消费typed Tag registry/artifact，rename/migration使Spawn artifact stale |
| ED28-P1-54 | Open | Weather/AI/Quest仍被静态行吸收；各owner经typed contributor/authority/receipt读写 |
| ED28-P1-55 | Open | 无partition/population ownership；定义cell activate/deactivate、region revision、migrate/drain与cross-cell owner |
| ED28-P1-56 | Open | 无Network/Save adapter；只消费stable schema/instance artifact，不从raw JSON/DynamicScene snapshot反推协议 |
| ED28-P1-57 | Open | script未接Spawn/World State service；提供scoped request/query/observe、typed handle/receipt/budget并删除旁路 |
| ED28-P1-58 | Partial | 34个focused declaration覆盖generic transaction/reload与host行为；没有domain compiler/determinism/authority/product matrix |
| ED28-P1-59 | Partial | generic profile/bytes/count/100K probe可复用；没有compile/state/net/query/journal的硬件/配置/digest基线 |
| ED28-P1-60 | Open | 两张ZUI、40个领域binding、route/feedback仍在产品；新链闭合后必须零引用硬删除，禁止长期双轨 |

P1 汇总：**52 Open / 8 Partial / 0 Closed**。Partial 编号为 18、19、22、23、26、51、58、59。

## 11. P2 current-source 状态

| ID | 状态 | 目标 |
|---|---|---|
| ED28-P2-01 | Open | Encounter/Population Director按强度、节奏、玩家状态与预算协调多个plan并输出decision trace |
| ED28-P2-02 | Open | partition cell级密度、预热、迁移、HLOD/streaming协调与远距离低频simulation |
| ED28-P2-03 | Open | versioned生态/长期population模型，覆盖出生、死亡、迁移、容量和离线推进 |
| ED28-P2-04 | Open | 可插拔generator/constraint solver、deterministic cache、局部rebuild与失败解释 |
| ED28-P2-05 | Open | client-predicted spawn、prediction key、confirmation/rejection、reconcile与rollback |
| ED28-P2-06 | Open | shard/zone instance与state ownership的two-phase handoff、timeout和idempotent recovery |
| ED28-P2-07 | Open | Scenario artifact/input/event/seed/time录制、确定性回放、branch fork与diff |
| ED28-P2-08 | Open | 按stable rule/key/transition ID执行semantic three-way merge与review |
| ED28-P2-09 | Open | platform/mode/difficulty/DLC的显式inherit/override与compiled flattening |
| ED28-P2-10 | Open | consent/redaction/schema/version治理下的analytics与调优闭环 |
| ED28-P2-11 | Open | 签名、权限、预算约束的mod/plugin contributor；卸载安全失效并保留unknown data |
| ED28-P2-12 | Open | immutable artifact/input bundle的分布式deterministic simulation farm与分位数资格 |

## 12. 验收门 current-source 状态

当前没有端到端 Spawn/World State/Scenario 产品 owner，因此即使 DynamicScene 支持局部原子性，所有门仍为 Fail，不能把底层 primitive 的 unit test 提升为产品门 Partial。

| Gate | 状态 | 必须证明的验收事实 |
|---|---|---|
| ED28-G01 | Fail | 默认入口不再显示固定rule/key/scenario/spawn/event业务事实 |
| ED28-G02 | Fail | 两张旧Workspace、40个领域binding、route/fixed feedback有inventory且迁移后零产品引用 |
| ED28-G03 | Fail | 无provider时明确Unavailable，Simulate/Validate不可调用且不产生queued/success |
| ED28-G04 | Fail | fake/test/real provider走同一registry、lease、request/receipt和lifecycle conformance |
| ED28-G05 | Fail | rename/reorder/display变化不改stable identity，delete/deprecated/redirect有migration report |
| ED28-G06 | Fail | Editor/cook/PIE/shipping对同输入产生同artifact digest与diagnostics |
| ED28-G07 | Fail | missing/stale/cyclic reference fail compile或明确LKG Stale |
| ED28-G08 | Fail | 相同seed/algorithm/clock/artifact/region重复simulation得到同digest |
| ED28-G09 | Fail | stale world/schema/source/owner lease原子拒绝SpawnRequest且不发布实体 |
| ED28-G10 | Fail | construct/initialize/observer故障不泄露entity/instance/虚假receipt |
| ED28-G11 | Fail | accepted request只有一个instance与terminal receipt，cancel/timeout/late无双终态 |
| ED28-G12 | Fail | whole-instance despawn在外部删除/unload/observer failure下返回可解释终态 |
| ED28-G13 | Fail | reload明确keep/replace/patch/drain/reject且旧新revision/instance可追溯 |
| ED28-G14 | Fail | script capability按self/owned/spawn/admin/rate/budget限制，不能任意修改/删除 |
| ED28-G15 | Fail | 每个World State key有stable ID/type/scope/owner/validation/policy |
| ED28-G16 | Fail | concurrent transaction按generation/CAS/idempotency确定执行且失败不部分写入 |
| ED28-G17 | Fail | layer/effective/conflict带provenance，debug override不保存回source |
| ED28-G18 | Fail | computed cycle在compile拒绝，transition与enter/exit在same tick确定 |
| ED28-G19 | Fail | Scenario所有终态携instance/authority/clock/source与唯一receipt |
| ED28-G20 | Fail | journal有界且gap触发full resync，100K keys/changes不造成无界内存 |
| ED28-G21 | Fail | server-only/redacted key不泄露到client/script/remote Editor/log/export |
| ED28-G22 | Fail | late join从schema-qualified baseline+delta收敛，wire ID不依赖注册顺序 |
| ED28-G23 | Fail | save/load保留schema/source/scenario/clock/spawn linkage，migration失败保持原世界 |
| ED28-G24 | Fail | cell unload阻止新spawn、取消/排空在途并终结instance，late callback不写复用world |
| ED28-G25 | Fail | Preview与authoring完全隔离，simulation前后authoring document/world hash不变 |
| ED28-G26 | Fail | server + N clients使用真实role/authority/replication/link配置 |
| ED28-G27 | Fail | trace可从condition input追到source/placement rejection/spawn receipt/state change set |
| ED28-G28 | Fail | source/dependency变化自动使compile/simulate/trace Stale，late job不覆盖新generation |
| ED28-G29 | Fail | overlay只消费bounded snapshot，关闭reader后无持续extract/evaluate成本 |
| ED28-G30 | Fail | 1/1K/100K矩阵记录CPU/allocation/publish/query/net/journal预算与typed bottleneck |
| ED28-G31 | Fail | Windows compiler/runtime/Editor/network/save/fault lanes通过，Linux需求另行证据验证 |
| ED28-G32 | Fail | 长期soak证明queue/journal/instance有界、无generation回退/authority泄漏/终态丢失 |

## 13. 分层重构里程碑

### M0 · Truthfulness、Inventory 与 owner 冻结

- 关闭两张 Workspace 的虚构成功语义；无provider时显示Unavailable。
- 冻结旧ZUI、40个领域binding、route、feedback、脚本旁路和公开surface inventory。
- 固定Editor28、Runtime99zb/99zc/99zd、Editor02/09/16/21/24/26/65/69 owner矩阵和删除门。

### M1 · Spawn 与 World State source contracts

- 建立stable document/rule/region/schema/key/scenario/action identity、typed value/reference、scope、layer、authority与migration。
- 用RED roundtrip/rename/reorder/delete/unknown-field tests证明identity与data守恒。

### M2 · Shared semantic compiler 与 immutable artifacts

- Runtime-owned compiler产出canonical artifact、digest、dependency manifest、cost、diagnostics和compatibility。
- Editor validate、Preview、PIE、cook、shipping使用同一compiler；加入determinism/cycle/stale/LKG tests。

### M3 · Spawn authority、transaction 与 instance registry

- 建立typed request/admission、owner lease、deferred initialize、atomic publish、stable instance/ownership与terminal receipt。
- 将DynamicScene作为内层commit substrate，补齐whole-instance despawn、reload/reconcile、query和teardown。

### M4 · World State runtime 与 Scenario lifecycle

- 建立scope/layer resolver、CAS/idempotent transaction、bounded journal/computed state和security projection。
- 建立Scenario instance/clock/timer/action contributor与所有terminal receipt。

### M5 · Transactional Editor 产品

- 接Editor02 document/history/save/conflict，建立schema inspector、graph/table、navigation、diff和LKG UX。
- 旧control-local field edit不得写领域source。

### M6 · Isolated simulation、PIE 与多人验证

- 接Editor69 Preview World/Time与Editor07/26 server + N clients topology。
- 输出deterministic trace、timeline、diff、overlay、heatmap与currentness/cancel receipt。

### M7 · Script、Network、Save 与 Partition adapters

- 脚本改为scoped request/query/observe facade并删除直接spawn/despawn产品旁路。
- 接stable wire/save artifacts、late join、migration、cell unload/drain与cross-cell ownership。

### M8 · Hard cutover、security、failure 与 scale

- 零引用删除两张旧Workspace、40 binding、route、fixed feedback和raw authority字段。
- 通过fault injection、100K矩阵、queue/journal bound、redaction、soak与Windows资格。

### M9 · Encounter、Population 与竞争性资格

- 建立Director、生态/长期simulation、prediction/rollback、shard handoff与simulation farm。
- 在同语义profile下形成可复现Unreal对照receipt；没有原始证据时不声称领先。

## 14. 禁止的临时修补

1. 禁止把 `DynamicScene`、`EntityRemap`、`RuntimeStateRegistry` 或 Session archive 改名成 Spawn Instance、World State 或 SaveGame。
2. 禁止只新增几个 DTO、descriptor、menu item、capability string 或 queued job 状态后宣称产品完成。
3. 禁止让Editor持有mutable Runtime service或直接遍历World推导authority truth。
4. 禁止让 Spawn Rules 调用现有 `gameplay.entity` 旁路；迁移完成后产品调用必须硬删除直接spawn/despawn。
5. 禁止另写Preview compiler、cook compiler和shipping compiler；同一source只能有一个semantic compiler和artifact schema。
6. 禁止把 control `value/value_text`、ZUI default、raw JSON、string tag/path 当canonical domain storage。
7. 禁止以append-spawn冒充reload/reconcile，也禁止以EntityId列表冒充instance ownership。
8. 禁止无界queue/journal/history、静默partial success、late result覆盖新generation或cancel后再发布success。
9. 禁止复制Network、Save、Partition、Tag、AI、Weather、Quest owner；只能通过typed adapter/contributor接入。
10. 禁止用功能缺失导致的低CPU/RSS数字证明优于Unreal。

## 15. 测试与验证矩阵

### 15.1 必须补齐的静态与单元证据

- schema/source canonical roundtrip、stable ID rename/reorder/delete/redirect、unknown preservation和migration；
- compiler golden/determinism/cycle/reference/currentness/LKG、artifact digest与cost bound；
- Spawn admission、deferred failure atomicity、observer order、instance lifecycle、despawn/reload和stale handle；
- World State CAS/idempotency/layer/computed/order/journal gap/redaction；
- Scenario clock/timer/action/cancel/failure/teardown与single terminal receipt。

### 15.2 必须补齐的产品与故障证据

- real document edit/save/reopen/undo/redo/conflict/recovery；
- isolated Preview重复运行、authoring hash守恒、late job/cancel/crash/currentness；
- server + N clients、late join、interest/redaction、predicted spawn/rejection；
- save/migrate/restore失败原子性、partition cell unload/drain、plugin/provider revoke；
- 1/1K/100K、burst/steady、long soak、queue/journal/instance memory bound与同语义benchmark。

### 15.3 本轮动态证据边界

本轮是review-only。没有运行Cargo、Editor、GUI/GPU、真实Spawn/World State compiler、Preview/PIE、server/client、save/load、fault、100K、soak、profile或跨引擎benchmark；tooling按用户要求排除。现有85个selected静态test declaration只用于定位已有测试深度，不声明通过。所有实现里程碑仍为待办，`source_recheck_required` 保持 `true`。

## 16. 产出记录

- canonical finding delta：**P0 0 / P1 0 / P2 0**；没有创建第二账本。
- current status：**P0 5 Open；P1 52 Open / 8 Partial / 0 Closed；P2 12 Open；Gate 32 Fail**。
- 可保留基础：DynamicScene generation-bound compile/preflight/no-fail publish、prepared/task cancel、reload revision/budget/report，以及脚本host的集中注册面。
- 必须删除或降级：两张静态Workspace、40个领域binding、fixed queued feedback、control-local领域编辑和直接spawn/despawn产品旁路。
- 首个实施切片是M0 truthfulness，不是继续装饰ZUI：先让缺provider明确失败，再建立source/compiler/owner RED contract。
