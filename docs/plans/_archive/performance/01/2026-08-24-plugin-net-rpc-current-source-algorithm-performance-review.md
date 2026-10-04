---
title: Plugin Net RPC Current-Source Algorithm Performance Review
date: 2026-08-24
status: static_complete_shared_changes_preserved_dynamic_pending
scope:
  - zircon_plugins/net/features/rpc/runtime
canonical_owners:
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_plugins/10-first-party-network-source-runtime-editor-dist-catalog-transport-rpc-replication-product-integration-review.md
  - docs/plans/optimize/zircon_editor/26-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetDriver.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetConnection.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/NetConnection.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/DataChannel.cpp
---

# Plugin Net RPC Current-Source Algorithm Performance Review

## 1. Status and frozen scope

The RPC feature completed E3 current-source static review over **20/20 Rust files** at revision `080fefe6acd449beded4497dee4a474b9e1f7383`:

| Module folder | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| `zircon_plugins/net/features/rpc/runtime` | 20/20 | 2,764 / 2,473 | 99,595 | 33 / 3 | `f1bc23a224818f71c80166a8e5af314a784228a39bf5bb39e1912eab041281ff` |

The fingerprint is SHA-256 over sorted `repository-relative-path|sha256(file-bytes)` rows joined by LF. Shared changes in `manager/channel.rs` and `manager/session.rs` were preserved. All 20 files parse through standalone rustfmt; 15/20 pass `rustfmt --check --edition 2021 --config skip_children=true`. The five formatting-only mismatches are import ordering in `manager/{channel,dispatch,session,state}.rs` and `tests/queue.rs`, plus wrapping in the shared session test. The scope passes `git diff --check`.

Managed Windows Cargo is unavailable, so none of the 33 tests ran. Product-call search across the other 156 Net Rust files found zero caller of this manager, invocation, queue or handshake API. There is no current-source multi-process RPC executable for WPR/ETW, and RenderDoc cannot diagnose CPU-side RPC dispatch. Dynamic latency, throughput, memory, wakeup, power and abuse acceptance remain pending.

This optional feature is a disconnected in-memory RPC model, not an accepted game-network RPC product.

## 2. Per-file review result

| Module | Reviewed files | Result |
|---|---|---|
| Feature/package surface | `capability.rs`, `feature.rs`, `lib.rs`, `plugin.rs` | Declares canonical NetManager dependency but ignores it and creates a private process-local manager. |
| Manager facade/state | `manager.rs`, `manager/state.rs` | One mutex owns sessions, mutable string registries, quotas, priority work, requests and channel queues; no World, authenticated connection generation or executor owner exists. |
| Channel and dispatch | `manager/{channel,dispatch}.rs` | Implements local DTO queues and synchronous closures, not packet/channel transport; timeout is detected only after the closure returns. |
| Handshake/session/quota | `manager/{handshake,session,quota}.rs` | Handshake token is discarded, challenge is static, peer-selected NetSpeed becomes quota, and closed sessions retain all related state. |
| Registration | `manager/registry.rs` | String-keyed descriptors, validators and handlers overwrite without owner lease, artifact version or unload generation. |
| Tests | `tests/{channel,dispatch,feature_registration,handlers,queue,session,support}.rs` plus inline modules | Direct single-process examples and ignored microbenchmarks only; no wire, transport identity, cancel, fault, flood, unload, cross-process or scale qualification. |

The framework contracts in `zircon_runtime/src/core/framework/net/{rpc,session}.rs` were reviewed with the feature. `RpcPeerRole` and `source_session` are caller-provided data. The default `NetSessionHandshakePolicy` uses the process-wide literal `zircon-rpc-challenge`.

## 3. Current-source local optimization assessment

Three local improvements are present but unexecuted:

- `manager/channel.rs` drains with exact vector capacity. Its ignored 8,192-message/64-byte benchmark models zero output-vector capacity growth versus repeated legacy growth.
- `manager/dispatch.rs` uses a `BinaryHeap` and exact-capacity bounded drain. Its ignored 100,000-depth/64-drain fixture avoids a full queue sort and modeled 99,936 front shifts.
- Shared `manager/session.rs` batches 256 close events over 4,096 sessions so the scan changes from event-by-session work toward events plus sessions.

These are valid allocation/complexity improvements only inside the temporary model. Channel enqueue still clones the full payload to return one copy and retain another. The channel store remains unbounded by bytes, age or owner. The global heap has no fairness, per-principal limit or byte budget. The batched close path intentionally repeats every session report for duplicate terminal events and can amplify output as `matching sessions x duplicate events`; its checked size calculation panics on overflow. None of the three improvements establishes transport correctness or product performance.

## 4. Structural algorithm findings

### P1: RPC has no canonical connection, World or transport owner

The factory ignores its NetManager dependency. The manager is not called by any other Net source file and has no receive/decode/send path. Channel sequence is global per `u8` channel, not per connection, while channel flags accept zero or both reliable and unreliable bits. No actual reliability, packet budget, ordering, congestion or response framing consumes the DTO.

The target is one RPC router per World/session generation owned by Runtime08E and attached to canonical authenticated connections. Incoming packet context supplies connection, principal, role, generation and limits; gameplay code cannot manufacture authority metadata. RPC encode/decode, reliability and delivery receipts use the same connection/channel scheduler as replication and control messages.

### P1: handshake and authority are forgeable

The wire frame decodes a token, then `into_hello_message` discards it. Login succeeds when `challenge_response` equals a static policy string and the player ID is non-empty. `RpcPeerRole`, `source_session` and control messages are direct caller inputs. `begin_handshake()` can create an addressless session. `NetSpeed` accepts an arbitrary remote value and immediately uses it as that session's quota.

Authentication must run before RPC allocation and dispatch. Use a fresh cryptographic challenge or authenticated transport credential, bind the resulting immutable principal/role/session generation to the connection, reject replay, and clamp negotiated rate to server policy and measured transport state. Remove public APIs that accept caller-supplied authority facts.

### P1: timeout does not bound execution or side effects

Handlers are synchronous closures executed on the invoking thread. The clock is checked only after the closure returns, so a 50 ms handler with a 10 ms timeout still blocks for 50 ms and may mutate state before returning `TimedOut`. Expiring a pending request likewise cannot stop work. A slow or re-entrant handler can therefore stall the main/network caller and create late ghost side effects.

Decode and permission validation publish an immutable job to a bounded executor. Admission carries one absolute deadline and cancellation token through queue, handler and response encoding. Timeout/cancel must prevent future commits; mutations cross a generation-checked commit boundary. Main-thread-only work is explicitly marshalled with a per-frame time budget rather than invoked under the network API.

### P1: validation and mutable registries share a global mutex

Schema validator closures run while `NetRpcRuntimeState` is locked. User validation can be slow, call back into the manager and deadlock, or block session, queue and channel work. Descriptor, handler and validator registration overwrites string keys without owner identity, conflict receipt, generation or unload cleanup.

Editor26 compiles a versioned immutable RPC manifest with stable numeric RPC IDs, parameter codecs, maximum encoded sizes, direction, reliability, authority/permission and execution policy. Runtime validates and atomically installs one artifact per BuildSet generation. Handler leases are generation-qualified and quiesced before unload. Codec/validator work occurs outside mutable global locks.

### P1: queue and channel backpressure are not product bounds

The invocation heap is bounded only by entry count, globally. It has no byte, age, in-flight, response, per-connection or per-principal cap. Strict priority can starve lower classes indefinitely. Quota is charged when an invocation is queued, even if it expires before execution. Channel queues have no entry limit at all and drain only by message count. Payload sizes exclude framing and responses.

Use hierarchical admission: global -> World -> connection/principal -> RPC class. Bound queued and in-flight items, encoded bytes, age, rate and response reservation. Apply weighted fair scheduling with aging, explicit overload receipts and no allocation before maximum frame size is validated. Charge final encoded request/response bytes at transport admission and reconcile on terminal outcome.

### P1: request correlation is process-global and non-terminal

Pending requests are keyed only by caller-provided `NetRequestId`; a duplicate silently overwrites the previous invocation clone. There is no connection/generation namespace, wire request/response codec, deduplication, retransmission contract, cancel packet or exactly-one terminal outcome. Queue expiration and handler completion are not integrated with transport delivery.

Correlation keys must include connection/session generation and a monotonic request ID. Keep bounded dedup/terminal caches with age limits. Every accepted request reaches exactly one terminal state: response, denied, overload, timeout, cancelled, disconnected or stale generation. Late completion cannot publish after terminalization.

### P1: quota state is remotely controlled, incomplete and retained

Fixed one-second HashMap windows allocate cloned RPC strings per RPC/session key and are never cleaned. NetSpeed counts only invocation payload bytes, not request framing, response or transport overhead. Closing a session merely changes its enum; sessions, quota rows, pending requests, queued invocations and channel messages remain.

Use stable manifest IDs and dense per-connection counters. Rate policy is server-owned, token-bucket or equivalent with burst limits, and accounts encoded traffic. Connection teardown must cancel/quiesce work and erase all generation-owned rows in bounded time. Duplicate close events are idempotent and yield one terminal receipt.

### P1: observation cannot prove cost or abuse handling

Reports are returned as copied DTOs but there are no bounded counters/traces for decode, auth, permission, queue delay, handler wall/CPU, main-thread wait, encoded bytes, drops, cancellation, stale completion, high-water memory or abuse closure. There is no principal redaction policy.

Publish generation-qualified RPC telemetry into the common diagnostics timeline. Payloads and credentials are never copied into traces. Sampling must still retain terminal errors and aggregate high-water/rate data required for WPR correlation and Editor inspection.

## 5. Unreal evidence and adopted policy

Unreal is the primary structural reference:

- `NetDriver.h:211-222` documents the full RPC route: NetDriver and NetConnection select the owning ActorChannel, the channel serializes the RPC ID/parameters, the receiving driver maps the sender address back to the connection, and the connection routes by channel ID. Authority is therefore transport-derived rather than supplied by the RPC payload.
- `NetDriver.h:233-240` makes packet sequence per connection and reliable bunch sequence per channel. Zircon's process-global channel sequence does not match that ownership model.
- `NetDriver.h:814-835` passes an explicit `UNetConnection*` through `InternalProcessRemoteFunction`; the call is not authorized from a free caller role/session pair.
- `NetConnection.cpp:617-643` creates RPC DoS detection per server connection, derives remote address and player identity from that connection, and closes the connection on an abuse decision.
- `NetConnection.cpp:651-659` installs the same connection and its RPC DoS owner into the replication system, showing that RPC and replication share connection identity rather than private managers.
- `NetConnection.cpp:2562-2583,5108-5143` accounts packet bytes into per-connection `QueuedBits` and drains by connection speed over elapsed time. The queue budget includes packet cost and is owned by the connection.
- `NetDriver.h:691` and `NetDriver.cpp:311` expose thresholded parallel connection ticking with explicit configuration and replication-system permission. Parallelism follows established ownership; it is not a substitute for fixing Zircon's synchronous handler contract.
- `NetConnection.cpp:3480-3542` rejects replay-prone RPC processing in invalid connection state and brackets received packets with per-connection RPC DoS accounting.

Zircon should adopt these boundaries, not copy Unreal classes. The minimum viable pipeline is authenticated connection -> validated compiled RPC ID/codec -> permission and hierarchical admission -> bounded cancellable execution -> generation-checked commit -> encoded response -> transport receipt -> terminal telemetry.

## 6. Required optimization sequence

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Truth and regression freeze | Keep RPC unavailable as a product; preserve local allocation wins and add failing identity, timeout, teardown and queue tests. | Tests expose token discard, role/session spoof, late side effects, duplicate request overwrite, priority starvation, duplicate close amplification and retained rows. |
| M1 Compiled RPC artifact | Stable IDs/codecs, max sizes, permission, direction, reliability, execution policy and BuildSet compatibility. | Golden codec/artifact tests reject malformed, oversize, wrong-build and unauthorized requests before allocation or state mutation. |
| M2 Authenticated connection binding | One Runtime08E owner derives principal, role, session generation and server rate policy from canonical transport. | No public dispatch/control API accepts authority facts; replay and stale-generation traffic are deterministically rejected. |
| M3 Bounded cancellable execution | Hierarchical item/byte/age/rate/in-flight budgets, fair scheduling, absolute deadline/cancel and explicit main-thread bridge. | Slow handler cannot block unrelated connections or the network loop; timeout prevents late commit; main-thread blocked wall is budgeted and traced. |
| M4 Correlation and terminal state | Generation-qualified request IDs, response/error/cancel wire messages, dedup and bounded terminal cache. | Loss/reorder/duplicate/disconnect produces exactly one terminal outcome and no cross-connection collision. |
| M5 Channel and transport integration | Per-connection/channel sequencing, reliability policy, packet-byte admission, congestion/pacing integration and response reservation. | Items/bytes/age remain within policy under flood; no class starves; wire bytes match admission receipts. |
| M6 Lifecycle and observation | Idempotent connection teardown, handler lease quiescence and common RPC metrics/trace schema. | Teardown leaves zero session/queue/request/quota/channel rows and zero stale task/callback; payload/credential data is absent from traces. |
| M7 Dynamic qualification | Current-source server/client processes, fault/flood/soak, WPR/ETW and power capture; RenderDoc only for a coupled render-frame regression. | Publish BuildSet-bound P50/P95/P99 decode/queue/handler/response latency, RPC/s, CPU, RSS, main-thread wait, wakeups, encoded bytes and joules per successful RPC. |

Static current-source review is complete. Dynamic/product acceptance is pending, shared source work is preserved, and no Git milestone commit or quantified WeCom notification is warranted.
