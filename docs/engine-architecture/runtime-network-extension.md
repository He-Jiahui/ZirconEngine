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
  - zircon_plugins/net/runtime/Cargo.toml
  - zircon_plugins/net/runtime/src/config.rs
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/module.rs
  - zircon_plugins/net/runtime/src/package.rs
  - zircon_plugins/net/runtime/src/service_types.rs
  - zircon_plugins/net/runtime/src/tests.rs
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
  - zircon_plugins/net/runtime/Cargo.toml
  - zircon_plugins/net/runtime/src/config.rs
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/package.rs
  - zircon_plugins/net/runtime/src/service_types.rs
plan_sources:
  - user: 2026-05-02 PLEASE IMPLEMENT THIS PLAN / ZirconEngine Net 插件完善计划
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - .codex/plans/多插件组合可选功能规则设计.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_plugins/net/runtime/src/tests.rs
  - passed: cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1
  - passed: CARGO_TARGET_DIR=target/codex-net-check cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --offline
  - passed: CARGO_TARGET_DIR=target/codex-net-check cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --offline
  - passed: CARGO_TARGET_DIR=target/codex-net-check cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --locked --offline
  - passed: CARGO_TARGET_DIR=target/codex-net-check cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --locked --offline
  - attempted: VS DevCmd + CARGO_TARGET_DIR=target/codex-net-check cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --offline -j 1
doc_type: module-detail
---

# Runtime Network Extension

## Purpose

`net` is an independent runtime/editor plugin. The shared engine contract lives in
`zircon_runtime::core::framework::net`, stable access remains under
`zircon_runtime::core::manager::NetManagerHandle`, and concrete behavior belongs to
`zircon_plugins/net/runtime`.

The first implementation milestone moves the previous UDP-only loopback MVP into a
Tokio-backed transport foundation:

- UDP socket bind/send/poll/close remains supported through the original `NetManager` API.
- TCP listener/client/accepted connection handles are now part of the shared manager contract.
- Runtime mode is explicit: `DedicatedServer`, `Client`, or `ListenServer`.
- The manager reports copied diagnostics and drains structured runtime events.
- HTTP route/request dispatch and WebSocket frame queue behavior now have a first runtime slice.
- RPC, replication, reliable UDP, and content-download contracts are represented as neutral DTOs and optional feature manifests.

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
  - plugin options, event catalog, and optional feature bundle declarations
- `zircon_plugin_net_editor`
  - network authoring view, drawer, menu operation, and template registration

`framework` stays DTO-only. It does not own Tokio, HTTP clients, WebSocket engines, asset caches,
or replication runtime state.

## Contract Shape

`NetManager` now covers the transport foundation while keeping the existing UDP calls:

- identity and mode: `backend_name()`, `runtime_mode()`
- UDP: `bind_udp`, `local_endpoint`, `send_udp`, `poll_udp`, `close_socket`
- TCP: `listen_tcp`, `listener_endpoint`, `accept_tcp`, `connect_tcp`, `connection_state`, `send_tcp`, `poll_tcp`, `close_connection`
- HTTP: `register_http_route`, `unregister_http_route`, `send_http_request`
- WebSocket: `open_websocket_loopback`, `send_websocket_frame`, `poll_websocket_frames`
- observability: `drain_events`, `diagnostics`

The manager intentionally exposes handles and copied DTOs only. It does not expose Tokio sockets,
tasks, streams, borrowed buffers, or runtime-owned connection objects.

## Runtime Implementation

`DefaultNetManager` is now the Tokio-backed runtime manager, with `NetRuntimeManager` as an alias
for the same concrete implementation. Internally it owns:

- one Tokio multi-thread runtime
- an atomic ID source per handle family
- UDP socket table
- TCP listener table
- TCP connection table
- HTTP route table
- WebSocket loopback connection table
- FIFO event queue

The synchronous `NetManager` trait uses Tokio nonblocking socket APIs internally. Binding and
connecting use the manager's runtime, while polling and sending use `try_*` methods so the existing
engine-facing call surface stays deterministic and budgeted.

The M2 starter slice keeps HTTP and WebSocket runtime behavior intentionally local and budgetable:

- HTTP routes are registered as `NetHttpRouteDescriptor` plus a stable `NetHttpResponseDescriptor`.
- `send_http_request` parses the request path and dispatches to registered routes by method/path.
- WebSocket loopback pairs use `NetConnectionId` handles, `NetWebSocketFrame` values, peer queues, close frames, and frame poll budgets.

This is enough for plugin/catalog/editor surfaces to exercise the feature family without binding the
framework contract to a specific external library. The production backend still needs the planned
`reqwest`/`hyper`/`tokio-tungstenite` integration before this becomes real network HTTP(S) or
WebSocket IO.

## Optional Features

`zircon_plugins/net/plugin.toml` and `zircon_plugin_net_runtime::package_manifest()` now declare
these feature bundles:

- `net.http` -> `runtime.feature.net.http`
- `net.websocket` -> `runtime.feature.net.websocket`
- `net.rpc` -> `runtime.feature.net.rpc`
- `net.replication` -> `runtime.feature.net.replication`
- `net.reliable_udp` -> `runtime.feature.net.reliable_udp`
- `net.content_download` -> `runtime.feature.net.cdn_download`

Each feature depends on `net/runtime.plugin.net` as its primary owner dependency. The runtime crates
listed for those features are feature-package targets for later milestones; the base plugin only
declares and gates them here.

## Reference Alignment

The shape follows Unreal's split between socket subsystem, net driver, connections, channels,
control messages, packet handlers, HTTP/WebSocket modules, replication systems, and build-patch
download services. It translates those ideas into Zircon's current Rust boundaries:

- driver/manager/service registration remains under `CoreRuntime`
- public access goes through `core::manager`
- user-facing feature families become plugin feature bundles
- replication/RPC/download behavior stays above the base transport manager

Mirror-style convenience maps to descriptors rather than language attributes:

- Commands and ClientRpc/TargetRpc map to `RpcDescriptor` plus `RpcDirection`
- SyncVar-style state maps to `SyncComponentDescriptor` and `SyncFieldDescriptor`
- interest management maps to the sync descriptor's interest group and later replication graph work

## Validation Status

New unit-test coverage was added in `zircon_plugins/net/runtime/src/tests.rs` for:

- net package optional feature bundle metadata
- UDP loopback preservation
- TCP listen/connect/accept/send/poll echo behavior
- runtime mode diagnostics and listener events
- RPC descriptor direction/schema/quota metadata

Validation is currently blocked by unrelated active workspace changes:

- The earlier asset metadata duplicate-field blocker no longer reproduces.
- The earlier navigation runtime manifest target blocker no longer reproduces.
- The plugin workspace member blocker for missing `asset_importers/{model,texture,audio,shader,data}/runtime`
  crates was cleared with minimal runtime skeleton packages.
- `ResourceRecord` in `zircon_runtime_interface` now has `with_state` and `with_diagnostics`, matching asset pipeline usage through `zircon_runtime::core::resource`.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --offline` passed with `CARGO_TARGET_DIR=target/codex-net-check`.
- The locked check variant also passed after refreshing `zircon_plugins/Cargo.lock` offline.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --tests --locked --offline` passed with the same target, proving the added HTTP/WebSocket tests type-check.
- Full `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --lib --locked --offline -j 1` was attempted inside VS DevCmd. It still timed out while compiling/linking large shared dependencies such as `wgpu`, `gltf`, `glyphon`, `fontsdf`, and `zircon_runtime_interface`; it did not reach the net test runner.

The next clean validation gate for this milestone is to rerun the same package test on a quieter
machine or after shared target artifacts are warm enough that the test binary can link within the
local timeout budget.
