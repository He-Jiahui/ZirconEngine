---
title: Runtime Gameplay Framework、Game Instance、World Context、Level、Game Mode、Game State、Local Player、Controller、Pawn、Possession、Spawn、Travel、Network、Save 与 Product Integration Current Source Review
category: zircon_runtime
report_id: Runtime127
review_date: 2026-08-23
baseline_head: 369ddacdd48498beaccc436a5c712d258c4b20d9
observed_head: 471bb732e3683fd7c12d7b69a9e85a22048efcba
baseline_epoch: 380
supersedes:
  - docs/plans/optimize/zircon_runtime/38-gameplay-framework-game-instance-world-context-level-game-mode-game-state-local-player-controller-pawn-possession-spawn-travel-network-save-scalability-product-integration-review.md
related_code:
  - zircon_runtime/src/core/framework/scene
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module
  - zircon_runtime/src/scene/world
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime/src/asset/project/manifest
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host
  - zircon_runtime/src/input
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/core/framework/net
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/session/session.rs
  - zircon_app/src/entry/entry_runner/headless.rs
  - zircon_app/src/entry/runtime_entry_app
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - examples/vampire/zircon-project.toml
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/scripts/vampire_game/main.zr
  - examples/woc/native/apps/woc_client/src/application.rs
  - examples/woc/native/apps/woc_client/src/shell/offline_session.rs
  - examples/woc/native/apps/woc_server/src/main.rs
  - examples/woc/native/plugins/woc_runtime/src/transaction.rs
  - examples/woc/scripts/woc_game/src/main.zr
tests:
  - zircon_runtime/src/scene/tests
  - zircon_runtime/src/dynamic_api/session/tests
  - zircon_runtime/src/dynamic_api/tests
  - zircon_runtime/src/script/vm/gameplay_host/tests.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests
  - zircon_runtime/src/input/tests
  - zircon_runtime/src/core/framework/net/tests.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards
plan_sources:
  - docs/plans/optimize/zircon_runtime/38-gameplay-framework-game-instance-world-context-level-game-mode-game-state-local-player-controller-pawn-possession-spawn-travel-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_runtime/37-camera-endpoint-director-rig-controller-blend-shake-cinematic-cut-history-multiview-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/40-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_app/06-vampire-roguelite-example-project-asset-script-gameplay-evidence-product-integration-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/zircon_runtime/runtime/13/failure-2026-08-15-gameplay-scene-transition-ledger-sync.md
  - docs/plans/zircon_runtime/runtime/10/failure-2026-07-19-project-script-scene-transition-host-request.md
  - docs/plans/zircon_runtime/runtime/07/failure-2026-07-22-level-system-runtime-state-frame-snapshot.md
  - docs/plans/zircon_runtime/runtime/10/failure-2026-07-17-woc-runtime-host-client-server-extensibility.md
  - docs/plans/zircon_runtime/runtime/04/failure-2026-07-18-project-source-index-targeted-import.md
  - docs/plans/zircon_runtime/frameworks/01/failure-2026-07-30-scene-animation-optional-feature-compile-boundary.md
  - docs/plans/zircon_editor/editor/05/failure-2026-07-31-scene-mode-input-ownership-hardcut.md
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
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
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99zb · Runtime Gameplay Framework Current Source Review

## 1. 结论

Runtime38 对 gameplay framework 系统缺口的主裁决仍然成立，但当前源码已经出现若干必须保留的工程底座：`zr.zircon.gameplay` 模块有真实 capability/function ledger；dynamic session registry 有有界 handle、closing/teardown retry 与 allocation drain；`LevelSystem` 有 world replacement epoch、generation-CAS、帧快照和动态场景 prepare/commit；Level scene save 有 bounded async ticket；Runtime UI 能先于 gameplay 消费输入；WOC 私有事务运行时有角色、generation、digest、故障与热重载回滚。这些能力不能被新框架重写或旁路。

但这些底座尚未组成 gameplay 产品。排除测试后，`GameInstance`、`WorldContext`、`GameMode`、`GameState`、`LocalPlayer`、`PlayerController`、`Pawn`、`Possession`、`ExperienceProgram`、`PlayerDirectory` 与 `TravelCoordinator` 均无 production owner；联合精确词检索只剩脚本热重载 fixture 中的 `PlayerState` 类型字符串。`RuntimeDynamicSession` 仍直接拥有一个 `CoreRuntime`、一个 `LevelSystem`、一个全局 `InputManager` 和一个固定 Orbit camera controller，不能代表跨 World 存活的 Game Instance，也不能表达同一实例内的多 World Context。

最明确的伪闭环仍是 Scene Transition。脚本 capability 与版本化 request/result DTO 已存在，但 producer 只把一个 request 写入当前 World resource，后写会覆盖前写；host request ABI 和 dynamic session consumer 只处理 IME、rumble 与 cursor。全产品没有 travel queue、preload、commit、rollback、terminal receipt 或旧 World retirement。返回 request ID 并不代表转场发生。

玩家链同样仍由产品私有约定替代。Vampire 以动态字符串 `role="player"` 扫描实体、读取全局 WASD，并把 camera entity `1` 硬编码给 `camera_follow`；WOC 有规模更大的私有 world/player/transaction 模型，却没有通过引擎级 Player、Rule、Possession、Travel、Save 和 Network lifecycle。两者分别证明脚本演示和产品私有运行时可运行，不能证明 Zircon Gameplay Framework 完成。

本轮对 Runtime38 原有账本按当前源码重判为：**0 项本地新增 P0；49 P1 Open、23 P1 Partial、0 Closed；16 P2 Open；30 Gate Fail、9 Gate Partial、1 Gate Pass**。没有新增 finding。目标不是复制 Unreal 的 UObject 类层次，而是建立同等工程语义：`GameplayFrameworkService -> GameInstanceRegistry -> WorldContextRegistry -> ExperienceRuntime/GameRuleAuthority -> LevelSet -> PlayerDirectory -> Controller/Pawn possession -> per-player Input/View/Audio -> Network/Save/Travel adapters`，所有转换必须 generation-scoped、异步可取消、事务发布、具有 terminal receipt，并在 client/listen/dedicated/headless/PIE/multi-world 下隔离。

“性能和表现优于当前 Unreal”目前没有可支持的结论：仓库没有同场景、同硬件、同画质、同网络条件的原始 benchmark receipt。正确顺序是先关闭 owner/currentness/lifecycle/correctness，再按 frame time、tail latency、CPU/RSS/VRAM、travel overlap、spawn throughput 与 dedicated-server capacity 建立可复现资格；不能用少做功能或空实现得到的低开销冒充性能优势。

本轮只做静态源码 review 和计划记录，没有修改 production、tests、Cargo、ABI 或参考源码；没有运行真实 Editor、Vampire/WOC 产品、网络、save/reopen、PIE、fault/soak/profile 或跨引擎 benchmark。Tooling 按用户要求不在范围内。MVP 未完成，`source_recheck_required` 保持 true。

## 2. 审查边界与物理冻结

### 2.1 Focused 集合

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Instance / world / level / session source | 40 / 7,895 / 7,159 / 287,164 / 67 / 1 | `28d67523898b05955e0a1caf1a9a0e2ccb10704348f51d1b1afac0ac57ff25d8` |
| Gameplay / input / net / save adapters | 70 / 9,257 / 8,408 / 310,992 / 41 / 1 | `4159e803c4cc1778da2aa3c06a67354e410e50e0e90e40c278819b30d64180a2` |
| App and product integration | 20 / 5,814 / 5,281 / 271,534 / 50 / 0 | `e16e3e31012076ded172b3272520ff1e35316ce911fd9999235cc4694093d152` |
| Focused tests | 25 / 7,013 / 6,467 / 258,463 / 138 / 4 | `be20152f0205d3831232ab71bd702d22855c0a840c29c0c0c0849b5c7f82225f` |
| Zircon deduplicated focused total | 153 / 29,074 / 26,502 / 1,094,769 / 281 / 6 | `37eadd8fb62b2e3b1a2a7888c60a6661c943b556c30ea9fc8ac2d6bea37e4ee3` |
| Selected five-engine evidence | 25 / 12,983 / 11,371 / 992,933 / 26 Rust test declarations / 0 | `62f38e5e073df3a0b5aa057c9bb7bcc1980a68cb9c86abcbf7bced5e24c9d055` |

fingerprint 算法与本系列 current-source review 一致：仓库相对路径转 `/`、小写、ordinal 排序去重；每项编码为 `lowercase-path + NUL + lowercase per-file SHA-256`，以 LF 连接且末尾无 LF，再对 UTF-8 payload 计算 SHA-256。它冻结本轮实际读取集合，不是 runtime artifact、save、network 或 release identity。

### 2.2 Currentness 与共享工作树

- registration baseline 是 `369ddacdd48498beaccc436a5c712d258c4b20d9`，baseline epoch 为 380；报告落笔时共享 HEAD 已前移到 `152dd4562574e3d420029756d8f4282ff2b204b6`。
- 工作树含大量其他 session 与用户在途修改；本轮只租用本报告、Runtime 索引、根索引和覆盖台账，不回退、不归因其他文件。
- 旧 Runtime38 的 129 文件冻结已经不能代表当前实现；本轮按四个 Zircon 分组重新去重为 153 文件，并重新读取 25 个参考文件。
- source zero scan 对 12 个 gameplay 概念仅得到 8 个 fixture/test 字符串命中，没有 production type/owner/consumer。零命中本身不作结论，结论来自 source、owner、consumer、lifecycle、product test 同时缺失。

### 2.3 纵向检查链

本轮沿 `Project manifest -> gameplay module/capability -> dynamic session registry/construction -> Game Instance/World Context -> Level registry/lifecycle -> scene transition -> Experience/Rules/Match -> LocalPlayer/Controller/Pawn/Possession -> Input/UI/Camera -> Spawn/Despawn -> Net session/replication -> Save/Restore -> Headless/PIE/Multi-world -> Vampire/WOC` 逐层检查。参考源码只提取 owner、状态机、失败、异步与规模合同，不以类名相似度判断完成度。

## 3. 当前实现事实

### 3.1 Project、Capability 与 Session

1. `ProjectManifest` 真实保存 default scene、UI roots、asset roots/settings/manifest、plugins、scripts 和 export profiles；`ProjectScriptManifest` 只有 package roots 与 startup packages。没有 Game Definition、startup Experience、GameMode、default Pawn、player policy、travel policy 或 save/network policy。
2. `zr.zircon.gameplay` 模块与 `gameplay.input/entity/navigation/scene_transition` capability、function-level required capability 都是真实 ledger。这使 capability identity 不再是完全空白，但它只描述脚本 host surface，没有编译后的 gameplay program、依赖闭包、产品 profile 或 activation receipt。
3. `RuntimeDynamicSession` 直接持有一个 Core、一个 Level、一个 InputManager、一个 camera controller、Runtime UI、operation registry 与 host request output。construction 从 project default/play override scene 创建唯一 Level；每帧也只 tick 这个 Level。
4. dynamic session global registry 有 checked handle allocation、Open/Closing/TeardownRetryPending、action/wake drain、foreign allocation drain 与 module shutdown。这是可靠的 FFI session registry，不是 GameInstance/WorldContext/Player lifecycle。

### 3.2 Level、World 与 Scene Transition

1. `LevelSystem` 有 `world_replacement_epoch`、generation-CAS replacement、runtime-state reset、animation/script derived frame snapshots 和动态场景 preflight/commit。它已经具备扩展为 World Context currentness 的基础。
2. `LevelManager` 中立 trait 只提供 create/exists/summary/load/save；`WorldHandle(pub u64)` 没有 owner、slot、generation。`DefaultLevelManager` 用 checked monotonic ID 和 `HashMap` 保存所有 Level，没有 remove/unload/retirement。
3. lifecycle 只有 Loaded/Unloaded；公开 raw setter 和 subsystem string list 没有形成 Requested/Loading/Visible/Unloading/Failed/Removed 状态机。private snapshot 能稳定排序，但没有公共 bounded enumerate/filter。
4. VM type sync 仍在锁住 Level 集合时深拷贝所有 World 用于 rollback；`LevelSystem::snapshot()` 也克隆 World。frame snapshot 的进展没有关闭全域 clone 与长锁风险。
5. Level scene save 已有 bounded async `SceneArtifactTicket`、project-root/currentness validation 与 atomic write 基础；load 仍同步并总是新建 Level。这是 project scene I/O，不是 Gameplay SaveGame。
6. scene transition producer 使用 versioned request/policy/result/status DTO，但 World 中只有单槽 request resource；sequence 可 saturate，u64 request ID 又被窄化为 i64 返回。host ABI 和 dynamic session 没有 consumer，因此没有任何 terminal transition result。

### 3.3 Gameplay、Player、Input 与 Spawn

1. gameplay host 的 lifecycle/combat/components 直接调用 `World::spawn_node`、remove_entity、set transform 和 string-keyed dynamic component/JSON mutation；没有 authority、owner、class/archetype、construction phase、collision policy、budget 或 typed receipt。
2. 通用 Dynamic Scene spawn 已有 bounded compile/preflight、world generation check、transaction commit/rollback，并有 100k entity 规模测试。这是 SpawnService 可复用底座，但当前 script spawn 不经过它。
3. 全局 `InputManager` 与 action evaluator 是真实基础，Runtime UI 也能优先消费 input；仍没有 platform user/device -> LocalPlayer -> Controller 路由、split-screen isolation 或 focus/ownership 完整证据。
4. Runtime126 已证明 camera 侧有 render endpoint/history 底座但没有 per-player Director。dynamic session 仍固定创建 1280x720 Orbit controller；Vampire 的 camera entity、player role 和 follower entity仍为硬编码/字符串真相。

### 3.4 Network、Save、Headless 与 Product

1. Net 层有 typed runtime mode、endpoint/session/connection ID、handshake、RPC、reliable datagram、replication identity/authority/snapshot/delta/budget DTO；全仓没有 `impl NetManager`，这些结构没有接入 gameplay login/player/possession/travel consumer。
2. handshake 的 player identity 仍是 `Option<String>`；没有 PrincipalId、PlayerId、LocalPlayerId、PlayerState、Controller/Pawn binding 或 reconnect lease。
3. Level scene save 与 native plugin save/restore callbacks、WOC private committed snapshot/hot-reload rollback 是三类不同底座；没有统一 SaveGame participant、capture barrier、schema migration、restore ordering 或 generation-qualified publication。
4. `EntryConfig::Headless` 可映射 ServerRuntime 且无 primary window，但 `RuntimeDynamicSessionProfile::Headless::target_mode()` 仍返回 ClientRuntime；`EntryRunner::run_headless()` 只 bootstrap 后立即返回。无窗口入口存在，authoritative dedicated gameplay host 不存在。
5. 多 dynamic session 可以各自拥有 Core/Level/InputManager；这只是进程内多个独立 session，不是同一 GameInstance 的多个 World Context，也没有 frontend/game/replay/preview world 的资源隔离合同。
6. WOC private transaction 有 Offline/Server/Client role、generation/tick/digest、install snapshot、hot reload save/migrate/restore/rollback 与故障状态，说明复杂产品逻辑可验证；它继续绕过统一 engine gameplay owner。Vampire 的 real-ZrVM 产品测试仍有 ignore，不能作为 shipping qualification。

## 4. 五套参考引擎的可迁移工程合同

| 参考 | 本轮实际证据 | Zircon 应吸收的合同 | 不应照抄的部分 |
|---|---|---|---|
| Unreal Engine / Lyra | GameInstance 持 WorldContext、LocalPlayers、OnlineSession 与 Init/Shutdown；GameMode 负责 PreLogin/Login/PostLogin/Logout、spawn/restart/travel；Controller/Pawn 双向 possession；LevelStreaming 有显式中间态；Lyra Experience 组合 Pawn Data、Game Features、Actions/Action Sets 并异步激活 | instance/world/player/rule/travel 唯一 owner、authority、状态机、preserved set、terminal receipt；compiled Experience 组合产品策略 | 不复制 UObject 层次、宏、历史兼容层和默认 Actor 成本；Lyra 自身 async deactivation TODO 也不能照抄 |
| Bevy | App 可拥有、插入和移除 SubApps；Plugin 有 ready/finish/cleanup；State 有 transition ordering 与 state-scoped despawn | 多 world/app 隔离、显式 readiness barrier、状态作用域资源回收 | 不把所有 gameplay 语义压成 ECS resource，也不复制 schedule API |
| Godot | SceneTree 支持 change/reload/unload current scene；Node 有 enter/ready/exit；MultiplayerAPI 有 authority/peer 边界 | 场景切换终态、节点生命周期、authority 与 owner 通知 | 不复制单全局 SceneTree 或 Node API |
| Fyrox | Engine 持 SceneContainer；Executor 管理 plugin/script init/update/deinit 并支持 headless | 多 scene handle、脚本/插件与 scene teardown 顺序、headless 同合同 | 不复制 scene graph 或 plugin trait 形状 |
| Unity Graphics | 本地镜像只证明 scene load 可触发 RenderPipelineAsset 选择/通知 | travel commit 必须显式切换 render pipeline/view generation | 该镜像没有完整 Unity Player，不能支撑 gameplay 完成度或性能结论 |

五套参考共同支持的不是“类越多越工程化”，而是：持久身份与 live handle 分离；实例、世界、玩家、占有和转场各有唯一 owner；authority 与中间态可枚举；异步工作有 deadline/cancel/terminal receipt；旧 generation 资源能被确定回收。

## 5. 目标 Owner 与发布合同

Runtime05 继续拥有 World/ECS identity、clone/serialization、scheduler 与 partition；Runtime08E 拥有 transport/replication；Runtime37/126 拥有 camera director；Runtime40 拥有通用 SaveGame；App01 拥有进程/窗口/host/shutdown；App03/06 拥有 WOC/Vampire 产品闭环；Editor07 拥有 PIE。本文只拥有它们之间的 Runtime gameplay composition/lifecycle，不重复登记父 P0。

建议新增 `GameplayFrameworkService`，内部至少分为：

- `GameInstanceRegistry`：instance identity、service scope、lifecycle 与 terminal shutdown。
- `WorldContextRegistry`：WorldType、owner instance、active/pending World、LevelSet、net/PIE/travel generation。
- `ExperienceRuntime` 与 `GameRuleAuthority`：编译、激活、规则、BeginPlay 与 match phase。
- `PlayerDirectory`：principal/local/remote/bot/replay identity、PlayerState 与 reconnect lease。
- `PossessionService`：Controller/Pawn 双向 binding、input/view/audio handoff 与 tick barrier。
- `SpawnService`：typed request、authority/collision/budget、deferred construction 与 rollback。
- `TravelCoordinator`：queue、preload、commit、retire、network/save/render handoff 与 terminal receipt。

核心只读发布合同 `GameplayFrameSnapshotV1` 至少包含 `build_set_id/game_instance_id/world_context_id/world_generation/experience_digest/rule_generation/match_phase/tick/local_players/player_states/controller_pawn_bindings/active_level_set/travel_epoch/network_epoch/save_epoch/disposition`。Input、Camera、Audio、UI、Network、Save 与 Script 只能消费同代 snapshot 或提交 typed command，不能跨阶段持有可变 World 借用。

## 6. P1：Capability、Source、Schema、Experience 与 Compiler

| Finding | 状态 | 当前源码判定 | 目标合同 |
|---|---|---|---|
| GPF-P1-001 | Partial | `zr.zircon.gameplay` 与四项 capability/function ledger 已存在，但没有产品 package/profile/成熟度真相 | client/server/editor/headless 分别声明 provider、version、maturity 与 dependencies |
| GPF-P1-002 | Open | Project 只选择启动 Scene，没有 Game Definition | 引用 instance class、startup experience、maps 与 policies |
| GPF-P1-003 | Open | 没有 Experience/Game Mode source | versioned Experience 定义 rules、pawn/controller/player state、feature actions 与 platform variants |
| GPF-P1-004 | Open | gameplay 类型仍靠字符串与动态 JSON | stable type/field ID、typed refs、units、default、unknown-field 与 migration policy |
| GPF-P1-005 | Open | 默认 pawn/规则散落于产品脚本 | Player/Pawn/Spawn/Match defaults 进入唯一 source truth |
| GPF-P1-006 | Partial | Project 已记录 scene/script/plugin/UI/asset roots，但不是闭合 gameplay dependency manifest | scene/prefab/script/input/camera/audio/UI/network/save/plugin 依赖可追踪 |
| GPF-P1-007 | Partial | module/capability admission 有基础，尚不校验 gameplay cross-field/product profile | client/server/headless 能力、default class、map 与 plugin 依赖 compile 前 fail-close |
| GPF-P1-008 | Open | 没有 deterministic gameplay compiler | 相同 source/dependency/toolchain/target 得到相同 diagnostic、artifact 与 digest |
| GPF-P1-009 | Open | 没有 immutable ExperienceProgram | 固化 rule graph、class factories、startup phases、replication/save adapters 与 cost |
| GPF-P1-010 | Open | 没有 gameplay activation/deactivation action plan | action 具 owner、order、rollback、timeout、role applicability 与 terminal state |
| GPF-P1-011 | Open | 没有 gameplay DDC/LKG/publication transaction | compile/activation 失败保留同源 last-good 并报告 stale/rollback |
| GPF-P1-012 | Partial | Project/scene/plugin 已有局部版本与 migration 基础，gameplay schema 无独立 receipt | schema 独立版本化，迁移生成 loss、backup、source/artifact 与 rollback 证据 |

## 7. P1：Game Instance、World Context、Level Registry 与 Travel

| Finding | 状态 | 当前源码判定 | 目标合同 |
|---|---|---|---|
| GPF-P1-013 | Partial | dynamic session registry 有稳定销毁/重试基础，但 session 仍代替 Game Instance | GameInstance 有 stable ID、owner、Init/Ready/Start/Stopping/Stopped 状态 |
| GPF-P1-014 | Partial | Core/module/session scope 存在，instance/world/player/match scope 未定义 | service 明确创建、查询、依赖与逆序 teardown |
| GPF-P1-015 | Open | 没有 WorldContext | 保存 WorldType、instance、current/pending world、net/PIE/travel identity 与 generation |
| GPF-P1-016 | Open | session 只能持一个 Level | World Context 拥有 persistent world 与 active/streamed level set |
| GPF-P1-017 | Partial | WorldHandle 分配已 checked exhaustion，但仍是裸 u64，无 owner/slot/generation | 拒绝 stale/cross-instance handle |
| GPF-P1-018 | Partial | manager 有稳定排序 private snapshot，但无 public bounded enumerate/filter/states | 提供 active/pending/retiring 状态与 diagnostic |
| GPF-P1-019 | Open | LevelManager 没有 remove/unload | request -> quiesce -> detach -> release -> removed，支持 deadline/cancel/receipt |
| GPF-P1-020 | Open | lifecycle 只有 Loaded/Unloaded | Requested/Loading/LoadedHidden/MakingVisible/Visible/Unloading/Failed/Removed |
| GPF-P1-021 | Open | load 总是插入新 Level | URI/source digest 幂等、duplicate policy、lease/refcount 与 replace transaction |
| GPF-P1-022 | Open | transition request 无 consumer | TravelCoordinator 消费 queue 并返回 Succeeded/Failed/Superseded/Rejected |
| GPF-P1-023 | Open | ReplaceActive 没有 prepare/commit/retire | preload/validate/activate/handoff/commit/retire，失败保持旧 World 可运行 |
| GPF-P1-024 | Open | 没有 seamless/non-seamless 策略 | 明确 preserved objects/player state/net connection、transition world 与 fallback map |

## 8. P1：Local Player、Controller、Pawn、Possession、Input 与 View

| Finding | 状态 | 当前源码判定 | 目标合同 |
|---|---|---|---|
| GPF-P1-025 | Open | 没有 LocalPlayer 实体 | platform user、device set、viewport、controller、profile 与 save identity 一一归属 |
| GPF-P1-026 | Partial | connection/session ID 已 typed，player identity 仍是可空字符串 | PrincipalId/PlayerId/ConnectionId/LocalPlayerId 分域且不可伪造 |
| GPF-P1-027 | Open | 没有 PlayerDirectory | join/leave/reconnect/spectate/bot/local split-screen 使用同一 registry 与 generation |
| GPF-P1-028 | Open | 没有 PlayerState | 复制公开状态、私有 owner 状态、persistent travel/save 状态明确分层 |
| GPF-P1-029 | Open | 没有 Controller owner | Player/AI/Replay/Spectator controller 有 authority、lifecycle 与 adapters |
| GPF-P1-030 | Open | 没有 Pawn contract | controllable body、movement/input receipt、controller backlink 与 player-state link typed 化 |
| GPF-P1-031 | Open | 没有 Possess/Unpossess 事务 | validate authority -> revoke old -> bind both sides -> input/view/audio handoff -> publish |
| GPF-P1-032 | Open | possession 不参与 tick ordering | controller intent 在 pawn simulation 前，结果在 camera/network extract 前稳定发布 |
| GPF-P1-033 | Open | gameplay 读取全局 InputSnapshot | InputUser/device -> LocalPlayer -> Controller 映射，支持 UI consume、focus 与 remap |
| GPF-P1-034 | Open | session 固定 Orbit controller | development navigation 按 profile 安装，shipping view 由 possessed controller/director 拥有 |
| GPF-P1-035 | Open | Vampire 硬编码 camera entity 1 | Controller 按 PlayerId/ViewId 请求 Runtime126 Director lease |
| GPF-P1-036 | Partial | UI 优先消费及 IME/rumble/cursor 回执有基础，但全部仍是 session-global | listener、feedback、cursor、IME 与 HUD 绑定 local player/view 并在 unpossess 时释放 |

## 9. P1：Game Mode、Game State、Spawn、Match 与 Script Facade

| Finding | 状态 | 当前源码判定 | 目标合同 |
|---|---|---|---|
| GPF-P1-037 | Open | 没有权威 Game Rule owner | server/listen/local authority 运行 GameRuleProgram，client 只消费复制结果 |
| GPF-P1-038 | Open | 没有 Game State snapshot | match phase、rules digest、players、time、score 与 travel state 按可见性复制 |
| GPF-P1-039 | Open | 没有 BeginPlay 屏障 | World ready、experience active、required levels visible、players ready 后统一开始 |
| GPF-P1-040 | Open | 没有 match phase 状态机 | Entering/Waiting/Starting/InProgress/Ending/Leaving 有 guard、timeout 与 receipt |
| GPF-P1-041 | Open | login 与玩法无连接 | PreLogin/admission -> PlayerState -> Controller -> spawn/possess -> PostLogin |
| GPF-P1-042 | Open | logout gameplay lifecycle 缺失 | unpossess、save/replication finalization、player removal 与 connection close 逆序执行 |
| GPF-P1-043 | Partial | generic Dynamic Scene spawn 有 bounded transaction，script spawn 仍直接创建裸 node | SpawnRequest 含 class/archetype、owner、authority、transform、collision 与 budget |
| GPF-P1-044 | Partial | dynamic scene 有 prepare/commit/rollback，gameplay entity construction 未接入 | allocate identity -> construct components -> validate -> finish/rollback 原子化 |
| GPF-P1-045 | Open | 没有 Player Start 选择 | typed start points、team/tag/reservation/occupancy/query 与 deterministic tie-break |
| GPF-P1-046 | Open | 没有 respawn/restart | death/spectate/cooldown/start selection/spawn/possess 生成单一 receipt |
| GPF-P1-047 | Open | script 直接 spawn/despawn/改 transform | script 只提交 bounded gameplay command，authority owner 执行并返回结果 |
| GPF-P1-048 | Open | `role="player"` 是产品私有真相 | 查询 typed Player/Pawn/Team/GameplayTag projection，不扫描动态 JSON 语义 |

## 10. P1：Network、Save、PIE、Headless 与 Multi-World

| Finding | 状态 | 当前源码判定 | 目标合同 |
|---|---|---|---|
| GPF-P1-049 | Open | handshake player_id 未绑定 framework | Welcome/Join 原子创建或恢复 PlayerState/Controller 与 principal binding |
| GPF-P1-050 | Open | authority 只存在于 Sync DTO | World/GameRule/Possession/Spawn command 统一执行 server authority admission |
| GPF-P1-051 | Open | Controller/Pawn 没有复制合同 | owner/observer relevance、prediction、correction 与 late binding 显式定义 |
| GPF-P1-052 | Open | 无 join-in-progress snapshot | experience/rules/game state/levels/players/possession 按同一 network epoch 发布 |
| GPF-P1-053 | Open | 无 disconnect/reconnect lease | grace deadline、controller/pawn policy、resume token 与 terminal cleanup |
| GPF-P1-054 | Open | 无 network travel 协议 | server announce、client preload/ack、commit tick、failure/kick/retry 与 map digest |
| GPF-P1-055 | Partial | scene save ticket 与 WOC/plugin snapshot 底座存在，但都不是统一 Game Save | SaveGame schema 区分 instance/player/game state/pawn/streamed level/transient 字段 |
| GPF-P1-056 | Open | 无 gameplay restore ordering | validate -> create context -> activate experience -> load levels -> players -> possess |
| GPF-P1-057 | Partial | scene ticket currentness 与 WOC private generation rollback 有基础，未统一 | async save 绑定 world/rule/player generation，late completion 不覆盖新 state |
| GPF-P1-058 | Partial | Editor play/process 有局部隔离基础，无 WorldContext/port/user/save namespace 合同 | 每个 PIE 实例严格 teardown 隔离 |
| GPF-P1-059 | Partial | App headless 无窗口入口存在，但 profile 仍映射 ClientRuntime 且 runner 立即返回 | dedicated/headless 与窗口产品共享规则、玩家、travel、save、shutdown 合同 |
| GPF-P1-060 | Partial | 多 dynamic session 可隔离 Core/Level/Input，单 instance 多 WorldContext 仍缺失 | frontend/menu/game/replay/preview world 不共享 input/net/script/camera/cache 状态 |

## 11. P1：Performance、Budget、Observability 与 Product Qualification

| Finding | 状态 | 当前源码判定 | 目标合同 |
|---|---|---|---|
| GPF-P1-061 | Open | Level manager HashMap 只增不减 | retirement/prune budget、leak counter、high-water mark 与 soak gate |
| GPF-P1-062 | Partial | frame snapshot/currentness 已进展，VM type sync 与 World snapshot 仍全量 clone | generation plan、bounded batches、copy-on-write journal 与短 commit 窗口 |
| GPF-P1-063 | Open | transition 单槽 resource 静默覆盖 | bounded priority queue、dedupe/supersede、backpressure 与每请求 receipt |
| GPF-P1-064 | Partial | Dynamic Scene 有 count/bytes transaction budget，gameplay spawn/despawn 无 owner/frame budget | per-world/per-owner count/bytes/time limits 与 overflow disposition |
| GPF-P1-065 | Open | player iteration 无稳定 snapshot，因为 PlayerDirectory 不存在 | contiguous generation snapshot、delta publication 与 reader lifetime budget |
| GPF-P1-066 | Open | travel 无 I/O/CPU/GPU 预算 | preload bytes、resident overlap、compile/upload time 与 minimum-memory fallback |
| GPF-P1-067 | Partial | 通用 profiling/log/session correlation 基础存在，无 gameplay 跨域 trace | instance/world/level/experience/player/possession/travel 共享 correlation IDs |
| GPF-P1-068 | Partial | session/operation/level 有局部 diagnostics，无统一 gameplay health snapshot | counts、states、ages、pending、leaks、stale rejects 与 last failure 可查询 |
| GPF-P1-069 | Partial | dynamic scene/WOC/session 有局部 fault tests，无 gameplay 阶段矩阵 | load/activate/login/spawn/possess/save/net/travel 注入失败并证明回滚 |
| GPF-P1-070 | Partial | 底层 contract tests 丰富，framework/product 层仍无 owner 可测 | contract/integration/fault/soak/perf/network/PIE/headless 分层测试 |
| GPF-P1-071 | Open | Vampire 只证明单玩家脚本演示，且 real-ZrVM 测试有 ignore | 验证 2 local players、AI possession、death/respawn、travel/save/reopen |
| GPF-P1-072 | Open | WOC 私有 player/world/transaction 旁路统一框架 | WOC client/server 通过统一 Player/Rule/Travel 合同并做 reference parity |

## 12. P2：完整性与长期演进

| Finding | 状态 | 目标 |
|---|---|---|
| GPF-P2-001 | Open | 支持 spectator、replay controller 与 kill-cam possession policy |
| GPF-P2-002 | Open | 支持 split-screen 动态 join/leave 和 viewport 重排 |
| GPF-P2-003 | Open | 支持 platform account 切换、guest 与 cross-play identity migration |
| GPF-P2-004 | Open | 支持 frontend/menu world 与 game world 并行预热 |
| GPF-P2-005 | Open | 支持 seamless travel transition world 与跨图 persistent actor policy |
| GPF-P2-006 | Open | 支持 level instance/data layer 与 per-player streaming source |
| GPF-P2-007 | Open | 支持 server migration/host handoff 的显式 Unsupported 或 qualified profile |
| GPF-P2-008 | Open | replay/save/network 共用稳定 event schema 但独立 retention policy |
| GPF-P2-009 | Open | bot/AI controller 与 human takeover 可逆切换 |
| GPF-P2-010 | Open | player-scoped accessibility、input、camera 与 audio profile |
| GPF-P2-011 | Open | world-context-aware console/debug command 与权限审计 |
| GPF-P2-012 | Open | game feature 热激活时 existing player/pawn 迁移或 fail-close |
| GPF-P2-013 | Open | mod/plugin 扩展 Game Rule 而不覆盖核心 authority |
| GPF-P2-014 | Open | 大规模 server 的 player/level 分片与 interest handoff |
| GPF-P2-015 | Open | 跨版本 save/travel/reconnect 兼容矩阵与 downgrade 说明 |
| GPF-P2-016 | Open | 建立同场景、同规则、同网络条件的 UE/Fyrox/Bevy/Godot 基准记录 |

## 13. Hard Cutover 与禁止双轨

1. 不把 `RuntimeDynamicSession` 政名为 GameInstance；它保留 ABI/session 责任，通过唯一 `GameplayFrameworkService` 组合 gameplay owner。
2. 不把 `LevelSystem` 政名为 WorldContext；WorldContext 拥有 LevelSet、net/PIE/travel identity，LevelSystem 继续拥有单 World runtime state。
3. 不保留 `WorldHandle(u64)` 与新 generational handle 双轨；迁移后删除 raw/public construction、compat alias 与 `pub use` shim。
4. 不保留 script 直接改 World 与 typed gameplay command 双 authority；所有 shipping mutation hard cut 到 admission/receipt。
5. 不保留单槽 scene-transition resource 作为 compatibility path；consumer/queue 落地前 capability 必须 fail-close 或明确 Unsupported。
6. 不把 WOC 私有 player/world/save transaction 包装成 engine API；先抽取通用合同，再让 WOC 成为真实 consumer。
7. `framework` 只保留中立 trait/schema/DTO；GameInstance、WorldContext、Rules、Player、Possession、Travel 的业务生命周期进入 runtime owner。

## 14. 重构里程碑

### GPF-M0 · Truth Freeze 与伪能力关闭

- 固化 72 P1/16 P2/40 Gate、owner 表、公开 surface 与 product negative tests。
- Scene Transition 在无 consumer 时 fail-close；禁止继续以 request ID 伪报成功。
- 定义 package/capability maturity，删除字符串 player/camera gameplay truth。

### GPF-M1 · Game Instance 与 World Context

- 建立 typed instance/context/handle/generation 与 service scope。
- dynamic session 只组合 owner；完成 Init/Ready/Start/Stopping/Stopped 与逆序 teardown。
- client/listen/dedicated/headless/PIE 共享 contract suite。

### GPF-M2 · Level Registry 与 Travel Transaction

- Level registry 支持 bounded enumerate、load state、unload/remove/retire 与 leak budgets。
- 建立 request queue、preload、commit、rollback、retire 和 terminal receipt。
- render/input/audio/UI/network/save 在同一 travel epoch 切换。

### GPF-M3 · Experience、Game Rule 与 Match State

- 增加 versioned Game Definition/Experience source、deterministic compiler 和 immutable program。
- 建立 activation plan、GameRule authority、GameState snapshot、BeginPlay barrier 与 match state machine。
- compile/activation 具 LKG、rollback、diagnostic 与 role applicability。

### GPF-M4 · Player、Controller、Pawn 与 Possession

- 建立 Principal/Player/LocalPlayer/Controller/Pawn typed identity 和 PlayerDirectory。
- Possess/Unpossess 原子更新双向 binding，并同步 input/view/audio/UI/network。
- 建立 spawn/start/respawn transaction 与 tick-order barrier。

### GPF-M5 · Network、Save 与 Script Facade

- login/reconnect/JIP/network travel 接入同代 gameplay snapshot。
- SaveGame participant/capture/restore/generation 接入 instance/world/player/rule。
- script hard cut 为 bounded typed command，关闭 direct World mutation。

### GPF-M6 · Product Cutover

- Vampire 移除 role/camera/entity 常量，完成双 local player、AI possession、respawn、travel/save。
- WOC client/server/headless 接入统一 Player/Rule/Travel/Save/Network owner。
- PIE 使用真实 WorldContext/namespace，而不是只靠进程或 session 物理隔离。

### GPF-M7 · Fault、Soak、Performance 与 Competitive Evidence

- 对 load/activate/login/spawn/possess/save/net/travel 注入失败并验证 rollback。
- 建立 10,000 次 load/unload、join/leave、possess、travel soak 与 memory/task/handle retirement。
- 用同场景/硬件/画质/网络的 raw receipt 比较 UE/Fyrox/Bevy/Godot；先证明语义等价，再比较性能。

## 15. 验收门（40 项）

| Gate | 状态 | 验收条件 / 当前证据 |
|---|---|---|
| GPF-G01 | Fail | 每个 GameInstance 有 stable ID、generation、owner 和唯一 terminal shutdown；owner 不存在 |
| GPF-G02 | Fail | WorldContext 明确 WorldType、instance、current/pending world 与 travel epoch；类型不存在 |
| GPF-G03 | Partial | App 有 client/server/headless 入口底座，但 headless dynamic profile 仍是 ClientRuntime，PIE/角色合同未统一 |
| GPF-G04 | Partial | allocation exhaustion 已 checked；handle 仍无 owner/generation，不能拒绝 stale/cross-instance |
| GPF-G05 | Partial | private stable snapshot 存在；public bounded enumerate 与完整中间态缺失 |
| GPF-G06 | Fail | load 重复 URI 没有显式 dedupe/replace policy |
| GPF-G07 | Fail | unload/remove/quiesce/release 不存在 |
| GPF-G08 | Fail | 无 10,000 次 load/unload 无泄漏 raw receipt |
| GPF-G09 | Fail | scene transition 没有 consumer/terminal result |
| GPF-G10 | Fail | 单槽覆盖、saturating sequence 与 u64->i64 窄化不能证明唯一终态 |
| GPF-G11 | Fail | 无 travel preload/失败后旧 World 持续运行证据 |
| GPF-G12 | Fail | 无 World/Input/Camera/Audio/UI/Network 同代 commit |
| GPF-G13 | Fail | 无 Experience artifact 寻址 |
| GPF-G14 | Fail | 无 activation action rollback |
| GPF-G15 | Fail | 无 GameRule authority owner/client rejection |
| GPF-G16 | Fail | 无 BeginPlay readiness barrier |
| GPF-G17 | Fail | 无 match phase guard/timeout/reason/receipt |
| GPF-G18 | Fail | 无 LocalPlayer/platform user/device/viewport/profile binding |
| GPF-G19 | Fail | player identity 仍是 string，未绑定 principal/local player |
| GPF-G20 | Fail | 无 join-in-progress 同 epoch snapshot |
| GPF-G21 | Fail | 无 admission/player/controller/spawn/possess login 链 |
| GPF-G22 | Fail | 无 logout/unpossess/cleanup idempotency |
| GPF-G23 | Fail | 无 Possession 原子双向 binding/rollback |
| GPF-G24 | Fail | 无 Controller/Pawn/Camera gameplay tick barrier |
| GPF-G25 | Fail | 无 split-screen player input/camera/audio/HUD 隔离 |
| GPF-G26 | Partial | UI consume 已能阻止本次全局 input 下发；per-player routing 与 focus/possession 测试未闭合 |
| GPF-G27 | Partial | generic Dynamic Scene transaction 可 rollback；gameplay spawn collision/admission 尚未接入 |
| GPF-G28 | Fail | 无 respawn transaction |
| GPF-G29 | Fail | script 仍可直接 spawn/despawn/transform 与写 transition resource |
| GPF-G30 | Fail | Vampire 仍含 camera entity 1 与 global active camera 假设 |
| GPF-G31 | Fail | 无 reconnect grace 与 terminal cleanup |
| GPF-G32 | Fail | 无 network travel map/experience/build digest 与 commit tick |
| GPF-G33 | Partial | scene/plugin/WOC snapshot 均未持久化锁/task/pointer，但统一 SaveGame schema 不存在 |
| GPF-G34 | Fail | 无 gameplay restore migration/dependency order |
| GPF-G35 | Fail | 无跨 world/player generation 的统一 async save overwrite gate |
| GPF-G36 | Partial | 多 dynamic session 有物理隔离；同 instance 多 WorldContext 隔离未证明 |
| GPF-G37 | Partial | dynamic scene/WOC/session 有局部 fault tests；gameplay 全阶段矩阵缺失 |
| GPF-G38 | Partial | 通用 diagnostics/correlation 有底座；无法重建 player/possession/travel 因果链 |
| GPF-G39 | Fail | Vampire 与 WOC 均未完成统一 framework 产品闭环 |
| GPF-G40 | Pass | 本报告、索引、coverage 的静态文档校验在本轮完成；不代表 runtime/product gate 通过 |

## 16. 状态与产出记录

- `review_status: review_complete` 仅表示 Runtime38 当前源码复核和差异文档完成；implementation、MVP 和产品资格仍为 pending。
- finding 数量保持 72 P1、16 P2，不因当前源码 Partial 重判新增或删除编号。
- 当前 P1 统计：Open 49、Partial 23、Closed 0；P2：Open 16；Gate：Fail 30、Partial 9、Pass 1。
- 当前可直接复用的底座是 module/capability ledger、session teardown、Level generation/currentness、dynamic scene transaction、UI-first input、typed net DTO、scene ticket 与 WOC private transaction；它们都必须进入统一 owner，不能继续各自证明“功能存在”。
- 首个实施切片应是 `GPF-M0 -> GPF-M1`：先关闭 scene-transition 伪成功，冻结 typed identity/scope/state/receipt，再建立 GameInstanceRegistry 与 WorldContextRegistry；不要先继续扩展产品脚本。
- 本轮未执行 Cargo 或产品动态验证，因为没有 production/test 代码改动；静态验证只覆盖文档格式、路径、链接、编号、计数与 currentness。
