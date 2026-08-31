---
title: Runtime Network / Transport / Socket / TLS / HTTP / WebSocket / Reliable UDP / Session / RPC / Replication / Prediction / Content Download / Editor 当前源码复审
category: zircon_runtime
report_id: Runtime140
review_date: 2026-08-24
baseline_head: ed543173cbd825fe3b7e1f6c81d52c9ca3391095
baseline_epoch: 422
verification_head: ed543173cbd825fe3b7e1f6c81d52c9ca3391095
verification_epoch: 422
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_plugins/10-first-party-network-source-runtime-editor-dist-catalog-transport-rpc-replication-product-integration-review.md
  - docs/plans/optimize/zircon_editor/26-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-review.md
related_code:
  - zircon_runtime/src/core/framework/net
  - zircon_plugins/net
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog
  - zircon_runtime/src/plugin/export_build_plan
  - zircon_app/src/entry
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/multiplayer
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
plan_sources:
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_plugins/10-first-party-network-source-runtime-editor-dist-catalog-transport-rpc-replication-product-integration-review.md
  - docs/plans/optimize/zircon_editor/26-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-review.md
  - docs/plans/zircon_plugins/07-net.md
  - docs/plans/zircon_plugins/07/2026-07-09-net-output-records.md
  - docs/plans/zircon_plugins/07/2026-07-17-m1-poison-recovery-hard-cut.md
  - docs/plans/zircon_plugins/07/2026-08-01-net-main-system-set-output-records.md
  - docs/plans/optimize/zircon_plugins/10/2026-08-19-lazy-replication-payload-clones.md
  - docs/plans/optimize/zircon_plugins/10/2026-08-19-rpc-priority-heap.md
  - docs/plans/performance/01/2026-08-24-plugin-net-replication-current-source-algorithm-performance-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetDriver.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetConnection.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/ReplicationGraph/Source/Public/ReplicationGraph.h
  - dev/UnrealEngine/Engine/Source/Runtime/Net/Iris/Public/Iris/ReplicationSystem/ReplicationSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Online/HTTP/Public/HttpRetrySystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Online/WebSockets/Public/IWebSocket.h
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices/Public/Interfaces/IBuildInstaller.h
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineSubsystem/Source/Public/Interfaces/OnlineSessionInterface.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/NetworkPrediction/Source/NetworkPrediction/Public/NetworkPredictionBuffer.h
  - dev/godot/scene/main/multiplayer_peer.h
  - dev/godot/modules/multiplayer/scene_multiplayer.h
  - dev/godot/modules/multiplayer/scene_rpc_interface.h
  - dev/godot/modules/multiplayer/scene_replication_interface.h
  - dev/godot/modules/multiplayer/editor/editor_network_profiler.h
  - dev/godot/modules/enet/enet_multiplayer_peer.h
  - dev/godot/modules/websocket/websocket_peer.h
  - dev/Fyrox/fyrox-core/src/net.rs
  - dev/bevy/crates/bevy_remote/src/lib.rs
  - dev/bevy/crates/bevy_remote/src/http.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime140 · Network 当前源码复审

## 1. 结论

当前Network不是工程级游戏网络系统，而是“同步socket facade、两个Tokio runtime、若干真实loopback、六个互不闭合的optional feature、内存RPC/Replication/RUDP模型、内存下载器、不可达Editor和静态多人Workbench”的组合。代码并非全是假实现：Runtime已有endpoint、security、HTTP/WebSocket、session、RPC、sync、reliable和download中立DTO；base插件能真实bind/connect/accept/send/poll TCP/UDP；HTTP和WebSocket有本机client/server测试；RUDP具备分片、ACK、重传和有序交付算法；RPC有direction/schema/quota；Replication有delta/interest/budget；Content Download有range/hash/mirror/resume bitmap。近期还完成了event drain、poison recovery、RPC priority heap、replication payload延迟clone、RUDP分组查找/移动payload和download bitmap/map等局部优化。这些都应保留，但不能继续被解释成多人网络产品已经闭环。

普通产品链首先就是断的。`zircon_app`默认是`target-client`，只带`net-contracts`，不带`first-party-runtime-plugins`；`target-server`甚至没有`net-contracts`和首方Net provider；`target-editor-host`也不链接base Net runtime。首方runtime catalog即使启用base feature，也只会把`RuntimePluginId::Net`映射到root runtime，不会收集HTTP、WebSocket、RPC、Replication、Reliable UDP和Content Download六个feature provider。首方Editor catalog只链接Navigation和Neural，完全没有Net Editor。builtin catalog里的feature row和crate name只是声明，不等于普通Client/Server/Editor中存在provider。

导出链需要分开判定。`SourceTemplate`会生成feature crate依赖和`plugin_feature_registration`调用，具备真实链接结构；`LibraryEmbed`计划虽然把feature crate列入`linked_runtime_crates`，实际command仍只构建`zircon_app --no-default-features --features target-client|target-server`，没有把计划中的crate依赖或注册函数注入宿主。NativeDynamic的Net dist又是stateless metadata entry，command/event/state/bridge/unload全部为空。因此source template局部可组合，不能替普通host、LibraryEmbed或NativeDynamic证明产品等价。

即使手工加载全部source provider，运行时仍不是一个network stack。HTTP和WebSocket factory忽略所声明的Net dependency，各自创建私有`DefaultNetManager`和后端；RPC、Replication、Reliable UDP分别创建独立内存manager；Content Download是唯一解析canonical `NetManager`的feature，却因此拿不到HTTP私有manager上的backend，测试通过直接注入`http_runtime_manager()`绕开了产品断点。feature dependency只表达声明顺序，没有形成实例注入、generation lease或行为接线。

base I/O也仍同步阻塞。公开`NetManager`方法把命令提交到单一有界worker，然后调用线程最长等待2秒；worker专用线程内再建multi-thread Tokio runtime并逐命令`block_on`，manager state另建第二个multi-thread runtime供HTTP/WebSocket使用。慢connect/send可产生队头阻塞，caller timeout不取消底层操作，worker ingress满时事件被静默丢弃。`net.poll_ingress`只发布事件、不搬运payload，`net.flush_egress`为空，diagnostics frame始终写0。TCP无frame/channel/backpressure合同；UDP每次poll分配65,535字节buffer；endpoint字符串拼接解析没有DNS/IPv6/Happy Eyeballs；裸`u64` ID没有owner/generation/exhaustion语义。

协议和安全存在不能以Beta掩盖的false surface。远端无显式port的HTTP URL仍可能仅按path命中本地route；client每次重建Hyper/Reqwest client并全量收集body，对所有method/error立即retry，没有幂等、backoff、jitter或`Retry-After`。HTTP pin路径先`danger_accept_invalid_certs(true)`再检查leaf，缺少完整chain/hostname/rotation产品合同。WebSocket的WSS只检查配置里存在pin字符串，实际`connect_async`没有安装custom root或pin verifier；server只有明文WS。inbound frame/event无界，没有max message、heartbeat和完整close/task fence。HTTP server无限accept/spawn、同步handler运行在executor、仅HTTP/1、固定1 MiB body且没有graceful drain。

RPC、Replication和Reliable UDP仍是算法模型。RPC handshake使用固定challenge，token没有认证用途，caller直接传role；channel是本地无界queue，handler同步执行后才检查timeout，没有wire codec、network correlation或transport dispatch。Replication没有World/Reflection/change detection/authority连接，没有spawn/despawn、baseline/ACK、stable wire schema、serializer/quantizer、dormancy或prediction；调度每轮排序snapshot，Transform插值只靠名字和首4字节f32。RUDP没有socket/peer/session连接，logic packet与wire packet字段宽度和channel模型不一致，ACK低16位可能跨wrap误确认，assembly无byte/age上限，也没有RTT/RTO、congestion、pacing、security或anti-amplification。

Content Download仍是同步内存下载原型：每次推进一个chunk/attempt，使用development security，request ID可跨下载冲突，chunk/partial/bitmap没有磁盘journal、startup recovery、atomic install、cache quota或repair；manifest没有签名、信任链、布局上限和URL policy；cancel只改状态，不取消正在进行的请求。其局部indexed bitmap和map优化是真实Partial，但不改变crash consistency与产品不可达。

Editor和多人产品表面是当前最严重的真实性问题。Net Editor注册view、drawer、command、inspector、asset、graph和palette descriptor，但引用的`authoring.zui`、三个inspector ZUI和default TOML都不存在，六个operation没有factory/handler，首方Editor catalog也不链接它。Workbench的Lobby/Matchmaking界面把房间、玩家、延迟、队列、backfill和告警写死在ZUI里；Simulate/Validate只回填预写字符串，没有Lobby、Online Session、Matchmaking、Allocation、provider或server+N clients会话。代码库没有可执行多人PIE拓扑、per-link network emulation、replication inspector或真实network profiler。

因此当前没有任何证据支持“性能和表现优于当前Unreal”。14个ignored release test只比较局部容器/clone/查找路径；没有同功能协议、相同安全策略、真实World、跨进程、多平台、loss/jitter、10k connection、100k object、24h soak或CPU/RSS/bandwidth/tail-latency竞争证据。本报告只刷新事实和重构门，不修改生产代码。

历史台账重判：Runtime08E的20项P1为 **18 Open、2 Partial**，5项P2全部Open；Plugins10的48项P1为 **43 Open、5 Partial**，12项P2全部Open；Editor26的5项P0、60项P1和12项P2全部Open。32项综合资格门为 **26 Fail、6 Partial、0 Pass**。

## 2. 审查边界、方法与currentness

### 2.1 冻结Network范围

统计口径为当前工作树物理行、非空行、文件bytes、Rust `#[test]`和`#[ignore`声明。fingerprint按normalized lowercase path排序，对每个文件拼接`path + NUL + lowercase(file SHA-256) + LF`后再取SHA-256。产品consumer集合是明确列出的App/catalog/export/Workbench文件，不代表这些crate的总规模。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Runtime Net中立合同全量 | **18 / 2,107 / 1,867 / 62,823 / 6 / 0** | `7f8c451aa840783548f654c772639b6f91ed1eb865981b3936f93a7deaec5a7d` |
| Net插件runtime/features/editor/dist/tests全量 | **186 / 17,074 / 15,404 / 601,569 / 149 / 14** | `c358454d07086153e696239f71480c9d9ed9e61b7ce122dc36c6d6d499f53cab` |
| App/catalog/export/Workbench产品consumer | **33 / 5,282 / 4,926 / 213,769 / 44 / 0** | `d90b23ea590184c18568adc7e7b298968da1d5a54957f74b495b5df94d47a420` |
| Zircon selected union | **237 / 24,463 / 22,197 / 878,161 / 199 / 14** | `aa375b2e5990c435dfdf3147c6180adbb19938df4f9d7d7fb409523e50363758` |
| 五引擎参考选择集 | **33 / 27,253 / 22,562 / 1,075,609 / 11 / 0** | `a74f4b964b48440e78b5ab806c74a58616ce724c72ffd3c6206eaaeb168b0d7c` |

Net插件中，base runtime为49文件/5,355行/39 tests/2 ignored，六feature合计126文件/10,966行/107 tests/12 ignored，Editor为6文件/421行/1 test，dist为1文件/98行/2 tests。近似production集合为123文件/11,771行/28 tests/14 ignored，专用test路径为59文件/5,069行/121 tests。测试规模证明局部算法被覆盖，不证明普通产品provider closure、跨进程session或World replication。

参考revision：Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`。`dev/UnrealEngine`没有独立Git元数据，由16个选择文件及参考集合fingerprint冻结。

### 2.2 检查方法

1. 逐文件读取18个Runtime Net合同和Net包全部186个文件，包括Cargo、manifest、runtime、六feature、Editor、dist和所有测试。
2. 沿`App target/profile -> builtin manifest -> first-party catalog -> registration -> Module factory -> service handle -> worker/backend`追踪普通产品链；不从类型名、feature row或单元测试推断可达性。
3. 沿`socket -> connection/session -> channel/RPC -> replication -> World`和`HTTP -> content -> disk/install`追踪真实effect、owner、budget与terminal cleanup。
4. 沿`Editor catalog -> contribution -> resource -> operation factory -> document/compiler -> runtime artifact -> PIE/diagnostics`核对authoring闭环，并核对Workbench static fixture。
5. 对Runtime08E、Plugins10、Editor26及三个局部performance record逐项重判。减少clone、sort或查找但不改变产品失败条件的项最多Partial。
6. 对照Unreal的NetDriver/Connection/Channel、Iris/ReplicationGraph、NetworkPrediction、Online Session、HTTP/WebSocket和BuildPatch；对照Godot scene multiplayer/ENet/WebSocket/editor profiler；Bevy/Fyrox/Unity按其真实参考边界降权。

### 2.3 动态证据边界

- Session基线、冻结HEAD为`ed543173cbd825fe3b7e1f6c81d52c9ca3391095` / epoch 422。
- Net相关源码包含用户或其他Session的working-tree修改；本报告读取当前内容，不覆盖、不回退，也不把未提交ignored benchmark写成集成资格。
- 本轮review-only，没有运行Cargo、App、Editor、dedicated server、NativeDynamic、真实公网/TLS、双进程、packet emulator、fuzz、sanitizer、scale、soak或竞争benchmark。
- 静态调用图足以证明的provider缺失、private manager、同步等待、空flush、WSS假pin、无World caller、内存download、缺资源/factory和固定Workbench反馈不因未跑Cargo而改变。
- Tooling按用户要求排除；LibraryEmbed只审查Runtime生成的计划和产品结果，不优化Python执行器。

## 3. 当前真实产品链路

```text
ordinary zircon_app Client
  default -> target-client -> net-contracts
  x no first-party-runtime-plugins -> no Net root provider

ordinary zircon_app Server
  target-server
  x no net-contracts
  x no first-party Net provider

ordinary zircon_app Editor Host
  target-editor-host -> advanced render/navigation/neural providers
  x no base Net runtime
  x first-party editor catalog has Navigation/Neural only -> no Net Editor

explicit base runtime catalog
  RuntimePluginId::Net -> Net root registration
  -> canonical DefaultNetManager with TCP/UDP worker
  x six optional feature providers are not collected

explicit feature construction
  HTTP private DefaultNetManager + HTTP backend
  WebSocket private DefaultNetManager + WS backend
  RPC / Replication / RUDP independent in-memory managers
  Content Download -> canonical NetManager -> normally no HTTP backend

export
  SourceTemplate -> generated dependencies + provider calls (structural path exists)
  LibraryEmbed -> report lists crates, build command does not link/register them
  NativeDynamic -> stateless metadata-only Net dist

Editor / Workbench
  Net descriptors -> missing resources + missing factories
  Lobby/Matchmaking -> fixed ZUI data + fixed feedback only
```

目标不是把所有逻辑继续堆进`DefaultNetManager`。App只应提交project/profile selection；Runtime创建generation-bound `NetworkRuntimeInstance`和per-purpose/per-World driver；I/O supervisor拥有transport task，secure session拥有principal/role，World replication拥有object/baseline/prediction，HTTP/Content拥有独立服务边界但共享executor/security/config，Editor只消费同一artifact、activation receipt和observation。

## 4. 必须保留的基础

1. 保留Runtime-owned中立Net DTO和typed report，但为endpoint、handle、request、session、object、wire artifact补owner、generation、version、deadline、cancel和budget。
2. 保留真实TCP/UDP/HTTP/WebSocket loopback与第三方库，迁移到唯一I/O supervisor，不回退直接在World线程调用`std::net`。
3. 保留`net.main`/`net.transport` system set和First/Last anchor；把它们接成真实bounded ingress/egress，不保留空flush。
4. 保留poison/reentry/失败零发布修复和局部move/COW/heap/index优化；将其纳入统一生命周期与动态资格。
5. 保留HTTP leaf pin读取peer certificate的局部能力，但必须重做完整TLS verifier、chain/hostname/root/pin/rotation语义。
6. 保留RUDP fragment/ACK/resend/order算法作为codec testkit；只允许在单一wire model和真实peer transport中复用。
7. 保留RPC direction/schema/quota与Replication delta/interest/budget词汇；caller role和String/raw-byte schema必须硬切。
8. 保留Content Download chunk hash/mirror/resume算法；迁入signed manifest、stream-to-disk和transactional installer。
9. 保留SourceTemplate生成provider注册的结构；普通source、LibraryEmbed和NativeDynamic必须消费同一resolver/receipt。
10. 保持Net为Beta/Partial/default-off。G01-G32通过前不得升级或用ignored microbenchmark宣称优于Unreal。

## 5. P0：产品真实性与安全底线

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| NET-P0-001 | Open | Lobby/Matchmaking Workbench把固定房间、玩家、延迟、队列、backfill和warning冒充在线产品 | 默认入口撤销或明确Demo/Unavailable；只有真实document/provider/session receipt可驱动产品文案 |
| NET-P0-002 | Open | 代码库没有Identity/Party/Lobby/Matchmaking/Ticket/Allocation/Online Provider runtime authority | 建立provider-neutral Online Services域；禁止把socket `NetManager`改名扩张为在线服务 |
| NET-P0-003 | Open | Net Editor默认不可达，手工注册后5个资源缺失且6个operation无factory | catalog/resource/factory端到端通过前隐藏capability并返回typed unavailable |
| NET-P0-004 | Open | Replication Schema只有asset/graph descriptor，没有document/compiler/artifact/runtime install，Runtime又不连World/transport | 建立stable wire artifact、server/client compat hash和真实World消费后才允许Create/Validate/Compile成功 |
| NET-P0-005 | Open | WSS界面接受certificate pin/custom roots，实际连接未应用它们；Simulate又没有server+N clients或网络仿真 | WSS先fail-close并完成真实verifier；多人Simulate必须来自可终止、可观测、可重放的session group |

## 6. P1：Package、Catalog、Capability、Editor与Distribution

| ID | 状态 | 当前差距 | 需要重构 |
|---|---|---|---|
| NET-P1-001 | Open | 默认Client/Server/Editor目标不链接Net provider，Server还没有net-contracts | target/profile生成完整selected provider closure；required缺失时启动fail-close |
| NET-P1-002 | Open | 首方runtime catalog只返回Net root，六个feature不进入普通provider collection | 生成root-feature provider graph和requested/linked/admitted/activated/degraded receipt |
| NET-P1-003 | Open | 首方Editor catalog没有Net Editor | 同一project selection解析runtime/editor closure；缺provider时隐藏所有Net菜单/asset/toolkit |
| NET-P1-004 | Open | feature dependency只存在manifest，HTTP/WS/RPC/Replication/RUDP factory忽略依赖实例 | activation transaction注入同代typed lease，禁止factory自行创建第二authority |
| NET-P1-005 | Open | HTTP和WebSocket各自创建私有`DefaultNetManager` | backend以extension transaction安装到目标`NetworkRuntimeInstance`并可quiesce/unload |
| NET-P1-006 | Open | RPC、Replication、RUDP各自创建独立内存manager | 消费同一secure session/connection/channel owner，保留模块边界但删除第二状态真相 |
| NET-P1-007 | Open | `NetConfig`和manifest option未驱动manager/system/worker | 建立validated `EffectiveNetConfig`，记录source/target/override/generation/apply receipt |
| NET-P1-008 | Open | runtime mode、target mode、role、feature和security不是单一activation snapshot | `NetworkActivationPlan`明确Client/Listen/Dedicated/Editor Preview允许的listener/service |
| NET-P1-009 | Open | event catalog只有4类且schema只是字符串，与实际event/feature不闭合 | 生成versioned event schema，覆盖transport/session/RPC/replication/download/drop/lifecycle |
| NET-P1-010 | Open | Net dist为stateless metadata shell，无command/event/state/bridge/host-ready/unload | 实现native provider与feature behavior、quiescence和state handoff，或撤销NativeDynamic支持 |
| NET-P1-011 | Open | ordinary source、SourceTemplate、LibraryEmbed、NativeDynamic行为不等价 | 全部路径消费同一ProviderResolver和golden activation receipt；LibraryEmbed命令必须真实链接 |
| NET-P1-012 | Open | Beta/Partial没有feature级升级、降级或撤销资格 | maturity绑定G01-G32和BuildSet artifact，feature不可执行时只能Unavailable/Degraded |
| NET-P1-013 | Open | `authoring.zui`、三个配置ZUI和default replication TOML不存在 | package resource manifest编译、hash、source/embed/dynamic解析并测试资源闭包 |
| NET-P1-014 | Open | 六个Net Editor operation只有descriptor | 每项绑定typed payload、permission、transaction/job、cancel/deadline和terminal receipt |
| NET-P1-015 | Open | asset/toolkit/graph没有document、save/undo/compiler/runtime artifact owner | 建立lossless source、semantic compiler、artifact install、preview与cook/export parity |
| NET-P1-016 | Open | Diagnostics surface没有producer，Workbench只有固定feedback，无多人拓扑/仿真 | typed Network Profiler接真实runtime trace；Editor启动server+N clients与per-link emulator |

## 7. P1：Base I/O、Transport、Security与Lifecycle

| ID | 状态 | 当前差距 | 需要重构 |
|---|---|---|---|
| NET-P1-017 | Open | `DefaultNetManager`是process级共享状态，不按purpose/World/Session/role分域 | process supervisor下建立driver registry和world/session-owned instance，teardown有generation fence |
| NET-P1-018 | Open | 同步API等待单一串行worker，manager和worker各有Tokio runtime | 收敛为唯一显式I/O executor与operation ticket；deadline/cancel必须停止底层I/O |
| NET-P1-019 | Open | 慢connect/send造成队头阻塞，caller timeout后worker仍可能成功并留下孤儿状态 | per-connection/transport task、公平调度、in-flight budget和exactly-once terminal publication |
| NET-P1-020 | Partial | event drain、poison处理和若干clone已改善；ingress满仍静默丢event，主event无界，flush为空，frame固定0 | producer到World全链限制entries/bytes/age/share，记录drop cause并实现egress completion receipt |
| NET-P1-021 | Open | socket/listener/connection/route/request为裸递增`u64` | 消费generation handle合同，带driver/World owner、near-exhaustion、retire和stale reject |
| NET-P1-022 | Open | TCP只有byte stream，无frame/channel/max message/partial write/backpressure/half-close | versioned bounded framing、channel QoS、flow control和完整wire byte accounting |
| NET-P1-023 | Open | UDP poll每次分配65,535字节buffer，缺peer admission/buffer pool/batch/truncation/fairness | 预分配buffer pool、batch receive、source quota、MTU/error语义和feature packet pipeline |
| NET-P1-024 | Open | `NetEndpoint`拼字符串parse，无DNS、IPv6 bracket、Happy Eyeballs、interface/path变化 | typed address/authority、async resolver、IPv4/IPv6 policy、socket options和path observation |
| NET-P1-025 | Open | 无显式port的远端HTTP URL可按path被本地route截获 | local dispatch使用独立scheme/authority或显式in-process route handle，禁止URL猜测 |
| NET-P1-026 | Open | HTTP每请求建client、clone/full-buffer body，retry不看method/idempotence且无backoff/jitter/Retry-After | pooled client、streaming、response cap、cancel、idempotency key与policy-driven retry |
| NET-P1-027 | Open | HTTP server无限accept/spawn，sync handler占executor，只有HTTP/1和固定body cap，无graceful drain | connection/request/header/body/rate预算、handler executor、stream backpressure和shutdown barrier |
| NET-P1-028 | Open | HTTP pin先接受无效cert再事后比leaf，root/system/hostname/rotation/credential/redaction不完整 | 统一TLS verifier与environment trust profile，校验chain/hostname/pin/expiry/rotation并secure-by-default |
| NET-P1-029 | Open | WSS custom roots/pin未进入`connect_async`，测试只证明配置字符串存在 | 实际读取peer chain并验证root/hostname/pin，stable security reason且禁止silent downgrade |
| NET-P1-030 | Open | WebSocket server只有WS，无WSS identity/reload/client auth | WSS listener、certificate rotation、mTLS/authorization adapter和shipping policy |
| NET-P1-031 | Open | WS outbound仅64帧局部有界，inbound/event无界，无max bytes/heartbeat/close timeout/task fence | 双向entry/bytes/age预算、heartbeat、close handshake、task owner和bounded teardown |
| NET-P1-032 | Open | diagnostics只有聚合计数；close多为删map/改state，不能证明accept/read/write task quiesce | typed per-driver/connection observation；Closing/Draining/Closed状态机和abort/join/leak receipt |

## 8. P1：RUDP、Session、RPC、Replication、Prediction与Content

| ID | 状态 | 当前差距 | 需要重构 |
|---|---|---|---|
| NET-P1-033 | Partial | RUDP已有payload move、分组lookup和局部分配改进；manager仍不使用UDP，logic/wire packet不一致 | 冻结唯一versioned codec并接真实peer/socket；旧双模型一次硬切删除 |
| NET-P1-034 | Open | ACK只按低16位匹配，ordered sequence全channel共享，assembly无byte/age/peer上限 | per-peer/per-channel window、wrap-safe ACK、assembly TTL/memory cap和abuse policy |
| NET-P1-035 | Open | RUDP没有RTT estimator、dynamic RTO、congestion、pacing、path MTU、crypto、anti-replay/anti-amplification | 优先采用成熟可靠datagram库；自研前必须有wire spec、interop、security review和emulator |
| NET-P1-036 | Open | session固定challenge，token未认证，principal/role/source session由caller输入 | connection-bound随机challenge/proof、principal/role派生、version/content/schema negotiation和replay protection |
| NET-P1-037 | Partial | RPC priority heap减少局部排序；channel仍是本地无界queue，handler timeout不可抢占 | compiled RPC ID/channel/correlation，bounded async executor、deadline/cancel、dedup和terminal response |
| NET-P1-038 | Open | RPC无wire codec、network send/receive、生产`apply_transport_events` caller或World/system绑定 | 从secure channel接收/验证/dispatch/response，handler affinity和authority由runtime决定 |
| NET-P1-039 | Open | Replication manager没有production caller，不接World、Reflection、transport或authenticated connection | per-World owner读取change tick，向per-connection channel发布spawn/update/despawn并transactional apply |
| NET-P1-040 | Open | descriptor authority/strategy/delta/type多为惰性metadata，String/raw-byte没有stable wire identity | compiler生成type/component/field/RPC ID、serializer、quantizer、condition、migration和compat hash |
| NET-P1-041 | Partial | 延迟payload clone和candidate结构减少局部分配；调度仍全snapshot排序且只按payload预算 | dirty queue、persistent per-connection priority/frequency结构和header/compression/encryption实际预算 |
| NET-P1-042 | Open | 无baseline/ACK/NACK、known object、dormancy、spatial/owner relevancy、priority debt和resync | connection replication state、checkpoint、interest graph、late join、loss recovery和bounded history |
| NET-P1-043 | Open | 插值按component名含`transform`和首4字节f32，只有receive-time固定延迟 | typed vector/quaternion serializer、clock sync、adaptive jitter buffer、extrapolation/teleport和smoothing policy |
| NET-P1-044 | Open | 没有input command、client prediction、server correction、reconciliation、rollback、rewind或lag compensation | clocked input/authoritative snapshot、bounded state history、resimulation和determinism/divergence artifact |
| NET-P1-045 | Open | Content factory解析canonical manager，但测试注入HTTP私有manager绕过产品composition | dependency lease必须解析同一instance的HTTP capability；普通App/export/native集成测试关闭断点 |
| NET-P1-046 | Partial | bitmap/indexed maps/attempt URL已降低局部复杂度；下载仍同步单chunk、内存partial/cache且用development security | async scheduler、bounded concurrency、production trust、stream-to-disk、cancel/resume和bandwidth/space quota |
| NET-P1-047 | Open | manifest无签名/发布身份/layout/URL policy，request ID冲突，resume bool可把未验证cache标完成 | signed manifest、checked size/offset/layout、content-addressed cache、verified journal和unique operation identity |
| NET-P1-048 | Open | 无temp/fsync/atomic publish、persistent cache、install/mount、repair/rollback/crash recovery；测试偏loopback/direct manager | BuildPatch式stage/verify/install/activate/retire/last-good；补产品、cross-process/platform、fuzz/fault/scale/soak矩阵 |

## 9. P2：工程级高级能力

| ID | 状态 | 能力差距 | 目标 |
|---|---|---|---|
| NET-P2-001 | Open | 大世界/海量对象Replication Graph未产品化 | spatial/grid/team/owner/shared nodes、parallel gather和persistent connection cache |
| NET-P2-002 | Open | 高级prediction、physics rollback、lag compensation和replay缺失 | shared simulation history、server rewind、spectator/replay、correction visualization和divergence artifact |
| NET-P2-003 | Open | NAT traversal、relay、P2P和platform socket未设计 | provider-neutral route negotiation、ICE/STUN/TURN/relay policy、privacy和fallback receipt |
| NET-P2-004 | Open | Lobby、party、matchmaking、allocation、backfill和Online Services缺失 | 独立Online Services域与provider adapters，不把transport owner变成平台服务 |
| NET-P2-005 | Open | voice/text chat/moderation不在当前package | 独立media/social service，复用identity和QoS并满足consent/privacy/platform policy |
| NET-P2-006 | Open | multi-region、server handoff、migration和fleet drain缺失 | session continuity、state transfer、reservation、duplicate suppression和failure rollback |
| NET-P2-007 | Open | QUIC/WebTransport/console/mobile path适配缺失 | capability-driven transport provider和network path change/background-resume contract |
| NET-P2-008 | Open | live protocol rollout和cross-version compatibility缺失 | support window、feature negotiation、dual-read/controlled-write、canary和rollback |
| NET-P2-009 | Open | DDoS、anti-cheat、abuse/ban与attestation不是完整产品 | admission/rate/anomaly/audit、server authority和shipping-only trust hooks |
| NET-P2-010 | Open | CDN delta、multi-source/P2P patch和内容优先级缺失 | signed chunk graph、delta/repair、adaptive source、QoS和transactional install |
| NET-P2-011 | Open | packet capture、protocol inspector和offline replay缺失 | privacy-aware bounded capture、schema decode、timeline/correlation和failure corpus |
| NET-P2-012 | Open | 第三方网络SDK生态与认证矩阵缺失 | stable provider SDK、conformance kit、sandbox/trust、migration和platform certification |

## 10. 历史台账与局部优化重判

### 10.1 Runtime08E

20项P1中只有“锁外回调/poison恢复”和“部分event/clone路径”可判Partial，其他18项仍Open；5项P2全部Open。`net.main`/`net.transport`注册和loopback行为是真实历史成果，但output record明确没有证明transport-connected RPC/Replication、product composition或Editor。

### 10.2 Plugins10

48项P1中`NET-P1-020/033/037/041/046`为Partial，其他43项Open；12项P2全部Open。2026-08-19/24的RPC heap、replication payload clone和algorithm performance记录只关闭局部排序/复制成本，不能关闭对应feature的owner、wire、security、World或产品门。

### 10.3 Editor26

5项P0、60项P1、12项P2全部Open。Workbench资源仍是固定fixture，Net Editor provider仍未进入catalog，5个插件资源和6个operation factory仍缺失，代码库仍没有Online Provider、多人PIE和network emulation。

### 10.4 Failure handoff

当前Network owning plan没有open `failure-*.md`。这只表示没有跨计划失败待接收，不表示G01-G32通过；历史output records继续作为局部实现证据，不作为current product资格。

## 11. 参考引擎差异

### 11.1 Unreal

`UNetDriver`按purpose/World拥有连接集合、地址路由、handshake、PacketHandler、timeout、network object list和replication driver；`UNetConnection`拥有per-connection channel、packet/bunch sequence、reliability、queued bits、owner/view、fault recovery和统计。Zircon可以采用Rust task/ECS模型，但不能以一个process manager、裸u64和caller传role替代这些owner/lifecycle。

Iris `ReplicationSystem`显式管理connection add/remove、replicated object handle、dirty/poll、view、filter、priority、owner、condition、delta compression、tear-off和cull distance；ReplicationGraph持有global/connection nodes并增量gather。NetworkPrediction又有frame ring buffer、input/sync/aux state、resimulation和Insights。当前Zircon只有String/raw-byte snapshot与receive-time插值，层级差异是架构性的。

Unreal HTTP retry包含verb/status/domain policy、exponential backoff、jitter和throttle响应；WebSocket接口有连接/错误/message/progress/close delegate；BuildPatch installer有start/pause/resume/cancel、verify/repair、progress/error/statistics、chunk source/store和install state；Online Session有create/start/update/end/destroy/find/join/register/matchmaking/invite/resolve-connect。Zircon下载器和Workbench不能用同名按钮替代这些状态机。

### 11.2 Godot

Godot `SceneMultiplayer`把MultiplayerPeer、pending/authenticated peer、auth timeout、object cache、RPC和replicator置于同一scene owner；Spawner/Synchronizer与ReplicationConfig接真实Node/property/visibility/authority。ENet backend提供server/client/mesh、peer/channel、bandwidth、connection status和refuse-new-connections。其规模小于Unreal，但仍反证把RPC/Replication/RUDP分成无transport内存manager的做法。

Godot `WebSocketPeer`明确限制inbound/outbound buffer和queued packets，维护heartbeat、close code/reason与state；Editor Network Profiler显示真实RPC/sync、bandwidth和replication数据，Replication Editor编辑真实Synchronizer config。Zircon当前无界inbound和空Diagnostics view不具备对等产品语义。

### 11.3 Bevy、Fyrox与Unity Graphics边界

Bevy主仓选择集是Remote JSON-RPC/ECS远程控制，不是游戏多人网络。可借鉴的是core method与HTTP transport分层、schedule handoff和反射ECS访问，不能用它降低session/replication/prediction标准。

Fyrox选择集只提供非阻塞TCP listener/stream、长度前缀和bincode消息，是诚实的小型framing原语，不是工程级多人栈。Unity `dev/Graphics`是SRP/渲染参考树，扫描没有Netcode/Online Services实现；本篇只记录参考缺口，不据此推断Unity网络能力或Zircon完成度。

## 12. 目标架构与Hard Cutover

```text
App Project/Profile Selection
  -> NetworkActivationPlan + CapabilityTruthReceipt
      -> Runtime-owned NetIoSupervisor
          -> DriverRegistry { game, beacon/service, replay, editor-test }
              -> NetworkDriverInstance { purpose, role, WorldGeneration? }
                  -> Listener/ConnectionTable { generational handles }
                      -> Transport task + bounded ingress/egress/channel queues
                      -> Secure Session { principal, role, version/content/schema }
                      -> Channels { control, RPC, reliable/unreliable data, input, replication }
                  -> NetWorldRuntime
                      -> NetObjectRegistry + compiled wire artifact
                      -> dirty/filter/prioritize/serialize/baseline/apply
                      -> clock/input/prediction/reconciliation/history
          -> HttpService / WebSocketService / ContentInstaller
          -> Observation/Trace/Capture bridge
  -> Editor consumes same artifacts, receipts, sessions and observations
```

必须硬切：

1. 删除HTTP/WebSocket feature factory中的私有`DefaultNetManager`；删除RPC/Replication/RUDP独立authority。
2. 将同步`NetManager` socket/HTTP/WS API硬切为ticket/bounded batch，不保留World/gameplay可继续调用`recv_timeout`/`block_on`的兼容层。
3. 删除空`net.flush_egress`、frame 0、静默`try_send`丢弃和diagnostics驱动全量搬运。
4. 删除裸ID跨driver/World使用及manager/worker双状态；完成generation migration后不保留数值alias。
5. 删除按“URL无显式port”猜测local route的逻辑，改显式in-process authority。
6. WSS verifier接通前撤销certificate pin/custom root capability，不能保留配置存在即安全的判断。
7. 删除RUDP logic/wire双模型和无socket manager；迁移到真实per-peer protocol后一次切换。
8. 删除固定challenge、unused token、caller role/source session和无wire RPC入口。
9. 删除String/raw-byte replication shipping主路径、名字包含Transform插值和未执行authority metadata。
10. 删除内存bool resume即cache hit、development security和全body拼接的Content Download生产路径。
11. 缺resource/factory/compiler/provider前隐藏Net Editor与Lobby/Matchmaking产品入口，不创建空文件或`Ok(())` handler伪闭环。
12. LibraryEmbed不能只在report声称linked；compile command、dependencies和registration必须与SourceTemplate等价。

## 13. 分层重构计划

### M0：Truth、Composition与Identity

- 建立默认Client/Server/Editor、六feature、SourceTemplate/LibraryEmbed/NativeDynamic的RED composition矩阵。
- 引入`NetworkActivationPlan`、唯一supervisor、feature extension transaction/lease和capability receipt。
- 接通effective config/security/role，删除private manager、false option和false capability。
- 固化driver/World/connection/session/operation/object generational identity。

### M1：I/O Executor、Operation与Shutdown

- TCP/UDP/HTTP/WS迁入唯一executor和per-connection task，移除caller同步等待。
- 实现deadline/cancel/exact terminal、bounded queue、公平调度和短锁registry。
- 完成StopAdmission/Drain/Close/Join/LeakReport与panic/stuck I/O恢复。
- 接通真实poll ingress/flush egress、wake/frame demand和drop observation。

### M2：Transport、Queue与Security

- 完成DNS/IPv6/Happy Eyeballs、socket options、TCP framing/half-close和UDP buffer pool/batch/truncation。
- 所有queue落地entry/bytes/age/per-peer share/fairness/drop policy。
- 统一TLS/DTLS/AEAD security profile、trust/pin/identity/rotation/shipping guard。
- 建立wire parser property/fuzz、malicious peer和resource exhaustion测试。

### M3：HTTP、WebSocket与Content Installer

- HTTP pool/stream/cancel/deadline/idempotent retry和bounded server/router/graceful shutdown。
- WebSocket WSS、bounded message/frame、heartbeat、close/task lifecycle和per-IP quota。
- Content signed manifest、stream-to-disk incremental hash、journal/resume/cache/atomic install/repair。
- 用普通catalog/App/Hub/Editor/export关闭test-only injection。

### M4：Secure Session、Channel与RPC

- connection-bound auth、principal/role、version/content/schema negotiation和admission quota。
- versioned channel/frame codec、reliable/unreliable routing、correlation和cleanup。
- compiled RPC table/serializer/permission，接通receive/dispatch/response和async affinity。
- 双进程join/RPC/disconnect/reconnect/world travel资格。

### M5：World Replication

- 接入NetworkIdentity、World change detection、spawn/despawn/ownership/subobject和client apply transaction。
- 编译stable descriptor、serializer/quantizer/change mask/condition/schema hash。
- per-connection known object/baseline/ACK、interest/priority/frequency/dormancy和wire budget。
- 验证late join、loss recovery、object reuse、World replacement和10k/100k object。

### M6：Input、Prediction与Time Sync

- server tick/time sync/input command/ack和authoritative snapshot合同。
- prediction history、reconciliation/resimulation、typed smoothing和visual correction。
- latency/jitter/loss/reorder/teleport/listen server parity与determinism artifact。
- 高级physics rollback/lag compensation/replay按P2切片实施，不做简化占位。

### M7：Reliable Datagram与Emulation

- 先选型成熟transport；若自研，先冻结wire spec、interop和security review。
- per-peer/channel window、wrap、ACK、fragment、RTO、congestion/pacing、MTU和anti-amplification。
- loss/reorder/duplicate/corrupt/stall emulator、parser fuzz和长时间wrap/soak。
- RPC/replication/input按policy使用真实channel，删除孤立manager。

### M8：Editor、Online Services、Diagnostics与Dist

- 交付真实Net resources、operation/document/compiler、Network Profiler、replication inspector和PIE topology/emulation。
- Lobby/Matchmaking由独立Online Services provider/document/simulator驱动，删除固定fixture成功文案。
- dist打包真实provider behavior、assets/config/schema和unload；source/library/native parity。
- 实施approved Replication Graph、platform provider和capture切片。

### M9：产品、安全与性能资格

- 完整App/Server/Editor/Hub/export、双进程/多进程、World travel和cross-version矩阵。
- fuzz/sanitizer/fault/TLS/security/DoS、24h soak与shutdown leak门。
- 1/100/10k connections、1k/10k/100k objects、0/1/60s consumer stall规模矩阵。
- 与Unreal/Godot同场景、同安全和质量记录CPU、p95/p99、bandwidth、allocation/RSS、recovery和server capacity。

## 14. G01-G32 综合资格门

| Gate | 状态 | 验收条件 |
|---|---|---|
| G01 Unique authority | Fail | 一个Core只有一个I/O/config/security authority，feature只持lease |
| G02 Product reachability | Fail | Client/Server/Editor按profile真实链接required Net provider |
| G03 Effective config | Fail | 所有manifest option有validated consumer和apply/restart receipt |
| G04 Scoped identity | Fail | driver/World/connection/session/object/ticket全部generation-bound |
| G05 Async lifecycle | Fail | main/World线程零`block_on`/`recv_timeout`，cancel停止底层I/O且终态唯一 |
| G06 Bounded queues | Partial | 局部command/writer有界；全链仍需entry/bytes/age/share/drop |
| G07 Transport correctness | Partial | loopback真实；framing/DNS/IPv6/half-close/UDP pool/interop未闭合 |
| G08 Security | Fail | TLS/WSS/secure session/credential/shipping guard与malicious peer门 |
| G09 HTTP | Partial | 本机HTTP和leaf pin局部存在；pool/stream/retry/server/drain仍缺 |
| G10 WebSocket | Fail | WSS verifier、bounded inbound、heartbeat、close/task fence通过 |
| G11 Reliable datagram | Partial | fragment/ACK/resend算法存在；真实peer/wire/congestion/security未闭合 |
| G12 Session auth | Fail | principal/role来自认证connection，challenge/proof可抗重放 |
| G13 RPC | Fail | stable codec、transport correlation、async handler/deadline/response闭环 |
| G14 World replication | Fail | 真实World spawn/update/despawn/apply和ownership闭环 |
| G15 Wire schema | Fail | stable IDs、serializer/quantizer/condition/migration/compat hash |
| G16 Baseline/relevancy | Fail | per-connection baseline/ACK/interest/dormancy/budget/loss recovery |
| G17 Prediction | Fail | clocked input、correction、resimulation、rollback/typed smoothing |
| G18 Content install | Partial | chunk hash/bitmap局部存在；signed/disk/journal/atomic install未闭合 |
| G19 Net Editor | Fail | 资源、factory、document/compiler/preview/undo/save全部真实 |
| G20 Online Services | Fail | identity/lobby/session/matchmaking/allocation/provider状态机存在 |
| G21 Multiplayer PIE | Fail | Dedicated/Listen + N clients、ports/accounts/sandbox/readiness/reap |
| G22 Observation | Fail | RTT/loss/jitter/queue/RPC/object/prediction/download typed trace |
| G23 Native parity | Fail | dist提供真实behavior/state/bridge/quiesce/unload和feature闭包 |
| G24 Export parity | Partial | SourceTemplate结构存在；ordinary/LibraryEmbed/Native仍不等价 |
| G25 Codec robustness | Fail | golden/property/fuzz/malformed/oversize/slowloris/fragment corpus |
| G26 Product integration | Fail | 普通App/Server/Editor/Hub不依赖test-only constructor |
| G27 Cross-platform | Fail | Windows/Linux/macOS及目标console/mobile transport互操作 |
| G28 Scale | Fail | 10k connections、100k objects和bounded memory/tail latency |
| G29 Fault recovery | Fail | DNS/TLS/reset/FD/port/disk/task panic/crash/path change矩阵 |
| G30 Soak/shutdown | Fail | 24h wrap/consumer stall/World travel/unload零orphan/leak |
| G31 Competitive benchmark | Fail | 同功能/安全/质量与Unreal/Godot可复现CPU/RSS/bandwidth/p99 |
| G32 Truthful maturity | Fail | 以上artifact驱动Beta/Partial升级，UI和manifest不再超前承诺 |

## 15. 禁止的临时修补

- 禁止再新增一个manager/runtime来“接通”某feature。
- 禁止给同步API外包一层async函数但内部继续`recv_timeout`/`block_on`。
- 禁止用更大的channel、Vec或timeout掩盖无界queue与队头阻塞。
- 禁止只检查pin字符串、关闭cert验证或把development trust带入Shipping。
- 禁止只创建5个空资源或6个返回`Ok(())`的operation factory。
- 禁止把固定Lobby/Matchmaking数字改成随机数、socket计数或计时器后继续称Simulate。
- 禁止用自由String、运行时排序index或component名字推断稳定wire ID/插值类型。
- 禁止把manager直调、单机loopback、DTO存在、descriptor注册或历史output record称为多人产品测试。
- 禁止把ignored microbenchmark的局部容器结果外推为优于Unreal。
- 禁止在G01-G32未通过时将Net或任一feature提升为Stable/Complete/Enabled-by-default。

## 16. 本轮完成定义

本review单元已完成：18个Runtime Net合同、Net包186个当前文件和33个产品consumer逐文件纳入审查；调用链从App/catalog/export追到root/六feature、I/O、安全、session/RPC/replication/RUDP/content、Editor/dist/Workbench，并由33个Unreal/Godot/Fyrox/Bevy/Unity Graphics参考文件校准。差距、目标owner、hard cut、里程碑和32项资格门已落账。

Network实现仍为`pending`。只有M0-M9按依赖实施，并通过真实App/Server/Editor/export、secure双进程session、World RPC/replication/prediction、signed content、cross-platform、fuzz/fault/scale/soak和竞争benchmark，才能把本篇标记implementation complete。实施前必须重取237个Zircon selected文件fingerprint并复核working-tree overlap。
