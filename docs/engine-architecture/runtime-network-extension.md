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
  - Tokio TCP/UDP state, base HTTP route tables, WebSocket loopback queues, backend injection traits
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
- WebSocket: `connect_websocket`, `listen_websocket`, `accept_websocket`, `open_websocket_loopback`, `send_websocket_frame`, `poll_websocket_frames`
- observability: `drain_events`, `diagnostics`

The manager intentionally exposes handles and copied DTOs only. It does not expose Tokio sockets,
tasks, streams, borrowed buffers, or runtime-owned connection objects.

The optional RPC feature is separate from `NetManager` so the base transport manager does not grow
upper-layer authority or handler state. Shared RPC/session contracts now include:

- `NetSessionHandshakePolicy`, `NetSessionHandshakeState`, and `NetSessionControlReport` for deterministic control-message progression.
- `NetSessionInfo` for copied session snapshots that expose the session handle, optional connection handle, handshake/lifecycle state, accepted login identity, and latest `NetSpeed` value.
- `RpcInvocationDescriptor`, `RpcPeerRole`, `RpcDispatchStatus`, and `RpcDispatchReport` for diagnostics-first registry validation.
- `RpcInvocationDescriptor` carries optional `NetRequestId`, timeout, and priority metadata so queued calls can preserve request correlation and deterministic dispatch order.
- `RpcDescriptor` remains the schema, direction, payload-limit, and call-rate contract.
- `RpcDescriptor::command`, `RpcDescriptor::client_rpc`, and `RpcDescriptor::target_rpc` provide Mirror-style descriptor helpers without introducing language attributes.
- `RpcDispatchReport` carries copied schema, diagnostic, and optional response payload data so handler execution stays observable without exposing runtime-owned closures.

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
- `net.http` enforces client-side `NetSecurityPolicy` before network I/O: `tls_required` rejects non-HTTPS requests unless insecure loopback is explicitly allowed, and certificate pinning is rejected until a real pin configuration surface exists.
- WebSocket loopback pairs use `NetConnectionId` handles, `NetWebSocketFrame` values, peer queues, close frames, and frame poll budgets.
- Base `listen_websocket` and `connect_websocket` return `ProtocolUnavailable { capability: "runtime.feature.net.websocket" }` unless `net.websocket` injects `TungsteniteWebSocketBackend`.
- `net.websocket` binds real Tokio TCP listeners, accepts/upgrades server streams, performs client handshakes with URL/custom headers/subprotocols/timeout, and runs read halves as Tokio tasks.
- Feature-backed WebSocket connections push received frames into the same budgeted `poll_websocket_frames` queue as loopback connections.
- `net.websocket` enforces client-side `NetSecurityPolicy` before network I/O: `tls_required` rejects non-WSS connections unless insecure loopback is explicitly allowed, and certificate pinning is rejected until pin configuration is available.

This keeps the framework contract DTO-only while making the base plugin lightweight when optional
network protocol crates are disabled. HTTPS and WSS use the rustls-enabled feature dependency stack.
Policy enforcement now fails closed for unconfigured certificate pinning; actual pin storage/matching,
proxy policy, retry policy, and production route handler callbacks remain later HTTP hardening work.

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
- queue admission through `enqueue_rpc`, bounded queue rejection through `QueueFull`, priority-ordered `drain_rpc_queue`, and timeout diagnostics through `TimedOut`

`dispatch_rpc` remains a validation-only path for diagnostics and queues. `invoke_rpc` runs the same
validation path and then executes a registered handler closure only after direction, payload, schema,
and quota gates accept the invocation. Handlers return copied response bytes or a copied failure
diagnostic; they do not expose runtime-owned state into `zircon_runtime::core::framework::net`.

The first M4+ contract crates add focused runtime managers while keeping the base transport manager
free of upper-layer state:

- `zircon_plugin_net_replication_runtime` registers `net.replication`, stores `SyncComponentDescriptor` entries, publishes `SyncObjectSnapshot` values, emits dirty-only `SyncDelta` reports, and filters visible snapshots through `SyncInterestDescriptor` groups.
- `zircon_plugin_net_reliable_udp_runtime` registers `net.reliable_udp`, assigns reliable datagram sequence numbers, fragments payloads according to `ReliableDatagramConfig::mtu_bytes`, tracks pending packets, and removes pending fragments by `ReliableDatagramAck`.
- `zircon_plugin_net_content_download_runtime` registers `net.content_download`, queues `NetDownloadManifest` values, tracks chunk progress through `NetDownloadProgress`, and fails closed on chunk hash mismatch.

These crates are contract-level stepping stones. They do not yet provide KCP/tokio-kcp transport I/O,
real HTTP range downloading, CDN mirror retry/failover, cache storage, or production replication graph
scheduling.

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

Unit-test coverage now spans the base runtime and the HTTP/WebSocket/RPC feature crates:

- net package optional feature bundle metadata
- UDP loopback preservation
- TCP listen/connect/accept/send/poll echo behavior
- base HTTP route-table dispatch without an explicit network backend
- disabled-base `ProtocolUnavailable` errors for real HTTP/WebSocket protocol calls
- `net.http` feature registration plus real HTTP socket route serving through `hyper` and `reqwest`
- `net.http` client-side TLS and unconfigured certificate-pinning policy rejection before network I/O
- `net.websocket` feature registration plus real WebSocket client/server handshake and text frame exchange through `tokio-tungstenite`
- `net.websocket` client-side WSS and unconfigured certificate-pinning policy rejection before network I/O
- `net.rpc` feature registration plus handshake success/failure and RPC no-handler/direction/payload/quota diagnostics
- `net.rpc` session snapshots, transport-event close teardown, joined-session client RPC gating, NetSpeed byte-budget blocking, schema validator, handler invocation, handler failure, schema unavailable, and Mirror descriptor helper coverage
- `net.rpc` request correlation, bounded queue admission, priority-ordered queue draining, timeout diagnostics, and no-double-count quota behavior for queued dispatch
- `net.replication` feature registration, dirty-field delta reporting, and interest-group snapshot filtering
- `net.reliable_udp` feature registration, MTU fragmentation, pending-packet tracking, and ack removal
- `net.content_download` feature registration, manifest progress accounting, and chunk hash mismatch failure diagnostics
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

Full plugin-workspace and root-workspace Cargo validation remain broader milestone gates because the
workspace is currently dirty with unrelated active asset/render/editor work, and the current scoped
net compile is blocked before reaching net crates by the unrelated render call-site mismatch above.
