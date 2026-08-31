---
title: Editor Multiplayer Lobby / Matchmaking / Online Services / Replication / Network Emulation / PIE Authoring 当前源码复审
category: zircon_editor
report_id: Editor100
review_date: 2026-08-26
baseline_head: 38c0e7f5d48189ac2637ed010e452b19c32f459d
verification_head: 38c0e7f5d48189ac2637ed010e452b19c32f459d
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/26-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-review.md
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
  - zircon_runtime/src/core/framework/net
  - zircon_app/src/entry/entry_runner
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/99zo-runtime-network-transport-socket-tls-http-websocket-reliable-udp-session-rpc-replication-prediction-rollback-content-download-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
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
---

# Editor100 · Multiplayer Lobby / Matchmaking / Replication / Network Emulation 当前源码复审

## 1. 结论

Runtime 的网络源码不是空白。`NetRuntimeMode` 已区分 Dedicated Server、Client、Listen Server；中立层有 TCP/UDP/HTTP/WebSocket/session DTO；Net 插件有 handshake、RPC quota、Replication delta/interest/budget、RUDP 和 content-download 局部算法。Runtime140 已拥有这些底层 transport、TLS/WSS、provider composition、wire、World replication、prediction 与 download installer 的差距，本报告不重复登记。

Editor 侧仍没有可执行的多人 authoring/product 闭环。Lobby 与 Matchmaking 两份 Workbench 是静态 fixture：固定 `Lobby_Default`、slot/player/crossplay、Ranked/Gold/Backfill、人数、延迟和 warning；feedback 函数直接返回 “Simulation queued”“Validation queued” 等文本。字段编辑只改变 retained control 的 `value/value_text`，没有 asset/document/revision/factory/provider/receipt。Workbench 导航把 Telemetry tab 当作普通 local selection，没有 Online Services provider。

Net Editor 比旧报告多了 descriptor 结构，但这是注册层进展而不是产品完成：`zircon_plugins/net/editor/src/authoring.rs` 注册 listener/route/schema operation、asset kind、graph editor、palette 和 three inspector customizations；`plugin.rs` 注册 `plugins://net/editor/authoring.zui`。当前 `zircon_plugins/net/editor` 只有 7 个源码/测试文件，实际不存在 `authoring.zui`、`listener_config.zui`、`route_config.zui`、`replication_schema.zui` 或 `replication_schema.default.toml`，也没有 operation factory/handler。first-party Editor catalog 只链接 Navigation/Neural，Net Editor 不在默认 Editor host provider closure。

PIE/Play 仍是单 child 进程。`PlayStartRequest` 仅携 `kind/project_root/requires_build/scene_source/running_document`；process command 只传项目、scene、runtime profile 和 report pipe，没有 server/client role、client count、server port、account set、network profile、seed、topology readiness 或 per-link emulation。`ProcessPlayBackend` 的 active state 只容纳一个 `PlayChild`，不能启动 Dedicated/Listen server 加 N clients，也不能为每个连接记录 join/auth/replication/prediction evidence。

因此旧 Editor26 的 5 项 P0、60 项 P1、12 项 P2 当前仍全部 Open；32 个 Editor multiplayer 资格门为 30 Fail / 2 Partial / 0 Pass。两项 Partial 只承认 Runtime 中立 DTO/Net mode 和 Net Editor descriptor 已存在，不能被解释为 Online Services、Replication Schema 或 Multiplayer PIE 已完成。当前没有任何证据支持“性能和表现优于 Unreal”。

## 2. 当前源码范围与证据

### 2.1 冻结选择集

| 选择集 | 文件 / 行 / 非空行 / bytes / test / ignored | fingerprint |
|---|---:|---:|---:|---:|---|
| Workbench multiplayer surfaces、asset入口、binding、navigation、feedback | 8 / 1,858 / 1,647 / 100,960 / 0 / 0 | `0133a6df5531e1c7bfdd1a15d1c2cd9bcdb5024750491f3c1e2898cebc2e2254` |
| Editor Net authoring、catalog、Play topology | 51 / 7,868 / 7,172 / 275,928 / 78 / 0 | `a369d75dcb3c7965d5d2fdbb18b3e6c1045884b372a4fc177918f9c88e2654b6` |
| Runtime Net handoff selected | 208 / 19,779 / 17,829 / 687,062 / 161 / 15 | `fec3d516db3e6a5b764a110221f0f039d8f58e9b9199969411c04bd1f8eeed9c` |
| focused multiplayer/Play tests | 93 / 12,271 / 11,113 / 426,171 / 127 / 5 | `93262630825671bbefe93a0638eb886f12c6cd6578c2f1109894878bbaaf61e4` |
| selected union | 266 / 30,116 / 27,192 / 1,082,023 / 252 / 15 | `aec9a8a5a842e5f75bfbd4a572b651e023ce344cdf1177eb1d65280d3b8653e2` |

Runtime handoff selection覆盖 `zircon_runtime/src/core/framework/net`、`zircon_plugins/net`、first-party Runtime catalog、App composition 和 entry runner；它不是本篇的第二套 Runtime report。focused tests 是静态 inventory，不是通过数。当前工作树有大量其他会话在途修改，本报告只读取和写文档，不回退、不格式化、不暂存。

### 2.2 动态边界

本轮没有运行 Cargo、Editor、Net plugin load、Online provider、match allocator、Dedicated Server、N clients、packet capture、network emulation、cross-platform login、soak、fuzz 或 scale。静态证据足以证明固定 Workbench feedback、缺失资源、缺失 factory、单 child Play request 和默认 catalog 未接 Net Editor；不能替代真实多进程资格。

### 2.3 Owner 边界

- Editor100 拥有 Lobby/Matchmaking/Replication authoring document、provider projection、Net Editor/catalog reachability、Multiplayer Test Session request、PIE topology/emulation UI 和 network observation projection。
- Runtime140 拥有 socket/transport/TLS/WSS/session/RPC/Replication/RUDP/content-download 运行时实现与其底层资格。
- Editor07 拥有 Play process/tree/world checkpoint 生命周期；Editor09 拥有 job admission/cancel/shutdown；Editor11 拥有 journal；Editor25 拥有 trace/metric/timeline。Editor100 只定义它们如何消费同一 Multiplayer Test Session/Online receipt，不复制基础设施。

## 3. 必须保留的真实基础

1. 保留 `NetRuntimeMode::{DedicatedServer, Client, ListenServer}` 和 typed session/control DTO；把它们接入真实 test topology，而不是重写成 Editor-only enum。
2. 保留 Net Editor 的 contribution batch、asset kind、graph descriptor、palette 和 capability declaration；在资源/factory/catalog闭合前隐藏 Available surface。
3. 保留 Runtime RPC direction/schema/quota、Replication descriptor/delta/interest/budget、late-join局部算法，待编译 artifact 与 World/transport owner 接入。
4. 保留 Editor07 的 process-tree cancellation、snapshot、output budget、plugin activation compensation 和 terminal state；扩展为 `PlaySessionGroupAuthority`。
5. 保留 Editor02/04/05/09/11/25 的 document、catalog、inspector、job、journal、observation 基础；多人域只提供 typed adapters。
6. 保留 Workbench 的视觉壳和固定尺寸布局，替换其数据源、control-local mutation 与假成功文案。

## 4. 当前拓扑与断路

```text
Lobby/Matchmaking ZUI -> local field mutation -> fixed feedback
                       \-> no MultiplayerDocument / provider / receipt

NetEditor contribution -> descriptor + missing URI + no factory
first_party_editor_catalog -> Navigation + Neural only

PlayStartRequest(kind, project, scene, document)
  -> ProcessPlayBackend -> one PlayChild -> runtime preview
  x no server/client topology, port/account/profile/emulation

Runtime Net modes/RPC/Replication DTO and local managers
  x no Editor-created schema artifact -> World -> wire -> server/client session
```

目标不是把 Online Services、Net Editor 和 Play process 继续堆到一个 callback。应建立 `MultiplayerDocument -> Compiler/Validator -> Versioned Artifact -> Provider/Runtime Activation -> PlaySessionGroup -> Network Observation` 的单向链。

## 5. P0：当前五项仍未关闭

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| P0-1 | **Open** | 两份 Workbench 固定 Lobby/Matchmaking 数字、40 条 binding/route 和 queued feedback，无 provider/document。 | 默认入口显式 Demo/Unavailable 或由真实 source revision/provider/session receipt 驱动。 |
| P0-2 | **Open** | 精确源码没有 Identity、Party、Lobby、Ticket、Allocation、Online Provider production authority；NetManager 只是 transport。 | 建立 provider-neutral Online Services registry、credential lease、environment/tenant 与 typed lifecycle。 |
| P0-3 | **Open** | Net Editor descriptor 存在，但 catalog 未链接；`authoring.zui`、3 个 inspector ZUI、default TOML 不存在，factory/handler 缺失。 | resource resolver、catalog closure、operation factory、asset/document/toolkit 端到端通过前撤销 capability。 |
| P0-4 | **Open** | `net.replication_schema` 只有 contribution/graph descriptor；没有 source document、compiler、wire artifact、World install 或 compatibility hash。 | 同一 source revision 产出 server/client 可安装 artifact，并以 golden wire/compatibility 测试驱动 Compile/Validate receipt。 |
| P0-5 | **Open** | Play request/backend 只有一个 child，Simulate 不读取 Net mode、账号、端口或 profile，也不启动 N clients。 | `PlaySessionGroupAuthority` 管理 server/listen/client 实例、ready/join/reap、sandbox、per-link emulation 和 observation。 |

## 6. P1 逐项 currentness 重判

旧 Editor26 的 P1 细节仍是当前 canonical 差距；本表逐项确认当前状态，避免把 Runtime140 的底层修复误算为 Editor 闭环：

| IDs | 状态 | 当前共同证据 | 目标边界 |
|---|---|---|---|
| P1-1..P1-10 | Open | Lobby/Matchmaking/Replication 没有正式 asset/document、stable reference、semantic diagnostics、artifact identity、factory 或默认 Editor 装配。 | 三类 versioned source document 接入 Editor02 transaction、Editor04 catalog、Editor05 inspector、Editor09 operation，并产出 typed diagnostics/artifact receipt。 |
| P1-11..P1-20 | Open | 没有 provider registry、environment/credential owner、identity login/logout/refresh、Lobby lifecycle、attribute schema、revision/CAS、presence/invite/crossplay policy。 | Provider registration/capability、secure credential lease、authenticated identity、environment scope、CAS member update 与 policy compiler。 |
| P1-21..P1-30 | Open | 没有 typed matchmaking ticket、queue/rule schema、quality sample source、party/team/capacity/backfill/allocation、可重放 offline match simulation。 | Ticket state machine、typed rule AST、party/team solver、reservation/allocation receipt、deterministic simulator 与 explanation artifact。 |
| P1-31..P1-42 | Open | Replication field/type 仍不由 Editor Reflection identity 驱动；runtime dense index 不能跨 build，缺 evolution/migration、RPC artifact、World entity ownership、spawn/despawn、interest graph、real wire budget、typed smoothing 和 compile-to-runtime test。 | `CompiledReplicationSchemaArtifact` 绑定 stable type/field/RPC ID、serializer/quantization/condition/schema hash、World/connection install、baseline/ACK/interest/smoothing contracts。 |
| P1-43..P1-52 | Open | `PlayStartRequest` 没 topology；backend single child；mode/port/temp root/account 未进入 CLI/config；无 per-link emulation/replay、unified session view、network observation provider 或 object replication inspector。 | `MultiplayerTestSessionRequest` + group authority + deterministic port/account/sandbox allocation + latency/loss/jitter/reorder/bandwidth profiles + Editor25 NetworkObservationProvider。 |
| P1-53..P1-60 | Open | Telemetry tab 无 governance；online errors 无 journal/action；tests 只证明 descriptor/isolated manager；没有 DoS、match scale、replication scale、fault/recovery 或 maturity evidence gates。 | 复用 Editor11 journal/Editor25 observation，建立 privacy/consent boundary、server/client scale matrix、fault corpus、release evidence manifest；不得升级 Beta/Stable。 |

## 7. P2 逐项 currentness 重判

| IDs | 状态 | 必须保留的专项 |
|---|---|---|
| P2-1..P2-2 | Open | 统一 Party/Lobby/Game Session/Match 术语与 provider compliance/adapter matrix。 |
| P2-3..P2-4 | Open | NAT traversal/relay/P2P、multi-region/fleet migration 与 session continuity 设计。 |
| P2-5..P2-6 | Open | deterministic network recording/replay、prediction/rollback/lag compensation authoring 专项。 |
| P2-7..P2-8 | Open | voice/chat/moderation、anti-cheat/server trust 与 privacy/security owner。 |
| P2-9..P2-10 | Open | graph scale/accessibility、team collaboration/merge 与 multiplayer document conflict policy。 |
| P2-11..P2-12 | Open | provider deployment/schema rollout/conformance SDK；参考引擎能力不得按按钮数量等同。 |

## 8. 五套参考的工程差异

| 参考 | 当前源码可验证机制 | Zircon Editor 应吸收 |
|---|---|---|
| Unreal Online Session | Create/Start/Update/End/Destroy、Find/Join、Start/Cancel Matchmaking、player register/unregister、delegate/typed join result；Session Settings 有 public connections、presence/invite、build ID、member settings、search/ping。 | provider-neutral async lifecycle、typed receipt、search/matchmaking result、build/environment/member policy；不复制 OnlineSubsystem 类层次。 |
| Unreal Play/Network Emulation | Play settings 区分 standalone/listen/client/dedicated、one-process/separate server、client count、server port/args；network emulation 按 server/client/all 与方向配置 latency/loss/bandwidth。 | TestSession topology、per-instance role、port/readiness/reap、per-link profile 和 deterministic seed；一个 Simulate button 不足。 |
| Unreal ReplicationGraph/Iris/NetworkPrediction | connection/global nodes、object handles、dirty/poll/filter/priority/frequency/dormancy、stable replication state descriptor、input/state ring、correction/resimulation 与 Insights provider。 | Editor artifact 必须含 stable schema、connection/object view、priority/interest、prediction trace；不要用 String/raw bytes/本地 dense index。 |
| Godot Replication Editor/Profiler | `SceneReplicationConfig` 真正绑定 `MultiplayerSynchronizer`、property picker、spawn/always/on-change/watch、undo/redo；Network Profiler capture/clear/autostart，统计 RPC/synchronizer in/out bytes/count/bandwidth。 | 真实 Scene/Reflection provider、property path validation、transactional undo、capture lifecycle、typed network counters；不是固定 queue table。 |
| Godot SceneMultiplayer/API | peer auth callback/timeout、pending/authenticated peers、connected set、root path、refuse-new-connections、RPC/replicator/cache/disconnect/clear。 | peer/session state machine、auth gate、disconnect retirement、object/schema ownership 与 recoverable Editor projection。 |

Bevy Remote/Fyrox/Unity Graphics 不提供可审查的 first-party lobby/matchmaking stack；不能用它们降低 Online Services 或多人 replication 标准，也不能从 Unity Graphics 包推断 Netcode/Services 完成度。

## 9. 分层重构路线

1. **M0 Truth / Reachability**：移除两份 Workbench 固定事实；Net Editor resource resolver、catalog closure、factory/handler 和 maturity/capability 先闭合，失败显示 Unavailable。
2. **M1 Documents / Artifacts**：建立 LobbyDefinition、MatchmakingConfig、ReplicationSchema source documents；接 Editor02/04/05 transaction、stable identity、semantic diagnostics、compiled artifact、CAS/save/recovery。
3. **M2 Online Provider**：Provider registry、environment、secure credential lease、identity/login、Lobby/Party/Matchmaking/Ticket/Allocation/Backfill state machine；所有 operation 返回 typed receipt，凭据不落资产/日志。
4. **M3 Replication Compiler**：Reflection/World schema ingestion、stable wire ID、serializer/quantizer/condition、RPC table、compat hash/migration、server/client install and conformance artifacts。
5. **M4 Multiplayer PIE Group**：将 Editor07 单 child 扩展为 server/listen + N clients group；ready/join/auth/reap、port/sandbox/account allocation、PlayKind/scene/build/network profile/seed 全量进入 request 与 CLI。
6. **M5 Network Emulation / Observation**：per-link latency/jitter/loss/reorder/duplication/bandwidth/MTU profile，deterministic replay seed；Editor25 provider 输出 connection/RPC/replication/object/prediction/download tracks。
7. **M6 Runtime World Integration**：由 Runtime140 负责 transport/session/World replication/prediction；Editor只消费 activation/artifact/session receipts，不维护第二个 Net manager。
8. **M7 Scale / Security / Release**：1/2/10 clients、1k/10k/100k objects、loss/jitter/consumer stall、TLS/auth/DoS、cross-platform、24h soak、failure recovery 与 Unreal/Godot same-scenario benchmark。

## 10. 资格门

| Gate | 状态 | 依据 |
|---|---|---|
| G01-G05 Truth/owner/resource/factory/artifact | Fail | fixtures、缺 provider、缺资源/factory、无 schema compiler/install。 |
| G06-G12 Provider/identity/environment/credential/lifecycle | Fail | 无 Online Services authority、secure credential lease、typed async lifecycle。 |
| G13-G18 Matchmaking/ticket/team/backfill/allocation/replay | Fail | 无真实 ticket/rule/solver/reservation/provider 或离线解释 artifact。 |
| G19-G24 Replication schema/World/wire/connection/observation | Fail | Editor source 未进入 Runtime artifact；没有 object inspector/network provider。 |
| G25-G28 Multiplayer PIE/topology/emulation/replay | Fail | Play request/backend 单 child，无 roles/client count/port/profile/per-link model。 |
| G29 | Partial | `NetRuntimeMode` 与 typed Runtime session DTO 存在，但 Editor/CLI/PIE 未消费。 |
| G30 | Partial | Net Editor descriptor/palette/capability 存在，但 catalog/resource/factory/operation 不闭合。 |
| G31-G32 Security/scale/recovery/competitive maturity | Fail | 无多进程、DoS、fault/soak/scale、同场景 Unreal/Godot benchmark 或发布证据。 |

## 11. 禁止的临时修补

- 禁止把 `Lobby_Default`、Gold、player/latency/warning 换成随机数、socket count 或 timer 后继续称真实 Online Services。
- 禁止只创建缺失 ZUI、返回 `Ok(())` factory、把 descriptor registration 当作 asset/document/compiler 完成。
- 禁止把 `PlayKind` 再包装一层而不传 server/client role、client count、port、account、profile、seed 和 readiness。
- 禁止在 Editor 复制 Runtime NetManager、RPC、Replication、RUDP 或日志/Timeline authority；严格消费 Runtime140、Editor07、Editor11、Editor25 的 canonical owner。
- 禁止用单机 loopback、isolated manager、descriptor string test 或 ignored microbenchmark 证明多人产品和 Unreal 竞争性能。
- 禁止在 credential/consent/tenant/retention/anti-abuse 边界未完成前接入真实平台或将 token 写入 project asset/session archive/log。

## 12. 本轮完成定义

本轮完成 Editor100 current-source review：Workbench/Net Editor/catalog/Play topology/Runtime handoff 共冻结 266 个 Zircon selected 文件、30,116 行、252 个 test attributes、15 个 ignored declarations；15 个 Unreal/Godot 参考文件冻结 7,660 行，fingerprint `64cf6be79cfac86a95be0f26a430ceb75fcb13150635662834738187b92641a8`，Godot `8c7e6c...`、Fyrox `8d815d...`、Bevy `fb89a8...`、Unity Graphics `a7e4c0...`。

本轮只修改 review 与索引，没有运行 Cargo、Editor、Net provider、Dedicated/Listen + N clients、packet emulation、fuzz、fault、scale、soak 或 competitive benchmark；Tooling 排除，也未查询或实时跟踪协调器。实现状态仍为 pending。只有 M0-M7 通过真实 source/document/artifact/provider/session/World/observation/security/scale 资格后，Editor Multiplayer 才能从当前 fixture/descriptor 状态升级。
