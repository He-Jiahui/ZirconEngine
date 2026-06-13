# 07 · Net 插件完善计划（TCP / UDP / WebSocket / HTTP / RPC / Replication）

> 状态：工程化细化版 v2 · 优先级：P2 · 前置：[01 插件架构核心](01-plugin-architecture-core.md) M1–M3
> 关联计划：`.codex/plans/ZirconEngine Net 插件完善计划.md`（M0–M7 分层路线维持有效） · 现状文档：`docs/zircon_plugins/net/{runtime,editor}.md`

## 1. 目标

把 `zircon_plugins/net` 推进到稳定的 TCP/UDP/HTTP(S)/WebSocket 四协议层、会话/RPC、ECS 状态复制、可靠 UDP、内容下载。**实查修正**：六个 optional feature 包（`net/features/{http,websocket,rpc,replication,reliable_udp,content_download}`）已作为独立 crate 存在且有骨架实现，契约层 replication DTO 完整——本计划主轴是**线程模型纠偏（专用 NetWorker 替代 block_on）**、各 feature 的深度补全、与 01 调度锚点/事件总线的对接。

## 2. 现状基线（实查）

总量约 6100 行（含 features），结构化良好：

- **契约层** `zircon_runtime/src/core/framework/net/`：`transport.rs`（`NetTransportKind`/`NetConnectionState`/`NetSecurityPolicy`/`NetCertificatePin`）、`sync.rs`（**replication DTO 全家已在**：`SyncAuthority`/`SyncFieldDescriptor`/`SyncComponentDescriptor`/`SyncReplicationBudget`/`SyncObjectSnapshot`/`SyncDelta`/`SyncInterestDescriptor`/`SyncReplicationScheduleReport`）、`session.rs`、`rpc.rs`、`reliable.rs`、`download.rs`、`http.rs`、`websocket.rs`、`packet.rs`、`endpoint.rs`、`socket_id.rs`、`event.rs`、`diagnostics.rs`、`manager.rs`。
- **本体** `net/runtime/src/`：`service_types.rs`（252 行，**已拆分**为 `service_types/{tcp,udp,http_routes,listeners,diagnostics}.rs`——早期"100KB 单文件"判断已过时）、`runtime_state.rs`、`package.rs`；**Tokio 已是依赖，但以 `block_on` 在调用线程同步执行**（`service_types/tcp.rs:26` `.block_on(TcpListener::bind(…))`）。
- **features**（六 crate 全部存在）：`http`（backend/server.rs 158 行）、`websocket`（connection/handshake/reader）、`rpc`（dispatch 342 行/state/handshake/session）、`replication`（manager/schedule.rs 121 行）、`reliable_udp`（manager/resend）、`content_download`（http_fetch/attempts/progress）。

缺口（按严重度，校准后）：

| # | 缺口 | 证据 |
|---|------|------|
| T1 | 线程模型：Tokio 以 `block_on` 阻塞调用线程，无专用 worker、无 ingress/egress 队列——bind/connect/IO 都可能卡主循环 | `service_types/tcp.rs:26,126` |
| T2 | 无 `net.poll_ingress`/`net.flush_egress` 系统锚点；事件走字符串目录而非类型化总线 | `runtime/src/lib.rs` |
| T3 | 重连 backoff、连接状态机（`NetConnectionState` DTO 在、驱动缺）；TLS/rustls 未接 | `transport.rs` vs 实现 |
| T4 | replication：DTO 全、`schedule.rs` 仅排程骨架——与 ECS change detection 的收集/应用闭环缺 | `features/replication/runtime/src/manager/schedule.rs` |
| T5 | reliable_udp：resend 骨架在，包头格式/ack 位图/fragment 未定 | `features/reliable_udp/` |
| T6 | content_download：fetch/attempts/progress 在，断点续传位图持久化与 hash 校验重拉缺；与 zrpack 的共享 DTO 未定 | `features/content_download/` |
| T7 | session/rpc：handshake/dispatch 在，channel 多路复用与方向校验/超时不完整 | `features/rpc/` |

## 3. 架构设计

中立契约维持在 `zircon_runtime::core::framework::net`；**Tokio 不进 zircon_runtime 本体依赖**（既定裁决），由插件自建独立 worker。

### 3.1 NetWorker 与队列（解决 T1/T2，`runtime/src/worker/` [新增]）

```rust
pub struct NetWorker {
    runtime: tokio::runtime::Runtime,       // multi-thread，插件 activate 时启动
    ingress: rtrb::Consumer<NetIngress>,    // worker → 主线程 SPSC（预分配环形）
    egress:  rtrb::Producer<NetEgress>,     // 主线程 → worker
}
pub enum NetIngress {
    ConnectionState { conn: NetConnectionHandle, state: NetConnectionState },  // 契约现枚举
    Datagram { conn: NetConnectionHandle, payload: Bytes },
    HttpResponse { req: HttpRequestHandle, status: u16, body: Bytes },
    WsFrame { conn: NetConnectionHandle, frame: WsFrameKind },
}
pub enum NetEgress {
    Listen(ListenerConfig), Connect(ConnectConfig), Send { conn: NetConnectionHandle, payload: Bytes },
    HttpRequest(HttpRequestConfig), Close(NetConnectionHandle), Shutdown,
}
```

- 关停协议：`deactivate` 发 `Shutdown` → worker drain 在飞任务（超时 2s 强制）→ join；泄漏检测断言进测试。
- 系统锚点（01 定稿）：`net.poll_ingress` ∈ First（drain ingress → 类型化事件总线 + 更新连接表）；`net.flush_egress` ∈ Last（收集本帧出站命令批量入队）。
- 句柄模型：`NetListenerHandle`/`NetConnectionHandle`（不透明 id，复用契约 `socket_id.rs` 形态）；现 `service_types/{tcp,udp}.rs` 的 `block_on` 路径**全部迁移**进 worker 内 async 任务，外部 API 变为命令式。

### 3.2 传输状态机与 TLS（解决 T3，`runtime/src/transport/` [新增]）

- `Transport` trait（worker 内部）：connect/listen/send/recv/close + 状态机 `Connecting → Open → Closing → Closed | Failed{reason}`（驱动契约 `NetConnectionState`）；状态变更一律产生 `ConnectionState` ingress。
- 重连：`ReconnectPolicy { base_delay, max_delay, jitter, max_attempts }`（指数退避），在 ConnectConfig 上声明。
- TLS：rustls（随 `net.http` feature 携带）；客户端默认 webpki roots + 契约 `NetCertificatePin` 校验，服务端证书经 `NetSecurityPolicy` 注入。

### 3.3 HTTP / WebSocket（features/http、features/websocket [深化]）

- HTTP 客户端：hyper 之上的 `HttpRequestHandle`（async → `HttpResponse` ingress 完成事件），支持 range request（content_download 复用）。
- HTTP 服务端：现 `backend/server.rs` 路由注册模型保留（`http_route_registered` 事件），handler 为字节级回调（VM/Rust 均可挂载）。
- WebSocket：tungstenite over Tokio；现 connection/handshake/reader 收编进 worker 任务，frame 统一走 ingress/egress。

### 3.4 会话与 RPC（解决 T7，features/rpc [深化]）

- `Session`：握手控制消息（**字节格式定稿**：magic `u32` + 协议版本 `u16` + 能力位 `u64` + token 长度前缀字节串）→ `SessionId`；channel 多路复用（每消息头 `channel_id: u8` + `flags: u8`，reliable-ordered / unreliable 两类；TCP 上退化为单 channel）。
- RPC（契约 `rpc.rs` 现 DTO 扩展）：`register_rpc(RpcDescriptor { id, direction: ClientToServer|ServerToClient|Bidirectional, payload_schema })`，方向在 dispatch 期校验；调用语义 fire-and-forget 与 request-response（关联 id `u32` + 超时）；payload 为序列化字节，schema 来自 [08 ZrVM](08-zr-vm.md) 反射描述或 Rust serde——**与 08 共用一份 schema 描述，禁止第二套**。

### 3.5 状态复制（解决 T4，features/replication [深化]）

- `NetworkIdentity` 组件：网络对象 id + `SyncAuthority`（契约现枚举）。
- 注册期：`SyncComponentDescriptor`（契约现 DTO）编译为 dense 复制表（组件类型 → 序列化器 + 策略 OnChange/Interval/Once）。
- `net.replication_collect` ∈ PostUpdate（01 锚点表）：ECS change detection 收集 dirty 组件 → `SyncDelta`（契约现 DTO）→ channel 发送，预算受 `SyncReplicationBudget`（现 DTO）限制；`net.replication_apply` ∈ PreUpdate：应用 delta（transform 类组件默认 100ms 插值缓冲窗口）。
- v1 明确不做（维持既定边界）：NAT 穿透/STUN/TURN、matchmaking、反作弊、生产级账号体系。

### 3.6 可靠 UDP（解决 T5，features/reliable_udp [深化]）

包头格式定稿（小端）：

```
seq: u16 | ack: u16 | ack_bits: u32 | channel: u8 | flags: u8 (FRAGMENT|LAST_FRAGMENT) | [frag_id: u16, frag_index: u8, frag_count: u8]
```

- ack 位图滑窗重发（现 `resend.rs` 扩展）、fragment/reassembly（MTU 1200 保守值）、per-connection 重发带宽预算；接口对齐 `Transport` trait，session/replication 无感切换。

### 3.7 内容下载与 zrpack 共享 DTO（解决 T6，features/content_download [深化]）

与 [09 发行](09-export-publishing.md) §3.3 共享同一格式（**DTO 定义在契约 `framework/net/download.rs` [改造]，双方引用，先定 DTO 再各自实现**）：

```rust
pub struct ZrPackManifest { pub version: u32, pub chunks: Vec<ZrChunkEntry>, pub total_size: u64 }
pub struct ZrChunkEntry  { pub hash: [u8; 32] /* blake3 */, pub offset: u64, pub size: u32 }
```

- 下载：manifest → range request 并行拉取 → 本地 chunk 位图持久化（`.partial` 旁文件，断点续传）→ blake3 校验失败重拉 → 合入；进度经事件总线供 Hub/Editor 显示。

## 4. 模块文件树

```
zircon_runtime/src/core/framework/net/download.rs   [改造] ZrPackManifest/ZrChunkEntry（与 09 共享）
zircon_plugins/net/runtime/src/
  worker/{mod,ingress,egress,shutdown}.rs   [新增] NetWorker/SPSC 队列/关停协议
  transport/{mod,state_machine,reconnect,tls}.rs  [新增] Transport trait/backoff/rustls
  service_types/{tcp,udp}.rs                [改造] block_on 路径迁入 worker，API 命令化
  lib.rs                                    [改造] net.poll_ingress/flush_egress 注册 + register_event
zircon_plugins/net/features/
  http/runtime/src/client.rs                [新增]；backend/server.rs [改造]
  websocket/runtime/src/backend/*           [改造] 收编进 worker 任务
  rpc/runtime/src/manager/{handshake,dispatch,session}.rs  [改造] 字节格式/方向校验/超时
  rpc/runtime/src/channel.rs                [新增] channel 多路复用
  replication/runtime/src/manager/{collect.rs [新增], apply.rs [新增], schedule.rs [改造]}
  reliable_udp/runtime/src/packet.rs        [新增]；manager/resend.rs [改造]
  content_download/runtime/src/manager/{bitmap.rs [新增], http_fetch.rs [改造]}
```

## 5. 里程碑与任务分解（沿用既有 M1–M7 编号）

### M1 NetWorker 基线

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M1-T1 | NetWorker + SPSC 队列 + 关停协议 | worker/* | — | `worker_shutdown_leaves_no_tasks` |
| M1-T2 | tcp/udp 迁入 worker；命令式 API | service_types/{tcp,udp}.rs | M1-T1 | `tcp_echo_round_trip_loopback`、`udp_echo_round_trip_loopback` |
| M1-T3 | poll_ingress/flush_egress 锚点 + register_event | lib.rs | 01-M1/M2、M1-T1 | `ingress_anchor_in_first_egress_in_last` |
| M1-T4 | 重连 backoff + 状态机驱动 | transport/* | M1-T2 | `reconnect_backoff_timing_sequence`、`state_changes_emit_events` |

#### M1 当前进度（2026-06-14）

- M1-T1 已落地：新增 `runtime/src/worker/{mod,ingress,egress,shutdown}.rs`，`NetWorker` 在插件自有线程内持有 Tokio runtime，并通过有界 `std::sync::mpsc::sync_channel` egress/ingress 队列执行 TCP/UDP 命令；`worker_shutdown_leaves_no_tasks` 覆盖 UDP/TCP listener 未关闭句柄统计、worker 关停状态和关停后的错误返回。
- M1-T2 已落地到基础 TCP/UDP：`service_types/{tcp,udp}.rs` 不再直接调用 `.block_on(...)`，bind/listen/connect/send/poll/close 转交 worker；既有 `net_runtime_manager_accepts_tcp_client_and_echoes_payloads` 与 `default_net_manager_sends_udp_packet_to_bound_socket` 继续覆盖 loopback 行为，`tcp_udp_service_paths_do_not_block_on_tokio_runtime` 增加源码结构守卫。
- M1-T3 已落地：`runtime_system.rs` 注册 `net.transport`、`net.poll_ingress`（`SystemStage::First`）、`net.flush_egress`（`SystemStage::Last`）和类型化 `NetEvent`；`plugin.toml` 与 runtime package manifest 同步声明 system set/anchors，`ingress_anchor_in_first_egress_in_last` 覆盖注册报告。
- M1-T4 已落地基础层：新增 `transport/{mod,reconnect,state_machine}.rs`，`ReconnectPolicy` 提供确定性指数退避序列，`TransportStateMachine` 负责 `Connecting/Open/Closing/Closed/Failed` 状态转换并产出 `ConnectionStateChanged` 事件；TCP worker 接入 connect/open/close/failed 状态事件。TLS policy/rustls 接入仍按 M2-T2 推进，不混入 M1。
- 验证记录：`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --locked` 仍被插件 workspace `Cargo.lock` 排序漂移阻止进入编译；本次使用锁文件备份保护的 `--offline` scoped 检查通过，并在还原锁文件后直接运行 warmed lib-test 二进制，`zircon_plugin_net_runtime` 18 个测试全通过。

### M2 HTTP / WebSocket / TLS

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M2-T1 | HTTP 客户端（hyper + range） | http/client.rs | M1 | `http_round_trip_against_local_hyper_server`、`range_request_returns_partial` |
| M2-T2 | rustls 接入（客户端 roots/pin、服务端注入） | transport/tls.rs | M2-T1 | `self_signed_cert_rejected_then_pinned_accepted` |
| M2-T3 | WebSocket 收编 worker | websocket/* | M1 | `ws_frame_order_preserved` |

#### M2 当前进度（2026-06-14）

- M2-T1 已落地到 HTTP feature：`features/http/runtime/src/backend/client.rs` 对 `http://` 请求走 hyper client，保留现有 reqwest/rustls HTTPS 路径给 M2-T2 收束；`Cargo.toml` 打开 hyper/hyper-util client 特性和 Tokio time，用统一超时包裹 hyper 请求。
- M2-T1 Range 契约已落地：`NetHttpRequestDescriptor::with_byte_range(start, end_inclusive)` 生成并替换标准 `Range: bytes=start-end` 请求头，`content_download/runtime/src/manager/http_fetch.rs` 改为复用该契约入口，避免下载层继续手写 Range 头格式。
- M2-T1 测试已补齐：HTTP feature 测试树新增计划命名覆盖 `http_round_trip_against_local_hyper_server` 与 `range_request_returns_partial`；内容下载既有 `content_download_manager_fetches_resumed_http_range_with_existing_prefix` 继续覆盖上层 Range 续传复用。
- 验证记录：使用锁文件备份保护的 `--offline` 检查通过 `zircon_plugin_net_http_runtime --tests` 与 `zircon_plugin_net_content_download_runtime --tests`；直接运行 warmed 测试二进制分别通过 HTTP feature 10 个测试和 content_download 13 个测试。`zircon_runtime` 公共契约焦点测试构建两次超时于冷目标目录编译/链接阶段，已停止残留进程并还原根/插件锁文件，未声明该测试执行通过。
- M2-T2 已落地到共享 TLS helper 与 HTTP feature：`NetSecurityPolicy` 增加自定义 root DER 列表，`runtime/src/transport/tls.rs` 提供 certificate SHA-256 pin 计算、pin 匹配、rustls root store、client config 和 `TlsServerIdentity`/server config 注入；HTTP HTTPS 路径开启 `tls_info`，pinning 时允许握手拿证书但在读取响应体前强制校验证书 SHA-256。
- M2-T2 测试已补齐：`self_signed_cert_rejected_then_pinned_accepted` 使用内联测试证书和 `tokio-rustls` 本地 HTTPS fixture，证明未 pin 的自签证书被 rustls 拒绝，配置正确 host pin 后可返回 `tls-ok`；HTTP feature warmed 测试二进制 11 个测试全通过。WSS 连接路径未纳入本次 HTTP/TLS 切片。
- M2-T3 已落地到 WebSocket feature 发送侧：`backend/connection.rs` 以 `tokio::sync::mpsc` 有界队列衔接 `send_websocket_frame`，客户端和服务端连接创建时把 Tungstenite sink 移入 runtime writer task；调用线程只负责入队，close frame 入队后同步标记 `Closing`，不再在发送路径执行 `runtime.block_on(...)`。
- M2-T3 测试已补齐：`ws_frame_order_preserved` 覆盖真实 WebSocket 握手后的双向多帧顺序，`websocket_connection_send_path_is_queue_driven` 源码守卫防止发送路径退回调用线程 `block_on`；本切片 `rustfmt --edition 2021 --check`、冲突标记/行尾空白扫描和发送路径防回退扫描已通过。锁文件备份保护的 `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_websocket_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never` 被活跃渲染会话持有的 `zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs` 初始化缺 `has_rolled_previous_transform` 字段阻塞于依赖编译阶段，根/插件锁文件已还原，未声明 WebSocket Cargo 通过。

### M3 Session / RPC

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M3-T1 | 握手字节格式 + 版本/能力协商 | rpc/manager/handshake.rs | M1 | `handshake_version_mismatch_rejected` |
| M3-T2 | channel 多路复用 | rpc/channel.rs | M3-T1 | `channels_isolate_message_order` |
| M3-T3 | RPC 方向校验/超时/关联 id；schema 与 08 对齐 | rpc/manager/dispatch.rs | M3-T1、08-M1（schema DTO） | `wrong_direction_rpc_rejected`、`request_response_timeout_fires` |

### M4 Replication

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M4-T1 | dense 复制表编译 + NetworkIdentity | replication/* | 01-M2 | `replication_table_compiles_from_descriptors` |
| M4-T2 | collect（change detection → SyncDelta）/apply 闭环 | collect.rs、apply.rs | M4-T1、M3-T2 | `dual_world_replicates_spawn_update_despawn` |
| M4-T3 | 插值缓冲 + 预算 | apply.rs、schedule.rs | M4-T2 | `interpolation_window_smooths_updates`、`budget_caps_bytes_per_tick` |

### M5 Reliable UDP

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M5-T1 | 包头/ack 位图/滑窗重发 | reliable_udp/packet.rs、resend.rs | M1 | `thirty_percent_loss_delivers_in_order` |
| M5-T2 | fragment/reassembly + 带宽预算 | packet.rs | M5-T1 | `oversize_payload_fragments_and_reassembles` |

### M6 Content Download

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M6-T1 | ZrPackManifest 共享 DTO（与 09 定稿） | framework/net/download.rs | — | DTO round-trip 测试 |
| M6-T2 | 位图断点续传 + blake3 校验重拉 | bitmap.rs、http_fetch.rs | M2-T1、M6-T1 | `interrupted_download_resumes_from_bitmap`、`corrupt_chunk_refetched` |

### M7 Editor / 导出

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M7-T1 | Network authoring 面板（listener/route/replication schema 配置）+ 连接诊断实时视图（布局 `ai-console-diagnostics-layout.png`） | net editor | [10 规范](10-editor-integration.md) | editor 契约测试 |
| M7-T2 | 带宽/延迟指标进 rolling diagnostics store | runtime、features | M1 | `diagnostic_paths_registered` |

## 6. 验收命令

```bash
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --locked
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --all-features --locked
```

## 7. 风险

- M1 的 block_on → worker 迁移改变全部现有调用方语义（同步返回 → 命令+事件）；features 六 crate 同窗口适配，借既有 loopback 测试兜底。
- replication 依赖 01-M1/M2 的 change detection 访问声明与事件总线；RPC/replication 的 schema 序列化器与 ZrVM 反射统一（避免两套 schema）必须在 M3 前与 [08](08-zr-vm.md) 对齐 DTO。
- rustls/hyper/tungstenite 版本组合锁定在 workspace 层，避免 features 间依赖漂移。

## 8. 附录 · dev 参考源码对位

实现各任务前**必须先读对应参考实现**，复制语义与可靠 UDP 协议细节对照真实代码核对，禁止凭空实现：

| 设计点 | 参考源码（已核验存在） | 看什么 |
|--------|----------------------|--------|
| 状态复制（authority/relevancy/delta） | `dev/UnrealEngine/Engine/Source/Runtime/Net/`、`Networking/` | 属性复制的 dirty 收集与条件复制、relevancy 裁剪形态（我们 v1 只取 OnChange/Interval/Once 子集） |
| 可靠 UDP（ack 位图/fragment/channel） | `dev/godot/modules/enet/`（及其 vendored ENet 源码） | seq/ack 滑窗、fragment 重组、channel 隔离——§3.6 包头设计的判例 |
| WebSocket 服务端/客户端生命周期 | `dev/godot/modules/websocket/` | 握手、分帧、关闭码处理 |
| TLS 接入形态 | `dev/godot/modules/mbedtls/` | 证书校验策略注入点（我们用 rustls，仅取结构形态） |
| WebRTC 形态（后续池参考，不在 v1） | `dev/godot/modules/webrtc/` | data channel 抽象与 Transport 对齐方式 |
