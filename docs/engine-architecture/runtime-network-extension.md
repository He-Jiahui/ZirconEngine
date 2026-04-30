---
related_code:
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/net/mod.rs
  - zircon_runtime/src/core/framework/net/endpoint.rs
  - zircon_runtime/src/core/framework/net/error.rs
  - zircon_runtime/src/core/framework/net/manager.rs
  - zircon_runtime/src/core/framework/net/packet.rs
  - zircon_runtime/src/core/framework/net/socket_id.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_plugins/net/runtime/src/mod.rs
  - zircon_plugins/net/runtime/src/config.rs
  - zircon_plugins/net/runtime/src/module.rs
  - zircon_plugins/net/runtime/src/service_types.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
implementation_files:
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/net/mod.rs
  - zircon_runtime/src/core/framework/net/endpoint.rs
  - zircon_runtime/src/core/framework/net/error.rs
  - zircon_runtime/src/core/framework/net/manager.rs
  - zircon_runtime/src/core/framework/net/packet.rs
  - zircon_runtime/src/core/framework/net/socket_id.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_plugins/net/runtime/src/mod.rs
  - zircon_plugins/net/runtime/src/config.rs
  - zircon_plugins/net/runtime/src/module.rs
  - zircon_plugins/net/runtime/src/service_types.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
plan_sources:
  - user: 2026-04-21 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-21 Later Milestones / M2 基础子系统补齐
  - user: 2026-04-21 继续
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - cargo check -p zircon_runtime --locked
  - cargo check --workspace --locked
  - cargo test -p zircon_runtime core::framework::tests::net_framework_root_stays_structural_after_folder_split --locked
  - cargo test -p zircon_runtime tests::extensions::manager_handles::externalized_runtime_plugins_keep_manager_handles_under_core_manager_contracts --locked
doc_type: module-detail
---

# Runtime Network Extension

## Purpose

这份文档记录 `M2` 的第一个真实子系统起手：把 `zircon_plugin_net_runtime` 从“只有 module descriptor 的占位壳”补成最小可用网络闭环。

当前完成线不是完整 multiplayer / replication / RPC，而是更薄的一层：

- `core::framework::net` 定义共享 socket/message-loop 合同
- `core::manager` 暴露稳定 `NetManager` contract / handle
- `zircon_plugin_net_runtime` 提供默认 runtime 实现
- 默认实现能在本机 loopback 上完成 `bind -> send -> poll -> close`

这正对应 roadmap 里 `M2` 的“socket/基础消息闭环”。

## Ownership

这轮之后网络子系统的 ownership 固定为：

- `zircon_runtime::core::framework::net`
  - `NetEndpoint`
  - `NetSocketId`
  - `NetPacket`
  - `NetError`
  - `NetManager`
- `zircon_runtime::core::manager`
  - `NET_MANAGER_NAME`
  - `NetManagerHandle`
  - `resolve_net_manager(...)`
  - `ManagerResolver::net()`
- `zircon_plugin_net_runtime`
  - `NetModule`
  - `NetDriver`
  - `DefaultNetManager`

也就是说：

- framework 只定义中性 DTO 和 manager trait
- core manager 只定义稳定服务名和 resolver surface
- runtime extension 才拥有 `std::net::UdpSocket` 的具体行为

## Contract Shape

`NetManager` 当前故意保持很小，只覆盖 `M2` 这一步必须成立的最小动作：

- `backend_name()`
- `bind_udp(...)`
- `local_endpoint(...)`
- `send_udp(...)`
- `poll_udp(...)`
- `close_socket(...)`

对应 DTO 也只保留最小必需集：

- `NetEndpoint { host, port }`
- `NetSocketId`
- `NetPacket { source, payload }`
- `NetError::{InvalidEndpoint, UnknownSocket, Io}`

这里没有提前引入连接状态机、频道、可靠重传、session、replication、RPC schema 或 host migration。那些都属于后续更高层网络协议，而不是这条 `M2` 起手 contract 该承载的范围。

## Runtime Implementation

默认实现 `DefaultNetManager` 目前基于 `std::net::UdpSocket`：

- `bind_udp` 绑定本地 endpoint，并切成 non-blocking
- `send_udp` 直接通过已绑定 socket 向目标 endpoint 发送 payload
- `poll_udp` 在 non-blocking 模式下收集最多 `max_packets` 个数据包，遇到 `WouldBlock` 立刻返回
- `poll_udp(max_packets)` 的 budget 是硬边界；超过 budget 的 datagram 留在 socket 中，等待下一次 poll
- `close_socket` 从 manager 内部 socket 表移除句柄

内部状态只是一张 `NetSocketId -> UdpSocket` 的表和一个递增 id 计数器，没有后台线程，也没有额外 runtime scheduler 依赖。

这让它满足两个条件：

- 真实可用，不是空壳 manager
- 范围够小，不会把 `M2` 的网络子系统一上来就做成半套复杂 runtime

## Module Wiring

`NetModule` 不再走 `module_descriptor_with_driver_and_manager::<_, _>(...)` 这种占位 helper。

现在它和 physics / animation 一样，显式注册三层服务：

1. `NetDriver`
2. `DefaultNetManager`
3. manager handle `NetManagerHandle`

这一步的意义不是为了复杂化，而是把“默认实现”和“稳定 manager contract”分开：

- 以后换掉默认 backend，不需要改 `NET_MANAGER_NAME`
- 上层 app / editor / plugin 继续只认 `core::manager::resolve_net_manager(...)`

## Validation

这轮直接跑过的验证包括：

- `cargo check -p zircon_runtime --locked`
  - 证明 `framework::net`、`core::manager` 和 `zircon_plugin_net_runtime` 的 production wiring 已经闭合
- `cargo check --workspace --locked`
  - 证明新的 shared manager contract / handle 没有破坏工作区主链
- `cargo test -p zircon_runtime core::framework::tests::net_framework_root_stays_structural_after_folder_split --locked`
  - 证明 `core::framework::net/mod.rs` 保持 structural root，而不是重新把实现堆回根文件
- `cargo test -p zircon_runtime tests::extensions::manager_handles::externalized_runtime_plugins_keep_manager_handles_under_core_manager_contracts --locked`
  - 证明 `NetManagerHandle` / `resolve_net_manager(...)` / `NET_MANAGER_NAME` 已经进入 core manager 稳定表面，而不是继续留在 extension 内部私有路径；网络行为闭环现在由 `zircon_plugin_net_runtime` 的独立插件 workspace 测试覆盖

## Next Steps

这条 `M2/net` 起手完成后，后续网络方向可以继续分层推进：

1. 在 `framework::net` 上加 session / connection surface
2. 把 UDP-only MVP 扩到 TCP listener / accepted stream surface
3. 再往上才是消息 schema、RPC、replication、多人状态同步

关键是这些后续层都应该建立在当前已经收口的 `NetManager` contract 之上，而不是重新回到“只有 module 壳、没有真实 runtime 行为”的状态。
