---
title: Runtime Network / Transport / Session / RPC / Replication / Prediction / Rollback / Security / Content / Product 当前工作树复审
category: zircon_runtime
report_id: Runtime217
review_date: 2026-09-01
baseline_head: 9963f8eb72e2d725d2536eb50b393b30387a1ffa
canonical_ledger_owner:
  - docs/plans/optimize/zircon_runtime/99zo-runtime-network-transport-socket-tls-http-websocket-reliable-udp-session-rpc-replication-prediction-rollback-content-download-editor-product-integration-current-source-review.md
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/173-runtime-network-current-working-tree-authority-transport-session-rpc-replication-editor-boundary-review.md
  - docs/plans/optimize/zircon_runtime/99zo-runtime-network-transport-socket-tls-http-websocket-reliable-udp-session-rpc-replication-prediction-rollback-content-download-editor-product-integration-current-source-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/233-editor-network-current-working-tree-authoring-profiler-multiplayer-boundary-review.md
related_cross_domain_reviews:
  - docs/plans/optimize/zircon_runtime/192-runtime-task-execution-job-scheduler-task-graph-worker-domain-scope-cancellation-deadline-shutdown-diagnostics-product-adoption-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/194-runtime-event-message-observer-bus-world-mirror-abi-ingress-delivery-lifecycle-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/195-runtime-failure-contract-error-taxonomy-panic-containment-health-recovery-shutdown-crash-abi-product-integration-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/199-runtime-plugin-profile-catalog-provider-resolution-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/208-runtime-product-build-export-profile-build-plan-platform-host-cross-compilation-package-launch-handoff-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/215-runtime-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-current-working-tree-review.md
  - docs/plans/optimize/zircon_app/09-product-launch-request-cli-config-resolution-provenance-diagnostics-exit-semantics-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/259-editor-plugin-provider-catalog-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/260-editor-extension-contribution-store-toolkit-reload-lifecycle-current-working-tree-review.md
related_code:
  - zircon_runtime/src/core/framework/net
  - zircon_plugins/net
  - zircon_app/src/entry
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog
  - zircon_runtime/src/plugin/package_manifest/builtin_catalog.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetDriver.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetConnection.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/ReplicationGraph/Source/Public/ReplicationGraph.h
  - dev/UnrealEngine/Engine/Source/Runtime/Net/Iris/Public/Iris/ReplicationSystem/ReplicationSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Net/Iris/Public/Iris/ReplicationState/ReplicationStateDescriptor.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/NetworkPrediction/Source/NetworkPrediction/Public/NetworkPredictionBuffer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Online/HTTP/Public/HttpRetrySystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Online/WebSockets/Public/IWebSocket.h
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices/Public/Interfaces/IBuildInstaller.h
  - dev/godot/scene/main/multiplayer_peer.h
  - dev/godot/modules/multiplayer/scene_multiplayer.h
  - dev/godot/modules/multiplayer/scene_rpc_interface.h
  - dev/godot/modules/multiplayer/scene_replication_interface.h
  - dev/godot/modules/enet/enet_multiplayer_peer.h
  - dev/godot/modules/websocket/websocket_peer.h
  - dev/Fyrox/fyrox-core/src/net.rs
  - dev/bevy/crates/bevy_remote/src/lib.rs
  - dev/bevy/crates/bevy_remote/src/http.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
doc_type: current-working-tree-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user
---

# Runtime217 · Network 当前工作树复审

## 1. 结论

当前 Network 仍不是工程级游戏网络系统。它是可复用的中立 DTO、真实本机 TCP/UDP/HTTP/WebSocket 原语、若干局部协议算法和较多 direct-manager/loopback 单元测试的组合；但产品 authority、provider composition、secure session、World RPC/replication、prediction/rollback、可靠传输、持久内容安装、Editor/PIE 和跨进程资格仍未闭合。不能因为类型、descriptor、feature row、静态 Workbench 或 ignored microbenchmark 存在，就把这些表面解释成与 Unreal 同级，更不能宣称性能优于 Unreal。

本轮逐文件复核 18 个 Runtime Net 合同文件和 `zircon_plugins/net` 的 188 个 Rust/TOML 文件，并追踪 11 个 App/catalog 产品文件、154 个插件测试、16 个插件 ignored benchmark，以及 Unreal、Godot、Fyrox、Bevy、Unity Graphics 的 19 个选定参考文件。选定 Zircon 路径相对 Runtime173 所在提交 `5798051603e7f7f565538125c9aba96d5beabae2` 没有源码差异，当前工作树在这些路径也为 clean；因此没有任何历史项可因 currentness 自动关闭。

Runtime140 继续作为唯一 Network 账本。本轮重判结果为：

- P0：**5 Open / 0 Partial / 0 Closed**。
- P1：**43 Open / 5 Partial / 0 Closed**；Partial 仍只有 `NET-P1-020/033/037/041/046`。
- P2：**12 Open / 0 Partial / 0 Closed**。
- G01-G32：**26 Fail / 6 Partial / 0 Pass**；Partial 仍只有 G06/G07/G09/G11/G18/G24。

Runtime173 的 `NET-RT-001..042` 不再作为第二套债务计数；它们在第 12 节映射回 `NET-P*` 或资格门。Runtime173 的 20 门表格实际是 **17 Fail / 3 Partial / 0 Pass**，原文汇总“15 Fail / 5 Partial”是算术错误；本报告修正该统计，但不回写历史证据文档。

本轮是 review-only。Tooling 按用户要求排除；未修改生产代码、Cargo、ABI、测试或 UI，未运行 Cargo、真实 TLS、公网、双进程、PIE、故障、fuzz、scale、soak 或竞争 benchmark，也未查询、轮询、等待或实时跟踪协调器。

## 2. 范围、指纹与测试证据

统计口径为当前工作树物理行、非空行、文件 bytes、Rust test attribute、`#[ignore]` 和 `unsafe` 词项。fingerprint 将相对路径转小写 `/`，按路径排序，以 `path + NUL + lowercase(file SHA-256) + LF` 拼接后再取 SHA-256。

| 范围 | files | lines | non-empty | bytes | tests | ignored | unsafe | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| Runtime framework net | 18 | 2,468 | 2,191 | 75,501 | 12 | 3 | 0 | `aac8c88b4068b9281c8b54e65bbd6f4c36f6777d65ff6443291e00d9aeef58fe` |
| `zircon_plugins/net` 全量 | 188 | 17,286 | 15,591 | 608,633 | 154 | 16 | 6 | `60aab4f4d460f3ebf357896a67f854d9c1e1a64f3fcba15c004f70c4b4f6e632` |
| App/catalog 产品选集 | 11 | 917 | 852 | 34,148 | 9 | 0 | 0 | `c720fc432332043428c2d9c3da05bc8ad4144f50271153c5363c617464a54e16` |
| 去重 Zircon 联合集 | **217** | **20,671** | **18,634** | **718,282** | **175** | **19** | **6** | `821098e7a3c371739a4fc56ebbabc066ddf975253e3f69966da4a300fe609a40` |
| 五引擎参考选集 | 19 | 12,186 | 9,985 | 482,300 | 2 | 0 | 0 | `714727170ea0a05cc3819e51e18e7b44ffa286f79aec008f19fdf3b40e76cef4` |

插件内部物理分布：

| 子域 | files | lines | bytes | tests | ignored |
|---|---:|---:|---:|---:|---:|
| base runtime | 52 | 5,547 | 186,926 | 43 | 4 |
| HTTP | 16 | 1,124 | 39,790 | 11 | 0 |
| WebSocket | 20 | 1,205 | 42,518 | 8 | 0 |
| RPC | 21 | 2,833 | 102,013 | 34 | 3 |
| Replication | 25 | 1,707 | 59,933 | 12 | 2 |
| Reliable UDP | 23 | 1,939 | 69,902 | 18 | 3 |
| Content Download | 21 | 2,223 | 81,184 | 25 | 4 |
| Editor | 7 | 403 | 16,279 | 1 | 0 |
| dist | 2 | 115 | 3,998 | 2 | 0 |

Runtime173 报告的是插件 186 files / 17,102 lines / 150 tests / 14 ignored，以及 framework 18 files / 2,207 lines / 8 tests / 1 ignored。当前选集相对该报告所在提交无源码差异，说明旧报告在落盘时已经少计同提交内的文件/测试，而不是本轮发生了功能增长。当前 19 个 ignored 全是 endpoint/range/TLS hex/UDP buffer、event/local HTTP、RPC heap/channel/session、replication lookup/clone、RUDP 容器和 download map/bitmap 等局部性能证据；没有同功能 World、secure session、跨进程或产品 benchmark。

## 3. 当前真实产品链

```text
Project / App target-profile selection
  -> builtin rows and feature crate names
  -> first-party runtime catalog
     -> root net runtime only
     x HTTP / WebSocket / RPC / Replication / RUDP / Content provider closure

root net runtime
  -> canonical DefaultNetManager
     -> manager-owned multi-thread Tokio runtime
     -> dedicated worker thread
        -> second multi-thread Tokio runtime
     -> synchronous request + fixed 2 s recv_timeout

optional features
  HTTP      -> private DefaultNetManager + HTTP backend
  WebSocket -> private DefaultNetManager + WebSocket backend
  RPC       -> private in-memory manager
  Replicate -> private in-memory manager
  RUDP      -> private in-memory manager, no UDP socket
  Content   -> resolves canonical NetManager
               x canonical manager normally has no HTTP backend

scene
  First -> drain at most 256 events, diagnostics frame = 0
  Last  -> no-op flush
  x session / RPC / replication / RUDP / download systems

editor / product
  -> descriptor/static Workbench surfaces
  x provider/resource/factory/document/compiler/runtime artifact
  x server + N client session group, network emulation and real profiler
```

feature declaration中的 dependency 只决定元数据/注册顺序，不是实例注入。HTTP/WebSocket factory 都忽略 `core` 并新建 `DefaultNetManager`；RPC/Replication/RUDP 同样直接新建各自 manager。Content Download 是唯一通过 `net_manager_handle(core)` 解析根 manager 的 feature，却因此无法获得 HTTP 私有 manager 中的 backend。direct tests 通过手工 constructor/injection 绕开了产品断点。

## 4. Authority、执行与生命周期

1. `DefaultNetManager::for_mode` 每个实例创建一个 multi-thread Tokio runtime，同时 `NetWorker::spawn` 再创建线程，线程内 `WorkerCore::new` 又创建一个 multi-thread Tokio runtime。HTTP/WS 每个私有 manager 重复这套成本；这些 worker 不属于 Runtime192 的统一 executor/inventory。
2. 公开 `NetManager` 是同步 facade。命令进入 1,024 容量的 `SyncSender` 后，caller 最长等待固定 2 秒；timeout 不取消底层 connect/send/listen，late side effect 可在 caller 已失败后继续发生。
3. worker ingress 也是 1,024 容量，`try_send` 结果被丢弃；manager 主 event queue 是无界 `VecDeque`。`diagnostics()` 会以 `usize::MAX` 抽干 worker ingress，使观测调用改变 delivery 行为。
4. scene First 每帧只 drain 256，且 diagnostics frame 固定 0；Last 阶段 `run_net_flush_egress` 直接 `Ok(())`。配置中的 runtime mode、TCP/UDP poll budget和 feature options没有进入同一 activation snapshot。
5. manager map 与 worker map是两份生命周期真相。多处 close 只是删 map、abort task或改 state，没有统一 StopAdmission/Drain/Close/Join/LeakReport，也没有 generation fence。
6. `NetError` 的大多数运行失败仍折叠为 `Io(String)` 或 `SecurityPolicyViolation { reason: String }`，缺少 Runtime195 所要求的 stage、owner、retryability、source chain和结构化 terminal receipt。

## 5. Transport、HTTP、WebSocket 与 Security

- `NetEndpoint::to_socket_addr` 只接受 IP literal；没有 DNS、IPv4/IPv6 racing、interface scope、path change或resolver cancellation。TCP只是字节流，没有 framing、channel、partial-write completion、max message、flow control或half-close contract。
- UDP socket的 65,535-byte receive buffer已从“每次 poll 分配”改善为每 socket 持久 buffer，这是可保留的局部进展；但每个 packet仍复制到 `Vec<u8>`，没有batch receive、pool ownership、truncation、peer admission和per-peer fairness。
- HTTP local route通过“URL没有显式port + path/method相同”猜测 in-process dispatch，远端 authority 可被错误截获。client每次请求创建 Hyper/Reqwest client并全量收集body；retry没有method/idempotence/backoff/jitter/`Retry-After`。
- HTTP pin路径先 `danger_accept_invalid_certs(true)`，再检查单张 peer certificate；这不是完整chain/hostname/root/pin rotation合同。server无限accept/spawn，handler同步执行，只有HTTP/1和固定body cap，无graceful drain。
- WebSocket custom roots/pin只做配置字段前置检查，`connect_async` 没有安装自定义root或pin verifier；server stream类型明确是明文 `TcpStream`，没有WSS identity。outbound仅64 frame局部有界，inbound/event无界；没有message byte cap、heartbeat、close deadline、reader/writer join或重连状态机。
- `NetSecurityPolicy::development()` 是默认值，Content Download也显式使用它。当前 shipping profile没有 fail-close guard，不能把配置表面当成安全能力。

## 6. Session 与 RPC

- 默认 `challenge_nonce` 固定为 `zircon-rpc-challenge`；handshake frame的 token只被编码/解码，不参与认证。Login只比较challenge字符串，player_id来自远端字符串，caller又直接提供 `RpcPeerRole` 和 source/target session。
- `apply_transport_events` 有测试，但没有生产 caller；session因此不受真实 connection lifecycle驱动。session、quota、handler、queue和pending request全部在单个大 mutex中。
- schema目前是字符串 ID加可选 closure validator，不是编译后的稳定 wire schema。channel queue是本地 `HashMap<u8, VecDeque<_>>`，没有连接、codec或transport route，且 enqueue没有queue depth/bytes cap。
- handler同步运行在调用线程；timeout只能在 closure 返回后标记 `TimedOut`，不能抢占、取消或阻止副作用。pending request可被相同裸 request ID覆盖，failure只保留字符串 diagnostic。
- priority heap、bounded invocation queue、per-session quota与部分 lookup优化是真实局部成果，所以 `NET-P1-037` 保持 Partial；它们不关闭认证、wire、authority、transport、World或async terminal缺口。

## 7. Replication、Prediction 与 Rollback

- Replication manager没有生产 caller，也不接 World、Reflection、change tick、authenticated connection或transport。所谓 `dual_world_replicates_spawn_update_despawn` 测试是两个内存 manager之间手工传 `SyncDelta`，不是两个 World或进程。
- component、field和interest仍由 String标识；descriptor的 authority、replication strategy和 `delta_compressed` 多数不驱动算法。`compile_replication_table` 只是对字符串排序并给本次运行分配 dense index，不是稳定 artifact或compat hash。
- `publish_snapshot` 对每个新 field线性查旧字段，移除字段不会生成删除语义；即使没有changed field也递增sequence并返回空delta。没有 serializer/quantizer/change mask/object reference/condition/migration。
- schedule每个 session/tick重建全 snapshot candidate并排序，按 payload bytes估预算，却不计header、object/component ID、compression、encryption、ACK。没有per-connection known-object/baseline、ACK/NACK、dormancy、priority debt、resync或bounded history。
- interest只有字符串group；`late_join_snapshots` 只是当前 visible snapshots别名。插值通过 component名称包含 `transform` 推断，并把字段前4字节当 little-endian f32；没有clock sync、typed vector/quaternion、jitter buffer、extrapolation、teleport policy。
- `despawn_object` 会删除sequence，而 `collect_despawn_deltas` 保留并递增sequence，生命周期入口语义不一致。裸 object ID复用缺owner/generation，无法证明 stale delta拒绝。
- 没有 input command、server tick、prediction history、authoritative correction、reconciliation、resimulation、physics rollback、rewind、lag compensation或deterministic replay。当前“插值”不能冒充 prediction/rollback。

## 8. Reliable UDP

- `ReliableDatagramPacket` 使用 `u64 sequence`、String channel和 `u16` fragment index/count；wire header使用 `u16 sequence/ack`、`u8 channel/fragment`。两套模型没有统一 codec，也没有实际调用根 UDP manager。
- `acknowledge_wire_header` 用 `packet.sequence as u16` 匹配 ACK bitmap；超过 16-bit wrap 后，不同逻辑 sequence可能被误确认。ordered delivery又以全 manager单一 `next_ordered_sequence` 排序，不按 peer/channel隔离。
- fragment assembly按 sequence索引，不带来源peer；没有总bytes、per-peer bytes、assembly count、TTL或malformed budget。攻击者可建立大量未完成 assembly。
- resend timeout固定，RTT只允许外部写一个 f32统计值；没有RTT estimator、dynamic RTO、cwnd、pacing、loss recovery、path MTU、AEAD、key epoch、anti-replay或anti-amplification。
- deterministic drop/reorder simulation、fragment/ACK/resend与局部容器优化可作为 codec testkit保留，所以 `NET-P1-033` 仍是 Partial；在接入真实 peer/socket和安全/congestion之前不能称为可靠传输产品。

## 9. Content Download

- chunk hash、range、mirror、resume bitmap和长度/overflow校验是真实底座；但 manifest无签名、publisher/key epoch、layout总上限、URL allowlist、redirect policy或content identity。
- partial chunks、bitmap、progress、cache hits和failed attempts全部在内存 HashMap/Vec中。resume bitmap中的 true可直接计为cache hit，没有磁盘artifact identity或重新hash证明。
- fetch同步调用 `NetManager::send_http_request`，每次只推进一个chunk/attempt，request ID由 attempt index生成，跨download可冲突。cancel只改状态，不取消网络请求。
- full response先进入内存，再和partial prefix拼接为另一份 Vec后hash；没有stream-to-temp、incremental hash、fsync、atomic publish、cache quota、startup recovery、repair、mount/install、rollback或last-good。
- indexed bitmap/map/attempt URL优化使 `NET-P1-046` 保持 Partial；它们不能替代 durable content installer和BuildPatch级事务。

## 10. Product、Editor 与跨域 owner

Editor产品细节仍由 Editor233 的 `ED-NET-*` 唯一账本持有，本篇不重复计数。Network的P0产品真实性仍保留在Runtime140：Net Editor provider/resource/factory/document/compiler未闭合，Lobby/Matchmaking/Online Services和server + N clients PIE没有运行时authority，静态页面不能作为成功证据。

后续横向审查进一步强化而非关闭Network债务：

| owner | 对 Network 的约束 |
|---|---|
| Runtime192 Tasks | 删除每 manager/feature私有runtime与未登记worker；I/O operation必须有owner、deadline、cancel、terminal和join receipt |
| Runtime194 Events | `NetEvent`进入编译taxonomy，携source/generation/sequence/time；bounded ingress丢失必须有receipt/resync |
| Runtime195 Failure | `Io(String)`/diagnostic String不能作为跨层错误合同；统一stage、source chain、retry、redaction和terminal state |
| Runtime199 Catalog | root + feature provider closure必须来自同一profile和generation，descriptor/crate name不等于activated provider |
| Runtime208 Export | ordinary/source-template/library/native必须链接并注册同一行为闭包，dist metadata shell不算parity |
| Runtime215 Identity | 7类Network ID和socket ID不能继续作为可serde裸u64；补owner/generation/exhaustion/replay合同 |
| App09 Launch | target/profile/config provenance必须生成Network activation plan；missing required provider fail-close |
| Editor259/260 | Net Editor contribution必须绑定provider owner/generation/capability和原子lifecycle transaction |

## 11. 唯一 canonical finding 账本重判

### 11.1 P0

| ID | 状态 | 当前差距 | 必须重构 |
|---|---|---|---|
| NET-P0-001 | Open | Lobby/Matchmaking固定房间、玩家、延迟、队列和feedback仍可冒充在线产品 | 未有真实document/provider/session receipt前隐藏或明确Demo/Unavailable |
| NET-P0-002 | Open | 没有Identity/Party/Lobby/Matchmaking/Ticket/Allocation/Online Provider runtime authority | 建立独立provider-neutral Online Services域，不把socket manager扩张成在线服务 |
| NET-P0-003 | Open | Net Editor默认不可达，资源/factory/lifecycle闭包未完成 | catalog/resource/factory端到端通过前不发布capability |
| NET-P0-004 | Open | Replication Schema无document/compiler/artifact/runtime install，Runtime不接World/transport | stable wire artifact、server/client compat hash和World消费共同验收 |
| NET-P0-005 | Open | WSS接收pin/root配置但未应用；Simulate没有真实多进程拓扑 | WSS fail-close；多人模拟必须由可终止、可观测、可重放session group驱动 |

### 11.2 P1

| ID | 状态 | 当前差距 / 重构目标 |
|---|---|---|
| NET-P1-001 | Open | 默认Client/Server/Editor provider不闭合；target/profile生成完整closure并fail-close |
| NET-P1-002 | Open | catalog只返回root；展开六feature DAG并发布requested/linked/admitted/activated receipt |
| NET-P1-003 | Open | Editor catalog无Net provider；runtime/editor closure必须来自同一project selection |
| NET-P1-004 | Open | factory忽略dependency；改为同代typed lease注入，禁止创建第二authority |
| NET-P1-005 | Open | HTTP/WS各自私有manager；backend以transaction安装到唯一instance |
| NET-P1-006 | Open | RPC/Replication/RUDP各自私有状态；消费同一secure session/connection/channel owner |
| NET-P1-007 | Open | `NetConfig`/options未消费；建立validated effective config和apply receipt |
| NET-P1-008 | Open | mode/target/role/feature/security分裂；冻结单一activation snapshot |
| NET-P1-009 | Open | event catalog只有4个字符串schema；生成完整versioned typed taxonomy |
| NET-P1-010 | Open | dist为stateless metadata shell；实现behavior/quiesce/state/unload或撤销NativeDynamic |
| NET-P1-011 | Open | ordinary/source/library/native不等价；统一resolver、registration和golden receipt |
| NET-P1-012 | Open | Beta/Partial无升级资格；maturity绑定本篇32门和BuildSet artifact |
| NET-P1-013 | Open | Net Editor资源闭包缺失；package manifest编译/hash/解析必须可验证 |
| NET-P1-014 | Open | operation只有descriptor；补typed I/O、permission、transaction、cancel和terminal |
| NET-P1-015 | Open | asset/toolkit/graph无document/compiler/runtime artifact owner；建立lossless source到install闭环 |
| NET-P1-016 | Open | Diagnostics/Workbench无真实producer和多人拓扑；接runtime trace与server + N clients emulator |
| NET-P1-017 | Open | process级manager不按purpose/World/session分域；引入supervisor/driver/world instance和generation fence |
| NET-P1-018 | Open | 同步API、串行worker和双runtime；收敛唯一I/O executor与async ticket |
| NET-P1-019 | Open | 队头阻塞和late side effect；per-connection task、公平调度、cancel和exact terminal |
| NET-P1-020 | Partial | 局部bounded drain/poison/clone改善；全链entry/bytes/age/share/drop、真实frame与flush仍缺 |
| NET-P1-021 | Open | 裸递增ID；迁移owner/generation/exhaustion/retire/stale-reject handle |
| NET-P1-022 | Open | TCP无frame/channel/backpressure/half-close；建立versioned bounded framing和QoS |
| NET-P1-023 | Open | UDP缺batch/pool/truncation/peer quota；建立buffer ownership和packet pipeline |
| NET-P1-024 | Open | endpoint无DNS/Happy Eyeballs/path observation；建立typed authority与async resolver |
| NET-P1-025 | Open | HTTP local route靠URL猜测；使用独立scheme/authority或显式local handle |
| NET-P1-026 | Open | HTTP per-request/full-body/naive retry；pool、stream、cap、cancel、idempotence、backoff/jitter |
| NET-P1-027 | Open | HTTP server无accept/handler/drain预算；bounded server和shutdown barrier |
| NET-P1-028 | Open | HTTP pin事后比leaf且先接受无效cert；统一chain/hostname/root/pin/rotation verifier |
| NET-P1-029 | Open | WSS root/pin未进握手；接真实peer chain验证与negative tests |
| NET-P1-030 | Open | WebSocket server仅明文；补WSS identity、rotation、mTLS/auth adapter |
| NET-P1-031 | Open | WS inbound/event无界且无heartbeat/close fence；双向byte/age预算和task lifecycle |
| NET-P1-032 | Open | diagnostics聚合、close不证明quiesce；typed per-connection observation与drain/join/leak receipt |
| NET-P1-033 | Partial | RUDP局部算法/分配改善；统一wire model并接真实peer/socket后硬切旧双模型 |
| NET-P1-034 | Open | ACK低16位、全局ordered sequence、assembly无上限；per-peer/channel wrap-safe window与TTL/quota |
| NET-P1-035 | Open | 无RTO/congestion/pacing/MTU/crypto；优先成熟库，自研需wire spec/interop/security review |
| NET-P1-036 | Open | 固定challenge、caller principal/role；connection-bound proof、identity和replay protection |
| NET-P1-037 | Partial | RPC heap/queue局部改善；compiled ID/channel、bounded async handler、deadline/cancel/dedup仍缺 |
| NET-P1-038 | Open | RPC无wire/transport/production caller；secure channel receive/validate/dispatch/response闭环 |
| NET-P1-039 | Open | Replication无World/Reflection/connection/transport；per-World extract和transactional apply |
| NET-P1-040 | Open | String/raw bytes与惰性metadata；compiler生成stable IDs/serializer/quantizer/condition/compat hash |
| NET-P1-041 | Partial | clone/candidate局部优化；dirty queue、persistent scheduler与真实wire预算仍缺 |
| NET-P1-042 | Open | 无baseline/ACK/known object/dormancy/relevancy/resync；建立per-connection replication state |
| NET-P1-043 | Open | 名字+首4字节f32插值；typed codec、clock/jitter/extrapolation/teleport policy |
| NET-P1-044 | Open | 无input/prediction/correction/reconciliation/rollback；建立bounded history和deterministic replay |
| NET-P1-045 | Open | Content解析canonical manager但HTTP在私有manager；同instance capability lease和产品集成测试 |
| NET-P1-046 | Partial | download map/bitmap局部优化；async scheduler、production trust、stream-to-disk和quota仍缺 |
| NET-P1-047 | Open | manifest无签名/identity/URL policy，request可冲突，bitmap可伪cache；signed manifest与verified journal |
| NET-P1-048 | Open | 无atomic install/persistent cache/repair/recovery；BuildPatch式stage/verify/activate/retire/last-good |

### 11.3 P2

| ID | 状态 | 工程级能力 |
|---|---|---|
| NET-P2-001 | Open | 大世界Replication Graph、spatial/team/owner nodes与parallel gather |
| NET-P2-002 | Open | physics rollback、lag compensation、replay和divergence artifact |
| NET-P2-003 | Open | NAT traversal、relay、P2P、ICE/STUN/TURN与privacy fallback receipt |
| NET-P2-004 | Open | Party/Lobby/Matchmaking/Allocation/Backfill和provider adapters |
| NET-P2-005 | Open | voice/text/moderation、consent/privacy/platform policy |
| NET-P2-006 | Open | multi-region handoff/migration/fleet drain/session continuity |
| NET-P2-007 | Open | QUIC/WebTransport/console/mobile/path-change/background-resume |
| NET-P2-008 | Open | live protocol rollout、cross-version support window和canary rollback |
| NET-P2-009 | Open | DDoS/anti-cheat/abuse/ban/attestation与shipping trust hooks |
| NET-P2-010 | Open | CDN delta、multi-source/P2P patch、QoS与transactional install |
| NET-P2-011 | Open | privacy-aware packet capture、schema inspector、offline replay和failure corpus |
| NET-P2-012 | Open | 第三方网络SDK、conformance kit、sandbox/trust和platform certification |

## 12. Runtime173 alias 收敛与统计纠正

| Runtime173 | 唯一owner |
|---|---|
| NET-RT-001..003 | NET-P1-001..003 |
| NET-RT-004..008 | NET-P1-004..008、NET-P1-017 |
| NET-RT-009..016 | NET-P1-009、020、018、019、021、023 |
| NET-RT-017..021 | NET-P1-022、024..032 |
| NET-RT-022..025 | NET-P1-033..038 |
| NET-RT-026..028 | NET-P1-039..044 |
| NET-RT-029..030 | NET-P1-046..048 |
| NET-RT-031..032 | NET-P1-010/011、001..003、016 |
| NET-RT-033..039 | NET-P1-020/022/032/034/035/037/041、NET-P2-001/011 |
| NET-RT-040..042 | G25、G27..G31 与 NET-P2-007 |

Runtime173 的 gate table中 Partial只有 G08、G15、G17；其余17项是Fail。正确汇总为 **17 Fail / 3 Partial / 0 Pass**。本篇采用更完整的Runtime140 G01-G32，不把两套gate相加。

## 13. 参考引擎差异

### 13.1 Unreal主对照

`UNetDriver`显式拥有client connection集合、connectionless handler、network object list、replication driver/system/bridge和server/client replication config；`UNetConnection`拥有open channels、packet/reliable sequence、queued bits、owning actor/player controller、close reason与resend state。Iris的 `ReplicationSystem`显式add/remove connection、set replication view/owner/filter/priority；`ReplicationStateDescriptor`保存稳定identifier、member offset、serializer/config、change mask、reference、condition、notify和delta traits。Zircon的process manager、caller role、String/raw bytes和本次运行排序index不具备同级语义。

ReplicationGraph按global/connection nodes增量gather，NetworkPrediction使用有界frame ring/sparse buffer保存连续和辅助状态。Zircon没有connection baseline或prediction frame history，不能用snapshot排序和receive-time f32插值替代。

Unreal HTTP retry有verb/status/domain policy、exponential backoff、jitter和cancel；WebSocket有message memory limit、connected/error/message/close event；BuildPatch installer有start/pause/resume/cancel、verify/repair、progress/error/statistics和install状态。Zircon的naive retry、无界WS inbound和内存download仍是架构差距。

### 13.2 Godot次对照

Godot `SceneMultiplayer`在同一owner下持有 `MultiplayerPeer`、pending/authenticated peer、auth timeout、connected peers、RPC、replicator和packet cache；RPC config包含mode、transfer mode和channel，replication interface处理spawn/despawn/sync与peer lifecycle。ENet/WebSocket peer提供真实connection status、channel/buffer/heartbeat/close语义。这反证把session/RPC/replication/RUDP拆成互不接transport的内存manager。

### 13.3 Fyrox、Bevy 与 Unity Graphics边界

Fyrox选择文件只是非阻塞TCP、长度前缀与bincode消息原语；可借鉴Rust ownership/error边界，但不是多人网络标杆。Bevy Remote明确把core methods与HTTP transport分离，并用bounded mailbox把请求回投World schedule；可借鉴插件边界和有界handoff，但它是远程ECS控制，不是game netcode。`dev/Graphics`选定包是SRP Core，package metadata中没有Netcode/Multiplayer/Online/Network依赖或实现；本报告只记录“不适用”，不从该树推断Unity完整网络能力，也不据此降低Zircon标准。

## 14. 目标架构与硬切

```text
Project/Profile Selection
  -> NetworkActivationPlan + CapabilityTruthReceipt
     -> Runtime-owned NetIoSupervisor
        -> DriverRegistry { game, beacon/service, replay, editor-test }
           -> NetworkDriverInstance { purpose, role, world_generation? }
              -> generational Listener/ConnectionTable
                 -> transport tasks + bounded lanes + secure session
                 -> versioned channels { control, RPC, input, replication, data }
              -> NetWorldRuntime
                 -> compiled wire artifact + object registry
                 -> dirty/filter/prioritize/serialize/baseline/apply
                 -> clock/input/prediction/reconciliation/history
        -> HttpService / WebSocketService / ContentInstaller
        -> typed observation / trace / capture
  -> Editor consumes same artifact, receipt, session and observations
```

必须硬切：

1. 删除HTTP/WS私有 `DefaultNetManager`，删除RPC/Replication/RUDP独立authority；feature只能扩展同代instance。
2. 同步socket/HTTP/WS facade切为ticket/bounded batch，World/gameplay路径不得继续 `recv_timeout`/`block_on`。
3. 删除空flush、frame 0、静默event drop和diagnostics驱动无限搬运。
4. 裸ID全部迁移到owner/generation qualified handle；完成迁移后不保留数值alias。
5. 删除HTTP local route URL heuristic，改显式in-process authority。
6. WSS verifier接通前撤销custom root/pin能力；development trust不得进入Shipping。
7. 删除RUDP logic/wire双模型和无socket manager；先冻结协议/选型，再一次切换真实peer transport。
8. 删除固定challenge、unused token、caller role和无wire RPC入口。
9. 删除String/raw-byte replication shipping主路径和component-name插值推断。
10. 删除bool resume即cache hit、全body拼接和无事务内容安装路径。
11. Net Editor/Workbench缺provider/resource/factory/compiler/session前不得报告产品success。
12. ordinary/source/library/native必须消费同一resolver和receipt，dist metadata不能冒充behavior parity。

## 15. 分层重构顺序

1. **M0 Truth/Composition/Identity**：默认target RED matrix、activation plan、唯一supervisor、feature lease、effective config和generation handle。
2. **M1 Executor/Lifecycle**：统一I/O executor、ticket/deadline/cancel/exact terminal、bounded lanes、Stop/Drain/Close/Join/LeakReport。
3. **M2 Transport/Security**：resolver/IPv6/framing/UDP pool、TLS/WSS verifier、shipping guard、malicious peer与parser fuzz。
4. **M3 HTTP/WS/Content**：pool/stream/retry、bounded server、heartbeat/close fence、signed manifest、stream-to-disk、journal与atomic install。
5. **M4 Secure Session/RPC**：connection-bound identity/version/schema negotiation、channel codec、compiled RPC table、async handler与双进程RPC。
6. **M5 World Replication**：NetworkIdentity、change extraction、spawn/despawn/ownership、stable serializer、baseline/ACK、interest/dormancy和transactional apply。
7. **M6 Prediction/Rollback**：server tick/time sync/input history、correction/reconciliation/resimulation、typed smoothing与determinism artifact。
8. **M7 Reliable Datagram/Emulation**：成熟transport选型或wire/interop/security review，per-peer window、RTO/congestion/pacing/MTU/AEAD和fault emulator。
9. **M8 Editor/Online/Dist**：真实resources/operations/compiler/profiler/PIE，独立Online Services，source/library/native行为和unload parity。
10. **M9 Qualification**：跨平台、fuzz/fault、1/100/10k connection、1k/10k/100k object、24h soak与同功能Unreal/Godot竞争benchmark。

## 16. G01-G32 资格门

| Gate | 状态 | 验收条件 |
|---|---|---|
| G01 Unique authority | Fail | 一个Core一个I/O/config/security authority，feature只持lease |
| G02 Product reachability | Fail | Client/Server/Editor按profile真实链接required provider |
| G03 Effective config | Fail | 所有option有validated consumer和apply/restart receipt |
| G04 Scoped identity | Fail | driver/World/connection/session/object/ticket generation-bound |
| G05 Async lifecycle | Fail | main/World零block_on/recv_timeout，cancel停止底层I/O且终态唯一 |
| G06 Bounded queues | Partial | 局部command/writer有界；全链entry/bytes/age/share/drop未闭合 |
| G07 Transport correctness | Partial | loopback真实；framing/DNS/IPv6/half-close/UDP pool/interop未闭合 |
| G08 Security | Fail | TLS/WSS/secure session/credential/shipping guard与malicious peer门 |
| G09 HTTP | Partial | 本机HTTP/leaf pin局部存在；pool/stream/retry/server/drain仍缺 |
| G10 WebSocket | Fail | WSS verifier、bounded inbound、heartbeat、close/task fence |
| G11 Reliable datagram | Partial | fragment/ACK/resend算法存在；peer/wire/congestion/security未闭合 |
| G12 Session auth | Fail | principal/role来自认证connection，proof抗重放 |
| G13 RPC | Fail | stable codec、transport correlation、async handler/deadline/response |
| G14 World replication | Fail | 真实World spawn/update/despawn/apply和ownership |
| G15 Wire schema | Fail | stable IDs、serializer/quantizer/condition/migration/compat hash |
| G16 Baseline/relevancy | Fail | per-connection baseline/ACK/interest/dormancy/loss recovery |
| G17 Prediction | Fail | clocked input、correction、resimulation、rollback/typed smoothing |
| G18 Content install | Partial | chunk hash/bitmap局部存在；signed/disk/journal/atomic install未闭合 |
| G19 Net Editor | Fail | resource/factory/document/compiler/preview/undo/save真实 |
| G20 Online Services | Fail | identity/lobby/session/matchmaking/allocation/provider状态机 |
| G21 Multiplayer PIE | Fail | Dedicated/Listen + N clients、sandbox/readiness/reap |
| G22 Observation | Fail | RTT/loss/jitter/queue/RPC/object/prediction/download typed trace |
| G23 Native parity | Fail | dist有behavior/state/bridge/quiesce/unload和feature closure |
| G24 Export parity | Partial | SourceTemplate结构存在；ordinary/LibraryEmbed/Native仍不等价 |
| G25 Codec robustness | Fail | golden/property/fuzz/malformed/oversize/slowloris/fragment corpus |
| G26 Product integration | Fail | 普通App/Server/Editor/Hub不依赖test-only constructor |
| G27 Cross-platform | Fail | Windows/Linux/macOS及目标console/mobile互操作 |
| G28 Scale | Fail | 10k connections、100k objects和bounded memory/tail latency |
| G29 Fault recovery | Fail | DNS/TLS/reset/FD/port/disk/task panic/crash/path-change矩阵 |
| G30 Soak/shutdown | Fail | 24h wrap/stall/World travel/unload零orphan/leak |
| G31 Competitive benchmark | Fail | 同功能/安全/质量对Unreal/Godot的CPU/RSS/bandwidth/p99 |
| G32 Truthful maturity | Fail | artifact驱动maturity，UI/manifest不超前承诺 |

## 17. 禁止的临时修补与完成定义

- 禁止新增manager/runtime来“接通”feature，或在同步等待外再包一层async。
- 禁止扩大channel/Vec/timeout掩盖owner、boundedness和队头阻塞。
- 禁止只检查pin字符串、关闭证书验证或把development trust带入Shipping。
- 禁止创建空resource、`Ok(())` operation、随机Lobby数字或静态success feedback伪造闭环。
- 禁止用自由String、运行时排序index或component名字推断稳定wire schema/typed interpolation。
- 禁止把direct manager、loopback、DTO、descriptor或ignored microbenchmark称为多人产品/性能资格。
- 禁止在G01-G32未通过前将Net或任一feature提升为Stable/Complete/default-on。

本review单元完成定义已满足：当前217个Zircon选集文件和19个参考文件已冻结，生产链、测试边界、唯一finding owner、alias、目标architecture、hard cut、milestone和32门已落账。实现仍为 `pending`；只有M0-M9依赖顺序完成，并通过真实App/Server/Editor/export、secure双进程session、World RPC/replication/prediction、signed content、cross-platform、fuzz/fault/scale/soak和竞争benchmark，才可标记implementation complete。
