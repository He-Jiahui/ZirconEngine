---
related_code:
  - zircon_runtime/src/core/framework/net/mod.rs
  - zircon_runtime/src/core/framework/net/diagnostics.rs
  - zircon_runtime/src/core/framework/net/download.rs
  - zircon_runtime/src/core/framework/net/endpoint.rs
  - zircon_runtime/src/core/framework/net/error.rs
  - zircon_runtime/src/core/framework/net/event.rs
  - zircon_runtime/src/core/framework/net/http.rs
  - zircon_runtime/src/core/framework/net/ids.rs
  - zircon_runtime/src/core/framework/net/manager.rs
  - zircon_runtime/src/core/framework/net/packet.rs
  - zircon_runtime/src/core/framework/net/reliable.rs
  - zircon_runtime/src/core/framework/net/rpc.rs
  - zircon_runtime/src/core/framework/net/session.rs
  - zircon_runtime/src/core/framework/net/socket_id.rs
  - zircon_runtime/src/core/framework/net/sync.rs
  - zircon_runtime/src/core/framework/net/transport.rs
  - zircon_runtime/src/core/framework/net/websocket.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_plugins/net/plugin.toml
  - zircon_plugins/Cargo.toml
  - zircon_plugins/net/runtime/Cargo.toml
  - zircon_plugins/net/runtime/src/config.rs
  - zircon_plugins/net/runtime/src/http.rs
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/module.rs
  - zircon_plugins/net/runtime/src/package.rs
  - zircon_plugins/net/runtime/src/runtime_state.rs
  - zircon_plugins/net/runtime/src/service_types.rs
  - zircon_plugins/net/runtime/src/tests.rs
  - zircon_plugins/net/runtime/src/websocket.rs
  - zircon_plugins/net/features/http/runtime/Cargo.toml
  - zircon_plugins/net/features/http/runtime/src/backend.rs
  - zircon_plugins/net/features/http/runtime/src/feature.rs
  - zircon_plugins/net/features/http/runtime/src/lib.rs
  - zircon_plugins/net/features/http/runtime/src/tests.rs
  - zircon_plugins/net/features/rpc/runtime/Cargo.toml
  - zircon_plugins/net/features/rpc/runtime/src/feature.rs
  - zircon_plugins/net/features/rpc/runtime/src/lib.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests.rs
  - zircon_plugins/net/features/replication/runtime/Cargo.toml
  - zircon_plugins/net/features/replication/runtime/src/feature.rs
  - zircon_plugins/net/features/replication/runtime/src/lib.rs
  - zircon_plugins/net/features/replication/runtime/src/manager.rs
  - zircon_plugins/net/features/replication/runtime/src/tests.rs
  - zircon_plugins/net/features/reliable_udp/runtime/Cargo.toml
  - zircon_plugins/net/features/reliable_udp/runtime/src/feature.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/lib.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests.rs
  - zircon_plugins/net/features/websocket/runtime/Cargo.toml
  - zircon_plugins/net/features/websocket/runtime/src/backend.rs
  - zircon_plugins/net/features/websocket/runtime/src/feature.rs
  - zircon_plugins/net/features/websocket/runtime/src/lib.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests.rs
  - zircon_plugins/net/features/content_download/runtime/Cargo.toml
  - zircon_plugins/net/features/content_download/runtime/src/feature.rs
  - zircon_plugins/net/features/content_download/runtime/src/lib.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests.rs
  - zircon_plugins/net/editor/src/lib.rs
implementation_files:
  - zircon_runtime/src/core/framework/net/mod.rs
  - zircon_runtime/src/core/framework/net/diagnostics.rs
  - zircon_runtime/src/core/framework/net/download.rs
  - zircon_runtime/src/core/framework/net/error.rs
  - zircon_runtime/src/core/framework/net/event.rs
  - zircon_runtime/src/core/framework/net/http.rs
  - zircon_runtime/src/core/framework/net/ids.rs
  - zircon_runtime/src/core/framework/net/manager.rs
  - zircon_runtime/src/core/framework/net/reliable.rs
  - zircon_runtime/src/core/framework/net/rpc.rs
  - zircon_runtime/src/core/framework/net/session.rs
  - zircon_runtime/src/core/framework/net/sync.rs
  - zircon_runtime/src/core/framework/net/transport.rs
  - zircon_runtime/src/core/framework/net/websocket.rs
  - zircon_plugins/net/plugin.toml
  - zircon_plugins/Cargo.toml
  - zircon_plugins/net/runtime/Cargo.toml
  - zircon_plugins/net/runtime/src/config.rs
  - zircon_plugins/net/runtime/src/http.rs
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/package.rs
  - zircon_plugins/net/runtime/src/runtime_state.rs
  - zircon_plugins/net/runtime/src/service_types.rs
  - zircon_plugins/net/runtime/src/websocket.rs
  - zircon_plugins/net/features/http/runtime/Cargo.toml
  - zircon_plugins/net/features/http/runtime/src/backend.rs
  - zircon_plugins/net/features/http/runtime/src/feature.rs
  - zircon_plugins/net/features/http/runtime/src/lib.rs
  - zircon_plugins/net/features/rpc/runtime/Cargo.toml
  - zircon_plugins/net/features/rpc/runtime/src/feature.rs
  - zircon_plugins/net/features/rpc/runtime/src/lib.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager.rs
  - zircon_plugins/net/features/replication/runtime/Cargo.toml
  - zircon_plugins/net/features/replication/runtime/src/feature.rs
  - zircon_plugins/net/features/replication/runtime/src/lib.rs
  - zircon_plugins/net/features/replication/runtime/src/manager.rs
  - zircon_plugins/net/features/reliable_udp/runtime/Cargo.toml
  - zircon_plugins/net/features/reliable_udp/runtime/src/feature.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/lib.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager.rs
  - zircon_plugins/net/features/websocket/runtime/Cargo.toml
  - zircon_plugins/net/features/websocket/runtime/src/backend.rs
  - zircon_plugins/net/features/websocket/runtime/src/feature.rs
  - zircon_plugins/net/features/websocket/runtime/src/lib.rs
  - zircon_plugins/net/features/content_download/runtime/Cargo.toml
  - zircon_plugins/net/features/content_download/runtime/src/feature.rs
  - zircon_plugins/net/features/content_download/runtime/src/lib.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager.rs
plan_sources:
  - user: 2026-05-02 PLEASE IMPLEMENT THIS PLAN / ZirconEngine Net 插件完善计划
  - user: 2026-05-03 continue net M2 real HTTP/WebSocket backend task
  - user: 2026-05-03 net feature crate hard-cut continuation
  - user: 2026-05-03 continue net M3 session/RPC feature slice
  - user: 2026-05-03 continue current net gaps and report remaining gaps
  - user: 2026-05-03 continue remaining net gaps
  - user: 2026-05-04 continue remaining net gaps
  - user: 2026-05-20 continue net replication scheduling/budget hardening
  - user: 2026-05-20 continue net RPC wall-clock handler timeout hardening
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - .codex/plans/多插件组合可选功能规则设计.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_plugins/net/runtime/src/tests.rs
  - zircon_plugins/net/features/http/runtime/src/tests.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests.rs
  - zircon_plugins/net/features/replication/runtime/src/tests.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests.rs
  - passed: cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1
  - passed: CARGO_TARGET_DIR=target/codex-net-check cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --offline
  - passed: CARGO_TARGET_DIR=target/codex-net-check cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --offline
  - passed: CARGO_TARGET_DIR=target/codex-net-check cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --locked --offline
  - passed: CARGO_TARGET_DIR=target/codex-net-check cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --locked --offline
  - attempted: VS DevCmd + CARGO_TARGET_DIR=target/codex-net-check cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --offline -j 1
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --locked
  - passed: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --lib --locked
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime --tests --offline
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime --tests --locked
  - passed: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime --lib --locked
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --tests --offline
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --tests --locked
  - passed: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --lib --locked
  - passed: cargo fmt --manifest-path Cargo.toml -p zircon_runtime
  - passed: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_replication_runtime -p zircon_plugin_net_reliable_udp_runtime -p zircon_plugin_net_content_download_runtime
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_replication_runtime -p zircon_plugin_net_reliable_udp_runtime -p zircon_plugin_net_content_download_runtime --tests --locked
  - passed: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_replication_runtime -p zircon_plugin_net_reliable_udp_runtime -p zircon_plugin_net_content_download_runtime --lib --locked
  - passed: cargo fmt --manifest-path Cargo.toml -p zircon_runtime
  - passed: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_replication_runtime -p zircon_plugin_net_reliable_udp_runtime -p zircon_plugin_net_content_download_runtime
  - attempted: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_replication_runtime -p zircon_plugin_net_reliable_udp_runtime -p zircon_plugin_net_content_download_runtime --tests --locked
  - attempted: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_replication_runtime -p zircon_plugin_net_reliable_udp_runtime -p zircon_plugin_net_content_download_runtime --tests --offline
  - passed: cargo metadata --manifest-path zircon_plugins/Cargo.toml --offline --format-version 1
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_runtime --tests --locked
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --tests --locked
  - passed: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --lib --locked
  - passed: cargo fmt --manifest-path Cargo.toml -p zircon_runtime
  - passed: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime
  - attempted: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --tests --locked
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --tests --offline
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --tests --locked
  - passed: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --lib --locked
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_websocket_runtime --tests --locked
  - attempted: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_replication_runtime -p zircon_plugin_net_reliable_udp_runtime -p zircon_plugin_net_content_download_runtime --tests --locked
  - attempted: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_websocket_runtime --lib --locked
  - attempted: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime --tests --locked
  - passed: cargo fmt --manifest-path Cargo.toml -p zircon_runtime
  - passed: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime
  - attempted: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --tests --locked
  - passed: cargo fmt --manifest-path Cargo.toml -p zircon_runtime
  - passed: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime
  - passed: git diff --check -- zircon_runtime/src/core/framework/net zircon_plugins/net/features/replication/runtime docs/engine-architecture/runtime-network-extension.md .codex/sessions/20260504-0024-net-full-gap-closure.md
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime --tests --locked
  - attempted: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime --lib --locked
  - passed: cargo fmt --manifest-path Cargo.toml -p zircon_runtime
  - passed: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime
  - passed: git diff --check -- zircon_plugins/net/features/rpc/runtime/src/manager.rs zircon_plugins/net/features/rpc/runtime/src/tests.rs docs/engine-architecture/runtime-network-extension.md .codex/sessions/20260504-0024-net-full-gap-closure.md
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --tests --locked
  - attempted: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --lib --locked --jobs 1 --message-format short --color never
  - passed: cargo fmt --manifest-path Cargo.toml -p zircon_runtime
  - passed: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime
  - passed: git diff --check -- zircon_plugins/net/features/reliable_udp/runtime docs/engine-architecture/runtime-network-extension.md .codex/sessions/20260504-0024-net-full-gap-closure.md
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --tests --locked
  - passed: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --lib --locked --jobs 1 --message-format short --color never
  - passed: cargo fmt --manifest-path Cargo.toml -p zircon_runtime
  - passed: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime
  - passed: git diff --check -- zircon_plugins/net/features/content_download/runtime docs/engine-architecture/runtime-network-extension.md .codex/sessions/20260504-0024-net-full-gap-closure.md
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime --tests --locked
  - passed: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime --lib --locked --jobs 1 --message-format short --color never
  - passed: cargo fmt --manifest-path Cargo.toml -p zircon_runtime
  - passed: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime
  - passed: git diff --check -- zircon_runtime/src/core/framework/net/http.rs zircon_plugins/net/features/http/runtime docs/engine-architecture/runtime-network-extension.md .codex/sessions/20260504-0024-net-full-gap-closure.md
  - passed: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime --tests --locked
  - attempted: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime --lib --locked --jobs 1 --message-format short --color never
  - passed: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Runtime Network Extension

## Purpose

`net` is an independent runtime/editor plugin. The shared engine contract lives in
`zircon_runtime::core::framework::net`, stable access remains under
`zircon_runtime::core::manager::NetManagerHandle`, and concrete behavior belongs to
`zircon_plugins/net/runtime` plus optional feature runtime crates under
`zircon_plugins/net/features/*/runtime`.

The first implementation milestone moves the previous UDP-only loopback MVP into a
Tokio-backed transport foundation:

- UDP socket bind/send/poll/close remains supported through the original `NetManager` API.
- TCP listener/client/accepted connection handles are now part of the shared manager contract.
- Runtime mode is explicit: `DedicatedServer`, `Client`, or `ListenServer`.
- The manager reports copied diagnostics and drains structured runtime events.
- HTTP route/request descriptors, base route-table dispatch, WebSocket frame queues, and loopback pairs remain available in the base runtime.
- Real HTTP route serving/client calls and real WebSocket handshake connections are implemented only by `net.http` and `net.websocket` feature runtime crates.
- RPC session/control descriptors now have a `net.rpc` optional runtime feature crate; replication, reliable UDP, and content-download now have optional runtime feature crates that exercise their shared DTO contracts without yet claiming production-grade backends.

This is not the full multiplayer stack yet. The current milestone establishes the low-level
transport and descriptor surface that later milestones can implement without changing the
manager boundary again.

## Ownership

The ownership split is fixed:

- `zircon_runtime::core::framework::net`
  - IDs: `NetListenerId`, `NetConnectionId`, `NetSessionId`, `NetRequestId`, `NetRouteId`, `NetDownloadId`, `NetSocketId`
  - transport contracts: `NetTransportKind`, `NetConnectionState`, `NetSecurityPolicy`, `NetEvent`, `NetDiagnostics`
  - higher-layer descriptors: HTTP routes/requests, WebSocket frames, RPC descriptors, sync descriptors, reliable-datagram config/stats, download manifests
  - `NetManager` trait
- `zircon_runtime::core::manager`
  - `NET_MANAGER_NAME`
  - `NetManagerHandle`
  - `resolve_net_manager(...)`
  - `ManagerResolver::net()`
- `zircon_plugin_net_runtime`
  - `NetModule`
  - `NetDriver`
  - `DefaultNetManager` / `NetRuntimeManager`
  - Tokio TCP/UDP state in `runtime_state.rs`, base HTTP route tables, WebSocket loopback queues, backend injection traits
  - plugin options, event catalog, and optional feature bundle declarations
- `zircon_plugin_net_http_runtime`
  - `NetHttpRuntimeFeature`
  - `HyperReqwestHttpBackend`
  - `NetHttpFeatureModule.Manager.NetHttpManager`
  - `reqwest` rustls client and `hyper` HTTP/1 route listener implementation
- `zircon_plugin_net_websocket_runtime`
  - `NetWebSocketRuntimeFeature`
  - `TungsteniteWebSocketBackend`
  - `NetWebSocketFeatureModule.Manager.NetWebSocketManager`
  - `tokio-tungstenite` client/server handshake, connection send, and read-queue implementation
- `zircon_plugin_net_rpc_runtime`
  - `NetRpcRuntimeFeature`
  - `NetRpcRuntimeManager`
  - `NetRpcFeatureModule.Manager.NetRpcManager`
  - session handshake state machine and RPC registry validation for `runtime.feature.net.rpc`
- `zircon_plugin_net_replication_runtime`
  - `NetReplicationRuntimeFeature`
  - `NetReplicationRuntimeManager`
  - `NetReplicationFeatureModule.Manager.NetReplicationManager`
  - descriptor-backed dirty snapshots, interest filtering, and deterministic schedule/budget selection for `runtime.feature.net.replication`
- `zircon_plugin_net_editor`
  - network authoring view, drawer, menu operation, and template registration

`framework` stays DTO-only. It does not own Tokio, HTTP clients, WebSocket engines, asset caches,
or replication runtime state.

## Contract Shape

`NetManager` now covers the transport foundation while keeping the existing UDP calls:

- identity and mode: `backend_name()`, `runtime_mode()`
- UDP: `bind_udp`, `local_endpoint`, `send_udp`, `poll_udp`, `close_socket`
- TCP: `listen_tcp`, `listener_endpoint`, `accept_tcp`, `connect_tcp`, `connection_state`, `send_tcp`, `poll_tcp`, `close_connection`
- HTTP: `register_http_route`, `unregister_http_route`, `listen_http`, `send_http_request`
- listener lifecycle: `close_listener` removes TCP, HTTP, and WebSocket listeners by handle and aborts feature-backed HTTP listener tasks when an abort handle is available
- WebSocket: `connect_websocket`, descriptor-backed `listen_websocket`, `accept_websocket`, `open_websocket_loopback`, `send_websocket_frame`, `poll_websocket_frames`
- observability: `drain_events`, `diagnostics`

The manager intentionally exposes handles and copied DTOs only. It does not expose Tokio sockets,
tasks, streams, borrowed buffers, or runtime-owned connection objects.

The optional RPC feature is separate from `NetManager` so the base transport manager does not grow
upper-layer authority or handler state. Shared RPC/session contracts now include:

- `NetSessionHandshakePolicy`, `NetSessionHandshakeState`, and `NetSessionControlReport` for deterministic control-message progression.
- `NetSessionInfo` for copied session snapshots that expose the session handle, optional connection handle, handshake/lifecycle state, accepted login identity, and latest `NetSpeed` value.
- `RpcInvocationDescriptor`, `RpcPeerRole`, `RpcDispatchStatus`, and `RpcDispatchReport` for diagnostics-first registry validation.
- `RpcInvocationDescriptor` carries optional `NetRequestId`, timeout, and priority metadata so queued calls can preserve request correlation, direct/queued handler execution can enforce wall-clock timeout diagnostics, and deterministic dispatch order remains observable.
- `RpcDescriptor` remains the schema, direction, payload-limit, and call-rate contract.
- `RpcDescriptor::command`, `RpcDescriptor::client_rpc`, and `RpcDescriptor::target_rpc` provide Mirror-style descriptor helpers without introducing language attributes.
- `RpcDispatchReport` carries copied schema, diagnostic, and optional response payload data so handler execution stays observable without exposing runtime-owned closures.

The optional replication feature is also separate from `NetManager`, keeping object-state authority above
the base transport layer. Shared sync contracts now include:

- `SyncComponentDescriptor::update_hz` as the descriptor-level replication frequency gate.
- `SyncComponentDescriptor::replication_priority` as a copied priority value for deterministic selection under contention.
- `SyncReplicationBudget` for per-schedule snapshot-count and byte budgets, with `SYNC_REPLICATION_UNBOUNDED_BUDGET` as the explicit unlimited sentinel.
- `SyncReplicationScheduleReport` for copied diagnostics about sent snapshots, used bytes, not-due skips, interest skips, and budget deferrals.

## Runtime Implementation

`DefaultNetManager` is now the base Tokio-backed runtime manager, with `NetRuntimeManager` as an
alias for the same concrete implementation. Internally it owns:

- one Tokio multi-thread runtime
- an atomic ID source per handle family
- UDP socket table
- TCP listener table
- TCP connection table
- HTTP route table and optional HTTP backend slot
- HTTP listener table populated only by an injected `HttpRuntimeBackend`
- WebSocket loopback table and optional WebSocket backend slot
- WebSocket network listener/connection tables populated only by an injected `WebSocketRuntimeBackend`
- FIFO event queue

The synchronous `NetManager` trait uses Tokio nonblocking socket APIs internally. Binding and
connecting use the manager's runtime, while polling and sending use `try_*` methods so the existing
engine-facing call surface stays deterministic and budgeted.

The M2 HTTP/WebSocket slice now has base local paths plus optional real protocol-backed feature paths:

- HTTP routes are registered as `NetHttpRouteDescriptor` plus a stable `NetHttpResponseDescriptor`.
- Local `send_http_request` calls without an explicit URL port still dispatch to registered routes by method/path for deterministic route tests and editor/catalog plumbing.
- Base `listen_http` and explicit-port/network `send_http_request` return `ProtocolUnavailable { capability: "runtime.feature.net.http" }` unless `net.http` injects `HyperReqwestHttpBackend`.
- `net.http` binds a real Tokio TCP listener, serves registered routes through `hyper` HTTP/1, and uses a `reqwest` rustls client for outbound requests.
- `NetHttpRequestDescriptor::max_retry_attempts` lets callers opt into bounded retry for transient HTTP failures/status codes; the default is zero retries, preserving one-attempt behavior.
- `net.http` enforces client-side `NetSecurityPolicy` before network I/O: `tls_required` rejects non-HTTPS requests unless insecure loopback is explicitly allowed, and certificate pinning requires a configured `NetCertificatePin` for the request host before network I/O proceeds.
- WebSocket loopback pairs use `NetConnectionId` handles, `NetWebSocketFrame` values, peer queues, close frames, and frame poll budgets.
- Base `listen_websocket` and `connect_websocket` return `ProtocolUnavailable { capability: "runtime.feature.net.websocket" }` unless `net.websocket` injects `TungsteniteWebSocketBackend`.
- `listen_websocket` now takes `NetWebSocketListenerDescriptor`, which keeps the bind endpoint together with optional server admission rules: allowed request paths, required headers, and allowed subprotocols.
- `net.websocket` binds real Tokio TCP listeners, accepts/upgrades server streams, performs client handshakes with URL/custom headers/subprotocols/timeout, and runs read halves as Tokio tasks.
- Feature-backed WebSocket listeners enforce the listener descriptor during the tungstenite server handshake: disallowed paths, missing/mismatched required headers, and unsupported/missing required subprotocols are rejected before a `NetConnectionId` is admitted, while matching subprotocols are selected into the handshake response.
- Feature-backed WebSocket connections push received frames into the same budgeted `poll_websocket_frames` queue as loopback connections.
- `net.websocket` enforces client-side `NetSecurityPolicy` before network I/O: `tls_required` rejects non-WSS connections unless insecure loopback is explicitly allowed, and certificate pinning requires a configured `NetCertificatePin` for the request host before network I/O proceeds.

This keeps the framework contract DTO-only while making the base plugin lightweight when optional
network protocol crates are disabled. HTTPS and WSS use the rustls-enabled feature dependency stack.
Policy enforcement now fails closed for unconfigured certificate pinning and has a shared pin DTO for
host/fingerprint configuration. WebSocket server admission policy now covers path, required-header,
and subprotocol gates. HTTP retry is descriptor-driven and bounded in the optional HTTP backend.
Actual peer certificate fingerprint extraction/matching and proxy policy remain later HTTP/WebSocket
hardening work.

The first M3 RPC/session slice adds `zircon_plugin_net_rpc_runtime` without changing the base
transport manager. `NetRpcRuntimeManager` owns feature-local state for:

- handshake sessions keyed by `NetSessionId`
- optional `NetConnectionId` binding for sessions that originate from transport connection handles
- copied session snapshots through `NetSessionInfo`, including `player_id`, `netspeed_bytes_per_second`, and `Closed` lifecycle state
- hello protocol-version and required-feature validation
- deterministic challenge/login/welcome/join transitions
- login identity capture only after accepted challenge response
- `NetSpeed` capture while a session is welcomed or joined
- `NetSpeed` byte-budget enforcement across one-second windows before per-RPC call quota accounting
- explicit session close, connection-close teardown helpers, and `NetEvent` bridge handling that mark bound sessions `Closed` after TCP/WebSocket close or failed-state events
- RPC descriptor registration by id
- Mirror-style descriptor helpers for Command, ClientRpc, and TargetRpc directions
- schema validators keyed by descriptor schema URI/name
- handler registration by RPC id for validated invocations
- caller-role direction checks for client-to-server, server-to-client, and target-client calls
- client-to-server source-session checks that require a known `Joined` session before payload, schema, quota, or handler gates run
- payload-size rejection before quota accounting
- schema-unavailable and schema-rejected diagnostics before handler invocation
- per-source-session call-rate windows plus per-source-session `NetSpeed` byte windows for DoS-style quota blocking
- copied dispatch reports for no-handler, direction-denied, session-unavailable, schema-unavailable, schema-rejected, payload-too-large, quota-exceeded, handler-failed, and accepted outcomes
- queue admission through `enqueue_rpc`, bounded queue rejection through `QueueFull`, priority-ordered `drain_rpc_queue`, and timeout diagnostics through `TimedOut` for zero-timeout invocations, expired queued calls, pending request expiry, and direct/queued handlers whose wall-clock execution exceeds the invocation timeout
- correlated request tracking through `pending_request` and `expire_pending_requests`, keyed by `NetRequestId`, so feature callers can observe outstanding request/response contracts and timeout diagnostics

`dispatch_rpc` remains a validation-only path for diagnostics and queues. `invoke_rpc` runs the same
validation path and then executes a registered handler closure only after direction, payload, schema,
and quota gates accept the invocation. Direct and queued handler calls measure wall-clock elapsed time
against `RpcInvocationDescriptor::timeout_ms`; late handler results are reported as `TimedOut`, their
response bytes are discarded, and any correlated pending request is still completed. Handlers return
copied response bytes or a copied failure diagnostic; they do not expose runtime-owned state into
`zircon_runtime::core::framework::net`.

The first M4+ contract crates add focused runtime managers while keeping the base transport manager
free of upper-layer state:

- `zircon_plugin_net_replication_runtime` registers `net.replication`, stores `SyncComponentDescriptor` entries, publishes `SyncObjectSnapshot` values, emits dirty-only `SyncDelta` reports, filters visible snapshots through `SyncInterestDescriptor` groups, serves late-join snapshot lists, removes object snapshots through explicit despawn lifecycle calls, and selects scheduled snapshots deterministically by interest, update frequency, priority, per-session last-send time, max-snapshot budget, and max-byte budget.
- `zircon_plugin_net_reliable_udp_runtime` registers `net.reliable_udp`, assigns reliable datagram sequence numbers, fragments payloads according to `ReliableDatagramConfig::mtu_bytes`, tracks pending packets, removes pending fragments by `ReliableDatagramAck`, exposes immediate resend batches and deterministic timeout-driven resend ticks, disconnects after `ReliableDatagramConfig::max_resend_attempts`, reassembles received fragments, applies deterministic loss/reorder simulation profiles, and records copied recovery diagnostics for connected/recovering/disconnected state.
- `zircon_plugin_net_content_download_runtime` registers `net.content_download`, validates `NetDownloadManifest` chunk shape before queueing, tracks chunk progress through `NetDownloadProgress`, builds primary/mirror candidate URLs, records cache-hit completion, builds resumable `NetDownloadAttemptDescriptor` values for chunk fetch attempts, advances mirror failover after failed attempts, records per-chunk failure diagnostics, supports explicit cancellation, and fails closed on chunk hash mismatch.

These crates are contract-level stepping stones. They do not yet provide KCP/tokio-kcp transport I/O,
real socket-backed reliable datagram transport, real HTTP range downloading, CDN mirror retry/failover
transport execution, persistent cache storage, ECS/entity integration for replication, or a production
spatial replication graph.

## Optional Features

`zircon_plugins/net/plugin.toml` and `zircon_plugin_net_runtime::package_manifest()` now declare
these feature bundles:

- `net.http` -> `runtime.feature.net.http`
- `net.websocket` -> `runtime.feature.net.websocket`
- `net.rpc` -> `runtime.feature.net.rpc`
- `net.replication` -> `runtime.feature.net.replication`
- `net.reliable_udp` -> `runtime.feature.net.reliable_udp`
- `net.content_download` -> `runtime.feature.net.cdn_download`

Each feature depends on `net/runtime.plugin.net` as its primary owner dependency. `net.http`,
`net.websocket`, `net.rpc`, `net.replication`, `net.reliable_udp`, and `net.content_download` now
have runtime crates in the plugin workspace. The base plugin declares and gates all features but only
advertises the base `runtime.plugin.net` capability from `runtime_capabilities()`.

## Reference Alignment

The shape follows Unreal's split between socket subsystem, net driver, connections, channels,
control messages, packet handlers, HTTP/WebSocket modules, replication systems, and build-patch
download services. It translates those ideas into Zircon's current Rust boundaries:

- driver/manager/service registration remains under `CoreRuntime`
- public access goes through `core::manager`
- user-facing feature families become plugin feature bundles
- replication/RPC/download behavior stays above the base transport manager

Mirror-style convenience maps to descriptors rather than language attributes:

- Commands and ClientRpc/TargetRpc map to `RpcDescriptor`, `RpcDirection`, `RpcInvocationDescriptor`, and `RpcPeerRole`
- `RpcDescriptor::command`, `client_rpc`, and `target_rpc` keep the Mirror convenience layer as explicit Rust descriptor construction rather than attributes or code generation
- SyncVar-style state maps to `SyncComponentDescriptor` and `SyncFieldDescriptor`
- interest management maps to the sync descriptor's interest group and later replication graph work

## Validation Status

Unit-test coverage now spans the base runtime and optional net feature crates:

- net package optional feature bundle metadata
- UDP loopback preservation
- TCP listen/connect/accept/send/poll echo behavior
- base HTTP route-table dispatch without an explicit network backend
- disabled-base `ProtocolUnavailable` errors for real HTTP/WebSocket protocol calls
- `net.http` feature registration plus real HTTP socket route serving through `hyper` and `reqwest`, including bounded retry of transient HTTP responses
- `net.http` client-side TLS rejection, missing-pin rejection, and configured-pin admission before network I/O
- `net.websocket` feature registration plus real WebSocket client/server handshake and text frame exchange through `tokio-tungstenite`
- `net.websocket` client-side WSS rejection, missing-pin rejection, and configured-pin admission before network I/O
- `net.websocket` server-side listener descriptor enforcement for allowed path, required header, and allowed subprotocol admission, including a successful policy-matched text frame exchange
- `net.rpc` feature registration plus handshake success/failure and RPC no-handler/direction/payload/quota diagnostics
- `net.rpc` session snapshots, transport-event close teardown, joined-session client RPC gating, NetSpeed byte-budget blocking, schema validator, handler invocation, handler failure, schema unavailable, and Mirror descriptor helper coverage
- `net.rpc` request correlation, pending request completion/expiry, bounded queue admission, priority-ordered queue draining, zero-timeout/expired-queue/pending-expiry/slow-handler timeout diagnostics, and no-double-count quota behavior for queued dispatch
- `net.replication` feature registration, dirty-field delta reporting, interest-group snapshot filtering, late-join snapshot copyout, despawn lifecycle removal, update-frequency scheduling, priority ordering, per-session snapshot/byte budgets, and budget deferral diagnostics
- `net.reliable_udp` feature registration, MTU fragmentation, pending-packet tracking, ack removal, resend batch copyout, deterministic resend-timeout ticks, resend attempt-cap disconnect, deterministic loss/reorder simulation, out-of-order fragment reassembly, recovery/disconnect state reporting, dropped-packet accounting, and RTT stats
- `net.content_download` feature registration, manifest progress accounting, invalid/partial manifest rejection, primary/mirror candidate URLs, cache-hit completion, range-resume attempt descriptors, mirror failover state, cancellation, and chunk hash mismatch failure diagnostics
- base runtime state extraction into `runtime_state.rs`, reducing `service_types.rs` from 1045 lines to 910 lines in this continuation
- listener shutdown for TCP listeners through `close_listener`, with HTTP/WebSocket listener-map removal sharing the same manager contract
- runtime mode diagnostics and listener events
- RPC descriptor direction/schema/quota metadata

Validation status for the net package is now:

- The earlier asset metadata duplicate-field blocker no longer reproduces.
- The earlier navigation runtime manifest target blocker no longer reproduces.
- The plugin workspace member blocker for missing `asset_importers/{model,texture,audio,shader,data}/runtime`
  crates was cleared with minimal runtime skeleton packages.
- `ResourceRecord` in `zircon_runtime_interface` now has `with_state` and `with_diagnostics`, matching asset pipeline usage through `zircon_runtime::core::resource`.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --offline` passed with `CARGO_TARGET_DIR=target/codex-net-check`.
- The locked check variant also passed after refreshing `zircon_plugins/Cargo.lock` offline.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --locked --offline` passed with the same target, proving the added HTTP/WebSocket tests type-check.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --locked` passed after the target was prewarmed.
- Before the feature-crate hard cut, `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --lib --locked` passed with 10 base-runtime tests including the then-colocated real HTTP/WebSocket coverage.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime --tests --offline` refreshed the plugin lockfile and passed for the base and feature crates.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime --tests --locked` passed after the lockfile refresh, with existing `zircon_runtime` warnings only.
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime --lib --locked` passed: 16 tests passed across base net, `net.http`, and `net.websocket`.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --tests --offline` passed after adding the `net.rpc` feature crate, with existing `zircon_runtime` warnings only.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --tests --locked` passed for the expanded scoped net set, with existing `zircon_runtime` warnings only.
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --lib --locked` passed: 20 tests passed across base net, `net.http`, `net.websocket`, and `net.rpc`.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_runtime --tests --locked` passed after the RPC schema/handler continuation, with existing `zircon_runtime` warnings only.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --tests --locked` passed after the RPC schema/handler continuation, with existing `zircon_runtime` warnings only.
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --lib --locked` passed: 22 tests passed across base net, `net.http`, `net.websocket`, and `net.rpc`.
- `cargo fmt --manifest-path Cargo.toml -p zircon_runtime` and `cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime` passed after the session lifecycle continuation.
- The first locked check after this slice reported that `zircon_plugins/Cargo.lock` required a refresh; the scoped offline check refreshed it without network access.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --tests --offline` passed after adding session snapshots, close lifecycle, and joined-session RPC gates, with existing `zircon_runtime` warnings only.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --tests --locked` passed after waiting on a Cargo build-directory lock, with existing `zircon_runtime` warnings only.
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --lib --locked` passed: 25 tests passed across base net, `net.http`, `net.websocket`, and `net.rpc`.
- `cargo fmt --manifest-path Cargo.toml -p zircon_runtime` and `cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime` passed after the gap-continuation slice.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime --tests --locked` passed after adding NetSpeed budgets, transport-event session close bridging, and HTTP/WebSocket policy enforcement, with existing `zircon_runtime` warnings only.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --tests --locked` passed for the full scoped net set after this continuation, with existing `zircon_runtime` warnings only.
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime --lib --locked` passed: 29 tests passed across base net, `net.http`, `net.websocket`, and `net.rpc`.
- `cargo fmt --manifest-path Cargo.toml -p zircon_runtime` passed after adding RPC queue/listener and M4+ DTO contracts.
- `cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_replication_runtime -p zircon_plugin_net_reliable_udp_runtime -p zircon_plugin_net_content_download_runtime` passed after adding the M4+ feature crates.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_replication_runtime -p zircon_plugin_net_reliable_udp_runtime -p zircon_plugin_net_content_download_runtime --tests --locked` first reported that `zircon_plugins/Cargo.lock` needed a refresh for the new feature crate workspace members.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_replication_runtime -p zircon_plugin_net_reliable_udp_runtime -p zircon_plugin_net_content_download_runtime --tests --offline` then refreshed the lockfile but stopped in unrelated shared runtime graphics code: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs:47` calls `render_compiled_scene` with 10 arguments while the current method takes 8. This sits outside the net subsystem and overlaps active render/VG work, so this net continuation did not modify it.
- `cargo metadata --manifest-path zircon_plugins/Cargo.toml --offline --format-version 1` passed after the new net feature crates were added to the plugin workspace.
- `cargo fmt --manifest-path Cargo.toml -p zircon_runtime` and `cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_replication_runtime -p zircon_plugin_net_reliable_udp_runtime -p zircon_plugin_net_content_download_runtime` passed after adding certificate pins, RPC pending request tracking, M4+ manager behavior, and runtime-state extraction.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_replication_runtime -p zircon_plugin_net_reliable_udp_runtime -p zircon_plugin_net_content_download_runtime --tests --locked` passed for the full scoped net package set, with existing unrelated `zircon_runtime` warnings only.
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_replication_runtime -p zircon_plugin_net_reliable_udp_runtime -p zircon_plugin_net_content_download_runtime --lib --locked` passed: 48 tests passed across base net and all six net feature runtime crates.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_websocket_runtime --tests --locked` passed after the WebSocket listener descriptor and server admission-policy slice.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_http_runtime -p zircon_plugin_net_websocket_runtime -p zircon_plugin_net_rpc_runtime -p zircon_plugin_net_replication_runtime -p zircon_plugin_net_reliable_udp_runtime -p zircon_plugin_net_content_download_runtime --tests --locked` is currently blocked before scoped net crate checking by unrelated active UI work: `zircon_runtime/src/ui/surface/surface/interaction_state.rs` calls `UiTree::node(...)` without the public `UiRuntimeTreeAccessExt` import in scope.
- A follow-up `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime -p zircon_plugin_net_websocket_runtime --lib --locked` is also blocked by the same unrelated UI compile issue after an attempted private-module import in `interaction_state.rs`. This continuation did not modify that active UI session area.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime --tests --locked` is currently blocked before the content-download crate can be type-checked by unrelated active render work: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs:160` attempts a second mutable borrow of `graph_execution_record` while the first borrow is still used later.
- `cargo fmt --manifest-path Cargo.toml -p zircon_runtime` and `cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime` passed after the reliable UDP simulation/recovery slice.
- The first `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --tests --locked` attempt timed out after 120 seconds while compiling shared runtime crates. Free space on `E:` was 46.38 GB, below the repository 50 GB cleanup threshold, so `cargo clean --manifest-path zircon_plugins/Cargo.toml` removed 7.8 GiB before retrying.
- The retry of `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --tests --locked` reached `zircon_runtime` compilation and is currently blocked before reliable UDP crate checking by unrelated active platform/input work: `zircon_runtime/src/dynamic_api/session.rs:604` uses the `?` operator in `RuntimeDynamicSession::handle_mouse_wheel`, which returns `ZrStatus` rather than `Result`/`Option`. This net continuation did not modify that active platform/input session area.
- `cargo fmt --manifest-path Cargo.toml -p zircon_runtime` and `cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime` passed after the replication scheduling/budget slice.
- `git diff --check -- zircon_runtime/src/core/framework/net zircon_plugins/net/features/replication/runtime docs/engine-architecture/runtime-network-extension.md .codex/sessions/20260504-0024-net-full-gap-closure.md` passed after the replication scheduling/budget slice with CRLF warnings only.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime --tests --locked` passed after the replication scheduling/budget slice, with one existing unrelated `zircon_runtime` dead-code warning for `World::entity_ids_matching_query_archetypes`.
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime --lib --locked` is currently blocked before replication tests execute by unrelated shared runtime native plugin loader errors: `zircon_runtime/src/plugin/native_plugin_loader/behavior_validation.rs:122` has an invalid format string, and `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs:258` borrows `behavior` after move. This net continuation did not modify the native plugin loader area.
- `cargo fmt --manifest-path Cargo.toml -p zircon_runtime` and `cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime` passed after the RPC wall-clock handler timeout slice.
- `git diff --check -- zircon_plugins/net/features/rpc/runtime/src/manager.rs zircon_plugins/net/features/rpc/runtime/src/tests.rs docs/engine-architecture/runtime-network-extension.md .codex/sessions/20260504-0024-net-full-gap-closure.md` passed after the RPC timeout slice with CRLF warnings only.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --tests --locked` passed after the RPC timeout slice, with an existing unrelated `zircon_runtime` dead-code warning for `World::entity_ids_matching_query_archetypes`.
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --lib --locked --jobs 1 --message-format short --color never` is currently blocked before RPC tests execute by unrelated active UI work: `zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs:11` imports `crate::ui::binding::runtime_state_update_with_source_kind`, which is not currently exported by `ui::binding`. This net continuation did not modify the UI binding/default-interaction area.
- The reliable UDP resend-timer slice adds feature-local deterministic resend tick state keyed by datagram sequence. `resend_due(now_ms)` returns packets only after `resend_timeout_ms`, increments resend attempts per sequence, keeps ack removal authoritative, and drops/disconnects capped sequences with the diagnostic `reliable datagram resend attempt cap exceeded`. This is still contract-level behavior, not real KCP/socket transport I/O.
- `cargo fmt --manifest-path Cargo.toml -p zircon_runtime` and `cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime` passed after the reliable UDP resend timer/cap slice.
- `git diff --check -- zircon_plugins/net/features/reliable_udp/runtime docs/engine-architecture/runtime-network-extension.md .codex/sessions/20260504-0024-net-full-gap-closure.md` passed after the reliable UDP resend timer/cap slice with CRLF warnings only.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --tests --locked` passed after the reliable UDP resend timer/cap slice, with existing unrelated `zircon_runtime` warnings for render scene-extract imports, `SystemState::state`, and `World::entity_ids_matching_query_archetypes`.
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --lib --locked --jobs 1 --message-format short --color never` passed after the reliable UDP resend timer/cap slice: 7 tests passed, 0 failed, with the same unrelated `zircon_runtime` warnings.
- The content-download manifest validation slice rejects empty manifests, duplicate chunk IDs, empty chunk IDs, empty URLs, zero byte lengths, empty hashes, and resume offsets outside the chunk byte range before storing manifest/progress state. Rejected manifests return a copied `Failed` progress report with a diagnostic instead of becoming queued work.
- `cargo fmt --manifest-path Cargo.toml -p zircon_runtime` and `cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime` passed after the content-download manifest validation slice.
- `git diff --check -- zircon_plugins/net/features/content_download/runtime docs/engine-architecture/runtime-network-extension.md .codex/sessions/20260504-0024-net-full-gap-closure.md` passed after the content-download manifest validation slice with CRLF warnings only.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime --tests --locked` passed after the content-download manifest validation slice, with existing unrelated `zircon_runtime` warnings for `SystemState::state` and `World::entity_ids_matching_query_archetypes`.
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime --lib --locked --jobs 1 --message-format short --color never` passed after the content-download manifest validation slice: 8 tests passed, 0 failed, with the same unrelated `zircon_runtime` warnings.
- The HTTP retry slice adds `NetHttpRequestDescriptor::max_retry_attempts` with a default of zero and a builder helper. The optional HTTP backend retries transient request errors and retryable response statuses `408`, `425`, `429`, `500`, `502`, `503`, and `504` up to that bound after security policy validation.
- `cargo fmt --manifest-path Cargo.toml -p zircon_runtime` and `cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime` passed after the HTTP retry slice.
- `git diff --check -- zircon_runtime/src/core/framework/net/http.rs zircon_plugins/net/features/http/runtime docs/engine-architecture/runtime-network-extension.md .codex/sessions/20260504-0024-net-full-gap-closure.md` passed after the HTTP retry slice with CRLF warnings only.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime --tests --locked` passed after the HTTP retry slice, with existing unrelated `zircon_runtime` dead-code warnings for `SystemState::state` and `World::entity_ids_matching_query_archetypes`.
- The first HTTP lib-test command hit the 15-minute timeout while rebuilding after the repository low-disk cleanup policy triggered at 45.51 GiB free on `E:`; `cargo clean --manifest-path zircon_plugins/Cargo.toml` removed 28.1 GiB before the timed-out rebuild attempt.
- Retried `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime --lib --locked --jobs 1 --message-format short --color never` passed after the HTTP retry slice: 6 tests passed, 0 failed, with the same unrelated `zircon_runtime` dead-code warnings.

Full plugin-workspace and root-workspace Cargo validation remain broader milestone gates because the
workspace is currently dirty with unrelated active asset/render/editor/platform-input work. Current
reliable UDP compile/test acceptance plus replication/RPC lib-test acceptance are blocked before the
net feature tests by unrelated shared-runtime compile errors above.
