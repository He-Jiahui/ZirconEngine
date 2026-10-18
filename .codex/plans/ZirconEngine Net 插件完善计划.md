# ZirconEngine Net 插件完善计划

## Summary

- 采用你选择的 **分层里程碑** 与 **Tokio 统一网络栈**：`net` 保持独立插件，底层 TCP/UDP/HTTP(S)/WebSocket 先稳定，再往上推进 RPC、状态同步、可靠 UDP、内容下载/CDN。
- 归属边界固定：共享契约在 `zircon_runtime::core::framework::net`，稳定访问入口在 `core::manager::NetManagerHandle`，真实实现放在 `zircon_plugins/net/runtime`，编辑器工具放在 `zircon_plugins/net/editor`。
- Unreal 为主参考，Mirror 为易用层参考：复刻 NetDriver/Connection/Channel、ControlMessage、RPC 防护、ReplicationGraph/Iris、PacketHandler、HTTP/WebSocket、BuildPatch 下载等能力；Mirror 的 NetworkBehaviour/Commands/ClientRpc/SyncVar/Interest Management 转译成 Zircon 的 descriptor + component + operation 体系。

## Key Changes

- **连接底座**
  - 把当前 UDP-only `NetManager` 扩展为 Tokio 驱动的 `NetRuntimeManager`：管理 runtime shutdown、listener、connection、poll budget、event queue、diagnostics。
  - 新增 role/mode：`DedicatedServer`、`Client`、`ListenServer`；`server_runtime` 允许监听和权威 session，`client_runtime` 允许主动连接和下载，`editor_host` 只做 authoring/diagnostics。
  - 新增 typed handles/DTO：`NetListenerId`、`NetConnectionId`、`NetSessionId`、`NetRequestId`、`NetDownloadId`、`NetTransportKind`、`NetConnectionState`、`NetEvent`、`NetSecurityPolicy`。

- **Tokio 依赖策略**
  - 基础插件统一使用 `tokio` 的 `rt-multi-thread/net/time/sync/io-util` 能力；TCP/UDP 进入基础 net runtime。
  - HTTP(S) 使用 `reqwest` 做 client/download，`hyper`/`hyper-util` 做 server/router 底层；TLS 默认走 rustls 系列。
  - WebSocket 使用 `tokio-tungstenite`；可靠 UDP/KCP 作为 `net.transport.kcp` 可选功能，优先验证 `tokio_kcp`，失败时保留同一 `ReliableDatagramTransport` 契约替换实现。

- **可选功能包**
  - 在 `plugin.toml` 和 `PluginFeatureBundleManifest` 中声明：
    `net.http`, `net.websocket`, `net.rpc`, `net.replication`, `net.reliable_udp`, `net.content_download`。
  - 每个功能提供明确 capability，例如 `runtime.feature.net.rpc`、`runtime.feature.net.replication`、`runtime.feature.net.cdn_download`，禁用后 runtime 服务和 editor UI 都必须消失。
  - 不复制 Unity/Mirror/Unreal 代码，只复刻架构语义。

- **上层网络能力**
  - RPC：提供 `RpcDescriptor`、`RpcDirection::{ClientToServer, ServerToClient, TargetClient}`、schema 校验、权限检查、频率/耗时/队列限额，参考 Unreal `MessagingRpc` 和 `RPCDoSDetection`。
  - 同步：提供 `SyncComponentDescriptor`、dirty tracking、snapshot/delta、interest graph、priority/frequency budget；参考 Unreal Iris/ReplicationGraph 和 Mirror SyncVar/Interest Management。
  - 可靠 UDP：提供 sequence/ack、fragment/reassembly、resend budget、RTT/loss stats、乱序/丢包测试；不把“可靠包”塞进基础 UDP API。
  - 下载/CDN/DCC：提供 manifest、chunk hash、range request、resume、mirror failover、cache validation、progress event；参考 Unreal BuildPatchServices。这里默认把 DCC 理解为内容制作/资源管线下载，不是 P2P 直连。

## Milestones

1. **M0 协调与基线**
   - 等当前 optional-feature/plugin-catalog 活跃会话稳定，或在实现前先合并其接口变化。
   - 记录 net 计划来源，确认当前 dirty workspace 不被回滚。
   - 验证现有 `net` UDP loopback 测试仍是底线，不删旧能力。

2. **M1 Tokio Net Runtime Foundation**
   - 建立 Tokio worker/service lifecycle，保持 `framework` 契约中性。
   - 扩展 TCP/UDP listener + connection handle、server/client/listen role、结构化事件。
   - 当前 `bind_udp/send_udp/poll_udp/close_socket` 继续兼容，但内部迁到新 transport 表。

3. **M2 HTTP(S) + WebSocket**
   - 增加 HTTP client/server route、request/response DTO、TLS policy、timeout/retry/proxy 配置。
   - 增加 WebSocket client/server、text/binary frame、close reason、message budget。
   - 用 optional features 控制编译、注册、导出和 editor capability。

4. **M3 Session / Handshake / RPC**
   - 建立 Unreal 风格 handshake/control message：hello/challenge/login/welcome/netspeed/failure/join 的 Zircon 版本。
   - 增加 RPC registry、direction 校验、authority gate、DoS quota、no-handler diagnostics。
   - 提供 Mirror 风格易用 façade：Command、ClientRpc、TargetRpc 对应 schema descriptor，不引入 C# attribute 模式。

5. **M4 Replication / Sync**
   - 建立 network object id、replication schema、dirty component snapshot、delta compression slot、interest graph。
   - Server authoritative 默认；client authority 必须按 component 显式声明。
   - 增加 transform/physics-friendly sync hook，但不把 physics/sound/animation 强耦合进 net。

6. **M5 Reliable UDP / KCP**
   - 在 `net.reliable_udp` 下实现 KCP 或同契约可靠 datagram backend。
   - 覆盖 packet loss、jitter、reorder、fragment、ack timeout、resend cap、disconnect recovery。
   - 基础 UDP 继续保持不可靠、低开销、可直接访问。

7. **M6 Content Download / CDN / DCC**
   - 增加 resource download manifest、chunked download、hash 校验、断点续传、CDN mirror fallback。
   - 下载结果只进入 resource/asset contract，不让 net 插件直接拥有 asset manager。
   - editor 增加下载诊断、缓存状态和失败重试操作。

8. **M7 Editor + Packaging + Docs**
   - 完善 Network authoring pane：连接 profiles、server/client launch preset、route/RPC/replication/download diagnostics。
   - 导出时只链接启用的 net feature crates；server/client/editor target 不匹配时给 fatal/warn 诊断。
   - 更新 `docs/engine-architecture/runtime-network-extension.md` 和对应 `docs/zircon_plugins/net/...` 模块文档。

## Test Plan

- **Manifest/catalog**：net base 和所有 optional features 的依赖、capability、target mode、导出链接开关都可被测试捕获。
- **连接回归**：UDP loopback、TCP echo、server accept/client connect、close/reconnect、poll budget、unknown handle、invalid endpoint。
- **HTTP/WebSocket**：HTTP route bind/unbind、HTTPS policy、client timeout/retry、WebSocket text/binary/close、禁用 feature 后 API 不注册。
- **RPC/session**：handshake happy path、version mismatch、challenge failure、unauthorized RPC、quota block、missing handler。
- **Replication**：dirty-only sync、interest culling、priority/frequency budget、late join snapshot、entity despawn、client authority denial。
- **Reliable UDP**：模拟 5-30% 丢包、乱序、重复包、fragment 超限、resend 上限、RTT 统计。
- **Download/CDN**：chunk hash failure、range resume、mirror fallback、cache hit/miss、partial manifest、download cancellation。
- **Validation commands**：root workspace `cargo check/test --workspace --locked`；插件 workspace `cargo check/test --manifest-path zircon_plugins/Cargo.toml --workspace --locked`；CI 对齐现有 GitHub Actions build/test。

## Assumptions

- 当前工作继续在 `main`，不创建分支/ worktree。
- 首轮不是一次性完成全部多人系统，而是按 M0-M7 分层推进；每个 milestone 有实现切片、测试阶段、修复循环和文档更新。
- Tokio 为统一 async runtime；HTTP/WebSocket/TLS/KCP 等连接库允许作为 optional feature dependencies 引入。
- v1 不做 NAT traversal/STUN/TURN、matchmaking、anti-cheat、生产级账号认证；这些以后作为 `net.online_services` 或平台插件补齐。
- 参考依据包括本地 Unreal 网络源码、当前 net 基线文档，以及最新库文档： [Tokio](https://docs.rs/tokio/latest/tokio/), [hyper](https://docs.rs/hyper/latest/hyper/), [reqwest](https://docs.rs/reqwest/latest/reqwest/), [tokio-tungstenite](https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/), [tokio-rustls](https://docs.rs/tokio-rustls/latest/tokio_rustls/), [Mirror README](https://github.com/MirrorNetworking/Mirror)。
