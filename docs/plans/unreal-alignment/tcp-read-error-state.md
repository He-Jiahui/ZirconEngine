# 网络：TCP 接收失败已发出 `Failed`，同一连接查询仍为 `Open`

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE/ZE 测试均未在本轮执行。

状态：单项源码实证，供独立问题 Draft 评估；本轮没有执行 ZE/UE 测试、启动网络对端、改引擎源码、写 Git 或访问远端。ZE 证据固定 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`，所引核心文件的本地内容按 Git 规范化重算与该提交 blob 一致。UE 取本机 [Build.version:2–9][ue-version]：6.0.0 / `UE5`，不推定上游 commit。

## 1. 差异与可达调用

- **ZE（已启用联编 `net` 的路径）**：App [engine_entry.rs:288–300][ze-app] 通过首方 catalog [lib.rs:73–105][ze-catalog] 取 `NetRuntimePlugin`，其 [plugin.rs:57–86][ze-plugin] 挂模块和 ECS system；[module.rs:28–55][ze-module] 把同一个 `DefaultNetManager` 注册为 `dyn NetManager`。`connect_tcp_impl` 写入 manager 的 `Open`；[service_types.rs:249–269][ze-api] 的 `NetManager::poll_tcp` 进入 [tcp.rs:63–114][ze-tcp]，走 [`NetWorker::poll_tcp`][ze-worker] → [`NetEgress::PollTcp` 分发][ze-dispatch] → [`WorkerCore::poll_tcp`][ze-read]。已建立连接的 `try_read` 如返回非 `WouldBlock` 的 I/O 错误，worker 先把连接转 `Failed` 并向 ingress 发 `ConnectionStateChanged(Failed)`，再回 `Err(NetError::Io)`；manager 的 `?` 在回写状态之前返回。之后 [connections.rs:13–23][ze-query] 的 `connection_state` 仍从 manager 表读 `Open`。事件由 [`drain_events_impl`][ze-drain] 读出，启用联编模块时也由 [`run_net_poll_ingress`][ze-ingress] 转为 ECS 事件。仅证明已存在的服务边界，不声称默认项目或某个游戏系统正在主动 poll。
- **UE（TCP 对照）**：启用 `TcpMessaging` 后，[TcpMessagingModule.cpp:240–255][ue-tcp-mod] 建立 transport/bridge，[MessageBridgeBuilder::Build][ue-builder] → [MessageBridge.cpp:73–85][ue-bridge] 启动 transport；[TcpMessageTransport.cpp:47–64、196–208、282–286][ue-tcp-owner] 的 listener accept 排队，transport 绑定状态回调、启动连接。对接受方（重试延迟为 0），[TcpMessageTransportConnection.cpp:165–245、308–337、431–436][ue-tcp-conn] 的接收失败终止循环，先更新 `STATE_Disconnected`，再发状态 delegate；[TcpMessageTransport.cpp:290–309][ue-tcp-notify] 的回调读取该连接同一个状态并反映节点失联，并非使用另一份只按接收成功回写的镜像。这是 TCP messaging，不把它当 Gameplay replication。

## 2. 有界影响

只需一条已是 `Open` 的正常 TCP 连接、一次**确实返回到调用方**的非 `WouldBlock` 读取错误、在下一次成功回执或主动清理前查询同一 ID：公开结果同时存在 `poll_tcp -> Err(Io)`、`drain_events -> Failed`、`connection_state -> Open`。若调用方看到错误后停止读，这是可以持续的旧状态；按 `Open` 决定是否保留连接的调用方与订阅事件的调用方会得出不同结论。未声称数据丢失、必然宕机、所有平台都会产生相同 socket 错误，亦不把 `Ok(0)` / EOF 的历史重复事件、worker 回执超时、发送错误、WS/RPC/复制策略列为本缺陷的影响。

## 3. 根因与 UE 处理

ZE 当前的 [`WorkerReply<T>` / `TcpPollResult`][ze-egress] 只允许在 `Ok(TcpPollResult)` 中携带状态，已发生的 socket 失败却从外层 `Err` 返回。`TcpPollResult.state` 在成功链才被消费，恰好吞掉了状态机已经认定的迁移。UE TCP 的 `Run` / `GetConnectionState` 与状态通知读同一所有者状态。Gameplay 的 [`UIpNetDriver::TickDispatch`][ue-ip-driver] → [`UIpConnection::HandleSocketRecvError`][ue-ip-connection] 在客户端、非阻塞等待且错误达到 resolver/关闭政策时走 `Shutdown` → [`UNetDriver::Shutdown`][ue-net-driver] → [`UNetConnection::Close`][ue-net-close] 写入 `USOCK_Closed`；不声称 UE 对全部 UDP 错误立即断开，与 ZE 的 `Failed` 名称无须相同。

## 4. Rust 沿 UE 思路，不新建架构

保留 socket worker 为迁移权威及现有 manager 同步镜像；将“worker 确实完成一次读取后的状态”与“读取给调用方返回字节还是 I/O 错误”分开传。worker/回执队列本身失败、超时、未知 ID 不拥有这一条连接的新状态，不能仅见 `NetError::Io` 就把 manager 猜成 `Failed`。

## 5. ABI / ECS 差异

UE 使用连接对象及 delegate，ZE 此处是联编 Rust `NetManager`、内部队列和可排空的类型化 `NetEvent`，ECS 只消费现有事件。修正不需要改 `NetConnectionState`、事件 DTO、`zircon_runtime_interface` 或复制 schema，也不要求加 UE channel、Iris 或 Actor 网络功能。

## 6. ZE 具体改法草案

仅在 [egress.rs:13–24][ze-egress] / [transport_runtime.rs:304–360][ze-read] 内把已存在连接的 poll 回执改为 `state` 和内层 `Result<Vec<u8>, NetError>`：worker 描述 `try_read` 的非暂态失败为“有效回执、读取失败、当前状态 `Failed`”，仍发布同一个事件；未知 ID/队列失败仍是外层错误。[tcp.rs:95–114][ze-tcp] 先把 worker 的 `state` 同步到原连接项，再将内层结果原样返回，仅按成功 payload 长度记字节。`max_bytes == 0`、`WouldBlock`、正常数据、既有 EOF 行为保留；不新增定时器、锁、外部 API 或架构。后续如获授权：先独立问题 Draft，再挂等价语义回归，再做窄修复。

### 对照测试与 ZE 落点（未执行）

| UE 名称、路径、输入 → 期望 | 等价边界 | ZE 现成落点与待加断言 |
|---|---|---|
| `FOnlineBeaconTestGarbageCollection` / `System.Engine.Online.OnlineSubsystemUtils.OnlineBeacon.GarbageCollection`，[OnlineBeaconUnitTest.cpp:2231–2234、2507–2545][ue-beacon]：已握手连接在回调中销毁客户端；推进 tick，从有 Host client 到断开通知 1 次且 Host 查询不再找到该 client。 | 只借鉴“同一连接的通知和查询一致”；这是 Beacon 拆除而非 TCP 读取错误，不能原样迁移。 | [`net_runtime_manager_accepts_tcp_client_and_echoes_payloads`][ze-test] 已有真实 listen/connect/accept、`Open` 与事件读取；扩展为接收错误后立刻断言同一 ID 的 `Err(Io)`、`Failed` 事件、`connection_state == Failed`，断言应在任何下一次 poll 前。 |
| 精确 TCP read-error / query 一致性：未找到 UE 自动化用例；检索范围为 `TcpMessaging` 模块全树、`Engine/Private/Tests`、`OnlineSubsystemUtils/Private/Tests` 的 `Tcp` / `SocketRecv` / `ConnectionLost` / `CloseConnection` 等。`FOnlineBeaconTestInvalidSocket` [OnlineBeaconUnitTest.cpp:2551–2604][ue-beacon-invalid] 只断言初始化失败，排除。 | 新回归期望来自 [UE TCP 源码][ue-tcp-conn]，不是移植或已运行的 UE 测试。 | 首个故障用例可在现有 TCP harness 注入一次已接受连接实际消费的读错误（或使用可控本地对端）；必要时给实际 worker 的 `try_read` 边界极小测试缝，仍贯穿原入队/回执/公开查询。不用手造的 `NetEvent` 代替 worker。健康对照是空等与正常收包；[`state_changes_emit_events`][ze-machine-test] 只是状态机单测，不足以作为回归。 |

### 覆盖记录与去重

| UE 模块 / 已检查点 | ZE owner | 测试与未清项 |
|---|---|---|
| `Messaging` / `TcpMessaging` / `Sockets` 接缝：bridge、accept、读错误和 disconnected publication | `zircon_plugin_net_runtime` worker / `DefaultNetManager` | 现成 TCP loopback、transport state machine；未动态运行，不扩大到重试、半关闭、worker timeout 或更高层协议。 |
| `OnlineSubsystemUtils` / `Engine` NetDriver/NetConnection | `NetManager`、`NetEvent`、ECS `net.poll_ingress`；可选 RPC 的 `apply_transport_events` 仅单独对读 | Beacon 两例与 [`batched_transport_close_preserves_event_order_and_duplicates`][ze-rpc-test] 仅邻近证据；不推断已接生产 RPC session 或任意网络错误都关闭。 |
| `Iris` 接口 [`AddConnection` / `RemoveConnection`][ue-iris]、[`ReplicationGraph::RemoveClientConnection`][ue-rg] | `net/features/replication` 的 collect/apply/delta/schedule 与 RUDP/RPC 属可选 owner | [`dual_world_replicates_spawn_update_despawn`][ze-rep-test] 实际使用两个内存 manager、无真实传输；未认证 wire、远端 World、ACK baseline、Iris、Relevancy、预测；不按功能缺失开题。 |

本地 PR 快照 #135–145 没有同路径或同触发；未远端查重。旧 [08e P1-8][ze-historical] 已有“双状态表/EOF 重复和连接生命周期”上位条目，及 [Runtime217 NET-P1-019/021/022/032][ze-historical-217] 的相关归属；本条把其中**不同于 EOF**的非 `WouldBlock` 错误回执矛盾收窄为单独可回归切片，不再登记一个广义生命周期或重复终态问题。

### 源文件指纹

ZE 链接均锚定 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。核心 blob：`service_types/tcp.rs` `5e3d1429bb6b27bb4adec77127eaaf86da735874`；`service_types/connections.rs` `71609922b27a61005ed3952110df1b84988abe19`；`worker/transport_runtime.rs` `d5ff917df3f17ac2e1a66e8d4c27f7fb5d34383c`；`worker/egress.rs` `f6ae7b68f5e37c2b505f55e7b09e3ed05fe78673`；`tests/tcp.rs` `85bf899b734f555cb0dbc28c3974f594487d8c43`。

| UE 本地源文件 | SHA-256（实文件字节） |
|---|---|
| [Build.version][ue-version] | `3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6` |
| [TcpMessagingModule.cpp][ue-tcp-mod] | `e46cb3ee3d54bbd93da3667bbe3b0deb183d8618f301727c19f8cb66db2ed8a2` |
| [MessageBridgeBuilder.h][ue-builder] | `6a8871b3e5c989e7104c3e5d4d3d9c4a28f4dbe8aa156e6548b513b7778bfe81` |
| [MessageBridge.cpp][ue-bridge] | `8743f57a048cef294dbe8a96c242ce1a5b62bccebf9f397f3d07370ca5e8dd26` |
| [TcpMessageTransport.cpp][ue-tcp-owner] | `ee118561723b9b1473064e43ddbfbe39e44caa077759ed6191f383725c16eef9` |
| [TcpMessageTransportConnection.cpp][ue-tcp-conn] | `a1aa01e6ea9847553c5ceecac617065ff5cf58c18497dc5a0aa725916268e691` |
| [IpNetDriver.cpp][ue-ip-driver] | `b1dd86a3bb45426025926d883863bd7c49cc7ade5c7ccf43da4b370909d39bbc` |
| [IpConnection.cpp][ue-ip-connection] | `49077746f6480ba9f8185735e6e5dde7c2419b79c88703983416e4808953b2bc` |
| [NetDriver.cpp][ue-net-driver] | `bc3c626f27e73ed7ff0f7da5988f169755df4c0969ce404dc3cf4ffd5a4f4c7d` |
| [NetConnection.cpp][ue-net-close] | `a0193501a7f28ce11290fd16f2a0e3040695160d3799b965f1ce37a6da7f2e37` |
| [OnlineBeaconUnitTest.cpp][ue-beacon] | `f9c8f190694a1d1e0eb155cd720044c022b732b6e75f606e07b6f9d402d18154` |
| [ReplicationSystem.cpp][ue-iris] | `d40cfb7d0ac62469665235918777d23715cc6b8d2f94d41c9471bb97a84323b6` |
| [ReplicationGraph.cpp][ue-rg] | `5c45a29969b3808920be2d92f2ff7a193d924e055dd6bddc438fd018840c2126` |

[ze-app]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/engine_entry.rs#L288-L300>
[ze-catalog]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/first_party_runtime_catalog/src/lib.rs#L73-L105>
[ze-plugin]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/net/runtime/src/plugin.rs#L57-L86>
[ze-module]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/net/runtime/src/module.rs#L28-L55>
[ze-api]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/net/runtime/src/service_types.rs#L249-L269>
[ze-tcp]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/net/runtime/src/service_types/tcp.rs#L63-L114>
[ze-query]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/net/runtime/src/service_types/connections.rs#L13-L23>
[ze-worker]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/net/runtime/src/worker/net_worker.rs#L161-L171>
[ze-egress]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/net/runtime/src/worker/egress.rs#L13-L24>
[ze-dispatch]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/net/runtime/src/worker/transport_runtime/dispatch.rs#L74-L81>
[ze-read]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/net/runtime/src/worker/transport_runtime.rs#L304-L360>
[ze-drain]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/net/runtime/src/service_types/diagnostics.rs#L28-L47>
[ze-ingress]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/net/runtime/src/runtime_system.rs#L33-L79>
[ze-test]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/net/runtime/src/tests/tcp.rs#L9-L91>
[ze-machine-test]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/net/runtime/src/tests/transport.rs#L31-L81>
[ze-rpc-test]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/net/features/rpc/runtime/src/tests/session_batched_transport.rs#L14-L54>
[ze-rep-test]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/net/features/replication/runtime/src/tests/collect_apply.rs#L15-L71>
[ze-historical]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md#L178-L182>
[ze-historical-217]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/217-runtime-network-transport-session-rpc-replication-prediction-rollback-security-content-product-current-working-tree-review.md#L246-L259>
[ue-version]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L2-L9>
[ue-tcp-mod]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Messaging/TcpMessaging/Source/TcpMessaging/Private/TcpMessagingModule.cpp#L240-L255>
[ue-builder]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MessagingCommon/Public/MessageBridgeBuilder.h#L89-L125>
[ue-bridge]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bridge/MessageBridge.cpp#L73-L85>
[ue-tcp-owner]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Messaging/TcpMessaging/Source/TcpMessaging/Private/Transport/TcpMessageTransport.cpp#L196-L208>
[ue-tcp-notify]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Messaging/TcpMessaging/Source/TcpMessaging/Private/Transport/TcpMessageTransport.cpp#L290-L309>
[ue-tcp-conn]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Messaging/TcpMessaging/Source/TcpMessaging/Private/Transport/TcpMessageTransportConnection.cpp#L165-L245>
[ue-ip-driver]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Online/OnlineSubsystemUtils/Source/OnlineSubsystemUtils/Private/IpNetDriver.cpp#L1098-L1150>
[ue-ip-connection]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Online/OnlineSubsystemUtils/Source/OnlineSubsystemUtils/Private/IpConnection.cpp#L748-L786>
[ue-net-driver]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/NetDriver.cpp#L2469-L2514>
[ue-net-close]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/NetConnection.cpp#L1107-L1156>
[ue-beacon]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Online/OnlineSubsystemUtils/Source/OnlineSubsystemUtils/Private/Tests/OnlineBeaconUnitTest.cpp#L2507-L2545>
[ue-beacon-invalid]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Online/OnlineSubsystemUtils/Source/OnlineSubsystemUtils/Private/Tests/OnlineBeaconUnitTest.cpp#L2551-L2604>
[ue-iris]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Net/Iris/Private/Iris/ReplicationSystem/ReplicationSystem.cpp#L1337-L1345>
[ue-rg]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Runtime/ReplicationGraph/Source/Private/ReplicationGraph.cpp#L669-L700>
