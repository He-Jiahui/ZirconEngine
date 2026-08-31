---
title: Runtime Network / Transport / Session / RPC / Replication 当前工作树 authority 与产品边界复审
category: zircon_runtime
report_id: Runtime173
review_date: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/99zo-runtime-network-transport-socket-tls-http-websocket-reliable-udp-session-rpc-replication-prediction-rollback-content-download-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_plugins/10-first-party-network-source-runtime-editor-dist-catalog-transport-rpc-replication-product-integration-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/233-editor-network-current-working-tree-authoring-profiler-multiplayer-boundary-review.md
related_code:
  - zircon_runtime/src/core/framework/net
  - zircon_plugins/net/plugin.toml
  - zircon_plugins/net/runtime/src/module.rs
  - zircon_plugins/net/runtime/src/plugin.rs
  - zircon_plugins/net/runtime/src/runtime_system.rs
  - zircon_plugins/net/runtime/src/config.rs
  - zircon_plugins/net/runtime/src/runtime_state.rs
  - zircon_plugins/net/runtime/src/service_types.rs
  - zircon_plugins/net/runtime/src/worker
  - zircon_plugins/net/runtime/src/transport
  - zircon_plugins/net/features/http/runtime/src/feature.rs
  - zircon_plugins/net/features/websocket/runtime/src/feature.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager
  - zircon_plugins/net/features/replication/runtime/src/manager
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager
  - zircon_plugins/net/features/content_download/runtime/src/manager
  - zircon_plugins/net/dist/src/lib.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
tests:
  - zircon_plugins/net/runtime/src/tests
  - zircon_plugins/net/features/rpc/runtime/src/tests
  - zircon_plugins/net/features/replication/runtime/src/tests
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests
  - zircon_plugins/net/features/content_download/runtime/src/tests
plan_sources:
  - docs/plans/optimize/zircon_runtime/99zo-runtime-network-transport-socket-tls-http-websocket-reliable-udp-session-rpc-replication-prediction-rollback-content-download-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_plugins/10-first-party-network-source-runtime-editor-dist-catalog-transport-rpc-replication-product-integration-review.md
  - docs/plans/performance/01/2026-08-24-plugin-net-replication-current-source-algorithm-performance-review.md
  - docs/plans/optimize/zircon_editor/26-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetDriver.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetConnection.h
  - dev/UnrealEngine/Engine/Source/Runtime/ReplicationGraph
  - dev/UnrealEngine/Engine/Source/Runtime/Iris/ReplicationSystem
  - dev/UnrealEngine/Engine/Source/Runtime/Online/HTTP
  - dev/UnrealEngine/Engine/Source/Runtime/Online/WebSockets
  - dev/UnrealEngine/Engine/Source/Runtime/NetworkPrediction
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices
  - dev/godot/scene/main/multiplayer_peer.h
  - dev/godot/modules/multiplayer/scene_multiplayer.h
  - dev/godot/modules/multiplayer/scene_rpc_interface.h
  - dev/godot/modules/multiplayer/scene_replication_interface.h
  - dev/godot/editor/editor_network_profiler.h
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

# 1. 结论

当前工作树已经有可保留的网络局部底座：`zircon_plugins/net` 共 186 个文件、17,102 行、602,589 bytes、150 个测试属性（14 个 ignored），包含 TCP/UDP worker、HTTP/WebSocket facade、TLS helper、RPC/replication/reliable-UDP/content-download 的 DTO 和算法测试。它证明了协议问题被开始建模，但不等于 Network 已经成为可运行的引擎服务。

本轮最重要的结论是 **authority 仍然分裂**：根 `DefaultNetManager` 承载 worker 和传输状态，而 HTTP/WebSocket/RPC/Replication/Reliable UDP feature factory 又分别创建新的 manager；feature manifest 中的 dependency 只是 DAG 元数据，并没有把根 manager 注入 feature。根 scheduler 只把至多 256 个事件送入 World，`frame_index` 固定传 `0`，Last 阶段的 `run_net_flush_egress` 是空函数。默认 client/server/editor-host 组合也没有形成完整的 Network provider 链，native dist 明确是 stateless metadata shell。因而当前实现不能声称具备 Unreal 的 NetDriver/NetConnection/ReplicationGraph/Iris、Godot 的 MultiplayerPeer/RPC/replication 或可部署的 dedicated-server/client 资格。

本报告新增 32 项 P1、10 项 P2、20 门资格门；没有重复登记旧报告已经拥有的 P0。当前状态为 **15 Fail、5 Partial、0 Pass**。这是 review-only 结论，没有修改生产代码、Cargo、ABI 或测试。

# 2. 当前工作树证据

## 2.1 选集与可重复指纹

| 范围 | files | lines | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---|
| `zircon_plugins/net` 全量（Rust/TOML） | **186** | **17,102** | **602,589** | **150** | **14** | `28336e23346b57dba0c9a41fd09d5ade1dcb56061933a43993a3f2d135d47c53` |
| Runtime framework net（`zircon_runtime/src/core/framework/net`） | **18** | **2,207** | **66,325** | **8** | **1** | current working-tree snapshot |
| App/catalog integration focused set | **5** | **528** | **19,555** | **4** | **0** | current working-tree snapshot |
| Net editor plugin | **7** | **403** | **16,279** | **1** | **0** | current working-tree snapshot |
| Multiplayer Workbench UI assets | **2** | **466** | **29,780** | **0** | **0** | current working-tree snapshot |
| Online-session callback/navigation routes | **2** | **273** | **11,465** | **0** | **0** | current working-tree snapshot |

统计口径：Rust 统计包含生产和测试源码；测试数为 `#[test]`/`#[tokio::test]` 等属性计数；不把 dev reference engine 代码混入 Zircon 指纹。没有运行 Cargo、真实 socket、TLS、PIE、跨进程、fault、soak 或 scale benchmark，因此表格不是性能资格证明。

## 2.2 装配、生命周期与调度

- `zircon_app/Cargo.toml:115-143` 只在 target-client/target-editor-host 暴露 `net-contracts`；target-server 没有 Network contract，target-client 也没有把 `first-party-runtime-plugins` 作为必选 provider。`zircon_app/src/entry/first_party_runtime_plugins.rs:109-131` 在 feature 未启用时返回空 Vec。
- `zircon_plugins/first_party_runtime_catalog/src/lib.rs:38-70` 能解析 `RuntimePluginId::Net`，但只返回 net root registration；`net.http`、`net.websocket`、`net.rpc`、`net.replication`、`net.reliable_udp`、`net.content_download` 没有在普通 catalog 中组成可选 feature 集合。
- `zircon_plugins/first_party_editor_catalog/src/catalog.rs:41-54` 只为 Navigation/Neural 提供 editor provider。即使项目 manifest 写了 `net`，EditorHost 也没有 Net editor registration 的产品入口。
- `zircon_plugins/net/runtime/src/module.rs` 的 root module 确实注册 `NetDriver`、lazy `DefaultNetManager` 和 `dyn NetManager`，这是应保留的唯一根 owner 候选。
- `zircon_plugins/net/runtime/src/runtime_system.rs:30-52` 注册 First poll 和 Last flush；`:54-72` 只 resolve root manager、以固定 256 budget drain event 并发送到当前 `World`；`:61` 以常量 `0` 写 diagnostic frame；`:75-77` 的 flush 没有任何 egress、RPC、replication、reliable-UDP 或 download 工作。
- `zircon_plugins/net/runtime/src/config.rs:4-20` 只定义 `NetConfig` 和 client 默认值。当前生产调用图没有把 `enabled/runtime_mode/tcp_poll_budget_bytes/udp_poll_budget_packets` 应用到 manager、worker、system 或 feature activation；配置因此是声明而非运行时快照。
- `zircon_plugins/net/runtime/src/runtime_state.rs:36-92` 把大量 socket/listener/route/connection/backend/events map 放进一个 `NetRuntimeState`，并在同一对象内建立 Tokio runtime 和 `NetWorker`。`new` 通过 `expect` 创建 runtime/worker，owner scope 仍是进程级 Arc，而不是 World/session/role 级 lease。
- `zircon_plugins/net/runtime/src/dist/lib.rs:15-42` 明确导出 `is_stateless: true`、state schema 0、空 command/event manifest、`invoke_command/save_state/restore_state/unload/on_host_ready: None`。这是 ABI registration shell，不是可保存、可热卸载、可恢复的 Network runtime。

## 2.3 根 manager 与可选 feature 的双 authority

- HTTP feature 在 `zircon_plugins/net/features/http/runtime/src/feature.rs:41-44` 通过 `DefaultNetManager::default().with_http_backend(...)` 创建私有 manager；`:51-64` 的 factory 只接受未使用的 closure 参数，虽然声明依赖 `NetManager`，却没有 resolve/inject 它。
- WebSocket feature 采用同样的私有 `DefaultNetManager` 模式（`features/websocket/runtime/src/feature.rs:42-64`）。RPC feature 在 `features/rpc/runtime/src/feature.rs:41-59` 直接 factory `net_rpc_runtime_manager()`；Replication 和 Reliable UDP 也各自返回独立内存 manager。Content download 是唯一尝试通过 `net_manager_handle(core)` 获取注册 manager 的 feature，但测试和 fallback 仍可注入私有 backend。
- 六个 feature 的 `RuntimePluginFeature::register` 都只做 `registry.register_module(module_descriptor())`，没有一个 feature 注册 `runtime_scene_system` 来消费 World、session tick、transport event 或 egress queue。manifest dependency 因此不能证明运行时共享状态。
- `zircon_plugins/net/runtime/src/service_types.rs` 的 `DefaultNetManager` facade 和 `NetManager` trait 是一个合理的 API 起点，但 trait 仍以同步 `Vec`/body DTO 为中心，没有 async ticket、cancel token、completion receipt、generation-qualified handle 或统一 transport/session capability。

## 2.4 Transport、协议与安全实现

- `zircon_plugins/net/runtime/src/worker/net_worker.rs:17-20,37-44,181-235` 使用双向容量 1024 的 sync channel；每个 command 通过 `try_send` 后以固定 2 秒 `recv_timeout` 等待。超时只返回调用者，不能取消已经在 worker 中运行的 socket 操作；shutdown 也复用相同固定 timeout。
- `zircon_plugins/net/runtime/src/worker/transport_runtime.rs:44-70` 在 worker 内再建一个 multi-thread Tokio runtime，导致 manager runtime 与 worker runtime 双 runtime。UDP poll 路径每次建立 65,535-byte buffer；TCP accept 使用 1 ms poll timeout。没有看到按 transport domain 建立的 persistent buffer pool、batch receive、fairness 或 per-connection budget。
- `zircon_runtime/src/core/framework/net/rpc.rs:37-231` 的 descriptor/id/schema/payload 仍以 String、`Vec<u8>` 和 caller-supplied timeout/priority 建模；`:139` 的注释承认真实 handler execution 还可后续叠加。`sync.rs` 的 field/component 也是 String/raw bytes，默认 budget 可为 0（无限），且 `allows_snapshot_count` 使用 `<` 语义，存在预算边界疑点。
- `features/rpc/runtime/src/manager/handshake.rs:10-70` 有可测试的 ZRPC header/length parser，但 token 只是 Vec bytes；默认 `challenge_nonce` 为固定字符串（同文件后续 policy），`accept_login` 不是 cryptographic proof，principal/role 仍可由 caller DTO 提供。没有 nonce freshness、key rotation、replay window 或 authenticated capability。
- `features/reliable_udp/runtime/src/manager/resend.rs` 有 resend/assembly 试验算法，但 ACK 仅比较 `packet.sequence as u16`，缺 peer/channel/generation namespace；RUDP manager 没有调用根 `NetManager` UDP。没有 RTT/RTO estimator、拥塞/ pacing、MTU discovery、加密和 anti-replay。
- TCP/UDP endpoint 仍通过字符串/`NetEndpoint` 转换到 `SocketAddr`；没有可证明的 DNS resolver、IPv6 policy、Happy Eyeballs、framing/QoS/backpressure。HTTP local-route heuristic 会在 URL 没有显式 port 时直接按 path 捕获，HTTP request 是完整 body DTO，retry/cache/circuit-breaker/streaming 皆未闭合。HTTP server accept/spawn 和 handler budget 也没有 bounded lifecycle。
- TLS helper 能构造 rustls roots 并比较 pin digest，但没有证据表明 hostname verification、custom roots、pin、WSS rotation/mTLS 在所有实际 backend connect path 上强制生效；WebSocket queue/heartbeat/close timeout 也没有统一 session owner。

## 2.5 Session、RPC、Replication 与 content download

- Session control DTO 有 Hello/Challenge/Login/Welcome/NetSpeed/Failure/Join，但 session map 由本地裸 `u64` 递增 id 管理；`apply_transport_events` 没有生产 caller，不能把 transport state 自动提升为 authenticated session。
- RPC manager 用 `Arc<Mutex<State>>` 保存 sessions/descriptors/handlers/quota/windows/heap/pending/channel queues，能够做内存校验和本地 priority queue，但没有 wire codec、connection send/receive owner、World system caller、cancellation receipt 或 timeout preemption。超时 helper 不能终止已执行的 handler。
- Replication manager 能 register component、set interest、publish/schedule/apply snapshot，但没有连接 `World`/Reflection/schema registry/transport/authenticated connection。interest 只有 group filter；schedule 每次 sort 所有 candidate；payload 是 raw bytes。`manager/apply.rs:119-123` 通过 component 名称包含 `transform` 判断插值，之后从 payload 前四字节解释 f32，属于临时 heuristic，不是 typed transform codec。
- Replication 没有 baseline ACK/NACK、dormancy、spatial relevancy、priority debt、resync、snapshot generation 或 deterministic rollback。客户端 prediction/reconciliation/rollback/lag compensation 没有产品 owner。
- Content download 的 manifest/chunk/progress DTO 能校验单块 hash，但 `manager/progress.rs:18-47,50-82` 在内存 `Vec<String>` 中记录 completed/cache chunks，`resume` 仍是内存 HashMap/bitmap；没有 streaming-to-disk、临时文件 fsync、原子 publish、持久 cache、mount/install/repair/rollback。没有 signed manifest、发布 identity、URL policy、request-id namespace 和 verified cache。

# 3. P1 差异与重构要求

| ID | 当前差异（证据） | 必须重构为 | 参考对照 |
|---|---|---|---|
| NET-RT-001 | 默认 target 组合没有完整 net provider | 用 target manifest 选择唯一 Net root + feature closure，并在 client/server/editor-host 中显式 fail-closed | Unreal NetDriver/WorldNetDriver |
| NET-RT-002 | catalog 只返回 root，六个 feature 未收集 | 将 feature DAG 展开为版本化 activation plan，记录 capability、owner、receipt | Godot MultiplayerPeer modules |
| NET-RT-003 | Editor catalog 没有 Net provider | 为 EditorHost 注册 Net editor，并与 runtime capability 同一 manifest generation | Godot editor network profiler |
| NET-RT-004 | feature factory 忽略 dependency，创建私有 manager | factory 只能 resolve root `NetManager` handle；禁止 feature 自建 transport authority | Unreal NetDriver/NetConnection |
| NET-RT-005 | feature 没有 scene system | 为 session/transport/RPC/replication/download 注册明确 stage、World scope 和 owner | Bevy Remote app/runtime boundary |
| NET-RT-006 | `NetConfig` 是未消费 DTO | 生成 immutable activation snapshot，校验 mode/role/budgets/security/backend 并绑定 generation | Unreal net driver config |
| NET-RT-007 | mode/role/security 不在一个快照内 | 建立 Server/Client/Listen/EditorSimulation profile，所有 feature 只读该 profile | Godot peer authority |
| NET-RT-008 | state 是进程级大 Mutex map | 拆成 process transport host、session lease、World replication domain、editor preview domain | Unreal World/NetDriver ownership |
| NET-RT-009 | event catalog 只有 4 个 string schema | 用稳定 event id、typed payload、source connection、sequence、timestamp 和 loss policy | Godot MultiplayerAPI signals |
| NET-RT-010 | ingress 1024 丢弃风险，main queue 无界 | 引入分级 bounded lanes、overflow receipt、backpressure 与 per-domain quotas | Unreal packet handler queues |
| NET-RT-011 | ingress 固定 256、frame=0，flush no-op | scheduler 注入真实 frame/tick，Last 阶段执行 egress/RPC/replication/RUDP flush 并产生 receipt | Bevy fixed/update schedules |
| NET-RT-012 | manager runtime + worker runtime 双 runtime | 统一 async executor/IO reactor，或明确隔离并实现 ownership/shutdown contract | Fyrox Rust networking split |
| NET-RT-013 | 2 秒等待不取消底层 command | 改为 async operation ticket + cancellation + deadline + terminal state；超时必须 quiesce | Unreal async network tasks |
| NET-RT-014 | caller 超时会遗留 worker side effect | command 带 operation generation，late reply 可安全丢弃并发布 orphan diagnostic | Godot peer close lifecycle |
| NET-RT-015 | id 是裸 u64 | 所有 socket/listener/connection/route/session/download 使用 generation-qualified handle 和 stale error | Unreal object/network handles |
| NET-RT-016 | UDP 每次 65,535 分配 | persistent slab/ring buffer、recv batch、packet budget、peer fairness 和 zero-copy boundary | Bevy transport resource discipline |
| NET-RT-017 | TCP 是 byte stream facade | 增加 framed channel、max frame、QoS、priority、backpressure、half-close 和 protocol error | Unreal packet handler chain |
| NET-RT-018 | endpoint 解析与 HTTP local-route heuristic 简化 | 统一 resolver/IPv4+IPv6/Happy Eyeballs/explicit local-vs-remote route policy | Godot ENet/WebSocket peer |
| NET-RT-019 | HTTP full-body/per-request client，server spawn 无界 | 连接池、streaming body、retry/backoff/circuit breaker、bounded accept/handler、cancel receipt | Unreal HTTP retry/build patch |
| NET-RT-020 | TLS/WSS policy 未证明全路径生效 | hostname verification、root/pin/mTLS/rotation 进入 handshake policy，并做 negative tests | Unreal WebSockets/online security |
| NET-RT-021 | WS 只有 queue/frame DTO | session-owned heartbeat、close handshake、bounded queue、backpressure、reconnect/rotation | Godot websocket_peer |
| NET-RT-022 | RUDP 不调用根 UDP，ACK 只有 low16 | RUDP channel adapter 必须绑定 root UDP，wire header 含 peer/channel/full sequence/generation | Unreal reliable channels |
| NET-RT-023 | RUDP 没有 congestion/crypto/anti-replay | 实现 RTT/RTO、loss estimator、cwnd/pacing、MTU、AEAD/key epoch/replay window | Unreal Iris transport/security |
| NET-RT-024 | 固定 challenge，caller 可供 principal/role | authenticated challenge-response、credential provider、capability negotiation、role admission audit | Godot authority/peer auth |
| NET-RT-025 | RPC 是本地 handler/raw bytes | schema registry + stable wire codec + connection transport + async call ticket + cancel/error receipt | Unreal RPC/RPC validation |
| NET-RT-026 | replication 无 World/Reflection/transport owner | World component schema、baseline store、connection scope、interest graph、typed apply transaction | Unreal ReplicationGraph/Iris |
| NET-RT-027 | transform interpolation 是字符串+前四字节 f32 heuristic | typed codec、schema version、clock sync、buffer policy、invalid payload rejection | Godot scene replication |
| NET-RT-028 | prediction/rollback 缺 product owner | deterministic input buffer、server correction、rollback window、replay/lag compensation service | Unreal NetworkPrediction |
| NET-RT-029 | download partial/cache 只在内存 | chunk stream to durable temp, fsync/atomic rename, resumable persistent cache, install/mount receipt | Unreal BuildPatchServices |
| NET-RT-030 | manifest/identity/request/cache 不可验证 | signed manifest、publisher/key epoch、content-addressed chunks、request namespace、verified cache | Unreal pak/build patch trust |
| NET-RT-031 | native dist stateless metadata shell | ABI command/event/state/restore/unload/conformance，source/library/native 必须同一 behavior receipt | Unreal module lifecycle |
| NET-RT-032 | 没有 session/PIE/server-client/World 产品闭环 | 建立 standalone server、client、listen、PIE multi-process 和 World attach/detach acceptance suite | Unreal PIE/net modes; Godot MultiplayerAPI |

# 4. P2 差异

| ID | 差异 | 重构方向 |
|---|---|---|
| NET-RT-033 | HashMap/Vec payload 分配不可预测 | slab、arena、small-vector、quota-aware allocator 与 zero-copy metrics |
| NET-RT-034 | timeout/resend 常量固定 | 自适应 RTT/RTO、loss profile、per-route policy、测试时钟 |
| NET-RT-035 | channel/QoS 只有 DTO | ordered/reliable/unreliable/priority lanes、fair scheduler、starvation proof |
| NET-RT-036 | interest 只有 group | spatial cell/owner/team/tag query 与 incremental interest graph |
| NET-RT-037 | snapshot schedule 每次全量 sort | dirty queue、priority debt、baseline delta、parallel extraction |
| NET-RT-038 | diagnostics 只有 aggregate counters | packet/session/RPC/replication trace、sampling、privacy redaction、profiler export |
| NET-RT-039 | bytes/time/queue 没有统一 quota | per-World/per-connection/per-feature budget ledger 与 admission receipt |
| NET-RT-040 | parser/handshake/codec 缺系统 fuzz | fuzz corpus、malformed packet budget、property tests、cross-version replay |
| NET-RT-041 | cross-platform backend 无资格矩阵 | Windows/Linux/macOS/mobile/WebSocket/TLS capability matrix 与 CI evidence |
| NET-RT-042 | 没有 scale/soak/fault baseline | 1/16/64/256/1K connection、packet loss、reorder、restart、hours-long soak benchmark |

# 5. 资格门

| Gate | 状态 | 必须满足 |
|---|---|---|
| NET-RT-G01 | Fail | client/server/editor-host 的 provider、catalog、feature closure 可从 manifest 重放 |
| NET-RT-G02 | Fail | 所有 feature factory resolve 同一个 root manager，禁止私有 transport authority |
| NET-RT-G03 | Fail | activation snapshot 原子包含 mode/role/security/budget/backend/generation |
| NET-RT-G04 | Fail | process/session/World/editor preview owner 和 shutdown lease 可证明 |
| NET-RT-G05 | Fail |真实 tick/frame ingress 与 egress 都有 bounded receipt，flush 不再是 no-op |
| NET-RT-G06 | Fail | async ticket deadline/cancel/late-reply/orphan 全部有 terminal state |
| NET-RT-G07 | Fail | handle generation、stale reference、exhaustion、cross-domain misuse 有测试 |
| NET-RT-G08 | Partial | rustls/pin helper 存在，但 hostname/WSS/mTLS 全路径和 negative proof 未完成 |
| NET-RT-G09 | Fail | TCP framing、UDP batching、resolver、IPv6、backpressure 有 wire tests |
| NET-RT-G10 | Fail | RUDP wire header、ACK namespace、RTO/congestion、AEAD/replay 与根 UDP 互通 |
| NET-RT-G11 | Fail | RPC schema/wire codec/auth/handler/world caller/cancel receipt 完成 |
| NET-RT-G12 | Fail | replication World/Reflection/baseline/interest/apply/rollback 形成单一数据流 |
| NET-RT-G13 | Fail | prediction/reconciliation/rollback/lag compensation 有 deterministic replay |
| NET-RT-G14 | Fail | download signed manifest、streaming、atomic install、resume/repair/recovery 完成 |
| NET-RT-G15 | Partial | source/library/native registration 可见，但 dist 仍 stateless、无 command/state lifecycle |
| NET-RT-G16 | Fail | Editor/PIE/standalone server/client 与 Runtime173 共用 generation/receipt |
| NET-RT-G17 | Partial | worker shutdown 有 report，但固定 timeout、late side effect 和 unified quiesce 未闭合 |
| NET-RT-G18 | Fail | multi-process、fault、packet loss/reorder/restart/soak 有自动化证据 |
| NET-RT-G19 | Fail | 1K connections、snapshot/RPC bandwidth、allocation/latency tails 达到预算并可回归 |
| NET-RT-G20 | Fail | 不允许 descriptor/static/local-loopback 成功冒充真实 network product success |

汇总：**15 Fail、5 Partial、0 Pass**。Partial 仅表示已有局部可复用证据，不表示可以发布。

# 6. 建议重构顺序

1. **M0 Truth gate**：先让默认 target 显式选择 provider；未支持的 target/feature 必须 fail-closed。把 stateless dist、无 handler 的 RPC、未绑定 UDP 的 RUDP、无 artifact 的 download 标为 unsupported，而不是报告成功。
2. **M1 Single authority**：保留 root `NetManager`，将所有 feature factory 改为 dependency-injected handles；拆分 process IO、session、World replication、editor preview 四种 owner；用 generation handle 和 activation snapshot 取代裸 id/未消费 `NetConfig`。
3. **M2 Async transport/security**：统一 executor/IO domain，加入 ticket/cancel/deadline/quiesce；实现 framing、resolver、persistent UDP buffers、TLS/WSS authenticated policy、RUDP wire/congestion/AEAD，并补 malformed/fault tests。
4. **M3 Session/RPC/replication**：先完成 authenticated session，再做 stable schema/wire codec；将 RPC 和 replication 接入 World/Reflection/transport，完成 baseline/interest/apply、prediction/reconciliation/rollback 的 deterministic replay。
5. **M4 Content delivery**：建立 signed manifest、content-addressed persistent cache、stream-to-temp、fsync/atomic install、repair/rollback/mount receipt。
6. **M5 Editor/profiler/product**：由 Editor233 提供真实 document/operation/session simulator/profiler；PIE、standalone server/client、reopen、fault、scale 必须共用 Runtime receipt。
7. **M6 Qualification**：按 1/16/64/256/1K 连接和 packet loss/reorder/restart/soak 运行跨平台基准，冻结性能和内存预算后再提升 maturity。

# 7. 参考引擎对照结论

- Unreal 是主对照：NetDriver/NetConnection、ReplicationGraph/Iris、NetworkPrediction、HTTP retry、WebSockets、BuildPatchServices 分别拥有 transport、connection、replication、prediction、content delivery 的长期 owner；Zircon 当前把这些能力压缩到 facade/manager/DTO，缺少跨层 authority 和生命周期。
- Godot 的 MultiplayerPeer、SceneMultiplayer、RPC/replication interface、ENet/WebSocket peer 说明 session、authority、RPC、replication transport 必须同一套 peer lifecycle；Zircon 当前 session 的 transport event 没有生产 caller。
- Fyrox 的 Rust net module 可作为 ownership/错误模型的轻量参考；Bevy Remote 的 HTTP/remote protocol 可作为 typed command、auth、bounded response 的参考，但两者都不能替代 Zircon 的 World replication 和 dedicated-server 资格。
- `dev/Graphics/Packages/com.unity.render-pipelines.core/package.json` 只适合包/模块元数据对照，不能证明 Network 功能。

# 8. 验证边界

本轮只做逐文件源码和资源审查，未运行 Cargo、socket/TLS、跨进程、PIE、真实 download/install、Render/Editor、fault、soak、scale 或 benchmark。后续实现每完成一个 M 阶段，都必须重新执行本报告的选集指纹、资格门和负向测试；旧 Runtime140/08e 的历史结论不能被局部单元测试自动标记为 Closed。
