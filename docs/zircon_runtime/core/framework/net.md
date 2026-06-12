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
  - zircon_runtime/src/core/framework/net/tests.rs
  - zircon_plugins/net/runtime/src/service_types.rs
  - zircon_plugins/net/runtime/src/runtime_state.rs
implementation_files:
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
  - zircon_runtime/src/core/framework/net/tests.rs
plan_sources:
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
  - .codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md
tests:
  - zircon_runtime/src/core/framework/net/tests.rs
  - endpoint_transport_and_security_policy_are_neutral_contracts
  - http_and_websocket_descriptors_keep_protocol_state_data_only
  - rpc_session_and_handshake_descriptors_are_runtime_mode_agnostic
  - reliable_datagram_and_download_contracts_record_recovery_state
  - sync_descriptors_share_interest_budget_and_delta_contracts
  - rustfmt --edition 2021 --check over touched Net event/runtime/test files (passed 2026-06-07)
  - cargo check -p zircon_runtime --tests --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607-runtime-core-min-check --message-format short --color never (passed 2026-06-07)
  - cargo test -p zircon_runtime --lib http_and_websocket_descriptors_keep_protocol_state_data_only --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607-runtime --message-format short --color never -- --test-threads=1 --nocapture (timed out during test-binary link on 2026-06-07)
  - cargo test -p zircon_runtime --lib http_and_websocket_descriptors_keep_protocol_state_data_only --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607-runtime-core-min --message-format short --color never -- --test-threads=1 --nocapture (timed out during test-binary link on 2026-06-07)
doc_type: module-detail
---

# Net Framework Contracts

## Purpose

`zircon_runtime::core::framework::net` is the neutral networking contract layer for transport, session, RPC, replication, reliable datagrams, HTTP, WebSocket, and content-download surfaces. It owns stable DTOs, IDs, manager traits, events, diagnostics, and security policy shapes. It does not own sockets, async runtimes, HTTP servers, WebSocket handshakes, replication stores, or gameplay protocol execution.

Concrete runtime behavior lives in `zircon_plugins/net/runtime` and optional feature crates. Runtime and editor callers should resolve a `NetManager` handle and exchange framework DTOs instead of sharing plugin-owned socket or backend objects.

## Related Files

The framework is folder-backed and `mod.rs` is only the re-export surface.

- `endpoint.rs`, `transport.rs`, `socket_id.rs`, and `ids.rs` define endpoint addressing, transport categories, security policies, connection states, and stable handles.
- `manager.rs` defines `NetManager`, the unified service facade for UDP, TCP, HTTP, WebSocket, event drain, and diagnostics.
- `http.rs` and `websocket.rs` describe protocol-level route, request, response, listener, connect, frame, and close DTOs.
- `rpc.rs` and `session.rs` define RPC direction, peer-role gates, invocation/report records, control messages, and handshake policy.
- `sync.rs` defines replication component schemas, object snapshots, deltas, interest groups, budgets, and schedule reports.
- `reliable.rs` defines reliable datagram configuration, simulation profiles, fragment packets, ACKs, delivery reports, receive reports, and recovery state.
- `download.rs` defines chunked content download manifests, range-resume attempts, and progress state.
- `event.rs` and `diagnostics.rs` expose transport-independent runtime events and copied status counters. Lifecycle events include socket bind/close, listener start/close, connection state/accept/close, HTTP route register/unregister, WebSocket pair open, and queued WebSocket frames.

## Behavior Model

The framework separates protocol authority from transport execution. `RpcDescriptor`, `RpcInvocationDescriptor`, `SyncComponentDescriptor`, and `NetSessionHandshakePolicy` can be created and validated without a socket. `NetHttpRequestDescriptor`, `NetHttpRouteDescriptor`, `NetWebSocketConnectDescriptor`, and `NetWebSocketListenerDescriptor` are data-only records until a plugin backend executes them. This matches the engine plugin rule: `zircon_runtime::core::framework` defines contracts, and plugin crates provide executable services.

The base transport set is explicit:

- UDP socket bind, send, poll, and close use `NetSocketId` and `NetPacket`.
- TCP listener, accept, connect, byte send, byte poll, and close use `NetListenerId` and `NetConnectionId`.
- HTTP routes, HTTP listener startup, and outgoing requests use `NetRouteId`, `NetRequestId`, and descriptor records.
- WebSocket listeners, connections, loopback pairs, frames, close reasons, and queued-frame events use the same connection/listener handle model.
- Reliable UDP, RPC, replication, and content download are higher-level contracts layered above those transports, not alternate manager traits.

`NetEvent` is the stable copied lifecycle stream for manager-owned handles. Runtime implementations should push start/open events after a handle enters the manager table, and push close/unregister events only after the handle has actually been removed or marked closed. Connection accepted and connection closed events include `NetTransportKind`, so consumers can classify TCP and WebSocket lifecycle changes without re-reading runtime tables after handle removal. This keeps event consumers from inferring lifecycle changes from diagnostics counters alone and lets RPC/session systems ignore non-connection events while still receiving deterministic connection-close notifications.

`NetDiagnostics` is the stable copied snapshot for manager-owned handles. It separates UDP sockets, TCP listeners, HTTP listeners, WebSocket listeners, TCP connections, HTTP routes, WebSocket connections, and queued events so editor tools and export/profile checks can distinguish passive listener exposure from outbound-only or data-only plugin selections.

`NetSecurityPolicy` is also a DTO. It records TLS requirement, certificate pinning, certificate pins, and local development loopback policy. The framework does not open TLS sessions or validate certificates by itself.

## Reference Alignment

The Net contract follows local reference-engine evidence rather than a one-off plugin shape.

- Bevy Remote separates protocol registration from transport startup. `dev/bevy/crates/bevy_remote/src/lib.rs` registers `RemotePlugin` methods without opening transports, while `dev/bevy/crates/bevy_remote/src/http.rs` provides the HTTP transport plugin. Zircon mirrors this by keeping RPC/session/schema DTOs independent from HTTP/WebSocket/TCP execution.
- Godot keeps low-level peers, HTTP client, UDP packet peers, ENet/multiplayer peer, WebSocket peer, and TCP server concepts as separate engine services under `dev/godot/core/io`. Zircon keeps comparable protocol concepts but routes access through `NetManager` instead of exposing concrete peer objects through runtime framework.
- Godot's UDP, TCP server, and TCP stream classes expose explicit close/stop/status lifecycle operations, while Bevy Remote keeps protocol method registration separate from the HTTP transport. Zircon translates those precedents into typed manager events instead of exposing socket objects through the framework.
- Bevy Remote records HTTP transport address and port as app resources and starts its HTTP transport separately from protocol method registration. Zircon translates that transport visibility into neutral diagnostics fields rather than feature-crate-specific inspector state.
- Fyrox's `dev/Fyrox/fyrox-core/src/net.rs` is a compact Rust-native network abstraction around listeners and streams. Zircon keeps that Rust-friendly service shape for TCP/UDP basics, then adds higher-level engine contracts for RPC, replication, download manifests, and diagnostics.

The deliberate divergence is that Zircon treats gameplay networking and developer remote protocols as profile-gated plugin capabilities. Adding the Net plugin should not make default client profiles listen on a port.

## Control Flow

`zircon_plugins/net/runtime` registers the concrete `net.runtime` module, `NetDriver`, and `DefaultNetManager`. The manager consumes the framework trait and stores plugin-owned runtime state. Optional feature crates such as HTTP, WebSocket, RPC, replication, reliable UDP, and content download register additional package features and executable backends while continuing to use framework DTOs.

App, editor, export, VM, and gameplay systems should:

1. Resolve the manager through the runtime manager layer.
2. Construct neutral descriptors or query copied diagnostics.
3. Let the plugin implementation decide whether a transport/backend exists for the selected profile and target.
4. Handle typed `NetError` values for unavailable protocol features, unknown handles, invalid endpoints, and backend IO failures.

## Edge Cases

Data construction is intentionally permissive. Runtime implementations still own validation for unsupported URL schemes, missing HTTP/WebSocket feature backends, invalid endpoint strings, oversized payloads, session mismatch, quota enforcement, replication budget exhaustion, unreliable network simulation, content hash mismatch, and resume range validation.

`SyncReplicationBudget::SYNC_REPLICATION_UNBOUNDED_BUDGET` uses `0` as the unlimited sentinel for local tests and tooling. A limited snapshot budget accepts `count < max_snapshots`; reaching the limit defers further snapshots. Interest filters allow snapshots without an interest group by default.

## Test Coverage

Framework tests now lock endpoint/transport/security DTO behavior, `NetDiagnostics` listener-count serde, HTTP/WebSocket descriptor defaults and serde, NetEvent lifecycle serde for close/unregister variants plus transport-qualified accept/close variants, RPC/session/handshake records, reliable datagram recovery/download resume records, and replication interest/budget/delta semantics. The current slice intentionally avoids real network IO so the framework contract remains deterministic and can run inside `zircon_runtime` without starting sockets.

Focused validation after transport-qualified connection lifecycle events ran on 2026-06-07. `rustfmt --edition 2021 --check` passed over the touched Net framework/runtime/feature Rust files; path-scoped `git diff --check`, conflict-marker scans, and trailing-whitespace scans passed for touched Rust/docs/session paths. Plugin workspace `cargo check --tests` passed for base Net runtime, RPC feature runtime, and WebSocket feature runtime with existing `zircon_runtime` warnings. Focused base/RPC tests passed for TCP accept/close transport, WebSocket explicit close transport, and transport-agnostic RPC session cleanup. The framework `cargo check -p zircon_runtime --tests --no-default-features --features core-min` command initially saw a concurrent runtime-assembly file-state mismatch outside the Net contract and failed on stale `RuntimeExtensionRegistry` visibility; the rerun against the settled worktree passed with existing warnings only, type-checking the transport-qualified `NetEvent` serde coverage.

Focused validation after adding explicit Net lifecycle events ran on 2026-06-07. `rustfmt --edition 2021 --check` passed over the touched Net Rust files; conflict-marker and trailing-whitespace scans over touched Net/doc/session paths returned empty; and path-scoped `git diff --check` passed with expected LF-to-CRLF warnings only. Focused Net runtime `cargo check` and the three lifecycle runtime tests passed. Direct framework test execution for `http_and_websocket_descriptors_keep_protocol_state_data_only` timed out twice while linking the `zircon_runtime` test binary, once with default features and once with `--no-default-features --features core-min`; no framework test execution pass is claimed. The fallback framework check `cargo check -p zircon_runtime --tests --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-net-lifecycle-events-0607-runtime-core-min-check --message-format short --color never` passed and type-checks the new `NetEvent` serde coverage.
