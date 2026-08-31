---
title: Plugin Net Core Runtime Current-Source Performance Review
date: 2026-08-24
status: static_complete_dynamic_pending
scope: zircon_plugins/net/runtime
canonical_owners:
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_plugins/10-first-party-network-source-runtime-editor-dist-catalog-transport-rpc-replication-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/54-runtime-scene-event-mirror-registration-subscription-cursor-backlog-overflow-reclaim-abi-consumer-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetDriver.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/NetDriver.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetConnection.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/NetConnection.cpp
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineSubsystemUtils/Source/OnlineSubsystemUtils/Private/IpNetDriver.cpp
  - dev/UnrealEngine/Engine/Plugins/Online/OnlineSubsystemUtils/Source/OnlineSubsystemUtils/Classes/IpNetDriver.h
  - dev/godot/core/io/http_client_tcp.cpp
  - dev/godot/core/io/packet_peer_udp.cpp
  - dev/godot/modules/websocket/wsl_peer.cpp
---

# Plugin Net Core Runtime Current-Source Performance Review

## 1. Status and frozen scope

`zircon_plugins/net/runtime` has completed a current-worktree E3 static review over **49/49 Rust files**. The frozen source is **5,355 physical lines**, **4,844 non-empty lines**, **180,531 bytes**, **39 test entry attributes** and **2 ignored release-only microbenchmarks**. Its path-and-file-hash fingerprint is `dff357b1221cc735708c0858c36fb5cce6a6b58cfc953cd0508f2c1ad340d555` at repository revision `16122ac757cf3b2e60e43477bda6c5fa94c63ddb`.

This supersedes only the core-runtime portion of the 2026-07-30 static report: the old core freeze had 4,464 lines and a different fingerprint. Focused current cross-boundary reads covered the synchronous `zircon_runtime::core::framework::net::NetManager` contract and the HTTP/WebSocket provider execution paths; the other Net feature packages remain owned by Plugins10 and are not re-accepted here.

`service_types/diagnostics.rs` and `service_types/http_routes.rs` contain shared uncommitted work and were preserved. The first change preallocates bounded event drains; the second moves a local HTTP request into its handler instead of cloning a 65,536-byte benchmark body. These are valid local improvements, but neither changes the network execution architecture.

All 49 files parsed through standalone `rustfmt`; **48/49** pass `rustfmt --check --edition 2021 --config skip_children=true`. The sole mismatch is import ordering in shared-modified `service_types/http_routes.rs`, so this review did not rewrite it. Managed Windows Cargo validation is unavailable for this session. No launchable current-source product, WPR tool or RenderDoc command was available; therefore there is no honest runtime, frame-time, power or GPU receipt and the module remains `dynamic_pending`.

## 2. Architectural conclusion

The dominant problem is not a small hot loop. A single `DefaultNetManager` creates two Tokio multi-thread runtimes plus a dedicated OS command thread, but the TCP/UDP path still executes one command at a time and calls `Runtime::block_on` inside that serial loop. The caller copies payloads, submits a command, waits synchronously for up to two seconds, and can return a timeout while the worker continues an uncancelled side effect.

```text
game/runtime caller
  -> manager registry mutex
  -> payload ownership copy
  -> 1024-entry egress channel
  -> synchronous reply wait (up to 2 s)
       -> one serial command loop on zircon-net-worker
            -> second multi-thread Tokio runtime
            -> block_on bind/connect/send/writable/accept
       -> 1024-entry ingress channel (event drop is silent when full)
  -> diagnostics moves all ingress with usize::MAX
  -> unbounded main event VecDeque
  -> runtime scene publishes at most 256 events per frame
```

This design pays thread-pool and synchronization cost without exposing useful transport concurrency. More worker threads, a `current_thread` builder swap, larger channels or faster `Vec` drains would leave the same head-of-line blocking, cancellation and ownership failures in place.

## 3. Foundations to retain

- Typed endpoint, socket/listener/connection IDs, transport states, errors, events and diagnostics are a usable contract vocabulary.
- Optional HTTP/WebSocket capabilities fail closed when no provider is installed; Beta/Partial truth must remain until product closure.
- The command and worker-ingress channels are entry-bounded at 1,024, and WebSocket egress is entry-bounded at 64; these are useful starting policies, not complete memory budgets.
- Registry locks are deliberately released around HTTP/WebSocket extension callbacks, and failed post-callback commits close or abort staged resources.
- State transitions, TLS policy helpers and shutdown reports are small, named owners that should survive the hard cut where their semantics remain valid.
- The two shared local optimizations remove avoidable growth and request cloning; retain them after the queue and HTTP ownership redesign.

## 4. Structural performance findings

### P0: execution ownership is duplicated but transport work is serialized

Production source contains **2** `Builder::new_multi_thread` runtimes, **1** extra OS thread and **6** worker-side `block_on` sites. The outer runtime owns optional HTTP/WebSocket tasks; the worker thread owns another multi-thread runtime for TCP/UDP, yet `dispatch::run_worker` receives and completes each command serially. `connect_tcp` can wait for the OS connection timeout, and `send_tcp` can wait indefinitely for one stream to become writable. Either operation prevents every other socket, listener and connection owned by that worker from progressing.

Plugins10 also proves that optional provider factories can create private managers. The effective thread/executor count therefore scales with manager/provider instances rather than World/session load. The target is one explicit `NetworkRuntimeInstance` per World/session, backed by an engine-owned I/O executor or one certified transport I/O owner. A dedicated receive lane is optional and workload-gated, not one Tokio pool per manager.

### P0: synchronous contracts make main-thread stalls and ghost side effects legal

`NetManager` exposes synchronous connect/send/poll/listen/request methods. Every TCP/UDP worker request uses `try_send` followed by `recv_timeout(2s)`. The command is not cancelled when the caller times out: a late bind/connect/send can still mutate worker state while the manager registry never commits the corresponding handle. Several TCP/UDP registry mutexes remain held across this wait, serializing unrelated public calls as well.

HTTP and WebSocket compound the problem. HTTP calls `Runtime::block_on` with a per-attempt default timeout of 30 seconds; retries multiply total caller wall time and have no backoff. WebSocket connect, listen and accept also block the caller. A timeout observed after submission is not a deadline and does not provide cancellation, generation fencing or zero-publication failure semantics.

Hard-cut the public transport contract to non-blocking admission plus typed operation handles/events, or an async interface that the Runtime scheduler can await off the frame thread. Every operation needs `(owner, generation, deadline, cancellation, terminal receipt)`. Late completions from timed-out or stale generations must close staged resources and publish no active handle.

### P0: frame work and queue memory are not bounded by the advertised configuration

`NetConfig.tcp_poll_budget_bytes`, `udp_poll_budget_packets`, manifest HTTP timeout and WebSocket message budget are not consumed by the core runtime. The per-frame `run_net_poll_ingress` first calls `diagnostics()`, which moves **all** worker ingress with `usize::MAX`, then calls `drain_events(256)`. The first step can perform unbounded work and expand an unbounded main `VecDeque`; the second merely limits World publication. It also polls ingress twice and locks all registries each frame. `run_net_flush_egress` is registered at `SystemStage::Last` but is empty.

Worker ingress silently discards an event when its 1,024-entry channel is full. Egress and WebSocket egress limit entries but not bytes or age. Main events, loopback WebSocket frames and network WebSocket inbound frames are unbounded; the WebSocket reader clones every owned frame and writes another global event. Large payloads can therefore make a small entry count consume arbitrary memory, while a stalled consumer has no deterministic drop/coalesce/disconnect policy.

The replacement needs entry, byte, age and per-owner quotas at every boundary, plus explicit overflow policy and counters. `TickDispatch` must consume within packet, byte and wall-time budgets; `TickFlush` must admit batches within connection bandwidth and frame budgets. Diagnostics must read an O(1) atomic/snapshot receipt, never drain work as a side effect.

### P0: the synchronous serial worker creates cross-connection head-of-line blocking

One backpressured TCP stream blocks the complete worker at `writable().await`. One slow connect blocks all sockets. TCP accept performs a one-millisecond timed wait after available accepts, so normal empty-tail polling introduces deliberate worker latency. The manager additionally copies TCP/UDP payloads before knowing whether the entry-bounded egress queue can accept them.

Replace request/reply RPC over a serial loop with readiness-driven per-connection state owned by one I/O authority. Receive/send completions should update bounded rings and compact connection receipts; frame stages consume or produce batches. Work complexity must be `O(ready connections + admitted packets/bytes)`, not `O(all API calls * blocking wait)`.

### P1: steady-state allocation and HTTP routing/client policy remain expensive

- Every non-empty UDP poll allocates and zeroes 65,535 bytes even when no packet is ready, then copies each received payload.
- Every TCP poll allocates `max_bytes`; an arbitrary caller value can force a large allocation that is immediately discarded on `WouldBlock`.
- TCP/UDP sends allocate owned payloads before queue admission.
- Local and server HTTP route matching linearly scans the route map, `O(routes * methods)`. The server still clones a dynamic request before invoking a handler.
- Plain HTTP creates a new Hyper client per request; HTTPS creates a new Reqwest/TLS client per request. Responses are fully collected without a response byte limit. The server caps matched request bodies at 1 MiB, but accepts and spawns connection tasks without an admission limit.

Use reusable packet slabs/buffer pools, shared immutable payload owners and admission-before-copy. Exact HTTP routes should use a method/path index, with parameterized routes compiled to a segment trie. HTTP client/TLS pools belong to a configuration generation and response bodies need streaming plus compressed/uncompressed limits. Server connection/request concurrency must be budgeted.

### P1: observability cannot prove the bottleneck is gone

Current diagnostics expose cumulative bytes, last latency, registry counts and queued main events. They omit executor/thread count, command queue entries/bytes/oldest age, caller blocked wall, worker service/queue wall, timeout-with-late-completion, event drops, socket readiness, per-connection backlog, payload copy/allocation bytes, HTTP client/TLS builds, task concurrency, shutdown wall and power-relevant wakeups. `record_net_diagnostics` is called with frame index `0` from the runtime system, so the timeline identity is also incomplete.

Instrumentation must be part of the redesigned queues and operation state machine, not added after implementation. Every dynamic result must bind source revision, BuildSet, target/profile, hardware, logical cores, World/session generation and workload seed.

### P1: skeleton owners create work and misleading surface area

`NetDriver` is empty, `run_net_flush_egress` is a no-op, `NetConfig` is exported but unused, and `ReconnectPolicy` is only referenced by its unit test. These are not reasons for a local delete while plans depend on the names. During the hard cut, either make each owner executable in the unified transport lifecycle or remove the declaration and every stale manifest/test/reference in one migration; do not leave compatibility shims.

## 5. Reference-engine evidence and adopted boundaries

Unreal is the primary source standard for this module:

- `NetDriver.h:113-117` defines `TickDispatch` as the receive/connection-dispatch phase. `NetDriver.cpp:1098-1203` keeps outgoing replication/connection work in `TickFlush`, instruments it, caches frame data before parallel work and only launches per-connection tasks after capability and minimum-connection gates pass.
- `IpNetDriver.cpp:58-83` makes the receive thread optional, limits its circular queue to 1,024 packets with explicit drop-on-full and offers preallocated `RecvMulti` batches with a configurable capacity. `IpNetDriver.h:350-356` and `IpNetDriver.cpp:384-400` bound receive processing by wall time checked every configured packet interval.
- `IpNetDriver.cpp:191-249` unifies direct receive, batch receive and receive-thread delivery behind one packet iterator; `924-968` does not enable receive threading and batch receive simultaneously without a certified path. This supports one transport authority with alternative ingestion modes, not stacked general-purpose runtimes.
- `NetConnection.cpp:5114-5144` clamps bandwidth time after a hitch and repays `QueuedBits` by `CurrentNetSpeed`; it avoids turning one late frame into an unrestricted burst. `PacketHandler.cpp:673-677, 794-798` times individual handler stages. Zircon needs equivalent per-stage cost and backlog evidence.

Godot supplies useful secondary budget evidence:

- `http_client_tcp.cpp:276-340, 496-546, 684-770` advances a persistent HTTP connection through `poll`, honors keep-alive and returns configurable chunks instead of constructing a client and collecting an arbitrary full response per request.
- `packet_peer_udp.cpp:284-329` receives into a retained buffer/ring and explicitly drops when the configured ring is full.
- `wsl_peer.cpp:294-296, 775-834` caps incoming message size, incoming packet capacity, outgoing packet count and outgoing bytes, and exposes current buffered amount.

The adopted design is therefore not simply "use more threads." It is: one owner, explicit dispatch/flush phases, optional gated receive parallelism, batch/readiness processing, per-connection bandwidth accounting, and entry+byte+age backpressure. Unreal's packet-only queue cap is not sufficient by itself; Godot's byte limits close that gap.

## 6. Required optimization sequence

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Product/currentness truth | Freeze canonical manager/provider/World/session ownership and make one launchable current-source client/server test product. | Activation receipt proves exactly one network instance and provider set per session; missing feature fails before gameplay. |
| M1 Baseline instrumentation | Add operation, queue, lock, executor, allocation/copy, drop and wakeup counters before behavior changes. | Fixed loopback and stalled-consumer workloads produce P50/P95/P99 and queue byte/age receipts without diagnostics draining work. |
| M2 Async operation hard cut | Replace synchronous request/reply waits with generation-qualified async/ticket operations, deadline and cancellation. | Frame/main thread network blocked wall is 0; timed-out/stale completion publishes no live handle or event. |
| M3 Unified I/O owner | Remove per-manager executor duplication and serial `block_on`; use engine I/O scheduling or one certified owner with readiness-driven connection states. | Executor/thread count is independent of manager/feature count; one stalled connection cannot stop unrelated connections. |
| M4 Dispatch/flush and backpressure | Implement bounded `TickDispatch`/`TickFlush` with entry, byte, age, per-owner and wall-time budgets plus typed overflow policy. | Per-frame work never exceeds configured packet/byte/time gates; queue memory is `O(configured bytes)` and all drops/coalesces/disconnects are counted. |
| M5 Allocation and HTTP/WS policy | Add reusable packet buffers, admission-before-copy, route index, pooled HTTP/TLS clients, streaming body limits and WebSocket dual budgets. | UDP no-data steady state allocates 0; exact route lookup is average `O(1)`; HTTP client builds are `O(config generations)`, not `O(requests)`. |
| M6 Product integration | Attach HTTP/WS/RPC/Replication/RUDP/Download to the same network instance and make source/native carriers behaviorally equivalent. | First-party App, Editor multiplayer tools and packaged product resolve the same owner/generation and capability receipt. |
| M7 Dynamic acceptance | Run current-source Windows tests, WPR/ETW CPU/allocation/context-switch/power traces and network soak; use RenderDoc only when network state changes rendered output. | Correctness parity plus quantified thread, blocked wall, queue, allocation, throughput, tail latency, RSS, shutdown and energy receipts meet gates below. |

Non-validation work must continue while a dynamic lane is blocked: contract tests, queue/operation types, instrumentation schemas, product activation receipts and hard-cut call-site inventory are all implementable. Performance claims and milestone closure remain blocked until the controlled executable and trace lane exist.

## 7. Dynamic workload and quantitative gates

Run at 1/100/10,000 connections; 1/1,000/100,000 packets or events; 0/1 KiB/1 MiB payloads; 1/2/8/64 logical cores; and 0/1/60-second consumer stalls. Include loopback UDP/TCP, one intentionally backpressured TCP peer, failed/slow connect, HTTP keep-alive and retry, WebSocket flood, queue overflow, cancellation, shutdown and session reload.

The acceptance gates are architectural and measurable:

- main/frame-thread synchronous network wait: **0 ms by contract**;
- executor/OS thread count: **O(engine I/O owners), independent of manager and optional-feature count**;
- queue retained memory: **bounded by configured bytes**, with oldest age and drop/coalesce/disconnect counts;
- dispatch/flush work: **bounded by packets, bytes and wall time per frame**;
- one slow connection: no measurable progress loss for independent connections beyond the configured shared bandwidth budget;
- UDP no-data poll: **0 steady-state heap allocations** after warm-up;
- payload transfer: at most one owned copy at the OS/ABI boundary unless encryption/compression explicitly requires another, with copied bytes reported;
- HTTP/TLS client construction: once per effective configuration/pool generation, not per request;
- route lookup: average `O(1)` for exact routes, `O(path segments)` for compiled parameterized routes;
- timeout/cancel/stale operations: zero orphan handles, zero late state publication and bounded cleanup wall;
- idle network instance: no busy polling; WPR wakeups/context switches and energy are compared on the same hardware/workload against the Unreal/Godot-style bounded design baseline.

No claim that power is "close to Unreal" is valid without the same machine, transport, payload, connection count, build mode and capture window. Report normalized CPU time/packet, bytes copied/payload byte, wakeups/second and joules/10k packets alongside absolute P50/P95/P99.

## 8. Implementation decision for this review

No production source was changed. A local switch from the worker's multi-thread runtime to `current_thread`, a larger channel or another microbenchmark would optimize a temporary architecture while leaving sync waits, uncancelled side effects, head-of-line blocking and unbounded byte retention intact. The current session also cannot run the managed Rust validation required for such a change.

Static review is complete for `zircon_plugins/net/runtime`; dynamic network/product acceptance remains pending. This is not a milestone-completion claim and does not warrant a Git milestone commit or quantified WeCom message.
