---
title: Editor Multiplayer Lobby / Matchmaking / Online Services / Replication / Network Emulation / PIE Authoring 当前源码复审
category: zircon_editor
report_id: Editor148
review_date: 2026-08-26
baseline_head: 166720dcb59c57fb4b33c34b859dc1a3f572b222
verification_head: 166720dcb59c57fb4b33c34b859dc1a3f572b222
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/26-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-review.md
  - docs/plans/optimize/zircon_editor/100-editor-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-current-source-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/multiplayer
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/index/workbench_extension_module_workspaces.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/online_sessions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/online_sessions.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/online_sessions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs
  - zircon_editor/src/core/play
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_plugins/net/editor
  - zircon_plugins/net/features/replication/runtime
  - zircon_plugins/net/features/rpc/runtime
  - zircon_plugins/net/runtime
  - zircon_plugins/first_party_editor_catalog
  - zircon_plugins/first_party_runtime_catalog
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/entry_runner
  - zircon_runtime/src/core/framework/net
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/99zo-runtime-network-transport-socket-tls-http-websocket-reliable-udp-session-rpc-replication-prediction-rollback-content-download-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_editor/147-editor-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineSubsystem/Source/Public/Interfaces/OnlineSessionInterface.h
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineSubsystem/Source/Public/OnlineSessionSettings.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Settings/LevelEditorPlaySettings.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Settings/LevelEditorPlayNetworkEmulationSettings.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/ReplicationGraph/Source/Public/ReplicationGraph.h
  - dev/UnrealEngine/Engine/Source/Runtime/Net/Iris/Public/Iris/ReplicationSystem/ReplicationSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Net/Iris/Public/Iris/ReplicationState/ReplicationStateDescriptor.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/NetworkPredictionInsights/Source/NetworkPredictionInsights/Public/INetworkPredictionProvider.h
  - dev/godot/modules/multiplayer/editor/replication_editor.h
  - dev/godot/modules/multiplayer/editor/replication_editor.cpp
  - dev/godot/modules/multiplayer/editor/editor_network_profiler.h
  - dev/godot/modules/multiplayer/editor/editor_network_profiler.cpp
  - dev/godot/modules/multiplayer/scene_multiplayer.h
  - dev/godot/modules/multiplayer/scene_replication_config.h
  - dev/godot/scene/main/multiplayer_api.h
  - dev/bevy/crates/bevy_remote/src/lib.rs
  - dev/bevy/crates/bevy_remote/src/http.rs
  - dev/Fyrox/fyrox-core/src/net.rs
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugFrameTiming.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
---

# Editor148 · Multiplayer / Online Services / Replication / Network Emulation 当前源码复审

## 1. 结论

Zircon当前没有工程级多人在线Editor产品。可保留的真实基础集中在Runtime：`NetRuntimeMode`区分Dedicated Server、Client和Listen Server；中立层已有TCP/UDP/HTTP/WebSocket/session DTO；Net插件具备handshake、RPC quota、Replication delta/interest/budget、RUDP与content-download局部算法。Editor07的Play链也已经有process-tree suspended spawn、取消、输出预算、scene materialization、retryable stop与终态清理。这些是未来多人测试会话可复用的底座，不是Online Services、Replication Schema或多人PIE完成证据。

Editor产品面仍由静态fixture构成。Lobby和Matchmaking两份ZUI固定显示`Lobby_Default`、8 slots、4 players、Ranked、Gold、Backfill、6 queues、128 players与48 ms；两份feedback函数直接返回queued/warning文本。40条template binding和field edit只改变retained control的`value/value_text`，没有多人document、source revision、provider、ticket、allocation或operation receipt。生产Rust中对Lobby、Party、Matchmaking Ticket、Allocation、Backfill和Online Service Provider等领域authority的精确类型扫描为零命中。

Net Editor只有descriptor层。`authoring.rs`声明2个view、6个operation、3个inspector customization、`net.replication_schema` asset kind、默认document与graph palette；插件目录仍只有7个文件，5个声明资源全部不存在，也没有operation factory/handler。first-party Editor catalog和App默认装配仍只支持Navigation/Neural，Net Editor不在production provider closure。

Play仍只能管理一个`PlayChild`。`PlayStartRequest`只有kind、project、build、scene与running document；CLI只有project、runtime profile、scene与report pipe。没有server/client role、client count、port/account、artifact revision、join plan、network profile、seed、readiness barrier或per-link emulation。因而Editor26/Editor100的5项P0、60项P1、12项P2仍全部Open；32个currentness资格门保持30 Fail / 2 Partial / 0 Pass。没有任何性能、规模、稳定性或同场景证据支持“优于Unreal”。

## 2. 冻结范围与方法

### 2.1 当前工作树选择集

本报告读取当前工作树内容，并以`166720dcb59c57fb4b33c34b859dc1a3f572b222`标记提交基线。工作树存在大量其他会话在途改动；本轮不回退、不格式化、不暂存它们。物理行按文件读取，test统计仅计Rust `#[test]`，ignored统计Rust `#[ignore...]`；fingerprint是排序后的相对路径与逐文件SHA-256再聚合所得。

| 选择集 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---:|
| Workbench multiplayer surface、binding、navigation、feedback | **8 / 1,858 / 1,647 / 100,960 / 0 / 0** | `49998ebf10576a3d0d60ae2c85972dcad056838a351c42f2b9acf09c9735e3b6` |
| Net Editor与first-party Editor catalog/App | **12 / 809 / 739 / 31,036 / 9 / 0** | `f4f7f5552ba6503c9e2a663eddecbddca223f7fa85cd1e3834a35d973d017547` |
| Play request/process/host topology | **43 / 8,121 / 7,402 / 284,087 / 82 / 2** | `c0dc5f32972f8840a3948564d2fd202fa60252836f446af5758c1d5821e28430` |
| Runtime Net handoff、catalog与App entry selected | **136 / 18,263 / 16,527 / 640,540 / 211 / 8** | `421211735c4a4c267667cb63f444a7ff4769a0aef45288ac50accf8bf01b8012` |
| Zircon selected union | **199 / 29,051 / 26,315 / 1,056,623 / 302 / 10** | `8c30767977db4907a343e0140cd2c3d6d0d80f9238a3e6a9dee1b48be67acff7` |

### 2.2 参考源码选择集

22个本地参考文件均存在，共14,354行、561,098 bytes，fingerprint为`231defdf9d74967117f6e2335daa27804d2163b3bc7283721c78f96e886962b7`。Godot revision为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`，Bevy为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`，Fyrox为`8d815db36494f1badb347547dfc7094bf4fbbdf8`，Unity Graphics为`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal是仓内vendored tree，以文件fingerprint冻结。

### 2.3 动态证据边界

本轮没有运行Cargo、Editor、Net plugin load、Online provider、Dedicated/Listen + N clients、packet emulation、network capture、fuzz、fault、scale、soak或competitive benchmark。静态证据足以判定固定事实、缺失资源/factory/catalog、单child拓扑和owner断路；不能证明运行时正确性或性能。Tooling按用户要求排除，后续迁移Rust时再独立审查。

## 3. 当前数据流与断点

```text
Lobby/Matchmaking ZUI
  -> local control value/value_text mutation
  -> fixed queued/warning feedback
  x no source document / revision / provider / receipt

Net Editor contribution
  -> view/operation/asset/graph descriptors
  x missing five resources
  x missing six factories/handlers
  x absent from first-party Editor catalog/App

PlayStartRequest(kind, project, build, scene, document)
  -> ProcessPlayBackend -> one PlayChild
  x no server/listen/client topology
  x no account/port/profile/seed/readiness/per-link emulation

Runtime Net mode/RPC/Replication local managers
  x no Editor schema artifact -> World/Reflection -> wire -> server/client install
```

目标链必须是：

```text
Versioned Multiplayer Documents
  -> Transaction / Validate / Compile
  -> Versioned Online + Protocol Artifacts
  -> Runtime provider and World activation
  -> PlaySessionGroupAuthority(server/listen + N clients)
  -> source-qualified Network Observation
```

## 4. Owner边界与必须保留的基础

| Owner | 唯一职责 | 禁止形成的第二authority |
|---|---|---|
| `zircon_runtime` | transport/session、authenticated identity binding、RPC/Replication wire、World install、prediction/rollback、download与运行时预算 | Editor-owned NetManager、Editor-owned replication runtime、调用者自报role/player ID |
| `zircon_editor` | versioned authoring document、provider capability投影、typed operation、多人测试会话编排、network observation/inspection | ZUI control作为document、固定queue/session数据、独立online state |
| `zircon_app` | first-party Runtime/Editor plugin组合与host lifecycle | 手写领域逻辑、只装Runtime不装配配对Editor却宣称产品可达 |

必须保留：

1. 保留`NetRuntimeMode::{DedicatedServer, Client, ListenServer}`和typed session/control DTO，将其接入真实topology request。
2. 保留Net Editor contribution batch、asset kind、graph descriptor与palette；在resource/factory/catalog闭合前将capability降为Unavailable。
3. 保留Runtime RPC direction/schema/quota及Replication delta/interest/budget/late-join局部算法，由Runtime140继续收敛World/transport owner。
4. 保留Editor07的process tree、scene snapshot、output budget、cancellation、retryable stop、plugin compensation与cleanup，扩展为group authority而非另写第二套launcher。
5. 保留Editor02/04/05/09/11/25的document、catalog、inspector、job、journal与observation基础，多人域只提供typed adapter。
6. 保留两份Workbench视觉布局壳；删除其中的固定事实、control-local authority和无receipt成功文案。

## 5. P0 currentness重判

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| P0-1 | **Open** | 两份Workbench共460物理行、54个node、38条route、0 provider；所有人数、队列、延迟和validation结果均为fixture。 | 默认入口显式Demo/Unavailable，或由真实document revision、provider snapshot与session receipt驱动。 |
| P0-2 | **Open** | production Rust对Identity/Party/Lobby/Ticket/Allocation/Backfill/Online Provider authority精确扫描为零；NetManager只是网络底层。 | 建立provider-neutral Online Services registry、environment、credential lease与typed lifecycle。 |
| P0-3 | **Open** | Net Editor只注册descriptor；`authoring.zui`、3个inspector ZUI和default TOML缺失，6个operation无factory，catalog/App未装配。 | resource resolver、factory closure、paired catalog与end-to-end operation测试通过前撤销Available。 |
| P0-4 | **Open** | `net.replication_schema`没有canonical source、compiler、protocol artifact、compatibility hash、World install或server/client conformance。 | 同一source revision产出可安装artifact，以stable ID、wire golden与mismatch测试驱动Compile/Validate receipt。 |
| P0-5 | **Open** | Play request/backend/CLI只描述并启动一个child，Simulate不启动server + N clients，也不应用网络仿真。 | 建立`PlaySessionGroupAuthority`、per-instance lifecycle与per-link deterministic emulation/observation。 |

## 6. P1 currentness重判

以下60项沿用Editor26 canonical ID与语义，不因Runtime140的底层进展重复计数或误判关闭。

| IDs | 状态 | 当前共同证据 | 重构目标 |
|---|---|---|---|
| P1-1..P1-10 | **Open** | Lobby、Matchmaking、Replication没有正式versioned document、stable reference、semantic diagnostic、artifact identity、factory或默认Editor装配。 | 三类source接Editor transaction/catalog/inspector/operation，提供lossless round-trip、CAS/save/recovery和typed diagnostic/artifact receipt。 |
| P1-11..P1-20 | **Open** | provider registry、environment/credential owner、identity login/refresh、Lobby lifecycle、attribute schema、member CAS、presence/invite/crossplay policy均缺失。 | provider capability negotiation、secure credential lease、authenticated identity、environment隔离、CAS lifecycle和policy compiler。 |
| P1-21..P1-30 | **Open** | 没有typed ticket、rule AST、QoS source、party/team solver、backfill、allocation/reservation或可重放match explanation。 | ticket state machine、typed rules、QoS freshness、party/team/capacity、allocation receipt和deterministic simulator。 |
| P1-31..P1-42 | **Open** | schema不由Reflection stable identity驱动；runtime dense index/raw bytes不能跨build，缺evolution、RPC artifact、World ownership、spawn/despawn、真实wire budget和typed smoothing。 | `CompiledReplicationSchemaArtifact`统一stable type/field/RPC ID、serializer/quantization/condition/hash、World/connection install和compile-to-runtime conformance。 |
| P1-43..P1-52 | **Open** | request无topology，backend单child，role/port/account/profile未进CLI/config；无emulation replay、关联状态视图、network observation provider和object inspector。 | `MultiplayerTestSessionRequest`、group authority、原子port/account/sandbox allocation、per-link profile与Editor25 typed network tracks。 |
| P1-53..P1-60 | **Open** | Telemetry无治理，online error无typed journal/action；测试只覆盖descriptor/局部manager，没有DoS、match/replication scale、fault/recovery或发布证据。 | privacy/consent/redaction、Editor11 journal、provider/multi-process/fault corpus、规模矩阵和evidence-backed maturity。 |

## 7. P2 currentness重判

| IDs | 状态 | 后续专项 |
|---|---|---|
| P2-1..P2-2 | **Open** | Party/Lobby/Game Session/Match术语与provider compliance/adapter矩阵。 |
| P2-3..P2-4 | **Open** | NAT traversal/relay/P2P、multi-region/fleet migration和session continuity。 |
| P2-5..P2-6 | **Open** | deterministic network recording/replay、prediction/rollback/lag compensation authoring。 |
| P2-7..P2-8 | **Open** | voice/chat/moderation、anti-cheat/server trust、privacy/security owner。 |
| P2-9..P2-10 | **Open** | graph scale/accessibility、team collaboration/structured diff/merge/conflict policy。 |
| P2-11..P2-12 | **Open** | provider deployment/schema rollout/conformance SDK；禁止按按钮数量等同参考引擎能力。 |

## 8. 参考引擎差异与适用边界

| 参考 | 当前源码可验证机制 | Zircon应吸收 | 不可错误外推 |
|---|---|---|---|
| Unreal Online Session | `Create/Start/Update/End/Destroy`、Find/Join、Start/Cancel Matchmaking、player register/unregister；settings包含public connections、presence、build ID与search/member配置。 | provider-neutral异步lifecycle、typed result/receipt、build/environment/member policy与provider capability。 | 不照搬OnlineSubsystem类层次，不把接口数量当实现质量。 |
| Unreal PIE / Network Emulation | Play settings显式区分standalone/listen/client、one/separate process、client count、server port/args；emulation按server/client与方向配置。 | typed topology、per-instance role、readiness/reap、port/account/sandbox和per-link deterministic profile。 | 一个Simulate按钮或单child loopback不能等价。 |
| Unreal ReplicationGraph / Iris / NetworkPrediction | global/per-connection graph node、dormancy/frequency、stable object/state descriptor、filter/prioritizer/poll、prediction trace/provider。 | stable protocol artifact、connection/object explanation、interest/priority/polling、prediction/correction trace与scale evidence。 | 不把Zircon local dense index/String descriptor称作同级协议。 |
| Godot Replication Editor | `SceneReplicationConfig`绑定`MultiplayerSynchronizer`和真实property path，支持spawn/always/on-change与UndoRedo。 | Scene/Reflection provider、stable property identity、transactional edits和真实runtime install。 | 只画graph/palette不等于authoring闭环。 |
| Godot Network Profiler / SceneMultiplayer | capture/autostart/clear、RPC/synchronizer count/bytes/bandwidth；peer auth callback/timeout、pending peers、disconnect/clear/refuse-new。 | capture lifecycle、typed counters、peer/session auth状态机、disconnect retirement和recoverable projection。 | 不能用固定Telemetry tab替代provider。 |
| Bevy Remote | `RemotePlugin`与`RemoteHttpPlugin`分离，BRP request/response/error、JSON-RPC 2.0、batch/stream和独立TCP I/O task。 | remote protocol与transport分层、typed error、bounded async dispatch，可用于diagnostic/control边界。 | Bevy Remote不是Lobby/Matchmaking/Replication产品参考。 |
| Fyrox | core net提供窄TCP listener/stream wrapper，Engine统一持有task pool与update lifecycle。 | Rust-native ownership、窄network primitive和engine lifecycle下限。 | 当前选择集没有first-party Online Services栈，不能降低Zircon多人标准。 |
| Unity Graphics | `DebugManager`注册/注销/reset/refresh debug data/panel；`DebugFrameTiming`维护bounded history与CPU/GPU/Present bottleneck projection。package仅Graphics 17.6.0。 | network observation可复用provider lifecycle、bounded history、显式refresh/reset的表现层模式。 | Graphics包没有Netcode/Services ownership，不能据此声称Unity多人能力缺失或完成。 |

Unreal和Godot是本篇主参考；Bevy、Fyrox和Unity Graphics只提供相邻边界下限。报告不会用缺少对应模块的参考选择集降低Online Services、多人PIE或Replication的验收标准。

## 9. 第二authority与产品断路

| Surface | 当前承诺 | 实际authority | 处置 |
|---|---|---|---|
| Lobby Workbench | 8 slots、4 players、crossplay、Simulate/Validate/Telemetry | 固定ZUI、control mutation、fixed feedback | M0改Demo/Unavailable；M1-M4后投影Lobby document/provider/session |
| Matchmaking Workbench | Ranked/Gold、6 queues、128 players、48 ms、Backfill | 固定ZUI、control mutation、fixed feedback | M0改Demo/Unavailable；M1/M5后投影config/ticket/simulator |
| Net Editor | Network/Diagnostics、listener/route/schema commands | 未进catalog；资源与factory缺失 | 闭合paired plugin或撤销发布 |
| Replication Schema | Create/Open/Validate/Compile和2个palette node | descriptor-only | 建立source/compiler/artifact/install后开放 |
| Runtime RPC/Replication managers | quota/delta/interest/budget/late join | 局部内存算法，未接authenticated World/transport | Runtime140继续收敛，Editor只交付artifact |
| Play/Simulate | Play/Simulate单child | 无role/topology/emulation | Editor07升级session group，本篇定义typed multiplayer request |
| Telemetry tab | Lobby/Matchmaking线上观测 | 无provider/schema/governance | 默认Unavailable；本地trace与线上Telemetry分权 |

## 10. 分层重构路线

1. **M0 Truth / Reachability**：删除固定成功事实；Net Editor resource、factory、catalog、App reachability与capability truth先闭合，失败显式Unavailable。
2. **M1 Canonical Documents**：建立LobbyDefinition、MatchmakingConfig、ReplicationSchema三类versioned source、stable identity、transaction、CAS/save/recovery、reference与semantic diagnostic。
3. **M2 Compiler / Artifact**：provider lowering、stable rule/type/field/RPC ID、schema/wire version、compatibility/content hash、golden/round-trip和atomic artifact publication。
4. **M3 Provider / Identity / Environment**：provider registry/capability、Development/Staging/Production隔离、secure credential lease、login/refresh/revoke、redaction与fake provider testkit。
5. **M4 Lobby / Online Session**：Create/Join/Update/Leave/Destroy、member/attribute/revision/capacity/invite/presence/build policy与typed async receipt。
6. **M5 Matchmaking / Allocation**：ticket/rule/expansion/party/team/QoS/backfill/allocation/reservation状态机、解释trace与deterministic simulator。
7. **M6 Runtime Protocol / World**：由Runtime140把compiled artifact接Reflection/World/connection/transport，完成ownership、spawn/despawn、interest、baseline/ACK、typed smoothing与真实wire budget。
8. **M7 Multiplayer PIE Group**：由Editor07把单child扩展为Dedicated/Listen + N clients group，提供port/account/sandbox/readiness/join/partial failure/stop/reap与per-link emulation。
9. **M8 Observation / Failure Workflow**：向Editor25注册ticket/connection/packet/channel/RPC/replication/prediction tracks和object inspector，向Editor11写typed diagnostic/action。
10. **M9 Scale / Security / Release**：100K ticket、128 client/100K object、loss/jitter、credential/PII、rate limit、跨平台、长时soak、rollback及同场景Unreal/Godot evidence。

M0-M5是Editor多人产品owner；M6是Runtime owner；M7复用Editor07；M8复用Editor11/25。禁止以本篇为由在Editor复制Runtime网络实现。

## 11. Currentness资格门

为保持Editor100状态映射不漂移，本轮沿用其32门currentness分组；Partial只承认已有底层结构，不能升级对应产品成熟度。

| Gates | 状态 | 当前依据 |
|---|---|---|
| G01-G05 | **Fail** | fixture truth、online owner、resource/factory/catalog、schema artifact均未闭合。 |
| G06-G12 | **Fail** | provider/identity/environment/credential/document lifecycle缺失。 |
| G13-G18 | **Fail** | ticket/rule/QoS/team/backfill/allocation/replay缺失。 |
| G19-G24 | **Fail** | stable protocol、World/connection install、wire/ownership/observation未闭合。 |
| G25-G28 | **Fail** | Play为单child，无topology、readiness、per-link emulation或replay。 |
| G29 | **Partial** | Runtime已有typed Net mode与session DTO，但Editor request/CLI/PIE未消费。 |
| G30 | **Partial** | Net Editor已有descriptor/asset kind/palette，但catalog/resource/factory/operation不闭合。 |
| G31-G32 | **Fail** | 安全、隐私、fault、scale、soak、跨平台与竞争证据缺失。 |

汇总：**30 Fail / 2 Partial / 0 Pass**。

## 12. 实现验收矩阵

| 场景 | 最低证据 |
|---|---|
| Document | 三类source的load-edit-undo-redo-save-reopen、external conflict、autosave/recovery与unknown-field preservation |
| Provider | fake provider覆盖login/refresh/revoke、Lobby race、ticket cancel/expire、allocation failure/rate limit和typed receipt |
| Protocol | server/client加载相同artifact完成spawn/update/despawn/RPC/late join；mismatch在join前以typed reason拒绝 |
| Topology | Dedicated与Listen各启动1/2/10 clients，验证role/port/account/sandbox/readiness、partial crash、stop/reap与无孤儿进程 |
| Emulation | 各方向latency/jitter/loss/dup/reorder/bandwidth生效；profile/seed/packet decision可捕获与重放 |
| Observation | 任一ticket/connection/entity可关联到source revision、instance、World、schema、bytes、interest/priority、why/why-not与failure action |
| Scale | 10K/100K tickets和1K/10K/100K objects记录CPU、memory、throughput、P50/P95/P99、per-client bytes与tail latency |
| Security | secret不进入asset/CLI/env dump/log/trace/crash；覆盖expiry/revoke、unauthorized RPC、DoS budget、PII redaction与tenant隔离 |
| Competition | 相同build、hardware、topology、dataset、quality与capture方法对照Unreal/Godot；保留raw artifact，不只报告平均值 |

## 13. 禁止的临时修补

- 禁止把固定Lobby、queue、player、latency或warning换成随机数、socket count或timer后继续称真实Online Services。
- 禁止只创建5个空ZUI、给6个operation注册`Ok(())` factory、或把descriptor registration称为document/compiler完成。
- 禁止把`NetSessionInfo`重命名为Online Session；transport session、Lobby和Game Session是不同产品层。
- 禁止把`PlayKind`再包装一层却不传role、client count、port、account、artifact/profile/seed和readiness。
- 禁止在Editor复制Runtime NetManager、RPC、Replication、RUDP、prediction或download authority。
- 禁止用caller提供的String/raw bytes/player ID/role或运行时排序index作为稳定协议、身份或权限。
- 禁止将provider token、platform secret或真实用户ID写入project asset、session archive、CLI、日志、trace或crash artifact。
- 禁止在UI线程同步等待login、match、allocation、compile、server ready或N个client join。
- 禁止用单机loopback、isolated manager test、descriptor string test或ignored microbenchmark证明多人产品或优于Unreal。
- 禁止默认启用线上Telemetry，或自动上传PIE trace；consent、tenant、redaction、retention和deletion必须先闭合。
- 禁止在M0-M9和安全/规模/恢复证据未通过前提升Stable/Complete成熟度声明。

## 14. 本轮完成定义

本轮完成Editor26/Editor100的current-source刷新：冻结199个Zircon selected文件、29,051行、1,056,623 bytes、302个Rust test attributes和10个ignored declarations；冻结22个Unreal/Godot/Bevy/Fyrox/Unity Graphics参考文件、14,354行、561,098 bytes。5项P0、60项P1、12项P2保持Open，currentness gates保持30 Fail / 2 Partial / 0 Pass，canonical finding总数不增加。

本轮只修改review与导航索引，不修改Runtime、Editor、App、plugin、ABI、测试或产品资源；没有运行Cargo或动态多人资格，也没有查询、轮询、等待或实时跟踪协调器。实现状态仍为pending。后续源码修正必须从M0开始，并在每一里程碑重新冻结受影响源码、验证真实artifact/receipt和owner终态。
