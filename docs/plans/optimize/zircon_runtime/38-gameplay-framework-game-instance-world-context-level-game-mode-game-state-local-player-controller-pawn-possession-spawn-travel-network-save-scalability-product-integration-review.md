---
related_code:
  - zircon_runtime/src/core/framework/scene/mod.rs
  - zircon_runtime/src/core/framework/scene/world_handle.rs
  - zircon_runtime/src/core/framework/scene/level_summary.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/level_manager_contract.rs
  - zircon_runtime/src/scene/module/level_manager_lifecycle.rs
  - zircon_runtime/src/scene/module/level_manager_project_io.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/identity.rs
  - zircon_runtime/src/scene/world/generation.rs
  - zircon_runtime/src/scene/world/schedule.rs
  - zircon_runtime/src/scene/world/events.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/input_events.rs
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime/src/dynamic_api/session/profile.rs
  - zircon_runtime/src/dynamic_api/session/script_systems.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host
  - zircon_runtime/src/core/framework/input/input_action_context.rs
  - zircon_runtime/src/core/framework/input/input_action_manager.rs
  - zircon_runtime/src/core/framework/input/input_manager.rs
  - zircon_runtime/src/core/framework/input/input_snapshot.rs
  - zircon_runtime/src/input/runtime/action_evaluator.rs
  - zircon_runtime/src/input/runtime/default_input_action_manager.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/core/framework/net/manager.rs
  - zircon_runtime/src/core/framework/net/session.rs
  - zircon_runtime/src/core/framework/net/sync.rs
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/session/session.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/runtime_entry_app/mod.rs
  - zircon_app/src/entry/runtime_entry_app/construct.rs
  - zircon_app/src/entry/runtime_entry_app/event_dispatch.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - examples/vampire/zircon-project.toml
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/scripts/vampire_game/main.zr
  - examples/woc/zircon-project.toml
  - examples/woc/assets/scenes/bootstrap.scene.toml
  - examples/woc/native/apps/woc_client/src/application.rs
  - examples/woc/native/apps/woc_client/src/input/intent.rs
  - examples/woc/native/apps/woc_client/src/shell/offline_session.rs
  - examples/woc/native/apps/woc_server/src/main.rs
tests:
  - zircon_runtime/src/scene/tests/level_system_frame_state.rs
  - zircon_runtime/src/scene/tests/world_basics/world_state.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/level_apply.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/world_driver.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_gameplay.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_runtime_support.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/dynamic_api/tests/session_entry_points.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/dynamic_api/tests/session_profiles.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/combat_lifecycle.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/component_state.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/property_animation.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/spawn_transform.rs
  - zircon_runtime/src/input/tests/action_mapping.rs
  - zircon_runtime/src/input/tests/input_manager.rs
  - zircon_runtime/src/input/tests/input_manager/frame_state.rs
  - zircon_runtime/src/core/framework/net/tests.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/frame_loop.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/host_requests.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/runtime_session.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/viewport.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/37-camera-endpoint-director-rig-controller-blend-shake-cinematic-cut-history-multiview-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_app/06-vampire-roguelite-example-project-asset-script-gameplay-evidence-product-integration-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/28-spawn-rules-encounter-population-world-state-scenario-quest-flag-authority-simulation-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/GameInstance.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/GameModeBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/GameStateBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/PlayerController.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/PlayerState.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/Controller.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/Pawn.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/World.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/LevelStreaming.h
  - dev/UnrealEngine/Samples/Games/Lyra/Source/LyraGame/GameModes/LyraExperienceDefinition.h
  - dev/UnrealEngine/Samples/Games/Lyra/Source/LyraGame/GameModes/LyraExperienceManagerComponent.cpp
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/sub_app.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_state/src/state/states.rs
  - dev/bevy/crates/bevy_state/src/state/transitions.rs
  - dev/bevy/crates/bevy_state/src/state_scoped.rs
  - dev/godot/scene/main/scene_tree.h
  - dev/godot/scene/main/scene_tree.cpp
  - dev/godot/scene/main/node.h
  - dev/godot/scene/main/multiplayer_api.h
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/SceneRenderPipeline.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 38 · Gameplay Framework、Game Instance、World Context、Level、Game Mode、Game State、Local Player、Controller、Pawn、Possession、Spawn、Travel、Network、Save、Scalability 与 Product Integration 工程化差距

## 1. 结论

Zircon已经有可保留的底层能力：`World`拥有ECS identity、generation、schedule、resource与project serialization；`LevelSystem`能封装一个World、保存metadata并在替换时增加epoch；`DefaultLevelManager`能创建、查找、加载和保存Level；动态Runtime能建立一个可tick的Level并把输入、脚本、渲染和UI接到同一session；network层也定义了transport、handshake、`player_id`与replication DTO。这些不是空壳，后续Gameplay Framework必须复用它们，不能另造第二套World或网络传输层。

但当前产品结构仍停在“一个进程、一个session、一个Level、一个全局输入、脚本直接改World”。没有Game Instance、World Context、Experience/Game Mode、Game State、Local Player、Player Controller、Player State、Pawn、Possession或Player Start的产品owner。对这些概念以及Travel相关词的联合检索只有18个命中，全部来自Editor静态展示文案或脚本热重载测试类型名，没有一个Runtime产品实现。Vampire用动态字符串`role="player"`寻找玩家、读取全局WASD，并把相机实体`1`硬编码传给`camera_follow`；这证明能演示玩法脚本，不证明存在工程级玩家框架。

最严重的断路不是“类名还没补”，而是已公开能力与执行事实不一致。`gameplay.scene_transition` capability公开`request_scene_transition`，脚本调用后仅把单槽`ZrRuntimeProjectSceneTransitionRequestV1`写入World resource；`ZrRuntimeHostRequestV1`只有IME、rumble、cursor，dynamic session也只收集这三类请求，全产品没有scene-transition consumer。调用者得到request ID，但不会发生加载、切换、失败回执或资源回收。与此同时LevelManager只有插入和查询，没有enumerate/active/unload/remove/travel；`Loaded/Unloaded`两态与字符串subsystem列表只在单测中写入，manager的HashMap会随load持续增长。

目标不是照抄Unreal类层次，而是补齐同等语义并可高性能实现：`GameInstanceService -> WorldContextRegistry -> ExperienceProgram/GameRuleAuthority -> World/LevelSet -> PlayerDirectory -> Controller/Pawn possession -> per-player Input/View/Audio -> Network/Save/Travel adapters`。所有转换都必须是generation-scoped、异步可取消、事务发布、有terminal receipt且能在client/listen/dedicated/headless/PIE/multi-world下隔离。父级P0继续由Runtime05/08E/37、App01/03/06和Editor07/28拥有；本文登记 **0个新P0 / 72个Runtime子P1 / 16个P2**。

## 2. 审查边界、语料与 currentness

### 2.1 冻结语料

| 输入 | 文件 / 行 / bytes | 说明 |
|---|---:|---|
| Runtime production与直接consumer | 61 / 13,195 / 485,700 | Scene/Level/World、dynamic session、gameplay host、Input、Net、runtime interface与App host |
| 聚焦测试 | 24 / 6,398 / 232,453 | 127个`#[test]`/`#[tokio::test]`；4个Vampire real-ZrVM测试被显式ignore |
| 产品/父计划控制面 | 19 / 9,393 / 590,847 | Vampire、WOC、Runtime05/08E/37、App01/03/06、Editor07/28 |
| 参考实现 | 25 / 24,578 / 992,933 | Unreal/Lyra 11、Bevy 6、Godot 4、Fyrox 3、Unity Graphics 1 |
| 合计 | 129 / 53,564 / 2,301,933 | 排序后逐文件SHA-256 manifest的复合SHA-256为`76af666bf94bb21b2debf112a8ea5dc6500b42335fa9fb266105ebe430630a19` |

冻结时集合内有11条Git status记录：`world_driver.rs`、dynamic session的`input_events/profile/script_systems`及host-request测试为用户在途修改；Runtime37、App01/03/06和Editor07/28为untracked计划文档。本文不回退、不归因这些改动，审查结论绑定当前物理文件；任何相关source fingerprint变化均要求复核。

### 2.2 检查方法

本轮沿Project bootstrap -> Game Instance -> World Context -> Level registry/load/unload -> Experience/Game Mode -> Game State -> local/remote Player -> Controller/Pawn possession -> Input/Camera -> spawn/respawn -> travel -> network/reconnect -> save/restore -> PIE/headless/multi-world -> product evidence逐层检查。关键词零命中只作为入口线索；只有owner、schema、consumer、lifecycle和测试共同缺失时才登记能力缺口。

### 2.3 动态证据边界

本篇是源码E3审查，不重复启动已知不可达的Editor和WOC验证lane，也不把127个聚焦测试写成已运行通过。4个real-ZrVM产品测试处于ignore，scene-transition只有source-contract测试而没有执行/回执测试；这些都不能作为shipping gameplay framework资格。

## 3. 当前可保留的真实基础

1. `World`已经有generation、entity/component/resource、schedule、derived state与project I/O边界。
2. `LevelSystem`把World锁、replacement epoch、physics/animation/script/frame state和metadata集中在一个owner内。
3. World replacement会重置局部运行态并用epoch防止部分陈旧发布，可扩展成World Context generation。
4. `DefaultLevelManager`已有Core service接入、typed `WorldHandle`、project-root校验和异步scene artifact ticket。
5. dynamic session已经形成Project config -> Core/modules -> Level -> Input/Script/Render/UI的真实启动链。
6. InputManager有bounded event/host request和action context/evaluator基础，可扩展为per-user路由。
7. network有client/listen/dedicated mode、handshake state、session/player ID和replication descriptor基础。
8. script host有capability admission、typed host value/error和实际World操作入口，可收敛为command facade。
9. runtime ABI已有bounded host-request批次、commit/rollback和foreign-output ownership，可复用为Travel receipt通道。
10. Vampire和WOC提供真实产品压力，但必须作为consumer接入统一框架，不能继续各自拥有字符串玩家语义。

## 4. 当前代码事实与断路

| 链路 | 当前事实 | 工程缺口 |
|---|---|---|
| Instance | `RuntimeDynamicSession`直接持有Core、一个Level、一个InputManager和一个Orbit camera controller | 没有跨World存活的Game Instance、service scope或init/start/shutdown状态机 |
| World Context | Level等同当前World，profile只区分session模式 | 没有WorldType、owner instance、active/pending world、net driver、PIE ID或travel state |
| Level registry | `AtomicU64 + HashMap<WorldHandle, LevelSystem>`只插入/查询 | 无checked exhaustion、generation、enumerate、remove、unload、retirement或active set |
| Level lifecycle | 只有`Loaded/Unloaded`，生产代码从不写Unloaded | 无Requested/Loading/LoadedHidden/Visible/Unloading/Failed/Removed及异步receipt |
| Transition | 脚本把ReplaceActive请求写成World单槽resource | 无consumer、queue、supersede执行、result、rollback或资源释放 |
| Gameplay rules | 产品没有Game Mode/Game State/Experience owner | 规则、默认pawn、join/spawn/begin-play/match全由脚本约定 |
| Player | Net handshake仅保存`Option<String> player_id` | 未绑定principal、LocalPlayer、PlayerState、Controller、Pawn、viewport或save identity |
| Possession | 没有Controller/Pawn类型或占有协议 | 输入、相机、AI、authority、tick order和network handoff无法原子切换 |
| Spawn | `spawn_empty/spawn_model`直接调用World并返回裸u64 | 无class/archetype、deferred construction、owner、authority、finish/rollback或receipt |
| Input | gameplay脚本读取全局InputSnapshot | 无platform user/device -> LocalPlayer -> Controller映射和UI consumption证明 |
| Camera | session固定Orbit；Vampire硬编码camera entity 1 | 无player view ownership，继续依赖Runtime37缺失的Director |
| Network | NetManager只负责transport，Sync多为DTO | 无login -> player framework、authority/possession复制、join-in-progress或travel握手 |
| Save | Scene project I/O复制整个World | 无GameInstance/PlayerState/Pawn/level streaming状态的save schema与restore order |
| Product | Vampire用`role`字符串；WOC有私有world/player模型 | 两个产品都没有证明统一Zircon gameplay lifecycle |

`LevelSystem::set_lifecycle`、`register_subsystem`和`LevelLifecycleState::Unloaded`的生产检索只命中其定义及同文件测试；LevelManager生产路径也没有`remove_level/unload_level/levels.remove`。因此不能把字段存在误报为生命周期完成。

## 5. 参考实现差异与适用边界

| 参考 | 可验证事实 | Zircon应吸收 | 不照抄 |
|---|---|---|---|
| Unreal Engine | GameInstance持WorldContext、LocalPlayers与OnlineSession；GameMode负责权威login/spawn/travel，GameState/PlayerState复制；Controller/Pawn双向占有；LevelStreaming有完整中间态 | instance/world/player/rule/travel的owner、authority、状态机与回执语义 | UObject层级、宏、Actor成本和具体线程模型 |
| Lyra | Experience定义组合Game Feature、Action Set和Default Pawn Data，并有async loading/activation状态 | 用compiled ExperienceProgram组合产品规则与能力，不在session构造中硬编码 | Lyra具体资产类与Game Feature URL机制 |
| Bevy | App可拥有多个SubApp；Plugin有ready/finish/cleanup；State有OnEnter/OnExit/transition及state-scoped despawn | 多world/app隔离、显式初始化屏障、状态作用域清理 | 不强制Gameplay全部ECS-resource化或复制schedule API |
| Godot | SceneTree有current scene change/reload/unload、node enter/ready/exit和multiplayer authority | tree/world切换终态、节点生命周期、owner与authority通知 | 不复制单SceneTree全局模型或Node API |
| Fyrox | Engine持SceneContainer，Executor管理plugin/script init/update/deinit并支持headless | 多scene handle、脚本/插件与scene teardown顺序、headless同合同 | 不复制Scene Graph或插件trait签名 |
| Unity Graphics | 本地仓只展示scene load时选择RenderPipelineAsset | Travel commit要显式通知render pipeline/view generation | 该镜像不含完整Unity Player，不能支持Gameplay Framework完成度结论 |

共同基线不是类数量，而是：持久身份和live handle分离；每个实例、世界、玩家、占有与转场有唯一owner；authority和生命周期可枚举；异步工作有deadline/cancel/terminal receipt；旧generation资源能被确定回收。

## 6. 唯一 Owner、父子 Finding 与目标合同

Runtime05继续拥有World/ECS identity、clone/serialization、scheduler和partition父问题；Runtime08E拥有transport/replication；Runtime37拥有per-player camera director；App01拥有进程/窗口/host/shutdown；App03/06拥有WOC/Vampire产品闭环；Editor07/28拥有PIE和spawn/world-state authoring。本篇只拥有它们之间的Runtime gameplay composition与lifecycle，不重复计父P0。

建议新增`GameplayFrameworkService`，内部至少拆成`GameInstanceRegistry`、`WorldContextRegistry`、`ExperienceRuntime`、`GameRuleAuthority`、`PlayerDirectory`、`PossessionService`、`SpawnService`和`TravelCoordinator`。不要把这些职责重新塞进`RuntimeDynamicSession`或`LevelSystem`大对象。

核心发布合同`GameplayFrameSnapshotV1`至少包含：`build_set_id/game_instance_id/world_context_id/world_generation/experience_digest/rule_generation/match_phase/tick/local_players/player_states/controller_pawn_bindings/active_level_set/travel_epoch/network_epoch/save_epoch/disposition`。Input、Camera、Audio、UI、Network、Save和Script只能消费同代snapshot或提交typed command，不能保留可变World借用跨阶段执行。

## 7. P1：Capability、Source、Schema、Experience 与 Compiler

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| GPF-P1-001 | 没有Gameplay Framework capability/package identity | client/server/editor/headless分别声明provider、version、maturity与dependencies |
| GPF-P1-002 | Project只选择启动Scene | 定义Game Definition source，引用instance class、startup experience、maps与policies |
| GPF-P1-003 | 没有Experience/Game Mode source | versioned Experience定义rules、pawn/controller/player state、feature actions与platform variants |
| GPF-P1-004 | gameplay类型靠字符串和脚本属性 | stable type/field ID、typed refs、units、default、unknown-field与migration policy |
| GPF-P1-005 | 默认pawn/规则散落产品脚本 | Player/Pawn/Spawn/Match defaults进入唯一source truth |
| GPF-P1-006 | 没有dependency manifest | scene/prefab/script/input/camera/audio/UI/network/save/plugin依赖可追踪 |
| GPF-P1-007 | 没有cross-field admission | client/server/headless能力、default class、map与plugin依赖compile前fail-close |
| GPF-P1-008 | 没有deterministic compiler | 相同source/dependency/toolchain/target得到相同diagnostic、artifact与digest |
| GPF-P1-009 | 没有immutable ExperienceProgram | 固化rule graph、class factories、startup phases、replication/save adapters与cost |
| GPF-P1-010 | 没有activation/deactivation action plan | action具owner、order、rollback、timeout、server/client applicability和terminal state |
| GPF-P1-011 | 没有DDC/LKG/publication transaction | compile/activation失败保留同源last-good并报告stale/rollback |
| GPF-P1-012 | 没有upgrade/downgrade receipt | schema独立版本化，迁移生成loss、backup、source/artifact与rollback证据 |

## 8. P1：Game Instance、World Context、Level Registry 与 Travel

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| GPF-P1-013 | dynamic session代替Game Instance | GameInstance有stable ID、owner、Init/Ready/Start/Stopping/Stopped状态 |
| GPF-P1-014 | 没有service scope | instance/world/player/match service明确创建、查询、依赖与逆序teardown |
| GPF-P1-015 | 没有WorldContext | 保存WorldType、instance、current/pending world、net/PIE/travel identity与generation |
| GPF-P1-016 | session只能持一个Level | World Context拥有persistent world与active/streamed level set |
| GPF-P1-017 | WorldHandle仅裸u64递增 | 使用owner+slot+generation并checked exhaustion，拒绝stale/cross-instance handle |
| GPF-P1-018 | Level registry不可enumerate | 提供bounded snapshot、filter、active/pending/retiring状态与diagnostic |
| GPF-P1-019 | LevelManager没有remove/unload | request -> quiesce -> detach -> release -> removed，支持deadline/cancel/receipt |
| GPF-P1-020 | lifecycle只有Loaded/Unloaded | 显式Requested/Loading/LoadedHidden/MakingVisible/Visible/Unloading/Failed/Removed |
| GPF-P1-021 | load总是插入新Level | URI/source digest幂等、duplicate policy、lease/refcount与replace transaction |
| GPF-P1-022 | transition request无consumer | TravelCoordinator消费queued request并返回Succeeded/Failed/Superseded/Rejected |
| GPF-P1-023 | ReplaceActive没有prepare/commit | preload/validate/activate/handoff/commit/retire，失败保持旧world可运行 |
| GPF-P1-024 | 没有seamless/non-seamless策略 | 明确preserved objects/player state/net connection、transition world与fallback map |

## 9. P1：Local Player、Controller、Pawn、Possession、Input 与 View

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| GPF-P1-025 | 没有LocalPlayer实体 | platform user、device set、viewport、controller、profile与save identity一一归属 |
| GPF-P1-026 | player_id只是可空字符串 | typed PrincipalId/PlayerId/ConnectionId/LocalPlayerId分域且不可伪造 |
| GPF-P1-027 | 没有PlayerDirectory | join/leave/reconnect/spectate/bot/local split-screen使用同一registry与generation |
| GPF-P1-028 | 没有PlayerState | 复制公开状态、私有owner状态、persistent travel/save状态明确分层 |
| GPF-P1-029 | 没有Controller owner | Player/AI/Replay/Spectator controller有authority、lifecycle和input/view adapters |
| GPF-P1-030 | 没有Pawn contract | controllable body、movement/input receipt、controller backlink与player-state link typed化 |
| GPF-P1-031 | 没有Possess/Unpossess事务 | validate authority -> revoke old -> bind both sides -> input/view/audio handoff -> publish |
| GPF-P1-032 | 占有不参与tick ordering | controller intent在pawn simulation前，结果在camera/network extract前稳定发布 |
| GPF-P1-033 | gameplay读取全局InputSnapshot | InputUser/device -> LocalPlayer -> Controller映射，支持UI consume、focus与remap |
| GPF-P1-034 | session固定Orbit controller | development navigation按profile安装，shipping view由possessed controller/director拥有 |
| GPF-P1-035 | Vampire硬编码camera entity 1 | Controller按PlayerId/ViewId请求Runtime37 Director lease，不使用裸entity常量 |
| GPF-P1-036 | 无玩家音频/反馈路由 | listener、rumble、cursor、IME与HUD绑定local player/view并在unpossess时释放 |

## 10. P1：Game Mode、Game State、Spawn、Match 与 Script Facade

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| GPF-P1-037 | 没有权威Game Rule owner | server/listen/local authority运行GameRuleProgram，client只消费复制结果 |
| GPF-P1-038 | 没有Game State snapshot | match phase、rules digest、players、time、score与travel state按可见性复制 |
| GPF-P1-039 | 没有BeginPlay屏障 | World ready、experience active、required levels visible、players ready后统一开始 |
| GPF-P1-040 | 没有match phase状态机 | Entering/Waiting/Starting/InProgress/Ending/Leaving有guard、timeout与receipt |
| GPF-P1-041 | login与玩法无连接 | PreLogin/admission -> PlayerState -> Controller -> spawn/possess -> PostLogin顺序可证 |
| GPF-P1-042 | logout直接缺失 | unpossess、save/replication finalization、player removal和connection close逆序执行 |
| GPF-P1-043 | Spawn直接创建裸World node | SpawnRequest含class/archetype、owner、authority、transform、collision与budget |
| GPF-P1-044 | 没有deferred spawn | allocate identity -> construct components -> validate -> finish/rollback原子化 |
| GPF-P1-045 | 没有Player Start选择 | typed start points、team/tag/reservation/occupancy/query与deterministic tie-break |
| GPF-P1-046 | 没有respawn/restart | death/spectate/cooldown/start selection/spawn/possess生成单一transaction receipt |
| GPF-P1-047 | script直接spawn/despawn/改transform | script只提交bounded gameplay command，authority/policy owner执行并返回结果 |
| GPF-P1-048 | `role="player"`是产品私有真相 | script查询typed Player/Pawn/Team/GameplayTag projection，不扫描动态JSON语义 |

## 11. P1：Network、Save、PIE、Headless 与 Multi-World

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| GPF-P1-049 | handshake player_id未绑定framework | Welcome/Join原子创建或恢复PlayerState/Controller并记录principal binding |
| GPF-P1-050 | authority只在Sync DTO | World/GameRule/Possession/Spawn command统一执行server authority admission |
| GPF-P1-051 | Controller/Pawn没有复制合同 | owner/observer relevance、prediction、correction和late binding显式定义 |
| GPF-P1-052 | 无join-in-progress snapshot | experience/rules/game state/levels/players/possession按同一network epoch发布 |
| GPF-P1-053 | 无disconnect/reconnect lease | grace deadline、controller/pawn policy、session resume token与terminal cleanup |
| GPF-P1-054 | 无network travel协议 | server announce、client preload/ack、commit tick、failure/kick/retry和map digest |
| GPF-P1-055 | Scene save代替game save | SaveGame schema区分instance/player/game state/pawn/streamed level与transient字段 |
| GPF-P1-056 | 无restore ordering | validate artifact -> create context -> activate experience -> load levels -> players -> possess |
| GPF-P1-057 | 无checkpoint/rollback generation | async save绑定world/rule/player generation，late completion不能覆盖新state |
| GPF-P1-058 | PIE没有独立World Context合同 | 每个PIE实例有ID、port/user/save/DDC namespace和严格teardown隔离 |
| GPF-P1-059 | Headless映射为ClientRuntime语义 | dedicated/headless拥有无window的规则、玩家、travel、save和shutdown同合同 |
| GPF-P1-060 | multi-world未隔离 | frontend/menu/game/replay/preview world不共享input、net、script、camera或cache状态 |

## 12. P1：Performance、Budget、Observability 与 Product Qualification

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| GPF-P1-061 | manager HashMap只增不减 | retirement/prune budget、leak counter、high-water mark与soak gate |
| GPF-P1-062 | VM type sync锁全部Level并clone全部World | generation plan、bounded batches、copy-on-write journal与短commit窗口 |
| GPF-P1-063 | transition单槽resource静默覆盖 | bounded priority queue、dedupe/supersede policy、backpressure与每请求receipt |
| GPF-P1-064 | spawn/despawn无frame budget | per-world/per-owner count/bytes/time limits与deferred overflow disposition |
| GPF-P1-065 | player iteration没有稳定snapshot | contiguous generation snapshot、delta publication和reader lifetime budget |
| GPF-P1-066 | travel没有I/O/CPU/GPU预算 | preload bytes、resident overlap、compile/upload time与minimum-memory fallback |
| GPF-P1-067 | 没有lifecycle trace | instance/world/level/experience/player/possession/travel span共享correlation IDs |
| GPF-P1-068 | 没有health snapshot | counts、states、ages、pending work、leaks、stale rejects和last failure可查询 |
| GPF-P1-069 | 没有fault matrix | load/activate/login/spawn/possess/save/net/travel每阶段注入失败并证明回滚 |
| GPF-P1-070 | 测试不覆盖framework | 建立contract/integration/fault/soak/perf/network/PIE/headless测试层 |
| GPF-P1-071 | Vampire只证明单玩家脚本演示 | 至少验证2 local players、AI possession、death/respawn、travel/save/reopen |
| GPF-P1-072 | WOC私有player/world旁路 | WOC client/server通过统一Player/Rule/Travel合同并做reference parity |

## 13. P2：完整性与长期演进

| Finding | 后续要求 |
|---|---|
| GPF-P2-001 | 支持spectator、replay controller与kill-cam possession policy |
| GPF-P2-002 | 支持split-screen动态join/leave和viewport重排 |
| GPF-P2-003 | 支持platform account切换、guest与cross-play identity migration |
| GPF-P2-004 | 支持frontend/menu world与game world并行预热 |
| GPF-P2-005 | 支持seamless travel transition world与跨图persistent actor policy |
| GPF-P2-006 | 支持level instance/data layer与per-player streaming source |
| GPF-P2-007 | 支持server migration与host handoff的显式Unsupported/qualified profile |
| GPF-P2-008 | 支持replay/save/network共用稳定event schema但独立retention policy |
| GPF-P2-009 | 支持bot/AI controller与human takeover可逆切换 |
| GPF-P2-010 | 支持player-scoped accessibility、input、camera和audio profile |
| GPF-P2-011 | 支持world-context-aware console/debug command与权限审计 |
| GPF-P2-012 | 支持game feature热激活时existing player/pawn迁移或fail-close |
| GPF-P2-013 | 支持mod/plugin扩展Game Rule而不覆盖核心authority |
| GPF-P2-014 | 支持大规模server的player/level分片与interest handoff |
| GPF-P2-015 | 支持跨版本save/travel/reconnect兼容矩阵与downgrade说明 |
| GPF-P2-016 | 建立同场景同规则同网络条件的UE/Fyrox/Bevy/Godot基准记录 |

## 14. Hard Cutover 与禁止保留的双轨

1. `request_scene_transition`在TravelCoordinator consumer与terminal result可用前必须标为Unavailable，不能继续返回伪成功ID。
2. 新框架启用后，脚本不得直接以裸entity执行player spawn/despawn/possession/travel；旧API一次性迁移并删除。
3. `RuntimeDynamicSession.level`不再代表全局当前世界；切换为`WorldContextHandle`后删除隐式单Level假设。
4. `role="player"`、硬编码camera entity和全局WASD路径迁移到typed Player/Pawn/Input binding，不保留第二authority。
5. `WorldHandle(u64)`旧值不得静默升级；必须经owner/generation migration或明确拒绝。
6. LevelManager在remove/unload/retirement完成前不得宣称支持multi-level或travel。
7. WOC私有player/world模型若继续存在，只能作为domain state挂接统一framework identity/lifecycle，不能并行拥有login/possession/travel真相。

## 15. 重构里程碑

### M0 · Truth Freeze 与伪能力关闭

- 将scene transition capability降为Unavailable或接入明确Rejected terminal result；
- 冻结Game Definition、World Context、Player/Pawn/Possession/Travel schema；
- 为现有single-session/single-level行为补source-bound contract baseline。

### M1 · Game Instance 与 World Context

- 建立instance/context typed identity、state machine与service scopes；
- dynamic session只持instance/context handle，不直接拥有当前Level真相；
- client/listen/dedicated/headless/PIE使用同一合同和不同profile。

### M2 · Level Registry 与 Travel Transaction

- Level handle升级owner+generation；补enumerate/load/visibility/unload/remove/retire；
- 实现prepare/commit/rollback和terminal receipt；
- 先完成non-seamless ReplaceActive，再扩展streaming与seamless travel。

### M3 · Experience、Game Rule 与 Match State

- Game Definition/Experience source进入compiler、artifact和activation plan；
- 权威GameRule与复制GameState分离；
- BeginPlay、match phase、feature action和teardown有确定顺序。

### M4 · Player、Controller、Pawn 与 Possession

- 接通principal/local player/player state/controller/pawn identity；
- 实现join/login/logout/spawn/possess/unpossess/respawn事务；
- Input/Camera/Audio/UI按local player/view重新路由。

### M5 · Network、Save 与 Script Facade

- network join/reconnect/travel绑定framework epoch；
- SaveGame按instance/player/world/gameplay state分层并验证restore order；
- script direct World mutation迁移到bounded command/result。

### M6 · Product Cutover

- Vampire删除role字符串玩家authority、硬编码camera ID与全局输入；
- WOC native client/server接入统一identity/rules/possession/travel；
- Editor PIE/Spawn工具只编辑source并消费runtime receipts。

### M7 · Failure、Soak、Performance 与 Competitive Gate

- 完成多world、多player、travel循环、reconnect、save/restore fault与soak；
- 建立CPU/RSS/alloc/lock/I/O/GPU overlap预算；
- correctness和failure gates通过后再与参考引擎做同负载性能比较。

## 16. 验收门（40项）

| Gate | 验收内容 |
|---|---|
| GPF-G01 | 每个Game Instance有stable ID、generation、owner和唯一terminal shutdown |
| GPF-G02 | World Context明确WorldType、instance、current/pending world与travel epoch |
| GPF-G03 | client/listen/dedicated/headless/PIE profile均通过同一contract suite |
| GPF-G04 | Level handle拒绝stale、cross-instance与exhausted allocation |
| GPF-G05 | Level registry支持bounded enumerate并显示全部中间态 |
| GPF-G06 | load重复URI按显式dedupe/replace policy处理 |
| GPF-G07 | unload/remove等待script/physics/render/network reader退出并释放内存 |
| GPF-G08 | 10,000次load/unload soak无Level/World/task/asset单调泄漏 |
| GPF-G09 | scene transition每个request都有唯一terminal result |
| GPF-G10 | transition supersede/cancel/timeout不会丢失或复用request ID |
| GPF-G11 | preload失败保持旧World可tick、可render、可network |
| GPF-G12 | commit tick对World/Input/Camera/Audio/UI/Network同代可见 |
| GPF-G13 | Experience artifact由source/dependency/toolchain/target完整寻址 |
| GPF-G14 | activation action失败按逆序回滚且不遗留provider |
| GPF-G15 | GameRule只在authority执行，client mutation被拒绝并记录 |
| GPF-G16 | BeginPlay不早于world/experience/required-level readiness |
| GPF-G17 | match phase所有转换有guard、timeout、reason与receipt |
| GPF-G18 | LocalPlayer绑定platform user、device、viewport与profile |
| GPF-G19 | network PlayerId不能伪造为另一个principal/local player |
| GPF-G20 | join-in-progress得到同一epoch的rules/world/player snapshot |
| GPF-G21 | login按admission/player state/controller/spawn/possess顺序完成 |
| GPF-G22 | logout/unpossess/cleanup在disconnect与shutdown下幂等 |
| GPF-G23 | Possession双向绑定原子提交，失败恢复old binding |
| GPF-G24 | Controller tick在Pawn simulation前且Camera extract读同代结果 |
| GPF-G25 | split-screen玩家输入、camera、audio、HUD互不串线 |
| GPF-G26 | UI consume/focus loss阻止gameplay action并有可执行测试 |
| GPF-G27 | Spawn collision/admission失败不留下entity或partial components |
| GPF-G28 | respawn transaction正确处理spectator、cooldown、start与possession |
| GPF-G29 | script无法绕过authority直接possess/travel或跨world改entity |
| GPF-G30 | camera与player绑定不含硬编码entity ID或全局active camera假设 |
| GPF-G31 | reconnect grace过期与成功恢复都有terminal cleanup证据 |
| GPF-G32 | network travel验证map/experience/build digest与commit tick |
| GPF-G33 | SaveGame不持久化裸live handle、锁、task或transient pointer |
| GPF-G34 | restore按schema migration与dependency readiness顺序执行 |
| GPF-G35 | async save late completion不能覆盖新world/player generation |
| GPF-G36 | 多World Context并行tick时Input/Net/Script/Camera/cache严格隔离 |
| GPF-G37 | fault injection覆盖load/activate/login/spawn/possess/save/travel各阶段 |
| GPF-G38 | diagnostics可重建instance/world/level/player/possession/travel因果链 |
| GPF-G39 | Vampire与WOC完成统一framework产品闭环和source-bound evidence |
| GPF-G40 | `git diff --check`、Markdown LF/BOM/trailing、frontmatter路径/重复、索引/coverage/链接与finding计数全部通过 |

## 17. 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Runtime/测试/产品/参考冻结 | review_complete | 2026-08-16 | 129文件、53,564行、2,301,933 bytes、复合SHA见第2节 |
| 产品概念入口检索 | review_complete | 2026-08-16 | 18命中/10文件，均为Editor静态文案或script migration测试，无Runtime owner |
| transition consumer审计 | review_complete | 2026-08-16 | request只写World resource；host ABI/collector无该variant；0产品consumer |
| Level lifecycle审计 | review_complete | 2026-08-16 | manager无remove/unload；Unloaded/subsystems仅测试写入 |
| 差距与里程碑 | review_complete | 2026-08-16 | 0 P0 / 72 P1 / 16 P2 / 40 gates |
| Production重构 | pending | - | 本篇不修改production、tests、Cargo或workflow |
