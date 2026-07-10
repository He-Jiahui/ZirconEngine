# 07-net 产出记录归档

> 来源：[`07-net.md`](../07-net.md) 的 `## 状态与产出记录`。

## 状态与产出记录

| 日期 | 任务 | 状态 | 产出文件/行为 | 验证记录 | 后续 |
|------|------|------|---------------|----------|------|
| 2026-06-14 | M1-T1~M1-T4 NetWorker 基线 | 已落地 | `worker/*`、`service_types/{tcp,udp}.rs`、`runtime_system.rs`、`transport/{reconnect,state_machine}.rs`；TCP/UDP 从调用线程 `block_on` 收束到 worker 命令和状态事件 | `zircon_plugin_net_runtime --tests --offline` scoped check 通过；warmed 测试二进制 18/18 通过；`--locked` 仍受 `zircon_plugins/Cargo.lock` 排序漂移阻塞 | 等锁文件刷新明确纳入范围后再跑插件 workspace locked 验证 |
| 2026-06-14 | M2-T1 HTTP client + Range | 已落地 | HTTP `http://` 走 hyper client；`NetHttpRequestDescriptor::with_byte_range(...)` 统一 Range 头；content_download 复用 Range 契约 | HTTP feature 10/10、content_download 13/13 warmed 测试通过；framework DTO 焦点测试冷链接超时，未声明通过 | 后续共享 DTO 改造进入 M6 |
| 2026-06-14 | M2-T2 TLS roots/pin | 已落地 | `NetSecurityPolicy` 自定义 root DER；`transport/tls.rs` rustls helper、SHA-256 pin、`TlsServerIdentity`；HTTP HTTPS pin 校验在读 body 前执行 | `zircon_plugin_net_runtime --tests` 与 `zircon_plugin_net_http_runtime --tests` scoped check 通过；HTTP feature 11/11 warmed 测试通过 | WSS 连接路径未纳入本切片 |
| 2026-06-14 | M2-T3 WebSocket send worker | 已落地 | `features/websocket/runtime/src/backend/connection.rs` 用 bounded Tokio `mpsc` 队列和 writer task 接管 Tungstenite sink；新增 `ws_frame_order_preserved` 与发送路径源码守卫 | rustfmt/冲突标记/尾随空白/发送路径防回退扫描通过；`zircon_plugin_net_websocket_runtime --tests --offline` scoped check 通过；`ws_frame_order_preserved` 和 `websocket_connection_send_path_is_queue_driven` focused tests 通过；早期合并长命令超时后已清理目标目录进程并还原锁文件 | M2 WebSocket 发送侧完成；WSS 仍不在本切片 |
| 2026-06-14 | M3-T1 RPC handshake byte frame | 已落地 | `NetRpcHandshakeFrame` 实现 magic `ZRPC`、u16 version、u64 capability bits、u16 token length + token 编解码，并映射到现有 Hello 状态机；新增 `handshake_version_mismatch_rejected` | `zircon_plugin_net_rpc_runtime --tests --offline` scoped check 通过；直接运行 warmed RPC 测试二进制通过 `rpc_handshake_frame_round_trips_magic_version_capabilities_and_token` 与 `handshake_version_mismatch_rejected`；cargo 外层双测试命令超时后已清理目标目录进程 | 推进 M3-T2 channel 多路复用 |
| 2026-06-14 | M3-T2 RPC channel 多路复用 | 已落地 | 新增 `manager/channel.rs`、`RpcChannelMessage`、`RPC_CHANNEL_RELIABLE_ORDERED`、`RPC_CHANNEL_UNRELIABLE`，按 `u8 channel_id` 分离队列和 per-channel sequence；新增 `channels_isolate_message_order` | `zircon_plugin_net_rpc_runtime --tests --offline` scoped check 通过；`channels_isolate_message_order` focused test 通过；根/插件锁文件已还原 | 推进 M3-T3 RPC 方向/超时/关联 id 与 schema 收束 |
| 2026-06-14 | M3-T3 RPC dispatch 方向/超时/schema | 已落地 | `RpcDirection::Bidirectional`、`RpcPayloadSchema` 反射 schema 请求 DTO、`allows_invocation(...)` 方向判定；RPC dispatch schema validator 继续按 schema id 查表但来源收束到 08 `ReflectSchemaRequest`；新增 `wrong_direction_rpc_rejected`、`request_response_timeout_fires`、schema/bidirectional 覆盖 | `zircon_plugin_net_rpc_runtime --tests --offline` scoped check 通过；`wrong_direction_rpc_rejected` cargo focused test 通过；直接运行 warmed RPC 测试二进制通过 `request_response_timeout_fires`、`rpc_payload_schema_uses_reflect_schema_request`、`bidirectional_rpc_accepts_valid_client_and_server_calls`；`zircon_plugin_net_runtime` descriptor focused test 通过；`zircon_runtime` framework focused test 在依赖编译阶段两次进程级 `-1`/`1` 中断且无 Rust 诊断，未声明通过；锁文件保持未改 | M3 进入测试收束/后续可推进 M4 replication |
| 2026-06-14 | M4-T1 dense replication table + NetworkIdentity | 已落地；后续 M4-T2 check 已覆盖 test 编译 | `NetworkIdentity`、`SyncReplicationStrategy`、`SyncComponentDescriptor::replication_strategy`；新增 `replication/manager/table.rs`、`NetReplicationTable`、`NetReplicationTableEntry`、`compile_replication_table()`，按 component type 稳定排序生成 dense index；新增 `replication_table_compiles_from_descriptors` | 初次 `zircon_plugin_net_replication_runtime --tests --offline` scoped check 被渲染侧 motion-vector 漂移阻断；后续 M4-T2 同 crate `--tests --offline` scoped check 通过并还原锁文件 | focused test 执行证据并入后续 M4 testing stage |
| 2026-06-14 | M4-T2 collect/apply spawn-update-despawn 闭环 | 已落地，focused test 受外部 UI 编译错误阻塞 | `SyncDelta` 增加向后兼容 tombstone 标记与 `despawn(...)`；新增 `replication/manager/{collect,apply}.rs`，支持快照变化收集、despawn delta、按 sequence 合并/忽略过期 delta；新增 `dual_world_replicates_spawn_update_despawn` | rustfmt 通过；`zircon_plugin_net_replication_runtime --tests --offline` scoped check 通过并还原锁文件；focused cargo test 被 `zircon_runtime` UI `tree_view`/`BeginEdit` 编译漂移阻塞，未进入 replication 测试体；根/插件锁文件保持未改 | 等 UI 编译恢复后在 M4 testing stage 重跑 focused test |
| 2026-06-14 | M4-T3 interpolation window + byte budget alias | 已落地，Cargo 验证受外部渲染/蒙皮编译错误阻塞 | `manager/apply.rs` 增加 Transform 默认 100ms 插值样本缓存、`apply_delta_at(...)`、`interpolated_f32_field(...)`；`manager/state.rs` 保存插值样本；despawn 清理插值缓存；新增 `interpolation_window_smooths_updates` 与 `budget_caps_bytes_per_tick` | rustfmt 已运行；`zircon_plugin_net_replication_runtime --tests --offline` 超时无诊断；`--lib --offline` 在 `zircon_runtime` 渲染/蒙皮私有导出漂移处失败（`SKINNED_MESH_MAX_JOINT_MATRICES` 私有 re-export、`gpu_scene::prev_skinned_palette` 访问私有 `mesh`），未进入 replication crate；根/插件锁文件保持未改 | 等渲染/蒙皮编译恢复后重跑 M4-T3 check/test，再进入 M5 reliable_udp |
| 2026-06-14 | M5-T1 Reliable UDP 包头/ack 位图/滑窗重发 | 已落地，scoped check 通过；测试执行仍待重跑 | 新增 `reliable_udp/packet.rs`，实现 plan header `seq/ack/ack_bits/channel/flags` 小端编解码、可选 fragment header 与 ack 位图展开；`resend.rs` 支持按 wire header 批量 acknowledge；`receive.rs` 增加按 sequence 顺序交付缓冲；新增 `reliable_udp_wire_packet_round_trips_header_ack_bitmap_and_fragment`、`reliable_udp_wire_ack_matches_pending_window_after_u16_wrap` 与 `thirty_percent_loss_delivers_in_order` | rustfmt --check、path-scoped `git diff --check`、冲突标记和尾随空白扫描通过；`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --tests --offline` 通过并还原锁文件；整包测试与 `reliable_udp_wire` focused test 均在编译/链接阶段超时无测试输出，已清理本轮 reliable_udp cargo/rustc 遗留进程；根/插件锁文件保持未改 | 等 Cargo lanes 空闲后重跑 reliable_udp focused tests，再推进 M5-T2 fragment/budget |
| 2026-06-14 | M5-T2 fragment/reassembly + 重发带宽预算 | 已落地，scoped check 通过；测试执行仍待重跑 | 复用既有 MTU 分片与乱序重组路径，新增计划命名 `oversize_payload_fragments_and_reassembles`；`resend.rs` 增加 `resend_due_with_byte_budget(...)`，按当前 tick payload byte budget 选择 due sequence 并保留未发送项到后续 tick；新增 `resend_due_with_byte_budget_caps_payload_bytes_per_tick` | rustfmt --check、path-scoped `git diff --check`、冲突标记和尾随空白扫描通过；`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --tests --offline` 通过并还原锁文件；测试执行沿用 M5-T1 当前超时缺口，未声明 pass | 等 Cargo lanes 空闲后重跑 M5-T1/M5-T2 focused tests，再进入 M6 content_download |
| 2026-06-14 | M6-T1 ZrPackManifest 共享 DTO | 已落地，scoped check 通过 | `framework/net/download.rs` 新增 `ZrPackManifest` 与 `ZrChunkEntry`，`framework/net/mod.rs` re-export；framework DTO 测试覆盖 serde round-trip、chunk end offset 与覆盖字节数 | rustfmt --check、path-scoped `git diff --check`、冲突标记和尾随空白扫描通过；`zircon_plugin_net_content_download_runtime --lib/--tests --offline` scoped check 通过并还原根/插件锁文件 | 与 09 发行侧对接时继续复用同一 DTO |
| 2026-06-14 | M6-T2 位图断点续传 + hash 校验重拉 | 已落地；运行时暂沿用既有 SHA-256 hash 路径，blake3 依赖待锁文件刷新范围明确 | 新增 `content_download/manager/bitmap.rs`；state/progress 维护 chunk 位图；`http_fetch.rs` 在 hash mismatch 后记录失败尝试并可切换 mirror 重拉；新增 `interrupted_download_resumes_from_bitmap`、`corrupt_chunk_refetched` | rustfmt/静态扫描通过；`zircon_plugin_net_content_download_runtime --lib` 与 `--tests --offline` scoped check 通过；focused test 执行 304s 超时无测试输出，已清理本目标目录 content_download 进程并保持锁文件未改 | 等 Cargo lanes 空闲后重跑 M6 focused tests；若要改为真实 blake3 运行时校验，需要单独纳入依赖与锁文件刷新 |
| 2026-06-14 | M7-T1 Network authoring + diagnostics 入口 | 已落地，scoped check 曾通过；最终重跑受外部 UI 文件阻塞 | 新增 `net/editor/src/authoring.rs`；注册 `net.authoring` 与 `net.diagnostics` 两个 view，listener/route/replication schema 配置 operation、component drawer、replication schema asset template、graph editor 和 palette；测试扩展 `net_editor_plugin_contributes_authoring_extensions` | rustfmt --check、path-scoped `git diff --check` 通过；`zircon_plugin_net_editor --tests --offline` 在本切片首次检查通过并还原锁文件；后续断言路径修正后重跑 Cargo 被未跟踪 `zircon_runtime/src/ui/surface/surface/default_interactions/tree_view.rs` 重复定义/缺失符号阻塞，未进入 Net editor 测试体；根/插件锁文件保持未改 | 等 UI tree_view 外部漂移恢复后重跑 Net editor focused check/test；实际 `.zui` 渲染接入由 UI lane 挂到已注册贡献 |
| 2026-06-14 | M7-T2 带宽/延迟 rolling diagnostics store | 已落地，scoped check 曾通过；最终重跑受外部 UI 文件阻塞 | `NetDiagnostics` 增加 outbound/inbound bytes 与 last latency；`NetRuntimeState` 记录原子计数；UDP/TCP/HTTP/WebSocket send/poll 路径更新计数；`runtime_system.rs` 导出 `NET_DIAGNOSTIC_*` 路径并写入 `CoreHandle::record_diagnostic(...)`；新增 `diagnostic_paths_registered`、`net_runtime_diagnostics_records_bandwidth_counters` | `zircon_plugin_net_runtime --tests --offline` 首次 scoped check 通过并还原锁文件；后续清理后重跑在外部未跟踪 UI `tree_view.rs` 编译错误处停止；rustfmt/diff-check 仍通过，锁文件保持未改 | 等 UI tree_view 外部漂移恢复后重跑 `zircon_plugin_net_runtime --tests` 与 focused diagnostics tests |

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
- M2-T3 测试已补齐并通过：`ws_frame_order_preserved` 覆盖真实 WebSocket 握手后的双向多帧顺序，`websocket_connection_send_path_is_queue_driven` 源码守卫防止发送路径退回调用线程 `block_on`；本切片 `rustfmt --edition 2021 --check`、冲突标记/行尾空白扫描、发送路径防回退扫描、`zircon_plugin_net_websocket_runtime --tests --offline` scoped check、两个 focused tests 均通过。早期合并长命令曾超时，目标目录残留进程已清理且根/插件锁文件已还原。

### M3 Session / RPC

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M3-T1 | 握手字节格式 + 版本/能力协商 | rpc/manager/handshake.rs | M1 | `handshake_version_mismatch_rejected` |
| M3-T2 | channel 多路复用 | rpc/channel.rs | M3-T1 | `channels_isolate_message_order` |
| M3-T3 | RPC 方向校验/超时/关联 id；schema 与 08 对齐 | rpc/manager/dispatch.rs | M3-T1、08-M1（schema DTO） | `wrong_direction_rpc_rejected`、`request_response_timeout_fires` |

#### M3 当前进度（2026-06-14）

- M3-T1 已落地到 RPC feature：`manager/handshake.rs` 新增 `NetRpcHandshakeFrame`，按计划固定二进制握手头为 magic `ZRPC`、协议版本 `u16`、能力位 `u64`、token 长度 `u16` 和 token 字节串；`RPC_HANDSHAKE_CAPABILITY_NET_RPC` 映射到现有 `runtime.feature.net.rpc` capability。
- M3-T1 复用现有状态机：`process_handshake_frame(...)` 解码字节帧后转为 `NetControlMessage::Hello`，版本不匹配和缺能力位继续走 `NetSessionHandshakePolicy` 的失败路径，避免并行维护两套握手状态。
- M3-T1 测试已补齐并通过：`rpc_handshake_frame_round_trips_magic_version_capabilities_and_token` 覆盖二进制帧 round-trip，`handshake_version_mismatch_rejected` 是计划命名的版本失败测试；当前 `zircon_plugin_net_rpc_runtime --tests` scoped check 已通过，直接运行 warmed RPC 测试二进制通过两个 focused tests。cargo 外层双测试命令曾超时，目标目录残留进程已清理且根/插件锁文件已还原。
- M3-T2 已落地到 RPC feature：新增 `manager/channel.rs`，以 `RpcChannelMessage { channel_id, flags, sequence, payload }` 固定 plan 中的 channel header 形态，`RPC_CHANNEL_RELIABLE_ORDERED` / `RPC_CHANNEL_UNRELIABLE` 表达可靠有序与非可靠 channel 标志；`NetRpcRuntimeState` 维护 per-channel 队列和 per-channel sequence。
- M3-T2 测试已补齐并通过：`channels_isolate_message_order` 交错写入两个 channel 后分别 drain，验证不同 channel 的 FIFO 顺序和 sequence 彼此隔离；当前 `zircon_plugin_net_rpc_runtime --tests` scoped check 与 focused test 均通过。
- M3-T3 已落地到公共 Net/RPC 契约和 RPC feature dispatch：`RpcDirection` 新增 `Bidirectional`，`RpcDirection::allows_invocation(...)` 用描述方向、实际调用方向和 caller role 统一校验，`manager/dispatch.rs` 不再用简单相等判断阻断双向 RPC。
- M3-T3 schema 已收束到 08 的反射 schema DTO：`RpcPayloadSchema` 持有 schema id 与 `ReflectSchemaRequest`，`RpcDescriptor::with_payload_schema(...)` 仍可接受现有 schema id 字符串，但会生成 `ReflectSchemaRequest::for_type(...)`；`with_reflect_payload_schema(...)` 可直接从 `ReflectTypePath` 构造，避免 RPC 自建第二套 schema 描述。
- M3-T3 测试已补齐并通过：`wrong_direction_rpc_rejected` 覆盖方向和调用方角色拒绝，`request_response_timeout_fires` 覆盖 request id + handler timeout 后不保留 pending request，`rpc_payload_schema_uses_reflect_schema_request` 和 `bidirectional_rpc_accepts_valid_client_and_server_calls` 覆盖 schema/双向方向；`zircon_plugin_net_rpc_runtime --tests` scoped check 与 `zircon_plugin_net_runtime` descriptor focused test 通过。`zircon_runtime` framework focused test 构建在依赖编译阶段返回进程级中断且无 Rust 诊断，本切片未声明框架测试执行通过。

### M4 Replication

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M4-T1 | dense 复制表编译 + NetworkIdentity | replication/* | 01-M2 | `replication_table_compiles_from_descriptors` |
| M4-T2 | collect（change detection → SyncDelta）/apply 闭环 | collect.rs、apply.rs | M4-T1、M3-T2 | `dual_world_replicates_spawn_update_despawn` |
| M4-T3 | 插值缓冲 + 预算 | apply.rs、schedule.rs | M4-T2 | `interpolation_window_smooths_updates`、`budget_caps_bytes_per_tick` |

#### M4 当前进度（2026-06-14）

- M4-T1 已落地到公共 Net sync DTO 与 replication feature：`NetworkIdentity { object, authority }` 固定网络对象身份与同步权威，`SyncReplicationStrategy::{OnChange, Interval, Once}` 进入 `SyncComponentDescriptor`，默认保持 OnChange，避免破坏既有 descriptor 构造路径。
- M4-T1 dense 表编译已落地：`manager/table.rs` 从已注册 `SyncComponentDescriptor` 生成 `NetReplicationTableEntry`，按 `component_type` 稳定排序并分配 dense index，保留 authority、strategy、fields、update frequency、priority、interest group，作为后续 collect/apply 调度和复制预算的静态表。
- M4-T1 测试已补齐：`replication_table_compiles_from_descriptors` 覆盖 NetworkIdentity、两个组件 descriptor、OnChange/Interval/Once strategy、dense index 顺序和字段/优先级/interest group 投影。早期 Cargo 验证曾被渲染侧 motion-vector 结构漂移阻塞；后续 M4-T2 scoped check 已覆盖 replication crate test 编译。
- M4-T2 collect/apply 闭环已落地到 replication feature：`manager/collect.rs` 将快照变化收集为 `SyncDelta`，并为 despawn 生成 tombstone delta；`manager/apply.rs` 按 sequence 在接收端创建/合并 `SyncObjectSnapshot`，忽略过期 delta，并在 tombstone 到达时删除 object/component snapshot。
- M4-T2 公共 DTO 已补齐：`SyncDelta` 增加 `despawned` serde-default 标记、`despawn(...)` 构造和 `is_despawn()` 查询，旧 `SyncDelta::new(...)` 默认保持非删除 delta，避免破坏现有序列化与测试构造路径。
- M4-T2 测试已补齐：`dual_world_replicates_spawn_update_despawn` 覆盖 source/replica 双 manager 的 spawn、update、过期 spawn delta 不回滚、despawn delta 传播和 late-join 快照清空。`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime --tests --offline` 已通过；focused test 执行目前被 UI `tree_view`/`BeginEdit` 编译漂移阻塞，未进入测试体。
- M4-T3 插值缓冲已落地到 replication apply 路径：Transform 类组件的 f32 字段在 `apply_delta_at(...)` 时记录接收时间样本，`interpolated_f32_field(...)` 以默认 100ms 延迟在相邻样本间线性插值；despawn delta、本地 despawn 和 collect despawn 均清理对应插值缓存。
- M4-T3 预算测试别名已补齐：`budget_caps_bytes_per_tick` 复用既有 `SyncReplicationBudget::max_bytes` 调度路径，明确锁住每 tick 字节预算只能发送预算内快照并把剩余快照计入 deferred。
- M4-T3 测试已补齐：`interpolation_window_smooths_updates` 覆盖 0ms/100ms Transform 更新在 150ms 渲染时按 100ms buffer 得到中点值，250ms 渲染时回到最新值；Cargo 验证当前被渲染/蒙皮私有导出漂移阻塞在 `zircon_runtime` 编译阶段，未进入 replication crate。

### M5 Reliable UDP

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M5-T1 | 包头/ack 位图/滑窗重发 | reliable_udp/packet.rs、resend.rs | M1 | `thirty_percent_loss_delivers_in_order` |
| M5-T2 | fragment/reassembly + 带宽预算 | packet.rs | M5-T1 | `oversize_payload_fragments_and_reassembles` |

#### M5 当前进度（2026-06-14）

- M5-T1 wire packet 已落地到 Reliable UDP feature：`packet.rs` 固定计划内包头布局 `seq: u16 | ack: u16 | ack_bits: u32 | channel: u8 | flags: u8`，按小端编码，并在 fragment flag 存在时追加 `fragment_id/index/count` 可选头。
- M5-T1 ack 位图路径已接入发送侧：`ReliableUdpWireHeader::acked_sequences()` 将当前 ack 与 32-bit 历史 ack 位展开为待确认序列，`NetReliableUdpRuntimeManager::acknowledge_wire_header(...)` 复用既有 outbound/resend 状态清理。
- M5-T1 顺序交付路径已接入接收侧：`receive_ordered_packet(...)` 在底层重组完成后按 sequence 暂存 payload，只在连续窗口完整时向上交付，`pending_ordered_payload_count()` 暴露测试用积压观测。
- M5-T1 测试已补齐：`reliable_udp_wire_packet_round_trips_header_ack_bitmap_and_fragment` 覆盖 wire header、fragment header 和 ack 位图；`reliable_udp_wire_ack_matches_pending_window_after_u16_wrap` 覆盖内部 `u64` sequence 超过 wire `u16` 后仍能确认当前窗口；`thirty_percent_loss_delivers_in_order` 用确定性 30% 丢包和 `resend_due(...)` 锁住滑窗重发后的顺序交付。
- M5-T1 格式、静态扫描和 `zircon_plugin_net_reliable_udp_runtime --tests --offline` scoped check 已通过；整包测试执行与 `reliable_udp_wire` focused test 在编译/链接阶段超时无测试输出，已清理本轮 reliable_udp cargo/rustc 遗留进程，根/插件锁文件保持未改。
- M5-T2 fragment/reassembly 的生产路径已确认由既有 `enqueue_reliable_datagram(...)` 与 `receive_packet(...)` 承担：发送端按 MTU 切分 `ReliableDatagramPacket::with_fragment(index, count)`，接收端用 `InboundFragmentAssembly` 支持乱序插入并在完整后合并 payload。
- M5-T2 重发带宽预算已落地：`resend_due_with_byte_budget(now_ms, max_payload_bytes)` 只重发本 tick 预算内的 due sequence，预算不足的待发项保留在 resend state，后续 tick 可继续发出，同时保持 resend attempt cap 和断线诊断路径不变。
- M5-T2 测试已补齐：`oversize_payload_fragments_and_reassembles` 覆盖 10-byte payload 在 4-byte MTU 下拆为 3 片并乱序重组；`resend_due_with_byte_budget_caps_payload_bytes_per_tick` 覆盖每 tick 4-byte 预算只发一个 4-byte payload、下一次同 timestamp 继续发送剩余 due packet。
- M5-T2 格式、静态扫描和 `zircon_plugin_net_reliable_udp_runtime --tests --offline` scoped check 已通过；测试执行仍沿用当前 reliable_udp test 编译/链接超时缺口，未声明 focused test pass。

### M6 Content Download

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M6-T1 | ZrPackManifest 共享 DTO（与 09 定稿） | framework/net/download.rs | — | DTO round-trip 测试 |
| M6-T2 | 位图断点续传 + blake3 校验重拉 | bitmap.rs、http_fetch.rs | M2-T1、M6-T1 | `interrupted_download_resumes_from_bitmap`、`corrupt_chunk_refetched` |

#### M6 当前进度（2026-06-14）

- M6-T1 共享 DTO 已落地到 `zircon_runtime::core::framework::net::download`：`ZrPackManifest` 固定 `version/chunks/total_size`，`ZrChunkEntry` 固定 `[u8; 32] hash/offset/size`，并提供 `covered_bytes()`、`is_complete_byte_plan()`、`end_offset()` 等确定性校验辅助。
- M6-T1 framework re-export 与测试已同步：`framework/net/mod.rs` 暴露 `ZrPackManifest` 和 `ZrChunkEntry`，`reliable_datagram_and_download_contracts_record_recovery_state` 覆盖 serde round-trip、chunk end offset、覆盖字节数与完整 byte plan 判断。
- M6-T2 断点续传位图已落地：新增 `manager/bitmap.rs`，显式 resume bitmap 优先于 progress 推导；progress 的 cache hit / chunk complete 路径会同步标记 bitmap，`apply_resume_bitmap(...)` 可按 manifest 顺序恢复已完成 chunk。
- M6-T2 hash mismatch 重拉已接入现有下载尝试模型：`http_fetch.rs` 在 chunk hash 不匹配时记录 failed attempt；存在 mirror 时保留下载态并切换到下一 mirror 重拉，尝试耗尽后仍保留原有失败诊断。
- M6-T2 测试已补齐：`interrupted_download_resumes_from_bitmap` 覆盖中断后按 bitmap 恢复完成 chunk，`corrupt_chunk_refetched` 覆盖首个 mirror 返回损坏 chunk 后从备用 mirror 重拉并完成。
- 依赖边界：计划 DTO 中的 hash 字段已按 `[u8; 32]` 形态固定；运行时校验本切片未引入 `blake3` 依赖，以避免改动根/插件锁文件，因此重拉路径暂沿用既有 SHA-256 字符串校验。若后续明确允许依赖和锁文件刷新，再把运行时 hash 实现切到 blake3。
- 验证记录：M6 touched files 的 rustfmt --check、path-scoped `git diff --check`、冲突标记和尾随空白扫描通过；`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime --lib --offline` 与 `--tests --offline` 均通过并还原锁文件；focused test 执行 304s 超时无测试输出，目标目录 content_download 进程已清理，未声明 focused test pass。

### M7 Editor / 导出

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M7-T1 | Network authoring 面板（listener/route/replication schema 配置）+ 连接诊断实时视图（布局 `ai-console-diagnostics-layout.png`） | net editor | [10 规范](10-editor-integration.md) | editor 契约测试 |
| M7-T2 | 带宽/延迟指标进 rolling diagnostics store | runtime、features | M1 | `diagnostic_paths_registered` |

#### M7 当前进度（2026-06-14）

- M7-T1 editor 注册契约已落地到 `zircon_plugins/net/editor/src/authoring.rs`：`NET_AUTHORING_SURFACES` 同时注册 `net.authoring` 与 `net.diagnostics`，后者作为连接诊断实时视图入口，布局参考锁定 `docs/ui-and-layout/ai-workbench-style/ai-console-diagnostics-layout.png`。
- M7-T1 listener/route/replication schema 配置入口已注册：`net.listener.configure`、`net.route.configure`、`net.replication_schema.{open,validate,compile,create}` operation 带 payload schema/capability；component drawer 指向 `plugins://net/editor/{listener_config,route_config,replication_schema}.zui`；replication schema asset template、graph editor、palette 节点已进入 editor extension registry。
- M7-T1 契约测试已扩展：`net_editor_plugin_contributes_authoring_extensions` 覆盖新增 diagnostics view、payload schema、component drawer、replication schema template、graph editor 与 palette。当前切片只实现注册数据面，不直接创建 `.zui` 渲染文件；实际模板渲染可由 UI/editor lane 复用这些已注册 id 接入。
- M7-T2 runtime diagnostics 已接入统一 rolling store：`NetDiagnostics` 新增 `outbound_bytes`、`inbound_bytes`、`last_observed_latency_ms`；UDP/TCP/HTTP/WebSocket send/poll 路径累计字节，HTTP request 记录最后一次延迟；`record_net_diagnostics(...)` 将 `net.bandwidth.*`、`net.latency.last_observed_ms`、连接数和 queued events 写入 `CoreHandle::record_diagnostic(...)`。
- M7-T2 测试已补齐：`diagnostic_paths_registered` 覆盖诊断路径、单位、当前值和 tag 投影；`net_runtime_diagnostics_records_bandwidth_counters` 覆盖 WebSocket loopback 的 outbound/inbound 字节计数。
- 验证记录：M7 touched Rust files 的 rustfmt --check、path-scoped `git diff --check` 通过；锁文件保持未改。`zircon_plugin_net_runtime --tests --offline` 与 `zircon_plugin_net_editor --tests --offline` 均曾在 M7 首轮实现后 scoped check 通过并还原锁文件；最终清理后重跑被外部未跟踪 `zircon_runtime/src/ui/surface/surface/default_interactions/tree_view.rs` 的重复定义/缺失符号编译错误阻塞，未进入 Net runtime/editor 测试体，因此最终 Cargo 重跑不声明 pass。

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
