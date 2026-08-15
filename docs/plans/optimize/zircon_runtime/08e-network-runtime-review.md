---
related_code:
  - zircon_runtime/src/core/framework/net
  - zircon_plugins/net/plugin.toml
  - zircon_plugins/net/runtime/src/package.rs
  - zircon_plugins/net/runtime/src/plugin.rs
  - zircon_plugins/net/runtime/src/runtime_state.rs
  - zircon_plugins/net/runtime/src/runtime_system.rs
  - zircon_plugins/net/runtime/src/service_types.rs
  - zircon_plugins/net/runtime/src/service_types/diagnostics.rs
  - zircon_plugins/net/runtime/src/service_types/http_routes.rs
  - zircon_plugins/net/runtime/src/worker/net_worker.rs
  - zircon_plugins/net/runtime/src/worker/transport_runtime.rs
  - zircon_plugins/net/runtime/src/worker/transport_runtime/dispatch.rs
  - zircon_plugins/net/runtime/src/transport/tls.rs
  - zircon_plugins/net/features/http/runtime/src/backend/client.rs
  - zircon_plugins/net/features/http/runtime/src/backend/server.rs
  - zircon_plugins/net/features/http/runtime/src/feature.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/connection.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/reader.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/security.rs
  - zircon_plugins/net/features/websocket/runtime/src/feature.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/packet.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/dispatch.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/handshake.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/schedule.rs
  - zircon_plugins/net/features/replication/runtime/src/manager/snapshot.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/http_fetch.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager/state.rs
  - zircon_plugins/net/editor/src/authoring.rs
  - zircon_plugins/net/editor/src/plugin.rs
  - zircon_plugins/net/dist/src/lib.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_plugins/07-net.md
  - docs/plans/zircon_plugins/07/2026-07-09-net-output-records.md
  - docs/plans/zircon_plugins/01/2026-08-01-net-main-system-set-output-records.md
  - docs/plans/performance/01/2026-07-30-net-runtime-http-websocket-static-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetDriver.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetConnection.h
  - dev/UnrealEngine/Engine/Source/Runtime/Net/Iris/Public/Iris/ReplicationSystem/ReplicationSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Net/Iris/Public/Iris/ReplicationState/ReplicationStateDescriptor.h
  - dev/godot/modules/multiplayer/scene_multiplayer.h
  - dev/godot/modules/multiplayer/scene_replication_config.h
  - dev/godot/modules/multiplayer/scene_replication_interface.h
  - dev/godot/modules/multiplayer/scene_rpc_interface.h
  - dev/godot/modules/enet/enet_connection.h
  - dev/godot/modules/enet/enet_multiplayer_peer.h
  - dev/godot/modules/websocket/websocket_peer.h
  - dev/godot/modules/websocket/packet_buffer.h
  - dev/bevy/crates/bevy_remote/src/lib.rs
  - dev/bevy/crates/bevy_remote/src/http.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 08E · Network Runtime 工程化差距

## 1. 结论

Zircon Network 不是空目录。当前代码已经有中立的 socket/HTTP/WebSocket/session/RPC/replication/reliable-download DTO，TCP/UDP loopback 行为，容量为 1024 的 worker command/event channel，typed connection lifecycle event，HTTP route handler、TLS certificate pin、WebSocket handshake policy，RPC direction/schema/quota，replication delta/interest/budget/interpolation，reliable UDP fragment/ack/resend，以及内容下载 range/hash/mirror/resume bitmap。插件 package 也声明了 server/client/editor target、六个 optional feature、`net.poll_ingress`/`net.flush_egress` system anchor、Editor authoring descriptor和native dist entry。这些局部合同和测试应当保留，不能在重构时退回一个直接在游戏线程调用 `std::net` 的简易网络模块。

但这些组成部分目前没有形成一条能运行游戏多人会话的产品闭环。根模块按 lazy service 创建一个 client-mode `DefaultNetManager`；HTTP 和 WebSocket feature 各自再创建一套带 backend 的私有 `DefaultNetManager`，却没有扩展 canonical `NetManager`。Content Download 生产构造解析的正是 canonical manager，因此即使 feature dependency 同时启用 HTTP，下载仍可能得到 `ProtocolUnavailable`；只有测试通过手工注入 `http_runtime_manager()` 才能工作。RPC、Replication、Reliable UDP 又各自注册与 `NetManager` 无数据连接的内存 manager，全仓 production caller 搜索没有发现任何 scene system、App、Editor或gameplay系统消费它们。feature dependency现在只是启动顺序声明，不是行为接线。

核心 IO 路径也没有实现历史计划描述的异步 NetWorker。公开 `NetManager` 全是同步方法；每个 TCP/UDP 操作 `try_send` 到单一串行 worker 后在调用线程 `recv_timeout(2s)`。连接表/监听表/套接字表的 mutex 多处跨这段等待持有。worker 自己又在专用 OS thread 内创建 multi-thread Tokio runtime并逐命令 `block_on`；manager另建第二个 multi-thread runtime给HTTP/WebSocket。一个慢 TCP connect会阻塞全部 socket命令；调用方两秒超时不会取消worker中的操作，晚到成功可留下只存在于worker、不存在于manager map的孤儿连接。shutdown也要向同一满载队列 `try_send`，失败后的Drop可能无限join。这里没有可证明的deadline、cancel、generation或终态合同。

`net.poll_ingress` 目前只搬运生命周期事件，不收包；TCP/UDP payload仍要求业务同步poll。系统先调用 `diagnostics()`，后者用 `usize::MAX` 将worker ingress全部搬到无界主事件队列，再只 `drain_events(256)`，因此256预算没有限制本帧搬运、锁占用或内存增长。worker ingress满时 `try_send` 的 lifecycle event被静默丢弃；WebSocket reader另向两个无界 `VecDeque` 写入frame和event。`net.flush_egress`完全为空，诊断frame index永远写0。manifest中的TCP/UDP/WebSocket/RPC/HTTP预算与 `NetConfig` 没有production consumer，runtime mode也永远默认Client。

协议层存在能力真实性和安全问题。WebSocket `certificate_pinning` 只检查“配置中有pin”，实际 `connect_async` 没有安装自定义TLS verifier也没有读取peer certificate，测试甚至只断言错误不是“缺少pin”；这不是证书固定。HTTP每请求重建Hyper/Reqwest/TLS client并全量缓冲response，对任何method和错误立即重试，没有幂等、退避、`Retry-After`、body byte limit或取消。HTTP server按连接无上限spawn，无header/connection/deadline/graceful-shutdown策略。更严重的是，`send_http_request_impl` 会把任何“URL没有显式端口”的请求仅按path匹配本地route；`https://example.com/health`可能被本地 `/health` handler静默截获。WebSocket入站无entry/byte/age上限，关闭network connection只从map删除并改state，不发送close frame也不终止reader/writer task。

Reliable UDP有wire header测试，却不是一个网络transport。逻辑packet用 `u64 sequence`、String channel和 `u16 fragment_count`，wire header用 `u16 sequence`、`u8 channel`和 `u8 fragment_count`；除ack辅助外没有完整转换和socket发送链。配置MTU只切payload，没有扣header；远端可让每个sequence预分配65,535个fragment slot，assembly、ordered gap和outbound queue均无byte/age/peer上限。ACK按低16位全扫所有outbound，在wrap时可能确认多个generation；没有per-peer状态、拥塞控制、pacing、RTT驱动RTO、anti-amplification或加密。

RPC/session和replication同样是可单元测试的算法模型，不是game network。handshake token解码后被丢弃，challenge response只需等于manifest中的静态nonce；caller role、source session和control message都由调用者直接传入，没有从认证connection推导。同步handler无法被timeout抢占，channel queue无界，也没有wire codec/transport dispatch。Replication不会扫描World change tick，不创建/销毁实体，不发包也不维护per-connection baseline/ack；authority、field type、delta flag和replication strategy多数只存DTO。每个session/tick仍在全局mutex内深clone并排序全部snapshot，byte budget只计field payload。所谓Transform interpolation只是对名字包含`transform`的任意字段取前4字节作f32，以receive time和固定100ms延迟插值。

Editor表面也没有产品实现。它注册了listener/route/replication schema命令、两个view、三个inspector、asset toolkit和graph palette，但引用的 `authoring.zui`、`listener_config.zui`、`route_config.zui`、`replication_schema.zui`、default TOML均不存在；六个network operation没有handler，唯一测试只断言descriptor被注册。Network Diagnostics没有实时connection/session/channel/packet/replication producer和可用UI。dist entry又明确stateless、无command/event manifest、无invoke/save/restore/unload，仅证明native descriptor可加载。

本轮登记20项P1、5项P2，没有新增P0。P1先建立唯一可组合runtime owner、per-world/session生命周期、真正异步且可取消的IO、bounded queues、安全协议、transport-connected session/RPC/replication/download和可用Editor。大规模replication graph、跨服务器迁移、深度rollback/lag compensation、NAT/relay/voice与live protocol rollout进入P2。完成这些重构及规模验收前，当前“TCP/UDP/HTTP/WebSocket/RPC/Replication/Reliable UDP/Content Download complete”的历史milestone只能说明局部API或测试存在，不能作为工程级多人网络完成声明，更不能支持性能或表现优于Unreal的结论。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 范围 | 文件 | 行数 | `#[test]` | 证据等级 |
|---|---:|---:|---:|---|
| `core/framework/net` | 18 Rust | 2,107 | 6 | E3：endpoint、transport、security、HTTP/WS、session/RPC、sync/reliable/download与manager contract |
| net core runtime | 49 Rust | 4,994 | 35 | E3：module/service、worker、TCP/UDP、HTTP/WS bridge、system、shutdown与diagnostics |
| HTTP feature | 15 Rust | 1,101 | 11 | E3：Hyper/Reqwest client、route server、安全策略与feature registration |
| WebSocket feature | 19 Rust | 1,185 | 8 | E3：client/listener、handshake、reader/writer、frame与安全策略 |
| RPC feature | 20 Rust | 2,142 | 27 | E3：session/handshake、registry/handler、quota、queue/channel与timeout |
| Replication feature | 24 Rust | 1,289 | 9 | E3：descriptor table、delta、interest、schedule、budget、lifecycle与interpolation |
| Reliable UDP feature | 22 Rust | 1,395 | 12 | E3：wire header、fragment/reassembly、ACK/resend、ordered delivery与recovery |
| Content Download feature | 20 Rust | 1,440 | 15 | E3：manifest、attempt/mirror、HTTP range/hash、partial/resume/progress |
| net editor | 6 Rust | 421 | 1 | E3 registration / E1 product：只有authoring descriptor，无UI文档和operation handler |
| net dist | 1 Rust | 98 | 2 | E3 ABI descriptor / E1 behavior：stateless registration entry |

物理统计以2026-08-15当前工作区为准，共194个Rust文件、16,172行、126个test属性。没有bench/fuzz/example、pcap/golden packet、Loom、sanitizer、loss/latency proxy、真实公网/TLS server、10k connection、长时间soak或跨平台产品证据。现有测试对loopback socket、worker shutdown、HTTP route/TLS pin、WebSocket frame order/handshake、RPC algorithm、snapshot/delta、fragment/resend和mirror retry有局部行为价值；但feature tests几乎都直接构造各自manager，恰好绕过了production service composition和scene/App调用链。

产品调用搜索覆盖 `zircon_app`、`zircon_editor`、`zircon_runtime`和全部 `zircon_plugins`。除feature自己的factory/API/tests外，没有发现 `NetRpcRuntimeManager`、`NetReplicationRuntimeManager`、`NetReliableUdpRuntimeManager` 的production consumer；Content Download虽然有production factory，却没有下载产品调用者，且连接到不带HTTP backend的canonical manager。HTTP/WebSocket private manager也只被其factory/tests和Content Download测试使用。`NetConfig`与六个plugin option key没有进入runtime构造或poll budget。

`zircon_plugins/net/runtime/src/plugin.rs`和`runtime_system.rs`当前包含其他Session未提交修改。本报告按current source承认新的`net.main` system set和diagnostics接线，但实现前必须重取fingerprint、复核overlap diff和现有failure/output record，故标记 `source_recheck_required`。

### 2.2 参考边界

- Unreal `UNetDriver`显式拥有World、连接集合、地址到connection路由、packet handler、control/channel、timeout/netspeed、network object list、replication driver和world reset/seamless travel。`UNetConnection`拥有每连接channel、package map、queued bits、owner/view target、RPC DoS检测和统计。Iris再把connection add/remove、dirty/poll、filter、priority、condition、stable replication descriptor、serializer和change mask拆成独立owner。Zircon不需要复制UObject类层次，但必须吸收“world-scoped driver + connection/session identity + protocol/channel + metadata-first replication + resource/DoS budget”的边界。
- Godot `SceneMultiplayer`把MultiplayerPeer、pending/authenticated peers、auth timeout、cache、RPC和replicator接在同一scene owner；Replication有spawn/despawn/sync/delta、per-peer visibility、authority检查、network process和packet MTU；RPC配置绑定mode、transfer mode、channel和object visibility。ENet backend提供peer/channel、in/out bandwidth、throttle、timeout、DTLS和refuse-new-connections。它证明即使较小的server API也不能把reliability、RPC和replication保留为互不连通的manager。
- Godot WebSocket同时限制inbound buffer bytes、outbound buffer bytes和queued packet count，维护heartbeat、close code/reason与明确clear路径。它直接反证Zircon只有64帧outbound、无界inbound/event和仅改state的close合同。
- Bevy主仓这里只有Bevy Remote HTTP JSON-RPC，用于远程检查/修改ECS，不是first-party游戏多人网络。可借鉴的是复用 `IoTaskPool`、bounded response channel和App system接线；不能用它为Zircon缺少session、replication、reliability或安全闭环辩护。
- Fyrox当前参考树没有可对等审查的first-party multiplayer/network stack，因此本篇不把“Fyrox没有某功能”作为设计依据。Unity `dev/Graphics`是渲染/RHI参考树，不是network authority；只在后续networked render streaming确有需求时定义跨域接口，不用Graphics代码推导传输协议。

### 2.3 明确未做

- 没有修改production code，没有运行Cargo、App、Editor、dedicated server、native dist、真实socket/TLS、network emulator、fuzz、sanitizer、soak或规模性能测试。本篇是current-source静态审查和重构计划，不是实现完成证明。
- 没有要求Zircon复制Unreal的每个legacy类或同时支持所有OnlineSubsystem。P1依据是正确性、安全、owner/lifecycle、产品闭环和可规模化复杂度；跨区域/平台高级服务进入P2。
- 没有否定现有typed DTO、worker shutdown测试、HTTP certificate pin、WebSocket policy、RPC direction/quota、replication budget和content hash。它们是可迁移基础，但必须接入同一per-world runtime generation与真实transport，不能继续用direct-manager unit test替代产品证明。

## 3. 当前闭环与必须保留的能力

### 3.1 framework DTO与feature packaging提供了可收敛的公共词汇

`NetEndpoint`/id/event/diagnostics、HTTP/WebSocket descriptor、session control、RPC direction/schema/report、sync descriptor/delta/budget、reliable packet/report和download manifest/progress已经给后续架构提供了命名基础。六个optional feature也有独立capability、target mode和crate边界。重构应收紧其identity/version/error/lifetime并生成prepared wire/runtime artifact，而不是在App、脚本和Editor各发明第二套网络DTO。

### 3.2 TCP/UDP worker与HTTP/WS backend已经有真实loopback路径

TCP/UDP不是固定假返回：worker确实bind/listen/connect/accept/send/read并产生lifecycle event。HTTP可运行本地route和TLS client pin，WebSocket可执行握手、subprotocol/header/path policy与network reader/writer。后续应保留这些第三方库和loopback行为测试，将其迁入唯一IO supervisor、ticket/event API与bounded connection owner，不必另写简易协议库。

### 3.3 typed policy/report比隐式布尔值更适合工程化扩展

RPC dispatch status、replication schedule report、reliable recovery report、download progress和plugin capability status已经能承载拒绝与诊断。目标架构应继续使用typed terminal result，补上session/world/generation、deadline/cancel、drop/backpressure、wire/schema version与security cause；不要把失败重新压成 `Io(String)`、`Option`或日志。

## 4. P1 差距清单

### P1-1：optional feature创建私有manager而不是扩展canonical runtime，production capability并未真正组合

core module lazy构造client-mode `DefaultNetManager`并包装成canonical `dyn NetManager`。HTTP `NetHttpManager`和WebSocket `NetWebSocketManager`的factory忽略所声明的dependency，分别 `DefaultNetManager::default().with_*_backend(...)`，形成各自runtime、worker、socket/event table；它们既不替换canonical service也不把backend注册到它。RPC/Replication/Reliable UDP factory同样忽略core dependency并创建独立内存state。Content Download是唯一解析canonical handle的feature，因此与HTTP的required dependency在运行时不相交。

目标只允许一个 `NetRuntimeSupervisor` service owner。optional feature通过activation transaction向该owner注册backend/protocol/replication codec，并取得generation-scoped lease；注册失败要回滚，unload先停止admission、等待lease和task后移除。HTTP、WS、RPC、Replication、Reliable UDP、Download manager变成同一owner的typed facade或world extension，不能拥有第二个worker/runtime/id space/event queue。增加产品composition test：用真实plugin catalog启用feature后，canonical handle必须看到backend且下载/RPC/replication能通过该handle完成loopback。

### P1-2：manifest option、NetConfig和runtime mode是false surface，dedicated/listen server无法由产品配置驱动

`NetConfig { enabled, runtime_mode, tcp_poll_budget_bytes, udp_poll_budget_packets }`只被导出/测试，module factory始终 `DefaultNetManager::default()`。`net.runtime_mode`、TCP/UDP poll budget、HTTP timeout、WebSocket message budget和RPC rate limit只注册在manifest；运行代码仍用硬编码2s command timeout、256 event budget、30s HTTP、64 frame queue、256 RPC queue等。`enabled=false`也没有阻止服务或system启动。

目标建立versioned `NetRuntimeConfigSnapshot`和validation/apply transaction，明确project/default/CLI/server profile/Editor PIE override优先级。构造owner时冻结需要restart的IO/executor/security参数；可热改预算通过generation publish并由所有producer/consumer读取。DedicatedServer、ListenServer、Client、Editor preview必须有不同admission/security/default endpoint策略和产品测试。无consumer的option必须删除，不能保留“配置看似存在但不生效”的UI。

### P1-3：网络状态不是per-world/per-session，singleton id和队列可跨PIE、world replacement与多实例污染

manager只有process-local AtomicU64和全局HashMap；socket、listener、connection、route、event、RPC session、replication object/interest、reliable queue、download state都不含WorldId、play session、replacement epoch或owner generation。Core下多个Level/PIE preview共享同一service，entity/object/session数值可碰撞；world替换没有停止admission、关闭连接、清理replication baseline或隔离晚到事件。

目标建立 `NetWorldKey { app_session, world, replacement_epoch }`、`NetDriverId`、`NetConnectionHandle { driver, slot, generation }`和 `NetSessionHandle`。一个Core可有game/demo/beacon/tool等多个driver，但每个driver显式绑定world或非world purpose。world unload/travel先进入Quiescing，取消/隔离operation，发送protocol close，retire session/replication state，再发布新generation。所有event/result带owner key；stale result返回typed `StaleGeneration`，不得进入新World。

### P1-4：公开API同步等待单一串行worker，mutex跨网络等待，主线程stall和head-of-line blocking是默认语义

每个TCP/UDP操作向容量1024的sync_channel `try_send`，随后调用线程最多阻塞2s。service层在bind/send/poll/connect等路径持有对应registry mutex直到reply。worker逐命令处理并在自己的multi-thread runtime上 `block_on`；一个connect、writable wait或DNS/OS timeout会阻塞所有连接的send/poll/close。HTTP/WS直接在调用线程对另一runtime `block_on`。`NetManager: Send + Sync`掩盖了方法会等待网络和锁的事实。

目标硬切为non-blocking admission：`bind/connect/listen/request/send/close`返回ticket或立即queue result，completion通过bounded typed stream发布；hot path send支持owned/shared buffer和partial/backpressure result。IO supervisor以per-connection async task或公平reactor驱动，不让一个peer阻塞全局。scene/main线程只在First/Last阶段按entry+byte+time budget drain/publish，绝不 `recv_timeout`/`block_on`。registry锁只保护短暂slot mutation，网络等待和user callback全部在锁外。

### P1-5：两个Tokio runtime加专用thread重复执行权，timeout不取消，panic/shutdown无法保证终态

每个manager在 `runtime_state`创建一个默认multi-thread runtime，worker thread内部再建一个默认multi-thread runtime，外加worker OS thread；启用HTTP/WS私有manager会继续倍增。构造用 `expect`，lazy service resolution可panic。调用方2s reply timeout不携带operation id/cancel token，worker晚到成功会创建orphan handle；send timeout也可能已实际发送。worker ingress满时event丢失。shutdown通过满载egress `try_send`，只统计当前map长度，没有task cancellation、protocol drain、deadline escalation或outer runtime report。

目标由App/Core统一提供 `NetIoExecutor`或明确配置的唯一network executor，线程数、affinity、timer和blocking pool受全局资源预算约束。每个operation有ticket、deadline、cancel state和exactly-one terminal result；timeout必须取消或标记late completion不可发布。supervisor shutdown分StopAdmission、Cancel/Drain、CloseTransport、Join、LeakReport阶段，并有总deadline和强制abort策略。构造/线程失败返回typed activation error，不在factory中panic。验证normal、queue full、runtime build failure、worker panic、stuck connect、drop last handle与process exit。

### P1-6：ingress预算被diagnostics绕过，事件/帧队列无界且会静默丢生命周期；flush系统为空

worker ingress虽有1024 entries，却在 `push_event` 忽略 `try_send`失败。manager events是无界VecDeque；`diagnostics()`每帧先 `poll_worker_ingress(usize::MAX)`，再scene system只drain 256。WebSocket reader每frame同时向per-connection无界inbound和全局event queue写入。single-frame ECS `NetEvent`没有byte/age/peer budget。`net.flush_egress`不做任何事，frame index固定0；UDP/TCP data根本不进入ingress，仍由外部同步poll。

目标为所有边界定义entry、bytes、oldest age、per-peer share、priority和drop/close policy。lifecycle/control不能静默drop；payload可按协议背压、丢旧或断开，但必须产生聚合drop telemetry。diagnostics读取atomic snapshot，不能搬运事件。`poll_ingress`按总时间/entry/byte公平drain connection ready/data/control并映射到world event；`flush_egress`收集本帧RPC/replication/transport command批次并一次提交。frame index取真实Core frame，队列watermark/drop/latency按driver/transport暴露。

### P1-7：基础transport缺少endpoint解析、socket policy、消息边界和主动IO模型

`NetEndpoint::to_socket_addr`只用字符串parse literal，不做DNS/Happy Eyeballs；IPv6 host格式没有方括号。TCP没有connect deadline、backlog、NoDelay、keepalive、buffer、DSCP、reuse、dual-stack或half-close policy。`send_tcp`一次 `try_write` 可部分写，API只返回usize且没有buffered message owner；`poll_tcp(max_bytes)`返回任意stream片段，没有frame codec、max message或EOF/error detail。UDP每poll分配65,535字节buffer并逐包copy；没有connected UDP、batch recv/send、source admission或truncation诊断。worker只响应显式poll，没有readiness-driven receive。

目标将socket transport限制在明确low-level层：async resolver/cache、IPv4/IPv6 candidate policy、typed socket options、connect/listen deadlines、read/write half lifecycle和platform capability report。上层protocol必须使用versioned length/packet framing、max frame、checksum/AEAD和shared buffer owner，不能把TCP stream chunk当消息。UDP用可复用buffer pool与批量API，报告truncated/oversize/source/drop。所有transport通过readiness主动产生bounded ingress，业务不在scene tick逐connection同步poll。

### P1-8：handle没有generation，超时/关闭/复用和状态迁移不足以保证exactly-once生命周期

所有ID只是裸u64；没有driver、slot generation、peer identity或nonce。Manager map和worker map维护两份connection state，失败/closed TCP entry不会自动remove；remote EOF可重复发 `ConnectionClosed`。WebSocket backend task可能在manager插入/push Connecting/Open前先发布frame/close，event顺序不稳定。close network WebSocket仅删除manager entry并 `set_state(Closed)`，未通知writer/reader或等待socket关闭。Error enum把timeout、backpressure、cancel、closed、stale、protocol/version、remote reset等压成 `Io(String)`。

目标以generational slot和operation sequence定义状态机：Resolving、Connecting、Authenticating、Open、Draining、Closing、Closed/Failed，每个迁移只发布一次并带cause。transport owner是状态唯一写者，manager只读immutable snapshot；close返回ticket并完成protocol close、task cancel、map retire。typed error区分invalid config、DNS、timeout phase、queue full、cancelled、stale、security、protocol、remote close和resource exhausted。模型测试覆盖late completion、double close、remote EOF、task failure、id wrap和world replacement。

### P1-9：TLS/安全策略没有形成端到端信任边界，WebSocket certificate pinning是虚假能力

HTTP pinning会读取peer leaf certificate并与配置hash比较，但policy默认development、content download强制development；server side HTTP/WS不支持TLS identity。`rustls_client_config`/server helper除测试外未进入transport。WebSocket security只验证URL scheme与“是否配置pin”，`connect_async`使用默认connector且不检查peer certificate，因此错误pin也不会由Zircon拒绝。host解析用手写split，对userinfo、IPv6、IDNA和端口不可靠。没有mTLS、certificate rotation/generation、secret store、key zeroization、ALPN/cipher policy、revocation策略或shipping insecure guard。

目标统一 `NetSecurityProfile`，用结构化URL/parser和平台/engine trust store；HTTP、WS和游戏transport共享可审计TLS/DTLS/AEAD owner。pin明确SPKI/leaf语义、algorithm/version、hostname和rotation set，验证在TLS handshake内完成；server支持identity lease、hot rotation和client-auth policy。shipping build禁止development policy和明文公网endpoint，loopback exception显式标记。增加wrong pin/hostname/expired/untrusted/root rotation、WSS server、downgrade、malformed URL与secret lifecycle测试。

### P1-10：HTTP client/server是逐请求临时实现，连接复用、流式body、取消、限额和幂等重试均缺失

plain HTTP每次创建Hyper client，HTTPS每次build Reqwest/TLS client；pool和DNS/TLS session随请求丢弃。request/response body都是Vec全量clone/collect，无compressed/decompressed byte上限、stream/sink、upload backpressure或cancellation。retry对GET/POST/PUT/PATCH/Delete和任何error同样立即重试，没有idempotency key、backoff/jitter、Retry-After、overall deadline或attempt telemetry。server无限accept/spawn，route在HashMap values线性搜索并复制response；handler同步运行在async task，可阻塞runtime，route mutex poison会panic。没有HTTP/2/3、TLS listener、header/URI/body/concurrency/idle limits或graceful shutdown。

目标建立generation-owned client pool keyed byproxy/security/DNS/config，支持HTTP/1.1与HTTP/2基线、streaming body、bounded decompression、overall/phase deadline、cancel和typed attempt。retry只按method/idempotency/status/policy执行指数退避。server有accept/connection/request/body/header/time/handler budget、indexed router、async handler dispatch与graceful drain。修复本地route：只有显式loopback/in-process endpoint或专用scheme才能short-circuit，绝不能按path截获任意无端口URL；route endpoint字段必须生效或删除。

### P1-11：WebSocket缺少真实pin/WSS server、入站背压、连接取消和完整close/heartbeat合同

network writer只有64个frame entry但无byte/age limit，reader inbound和event无界；tungstenite默认message/frame上限没有投影到descriptor或telemetry。每个frame被转owned DTO后clone一次再排队，并产生全局event mutex写。accept每次用1ms timeout同步block_on，握手没有总header/body/concurrency deadline。listener只有plain TCP，无法WSS。Ping/Pong、heartbeat、idle timeout、fragment/compression、close handshake、peer close code和task join没有runtime owner。mutex poison使用expect导致task panic。

目标让WebSocket成为同一IO supervisor的protocol connection，WSS复用统一security profile。descriptor声明max frame/message、in/out bytes、entries、age、compression、subprotocol、heartbeat/idle和close deadline。reader通过bounded byte queue背压或按policy关闭1009/1013，writer completion与drop可见；control frame和close state由单owner处理。listener accept/handshake有per-IP/global quota。测试覆盖consumer stall、large/fragmented message、ping timeout、simultaneous close、writer failure、wrong pin与10k idle connections。

### P1-12：Reliable UDP与真实UDP/connection完全断开，wire/logical模型冲突且资源/拥塞安全不足

feature manager没有NetManager/socket/endpoint/peer字段。logical `ReliableDatagramPacket`与 `ReliableUdpWirePacket` 的sequence/channel/fragment宽度不一致且无完整encode/decode/admission。MTU按payload切分导致实际datagram超限；u16 wire ACK低位匹配u64 pending sequence，wrap时generation歧义。outbound、resend、inbound assembly和ordered gap均全局无界；远端fragment_count可预分配65,535 slot且无age/byte/source限制。ACK/resend反复全扫/clone，RTO固定，记录RTT不影响调度，没有congestion/pacing、loss window、bandwidth fairness、anti-amplification、cookie或encryption。

目标选择成熟可靠datagram库/协议作为core transport，而不是继续扩展孤立算法；若保留自有协议，先写规范与互操作fixture。每connection/channel拥有wrap-safe sequence window、selective ACK、fragment byte/entry/age limit、reassembly timeout、ordered gap policy、RTT/RTO、congestion control、pacing和MTU discovery。wire header/version/auth tag纳入budget；source在handshake验证前受cookie/anti-amplification。send/receive直接接NetIo UDP buffer pool和session channel，所有drop/retransmit/RTT/cwnd可观测。fuzz parser并用loss/reorder/duplicate/corrupt/wrap/MTU矩阵验证。

### P1-13：session handshake是调用方驱动的静态状态机，没有绑定transport身份、认证、版本兼容或travel生命周期

`begin_handshake_for_connection`只保存裸connection id；control message由外部直接调用。wire handshake frame只承载version/capability/token，token随后丢弃。Challenge nonce来自静态policy，Login只检查response等于该nonce，任何观察到配置者都能登录；player_id没有认证provider、重复/长度/字符/ban/entitlement检查。NetSpeed由客户端自报并直接成为RPC byte quota。无handshake deadline、attempt/IP rate limit、encryption requirement、protocol/content/build checksum、map/package negotiation、join-in-progress/travel/reconnect/rejoin与close cleanup。

目标让 `NetSessionProtocol`只消费已建立且经过security transport的control channel，由connection owner提供不可伪造peer role/address/identity。握手协商engine/game protocol、schema/content/build feature set、compression/security，并调用可插拔auth/authorization provider；challenge为每connection随机nonce并防重放。server clamp netspeed和all quotas。session有phase deadline、failure code、admission/rate limit和exact cleanup；world travel/reconnect/rejoin以generation/ticket定义。产品测试从socket connect走到authenticated joined world，而不是直接调用state方法。

### P1-14：RPC没有wire/transport dispatch，角色可伪造，schema/handler/queue/timeout不足以作为远程调用系统

RPC manager不持有NetManager或session owner。`dispatch_rpc`/`invoke_rpc`由调用者传 `RpcPeerRole`、direction和session；server-to-client/target路径没有完整target membership/ownership检查。descriptor重复注册会静默覆盖，reflect schema request没有编译成稳定wire serializer/version。schema validator在全局mutex内执行；handler虽锁外但同步跑在caller frame，timeout只能等待返回后丢结果。RPC invocation queue仅限256 entries，不限bytes/age/per-session；channel queue完全无界，也没有encode、send、ack、response correlation transport。session close不统一清quota/netspeed/pending/channel state。

目标在compile/cook期生成稳定RPC table：dense id、direction、reliability/channel、request/response schema hash、max bytes、rate/permission和compat version。receive path从authenticated connection推导role/session/object authority，先做frame length、table id、permission、quota和schema validation，再投递到指定thread/World command queue。handler支持async/cancel/affinity和bounded response；request id有generation、deadline和late-response policy。connection/session close原子清理全部state。Editor/脚本/Rust共用同一descriptor与codec，不允许字符串id+任意Vec作为shipping主路径。

### P1-15：Replication没有接World/change detection/transport，authority、strategy、field contract只是未执行的metadata

Replication manager只接受调用者手工 `publish_snapshot(object, component_type, Vec<SyncFieldValue>)`。全仓没有scene collect/apply system、NetworkIdentity component projection或manager production consumer。descriptor的field list/value_type/delta_compressed不校验输入；authority不限制publish/apply；OnChange/Interval/Once不控制collect，publish即使零changed field也递增sequence并返回delta。apply接受任意field bytes，Transform识别靠字符串contains。

目标在World内建立generational `NetObjectRegistry`和compiled replication descriptor。descriptor从reflection/codegen生成stable type/member id、serializer/quantizer、external/internal layout、change mask、condition、authority、init-only和migration hash。server collect读取ECS change tick/explicit dirty bit，spawn/despawn/ownership/subobject进入同一object lifecycle；client apply在PreUpdate以validated command transaction写World。无authority、未知schema、stale object/generation和invalid bytes必须typed reject并计数，不能修改snapshot cache。

### P1-16：Replication没有per-connection baseline/ACK/relevancy/dormancy，调度深clone全量snapshot且byte budget失真

每session/tick在全局mutex内clone所有snapshot和String，再全排序，之后才做interest/due/budget；发送的是full snapshot，collect产生的delta没有消费链。interest只有字符串group，未设置即全可见；没有spatial/view/owner/team/condition、dependency、dormancy、tear-off或优先级aging。没有per-connection baseline、packet ACK、delta compression、reliable create/destroy、late-join ordering或loss recovery。byte cost只累加field payload，忽略object/component/field names和协议头。session/object清理不完整，last-replication key会增长。

目标采用metadata-first pipeline：dirty object bitset -> connection filter/relevancy -> priority/frequency/dormancy -> byte budget -> serialize selected change mask against acknowledged baseline。连接拥有known objects、baselines、pending reliable lifecycle、dependencies和retirement state；disconnect/driver reset O(owned state)清理。预算使用实际encoded bits并保留priority debt，不能先materialize全量payload。spatial filter/prioritizer可插拔但首个MVP至少完成view distance、owner/always relevant、group condition与late join。规模目标以10k/100k objects和1/100/1k clients建立CPU/bytes/allocation曲线。

### P1-17：没有客户端输入、prediction/reconciliation、时间同步和server simulation ownership闭环

当前network framework没有input command sequence、server tick、clock offset、snapshot interpolation clock、client prediction key、correction/ack或rollback history。Replication固定用receive-time f32 sample和100ms delay；不理解Transform vector/quaternion、velocity、teleport、parent、physics body或animation root motion。没有listen-server local path一致性、deterministic command admission、server authoritative movement、packet loss correction和cheat bounds。

目标先定义server-authoritative simulation合同：tick/time sync、input command schema/sequence/budget、server ack、authoritative state snapshot、client interpolation与bounded prediction/reconciliation buffer。character/physics/animation/network对Transform写入通过单一movement ownership和correction command协作。不可预测组件走snapshot interpolation，可预测组件显式注册save/restore/replay hook；teleport/travel/ownership change清history。确定性、latency/jitter/loss和listen-server parity进入产品测试，不能用任意f32字节插值冒充network motion。

### P1-18：Content Download是同步全内存测试模型，生产HTTP接线、安全manifest、持久cache和原子安装均不成立

production manager解析canonical NetManager，因P1-1无法获得HTTP backend。即使手工注入，`fetch_next_chunk`同步等待最多30s并将response全量读入Vec；resume同时保留state prefix、prefix clone、response和combined Vec，成功后完整chunk仍留在 `partial_chunks`。manifest没有签名/root hash/content version/host allowlist，总字节和offset加法可溢出，未检查overlap/gap/order；mirror URL由字符串拼接chunk id。SecurityPolicy强制development。resume bitmap可仅凭bool把未验证cache标记complete。cancel不取消in-flight，也不阻止后续fetch；没有磁盘temp、fsync、原子rename、quota/eviction、parallel/bandwidth/priority和terminal cleanup。

目标把download建成异步ticket service：签名且versioned manifest、可信host/security profile、streaming range写入bounded temp/cache、增量hash、resume metadata校验、mirror/backoff和cancel。chunk/pack安装使用staging、space reservation、fsync/atomic publish/rollback；cache有content-addressed identity、lease、quota、eviction和corruption repair。App/Hub/Editor消费bounded progress event；terminal state按policy回收memory/task。测试覆盖断电/磁盘满/权限/代理/TLS/恶意manifest/range错误/大于内存资源与跨进程resume。

### P1-19：observability和Editor只有注册外壳，无法诊断真实connection/session/channel/replication或执行authoring操作

runtime diagnostics只有累计bytes、最后HTTP latency、open map count和queued event，且frame index写0；没有per-driver/peer RTT、loss、jitter、cwnd、queue bytes/age/drop、RPC/replication/download、handshake/security/close cause或trace correlation。event catalog只列四类，和实际 `NetEvent`不完整对应。Editor view/ZUI/default asset不存在，operation无handler；graph schema不会编译到runtime descriptor，diagnostics view没有producer。dist behavior为空manifest/stateless/no unload。

目标定义低基数metric schema、connection/session trace id、packet/channel/RPC/replication/download counters和bounded diagnostic snapshot。敏感endpoint/token/payload默认redact，capture需权限与size/time budget。Editor交付真实Network Profiler：driver/connection timeline、bandwidth/loss/queue、RPC、object relevancy/priority、packet content schema视图与emulation controls；listener/route/schema操作具备validation、undo、compile/cook和PIE apply。ZUI/asset/handler缺一即capability unavailable，不得只注册descriptor。dist导出必须包含真实feature behavior与shutdown合同。

### P1-20：测试数量掩盖product/scale/security/cross-platform空白，shipping资格没有可复现证据

126个test属性主要直接构造manager、loopback和descriptor registration；没有通过真实plugin catalog+Core+LevelSystem+App启用组合feature。没有dedicated/listen/client双进程、真实World spawn/RPC/replication、Editor operation、exported dist、Windows/Linux/macOS互操作。没有protocol parser fuzz、malformed/hostile peer、FD/port exhaustion、consumer stall、thread panic、packet loss/reorder/duplicate、TLS rotation、world travel、shutdown soak或benchmark。历史计划多次把DTO/注册/单元测试记作milestone完成，与current product reality冲突。

目标建立分层qualification：codec/property/fuzz；transport loopback和fault injection；双world/double-process session/RPC/replication；plugin composition与hot unload；Editor/PIE/export；跨平台interop；规模/perf/soak/security。每项记录source/config/build hash、机器、网络profile、trace和阈值。任何“complete”必须证明真实入口能到真实effect与terminal cleanup；registration、文件文本、manager直调或mock不能单独作为完成证据。

## 5. P2 能力差距

### P2-1：缺少Replication Graph级大世界分片、interest cell、streaming/travel与海量连接调度

P1完成单World/per-connection replication闭环后，仍需spatial grid/graph、always relevant/owner/team node、frequency bucket、dormancy node、world partition cell、streaming level visibility、seamless travel和priority debt。目标用100k-1M objects、1k connections和分区加载验证增量复杂度，禁止每connection扫描全World。

### P2-2：缺少高级prediction、physics rollback、lag compensation、rewind与network replay/spectator

复杂动作/载具/物理需要input resimulation、deterministic state history、server rewind hit validation、prediction key dependency、visual smoothing和mis-prediction diagnostics。Replay/Demo driver、spectator catch-up、scrub/index、recording compatibility也应复用相同schema和packet/event语义，而不是复制第二套serialization。

### P2-3：缺少跨服务器扩缩容、handoff、seamless migration和多区域容灾

大型在线世界需要session directory、shard/instance identity、connection handoff ticket、state transfer、duplicate suppression、grace window、region failover与version compatibility。此能力必须建立在P1稳定session/object identity和content/schema negotiation之上，不能由singleton manager加几个URL字段实现。

### P2-4：缺少NAT traversal、relay/P2P、平台在线服务、voice与移动网络适配

STUN/TURN/ICE、relay、P2P mesh、console/mobile platform socket、voice channel、IPv6-only/NAT64、network path change和background/resume需要独立feature与security/privacy policy。Godot WebRTC/ENet可作为结构参考，但实现优先选成熟库/平台SDK，不自写ICE或语音codec。

### P2-5：缺少live protocol rollout、兼容窗口、anti-cheat与在线运维控制面

工程级服务需要schema/version rollout、feature flag、canary、backward/forward compatibility、server drain、kick/ban/reason、DDoS/rate policy、anti-cheat attestation hook、privacy/redaction和SLO告警。协议变化必须有golden corpus与跨版本matrix；运维控制面必须权限隔离，不能复用无认证的game RPC/HTTP route。

## 6. 目标架构

```text
Core/App NetIoExecutor
  -> NetRuntimeSupervisor (single process owner, config/security/backend generations)
      -> NetDriverRegistry
          -> NetDriver { purpose, NetWorldKey?, mode, transport listeners }
              -> ConnectionTable { generational handles, protocol tasks, bounded queues }
                  -> Transport (UDP/TCP/WS/HTTP client service)
                  -> Secure Session / Control Channel
                  -> Game Channels { RPC, reliable/unreliable data, replication, input }
              -> NetWorldRuntime
                  -> NetObjectRegistry + compiled replication descriptors
                  -> collect/filter/prioritize/serialize/baseline/apply
                  -> input/prediction/reconciliation/time sync
      -> HttpService / ContentDownloadService (same executor/security/config authority)
      -> DiagnosticsSnapshot / Trace / Editor bridge
```

关键所有权规则：

1. 一个Core只有一个network execution/config/security authority；optional feature只能向它注册extension lease，不能新建私有runtime或id space。
2. 一个driver显式绑定purpose和可选World generation；connection/session/object/ticket都带driver/slot/generation，不能只用裸u64。
3. IO task拥有mutable transport state；World/main thread只交换bounded immutable batch/command，registry mutex不跨await/callback。
4. Session从secure connection推导identity/role；RPC、input和replication不能接受调用方自报caller role。
5. Replication descriptor在compile/cook期固定wire identity、serializer、change mask和compat hash；runtime只处理prepared metadata和bounded payload。
6. 所有队列同时限制entries、bytes、age和per-peer share；所有timeout/cancel/close产生exactly-one terminal result，所有drop可观测。
7. Editor、App、script和dist使用同一runtime status/descriptor/diagnostic，不维护第二份connection或schema真相。

## 7. 必须硬切的旧实现

- 删除HTTP/WebSocket feature factory中创建私有 `DefaultNetManager` 的路径；禁止任何feature manager自建第二套runtime/worker/event/id map。
- 将同步 `NetManager` socket/HTTP/WS方法硬切为ticket/bounded batch合同；不保留内部 `recv_timeout`/`block_on` 的兼容包装供scene/gameplay继续调用。
- 删除 `diagnostics()` 搬运ingress、忽略 `try_send` 和空 `net.flush_egress`；预算必须覆盖producer到World的完整链。
- 删除裸id跨driver/world使用、manager/worker双状态和timeout后继续发布的路径；generation migration完成后不保留数值alias。
- 删除无完整URL/endpoint identity的local HTTP path short-circuit；仅保留显式in-process route API。
- 在真实peer certificate验证接通前移除WebSocket certificate pinning capability/status，不允许继续以“配置中有pin”宣称支持。
- 删除与socket无关的Reliable UDP manager及逻辑/wire双模型；迁移到真实per-connection protocol后一次性切换。
- 删除静态nonce等于challenge response、unused token和调用方传caller role的session/RPC入口。
- 删除手工Vec field snapshot作为shipping replication主路径、字符串contains Transform插值和未执行authority/strategy metadata。
- 删除内存bool resume bitmap即cache hit、development security和全body拼接的Content Download生产路径。
- 缺失ZUI/handler/runtime compiler前隐藏Network authoring/diagnostics capability；不保留可点击但必然无effect的菜单。

## 8. 分阶段重构计划

### M0：能力真实性、唯一owner与配置硬切

- 为core/HTTP/WS/RPC/Replication/Reliable/Download画出现有service graph并增加production composition RED tests。
- 引入唯一 `NetRuntimeSupervisor`、feature extension transaction/lease和versioned config/security snapshot。
- 接通runtime mode/options；删除private manager、无consumer option和false capability。
- 固化generational driver/world/connection/session/ticket identity和typed error taxonomy。

### M1：唯一IO executor、异步operation与shutdown

- 将TCP/UDP/HTTP/WS迁入统一executor和per-connection task/reactor，移除caller `block_on`/`recv_timeout`。
- 实现ticket/deadline/cancel/exact terminal result、bounded command/completion queue和短锁slot registry。
- 完成StopAdmission/Drain/Close/Join/LeakReport shutdown与panic/stuck IO恢复。
- 接通真实 `poll_ingress`/`flush_egress` batch和reactive frame demand/wake。

### M2：基础transport、queue budget与security

- 完成DNS/IPv6/Happy Eyeballs、socket options、TCP framing/half-close、UDP buffer pool/batch/truncation。
- 所有队列落地entries+bytes+age+fairness+drop/close policy与telemetry。
- 统一TLS/DTLS/AEAD security profile、trust store、pin、server identity和shipping guard。
- 建立codec/parser property/fuzz corpus和malicious peer admission tests。

### M3：HTTP、WebSocket与Content Download产品化

- HTTP client pool/stream/cancel/deadline/idempotent retry和bounded server/router/graceful shutdown。
- WebSocket WSS、bounded frame bytes、heartbeat、close/task lifecycle和per-IP quota。
- Content Download signed manifest、stream-to-disk incremental hash、resume/cache/atomic install/cancel。
- 用真实catalog+canonical supervisor跑通App/Hub下载与Editor diagnostics。

### M4：secure session、channel与RPC

- 实现connection-bound handshake、auth/authorization、version/content/schema negotiation和admission quota。
- 建立versioned channel/frame codec、reliable/unreliable routing、request correlation和session cleanup。
- compile RPC dense table/serializer/permission，接通transport receive/send、async handler affinity与deadline。
- 完成双进程join、RPC request/response、disconnect/reconnect/world travel测试。

### M5：World replication闭环

- 接入NetworkIdentity、World change detection、spawn/despawn/ownership/subobject和client apply transaction。
- 编译stable replication descriptor、serializer/quantizer/change mask/condition/schema hash。
- 完成per-connection known object/baseline/ACK、relevancy/priority/frequency/dormancy/byte budget。
- 验证late join、loss recovery、object reuse、world replacement和10k object scale。

### M6：input、prediction、time sync与reconciliation

- 定义server tick/time sync/input command/ack和authoritative snapshot合同。
- 接通character/physics/animation的movement authority、interpolation和bounded prediction history。
- 完成latency/jitter/loss/reorder、teleport/travel、listen server parity与cheat-bound验证。
- 高级physics rollback/lag compensation留接口与P2门，不在MVP伪实现。

### M7：Reliable Datagram与网络仿真

- 选型成熟可靠datagram transport；若自研，先冻结wire spec、interop与security review。
- 实现per-peer/channel window、wrap、ACK、fragment、RTO、congestion/pacing、MTU和anti-amplification。
- 建立loss/reorder/duplicate/corrupt/stall emulator、parser fuzz和长时间wrap/soak。
- RPC/replication/input按policy切换channel，删除孤立manager。

### M8：Editor、diagnostics、dist与高级能力

- 交付真实Network Profiler、schema authoring/compile、listener/route操作、PIE network emulation与redacted capture。
- dist打包实际feature behavior、assets/config/schema和unload；验证Windows/Linux/macOS与dedicated export。
- 实施Replication Graph/partition/replay等已批准P2切片，保持owner/协议边界。

### M9：产品、安全与性能资格

- 跑完整plugin composition、App/Editor/Hub/export、双进程/多进程、world travel和cross-version矩阵。
- 跑fuzz/sanitizer/fault/TLS/security/DoS、24h soak与shutdown leak门。
- 跑1/100/10k connections、1/1k/100k events/packets、0/1KiB/1MiB/256MiB payload、1/2/8/64 cores、0/1/60s consumer stall矩阵。
- 与Unreal/Godot在相同场景、协议、安全和质量设置下记录CPU、p95/p99、bandwidth、allocation/RSS、loss recovery和server capacity；没有可复现实测不得宣称更优。

## 9. 验收门

### 9.1 正确性与生命周期

- 真实plugin catalog启用任意feature组合后只存在一个supervisor/executor/id space，canonical handle可使用已启用能力。
- 两个World/PIE session并行、replace/travel/unload时connection/session/object/event/result不串线，stale generation全部typed reject。
- 每个operation在success/error/timeout/cancel/shutdown/panic下exactly-one terminal result；无orphan socket/task、double event或无限join。
- 双进程从connect、安全握手、auth、join、RPC、spawn/update/despawn、input correction到disconnect完整闭环通过。

### 9.2 安全与故障

- malformed frame/URL/header/schema/fragment、oversize、slowloris、queue flood、auth replay、wrong pin/cert/hostname和untrusted content均在分配/执行前受限并产生typed cause。
- shipping profile不能启用明文公网/development trust；token/key/payload不进入普通日志和未授权capture。
- worker/runtime/handler panic、DNS/TLS failure、remote reset、port/FD exhaustion、disk full、process exit和network path change可恢复或有界失败。
- parser/codec fuzz、sanitizer和resource leak门覆盖所有wire入口。

### 9.3 性能与规模

- 主/scene线程网络API零 `block_on`/`recv_timeout`，稳定帧无全connection poll、无全snapshot clone/sort、无65KiB no-data UDP allocation。
- metric至少记录caller blocked time、IO task/thread、lock hold/wait、queue entries/bytes/age/drop、payload clone bytes、HTTP pool/TLS build、RTT/loss/cwnd、replication selected/encoded/deferred和RSS。
- 100k dirty candidates按bitset/filter增量处理，未选对象不materialize payload；实际encoded bits服从per-connection budget。
- 10k idle/active connection、consumer stall和24h wrap/soak不出现无界内存、句柄增长、event loss无诊断或shutdown超时。

### 9.4 产品与跨平台

- App、dedicated/listen/client、Editor PIE、Hub download和exported dist走同一配置/能力/status，不依赖test-only constructor。
- Network Editor的ZUI、operation handler、schema compile/cook、profiler和emulation均有真实effect与undo/error/cleanup。
- Windows/Linux/macOS至少完成TCP/UDP/TLS/WSS/session/RPC/replication/content互操作；平台差异以capability报告而非silent fallback处理。
- source/config/build/schema hash、机器与network profile随qualification artifact归档，历史测试文本不能替代current-source重验。

## 10. 与既有计划的关系

- `docs/plans/zircon_plugins/07-net.md`仍是实现owner，但其M1-M7历史进度需要按本篇重新开门。NetWorker、TLS、RPC、Replication、Reliable UDP、Content Download和Editor“已落地”仅能作为局部基础，不再代表产品完成。
- `docs/plans/performance/01/2026-07-30-net-runtime-http-websocket-static-review.md`的 `PERF-MVP-575..580` current-source结论仍有效，本篇将其扩展为correctness/security/product architecture并补入private manager composition、WebSocket false pin、local HTTP hijack、session auth、Editor false surface等差距。
- `docs/plans/zircon_plugins/01/2026-08-01-net-main-system-set-output-records.md`只证明 `net.main`/`net.transport` registration合同。它明确不能证明transport behavior，本篇保留该结论；M1需要把空flush和失效budget变成真实执行。
- 实现必须服从 `docs/plans/mvp/index.md` 的baseline gate和coordinator ownership。当前MVP未完成，本篇只做user-authorized review；任何代码切片开始前重新检查source、lease、依赖和现有failure handoff。

## 11. 完成定义

本审查单元的review定义已满足：194/194个目标Rust文件逐个纳入物理覆盖，关键调用链从framework contract追到core runtime、六feature、Editor/dist和production consumer，并以Unreal/Godot/Bevy/Fyrox/Unity参考边界校准；差距、目标owner、硬切、里程碑和验收门已记录。

Network实现仍为 `pending`。只有M0-M9按依赖完成，并通过真实catalog/App/Editor/export、双进程World/RPC/replication、security/fault/fuzz、跨平台、规模与soak门，才能关闭08E。单元测试通过、manager直调、descriptor注册、DTO存在、loopback一次成功或历史output record都不能单独把本篇标记implementation complete。
