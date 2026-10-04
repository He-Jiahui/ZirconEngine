---
title: Plugin Net HTTP and WebSocket Providers Current-Source Performance Review
date: 2026-08-24
status: static_complete_dynamic_pending
scope:
  - zircon_plugins/net/features/http/runtime
  - zircon_plugins/net/features/websocket/runtime
canonical_owners:
  - docs/plans/optimize/zircon_plugins/10-first-party-network-source-runtime-editor-dist-catalog-transport-rpc-replication-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
source_review_dependency: docs/plans/performance/01/2026-08-24-plugin-net-core-runtime-current-source-performance-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Online/HTTP/Private/HttpManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Online/HTTP/Private/HttpThread.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Online/HTTP/Private/HttpModule.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Online/HTTP/Private/HttpRetrySystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Online/HTTP/Public/HttpRetrySystem.h
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineSubsystemUtils/Source/OnlineSubsystemUtils/Private/IpNetDriver.cpp
  - dev/godot/core/io/http_client_tcp.cpp
  - dev/godot/modules/websocket/wsl_peer.cpp
---

# Plugin Net HTTP and WebSocket Providers Current-Source Performance Review

## 1. Status and frozen scope

The two optional transport providers completed an E3 current-source static review over **34/34 Rust files**:

| Module folder | Files | Physical / non-empty lines | Bytes | Tests / ignored | Final fingerprint |
|---|---:|---:|---:|---:|---|
| `zircon_plugins/net/features/http/runtime` | 15/15 | 1,102 / 998 | 38,859 | 11 / 0 | `942526f2c2cfc70f378fdf8e4b76fb287810e34db97840d6710b85e354dfbce3` |
| `zircon_plugins/net/features/websocket/runtime` | 19/19 | 1,186 / 1,082 | 41,741 | 8 / 0 | `446a6b490a7cc0e1d01c57baf9c09535e7a0ba231ef1b480594df40118daf7bf` |

All 34 files pass standalone `rustfmt --check --edition 2021 --config skip_children=true`; the two owned source diffs pass `git diff --check`. Managed Windows Cargo and product traces are unavailable, so Rust tests, real TLS/WSS, scale, shutdown and power acceptance were not run. WPR/RenderDoc remain inapplicable until a launchable current-source product exists; RenderDoc is only useful if a network event changes rendered output.

This review retains Beta/optional/default-off truth. It does not accept the providers as a product capability merely because direct loopback tests and manifests exist.

## 2. Implemented M0 copy reductions

Two clean, behavior-preserving local changes were applied after the full related-module read:

- HTTP server dynamic routes now move the owned `NetHttpRequestDescriptor` into the handler. They no longer clone the request and its body. Saved copy bytes per dynamic request equal `request.body.len()`; at the current matched-route limit that is up to **1 MiB per request**.
- WebSocket reader now records whether an incoming frame is Close before moving the owned frame into the inbound queue. It no longer clones each frame and payload. Saved copy bytes per Text/Binary/Ping/Pong frame equal its payload length.

Existing route-forwarding, oversized-body, real-handshake and frame-order tests cover the changed semantics, but were not executable in the current validation lane. These changes remove obvious redundant copies; they do not bound retained bytes, make calls asynchronous or unify provider ownership.

## 3. Structural findings

### P0: both features create private network authorities

Each feature descriptor declares a dependency on `net.runtime.Manager.NetManager`, but its factory ignores the dependency and creates a fresh `DefaultNetManager::default()` with only its own backend. Each private manager therefore creates the core runtime's executor/worker/thread set and independent sockets, events, diagnostics and generations. HTTP, WebSocket and the canonical base manager cannot observe or govern one another.

Feature activation must install a typed backend extension into the selected World/session `NetworkRuntimeInstance`, not construct another manager. Installation/removal must be transactional and generation-qualified. The feature factory must resolve and consume its declared dependency; a dependency row without instance injection is metadata, not composition.

### P0: provider calls still block the caller

HTTP has **2** production `block_on` sites: listener bind and the complete request/retry operation. WebSocket has **4**: connect, listener bind, accept poll and server handshake. WebSocket accepts only time-limit the socket accept; after a TCP peer is accepted, `accept_hdr_async` has no handshake deadline. One client that opens TCP and never completes the upgrade can block the caller and prevent later accepts on that listener.

All connect/listen/request/accept paths must become scheduled operations owned by the unified I/O authority. Connect, handshake, header/body activity and total deadlines need distinct policy; cancellation must close the socket/task and publish one terminal receipt. Completion delivery to Runtime/World occurs only within the bounded dispatch phase.

### P0: advertised WebSocket certificate pinning does not validate the peer

`validate_websocket_security_policy` checks that a pin string exists for the parsed host, then calls ordinary `tokio_tungstenite::connect_async`. No provider source calls `certificate_pin_matches`, reads a peer certificate or installs a pin-verifying TLS connector. The test named `accepts_configured_certificate_pin_before_network_io` only proves that preflight no longer reports a missing configured pin; it does not prove that the server certificate matches.

This is fail-open product behavior, not a micro-optimization. Until a custom rustls connector validates the actual certificate/chain and hostname against the effective policy, WSS pinning must return `ProtocolUnavailable`/`SecurityPolicyViolation` instead of advertising success. Add a real fixture matrix: trusted-unpinned, self-signed-unpinned, exact leaf pin, wrong pin, wrong hostname, expired chain and rotated generation.

### P0: retained data and producer frequency are unbounded by bytes and age

WebSocket egress limits frames to 64 entries, but not payload bytes or oldest age. Inbound uses an unbounded `VecDeque` per connection, and the reader produces a global `WebSocketFrameQueued` event for every frame under a shared mutex. A stalled consumer can retain arbitrary payload memory and create an equally fast event stream. The manifest's `net.websocket_message_budget` is not wired into this provider.

HTTP matched request bodies are limited to 1 MiB, but response bodies are fully collected without a byte limit. Unmatched request bodies are drained without an activity/total deadline. The server accepts and spawns one task per TCP connection with no configured concurrency, per-host, header or idle budget.

Every provider queue needs entry, byte, age and owner limits plus typed overflow policy. WebSocket should expose queued bytes and coalesce readiness notifications per connection/generation rather than emit one global event per frame. HTTP needs compressed and uncompressed body limits, streaming/file sinks, header limits and server admission.

### P1: HTTP destroys pooling and amplifies retries

Plain HTTP builds a new Hyper client for every attempt; HTTPS builds a new Reqwest/rustls client for every attempt. Dropping the client forfeits connection/TLS pooling. Request bodies are cloned for each backend attempt. The response body is fully collected before the outer loop checks whether status 408/425/429/5xx should retry. Retries have no `Retry-After`, exponential backoff, jitter, idempotency policy or end-to-end deadline; `u8` attempts permit a large immediate retry burst.

Create client pools per effective proxy/TLS/root/pin/config generation, keyed by authority and protocol policy. Retry admission must be method/idempotency-aware, governed by a total deadline and attempt/byte budget, respect `Retry-After`, and use capped exponential backoff with jitter. A retryable response should be drained or discarded under a small policy without retaining an arbitrary body.

### P1: route lookup, handler execution and URL parsing are not compiled policies

Both core local dispatch and Hyper server dispatch linearly scan all route entries while holding a standard mutex, then clone the stored static response. Path/method policy is repeated and URL helpers use manual splitting rather than the structured URI parser already available through Hyper/HTTP dependencies. The current local shortcut can select a local route for an HTTP URL solely because it lacks an explicit port, without proving the request authority owns that route.

Compile exact method/path routes into an immutable index and parameterized routes into a segment trie. Publish route generations atomically so request tasks read a snapshot without a global route mutex. Parse authority, IPv6, userinfo, port, path and query through structured URI APIs; local dispatch must require an explicit loopback/local-listener identity, not a no-port heuristic.

### P1: connection task lifecycle has no explicit close receipt

WebSocket creates a writer and reader task per connection but retains no task/abort handles. Removing a network connection from the manager and setting its state to Closed does not explicitly close the sink or cancel/join both tasks; the reader can live until the remote socket ends. HTTP listener stores only an abort handle and per-connection tasks are detached. Neither provider reports active tasks, cancel latency, orphan tasks or shutdown wall.

Store connection/listener operation ownership and cancellation handles in the network instance. Close and shutdown should quiesce admission, signal protocol close when allowed, cancel after deadline, join/harvest tasks and emit one terminal receipt with open resources and abandoned work.

## 4. Reference-engine evidence and adopted policy

Unreal is the primary provider reference:

- `HTTP/Private/HttpManager.cpp:94-108` tracks queued/in-flight requests, maxima and maximum queue wait. `499+` advances completion from a manager tick rather than synchronously blocking a gameplay caller.
- `HttpThread.cpp:201-217, 274-471` separates new, cancelled, running, rate-limited and completed requests; it starts work only below `MaxConcurrentRequests` and returns completion according to thread policy.
- `HttpModule.cpp:58-109` owns connection/activity/receive/send/total timeout policy, max connections per server, and active/idle/event-loop timing as one effective configuration.
- `HttpRetrySystem.h:77-99` defines capped exponential backoff and jitter. `HttpRetrySystem.cpp:630-652` honors response lockout such as throttling information before applying backoff; `684-711` has explicit cancellation.
- Unreal IP receive remains separately bounded by optional-thread and dispatch policies; HTTP/WS providers should consume the same network/task ownership principles rather than create more runtime pools.

Godot supplies secondary protocol evidence:

- `http_client_tcp.cpp:276-340, 496-546, 684-770` uses a pollable connection state machine, preserves HTTP/1.1 keep-alive and reads configurable chunks.
- `wsl_peer.cpp:294-296, 775-834` limits receive message size, incoming packet capacity, outgoing packet count and outgoing bytes, and reports current buffered bytes.

The target retains Hyper/Tungstenite as proven protocol engines. Zircon must own scheduling, pooling, identity, security admission, backpressure, cancellation, observation and product composition around them.

## 5. Required optimization sequence

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Local copy floor | Preserve the two implemented move-based handoffs and add allocation/copy counters. | Dynamic route and WS receive semantics pass; copy bytes fall by exactly one body/frame payload per operation. |
| M1 Provider composition | Install HTTP/WS backends into the canonical World/session network instance. | One activation receipt and executor set; no private `DefaultNetManager` or independent diagnostics authority. |
| M2 Async lifecycle | Operation handles for bind/connect/accept/request/close with phase deadlines, cancel and task harvest. | Main/frame blocked wall is 0; slow handshake cannot block other accepts; close leaves zero orphan tasks. |
| M3 HTTP policy | Config-generation client pools, authority reuse, route index, server concurrency, streaming limits and structured URI parsing. | Client/TLS builds scale with config generations; exact routes average `O(1)`; retained bodies obey byte limits. |
| M4 Retry and security | Idempotency-aware retry budget/backoff/jitter/`Retry-After`; actual WSS peer-certificate pin validation. | Retry attempts/bytes/wall are bounded; wrong pin/hostname/chain always fails before publication. |
| M5 WebSocket backpressure | Entry+byte+age budgets, coalesced readiness, bounded message/frame size and explicit close ownership. | Stalled consumer memory is `O(configured bytes)`; event work is `O(active ready connections)`, not `O(frames)`. |
| M6 Dynamic acceptance | Current-source TLS/WSS fixtures, concurrency/flood/stall/soak and WPR/ETW captures. | BuildSet-bound P50/P95/P99, allocations/copies, pool hit, queue bytes/age/drop, tasks, RSS, wakeups and energy meet configured gates. |

## 6. Dynamic workload and gates

Exercise HTTP 1/100/1,000 concurrent requests across 1/10/100 authorities, keep-alive reuse, 0/1 KiB/1 MiB/256 MiB declared bodies, chunked bodies, retryable responses, `Retry-After`, slow headers/body and cancellation. Exercise WebSocket 1/100/10,000 connections, 0/1 KiB/1 MiB frames, 1/60-second stalls, slow/invalid handshakes, close races, wrong pins and queue overflow.

Required gates: zero frame-thread wait; client/TLS builds `O(config generations + authorities)`; HTTP connection/task concurrency bounded by config; response/inbound/outbound retained bytes bounded; WebSocket event publications coalesced per ready connection/frame boundary; handshake and close terminal within their deadlines; zero wrong-pin acceptance; zero orphan tasks; and normalized CPU/request, CPU/frame, copied bytes/payload byte, wakeups/second and joules/work unit.

Static source review and the two M0 copy reductions are complete. Dynamic/product acceptance is pending, so this does not warrant a Git milestone commit or quantified WeCom message.
