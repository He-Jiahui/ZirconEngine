---
title: Plugin Net Reliable UDP Current-Source Algorithm Performance Review
date: 2026-08-24
status: static_complete_shared_changes_preserved_dynamic_pending
scope:
  - zircon_plugins/net/features/reliable_udp/runtime
canonical_owners:
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_plugins/10-first-party-network-source-runtime-editor-dist-catalog-transport-rpc-replication-product-integration-review.md
  - docs/plans/zircon_plugins/07-net.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Net/NetPacketNotify.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Net/NetPacketNotify.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/NetConnection.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Net/DataBunch.h
  - dev/godot/thirdparty/enet/protocol.c
  - dev/godot/thirdparty/enet/peer.c
---

# Plugin Net Reliable UDP Current-Source Algorithm Performance Review

## 1. Status and frozen scope

The Reliable UDP feature completed E3 current-source static review over **22/22 Rust files** at revision `080fefe6acd449beded4497dee4a474b9e1f7383`:

| Module folder | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| `zircon_plugins/net/features/reliable_udp/runtime` | 22/22 | 1,929 / 1,728 | 69,562 | 18 / 3 | `282ba9a2457beb1c9ba50ef2a088318a3e5ac0747cd6867b474e42604caea1d0` |

The fingerprint hashes sorted repository-relative path plus file bytes. Four manager files contain shared uncommitted work and were preserved: `assembly.rs`, `delivery.rs`, `receive.rs` and `resend.rs`. All 22 files parse through standalone rustfmt; 20/22 pass `rustfmt --check --edition 2021 --config skip_children=true`. The two failures are import ordering in shared-modified `delivery.rs` and `receive.rs`. The scope passes `git diff --check`.

Managed Windows Cargo is unavailable in the current validation lane. None of the 18 tests ran, including the three ignored release microbenchmarks. There is no production caller outside this feature, no real UDP/peer integration and no current-source executable on which WPR can measure this code. RenderDoc is inapplicable because this is CPU/network protocol work with no rendered-output workload.

This module remains Beta/optional/default-off. Static review does not promote it as a transport.

## 2. Per-file review result

| Module | Reviewed files | Result |
|---|---|---|
| Feature/package surface | `capability.rs`, `feature.rs`, `lib.rs`, `plugin.rs` | Declares a dependency on canonical NetManager but the factory ignores it and creates a private in-memory manager. |
| Public manager and wire codec | `manager.rs`, `packet.rs` | Exposes direct synchronous state operations and an isolated 16-bit packet header; it has no endpoint, peer, socket, task or lifecycle identity. |
| Protocol state | `manager/{assembly,delivery,receive,recovery,resend,send,state,stats}.rs` | Implements process-local fragmentation, simulated loss/reorder, ACK removal, fixed-time resend and ordered buffering under one mutex. It is not connected to wire I/O. |
| Tests | `tests/{delivery,feature_registration,mod,packet,receive,recovery,resend,send}.rs` plus three inline benchmark modules | Covers direct manager examples and descriptor registration. It has no malformed/fuzz/property, real socket, cross-process, peer isolation, ACK-loss, congestion, soak or power test. |

The framework DTO boundary was followed through `zircon_runtime/src/core/framework/net/reliable.rs`. Its config exposes only payload MTU, fixed resend timeout, attempt cap and receive-window length. The receive window is used only to trim a duplicate-history deque; it is not a sender window, flow-control window or wire wrap-safety contract. `rtt_ms` can be assigned for display but never influences resend scheduling.

## 3. Shared current-source optimization assessment

The shared patch contains three valid local improvements:

- Fragment insertion moves the owned payload instead of cloning it, eliminating one clone and `payload.len()` copied bytes for each accepted fragment.
- Deterministic delivery simulation precomputes exact delivered/dropped capacities when the iterator size and drop cadence are known, avoiding vector growth for that synthetic workload.
- Due-resend grouping replaces a legacy repeated outbound scan with a `HashSet`/`HashMap` pass. For the included 1,024-packet all-due fixture, packet inspections fall from **1,048,576 to 1,024** before hash operations and payload cloning.

The benchmarks are useful local evidence designs, but they are ignored and unexecuted. They also benchmark an in-memory simulator, not socket throughput, congestion behavior, main-thread cost or energy. These changes must be preserved during migration only where the final protocol still owns the same handoff; they do not validate the current architecture.

## 4. Structural algorithm findings

### P1: the feature is not a reliable UDP transport

The feature factory discards its declared NetManager dependency. The manager has no UDP socket/connection/peer, no I/O task, no receive/flush phase and no shutdown generation. Every production call site search outside the feature returns zero. `simulate_outbound_delivery` only partitions caller-owned packets in memory.

The repair must install one per-connection reliable protocol instance into the canonical World/session network owner. The I/O task owns mutable protocol state; Runtime/main thread exchanges bounded commands and immutable completion batches. A second private manager or new polling thread is prohibited.

### P1: wire identity and ACK wrap are unsound

Logical packets use `u64 sequence`, string channel and `u16` fragment fields; the wire header uses `u16 sequence`, `u8 channel`, `u8` fragment count and an unrelated `u16 fragment_id`. No encode/decode adapter connects the models.

`acknowledge_wire_header` compares only the low 16 bits of every unbounded `u64` outbound sequence. Once more than one wrap-equivalent sequence is retained, one ACK can remove multiple messages. The existing wrap test queues only one full 16-bit space and therefore cannot expose the second collision. A bounded half-range-safe send window and generation-relative sequence comparison must exist before ACK processing.

Packet sequence, reliable channel sequence and fragmented-message identity must be separate typed fields. Wire version, peer/session generation, length, flags and ACK-history validity need a written protocol specification and golden vectors.

### P1: ACK loss can force a false disconnect

After a sequence is completed, a duplicate fragment returns `DuplicateFragment` without another ACK. If the original ACK is lost, the sender retransmits, the receiver withholds the replacement ACK, and the sender eventually drops the sequence at `max_resend_attempts` and marks the whole manager disconnected.

Duplicate valid data inside the receive history must re-emit current ACK history without redelivery. ACK generation and ACK-of-ACK/history retention belong to the per-peer packet notification state, not a one-shot receive report.

### P1: memory and ordered delivery are unbounded

Outbound packets, resend state, incomplete assemblies and ordered payloads have no entry, byte, age or peer limits. A first fragment may allocate up to 65,535 optional fragment slots, and incomplete assemblies never expire. A single ordered gap retains every later payload indefinitely. All string channels share one `next_ordered_sequence`, so loss on one logical channel creates head-of-line blocking for every channel.

Admission must bound per-peer and global messages, fragments, payload bytes, age and in-flight wire bytes before allocation. Ordered state belongs per channel. Overflow, gap timeout and disconnect/drop policy must be typed and observable. Retained memory must remain `O(configured peer/channel byte windows)`, independent of attacker-declared counts.

### P1: reassembly is `O(F^2)` and resends all fragments

Every accepted fragment scans the entire `Vec<Option<Vec<u8>>>` with `all()`. Completing `F` fragments therefore performs `O(F^2)` slot inspections. Only a whole-message ACK exists, so losing one fragment resends and clones every fragment in the message.

Track received count and a bounded bitset for `O(F)` total insertion bookkeeping. Use distinct packet ACK history plus message/fragment metadata so only missing packets are retried. Validate fragment count, index, declared total length and accumulated bytes before storage, and expire partial messages by generation-qualified deadline.

### P1: resend scheduling, budget and timeout are not viable

Each tick scans all resend states, sorts due sequences, scans outbound, builds hash tables and clones every due payload while holding the single mutex. `resend_due()` disables the byte budget with `usize::MAX`. The budget counts payload only, not headers/wire bytes.

More seriously, if all fragments for one sequence exceed the tick byte budget, that sequence is skipped without updating send time or attempts. It remains due forever and can never reach the attempt cap. Fixed `resend_timeout_ms` ignores the recorded RTT; new messages start with `last_sent_at_ms = 0`, so correctness depends on callers using the same implicit time origin.

Use a timer heap/wheel keyed by next due packet, a packet-level byte-accurate pacing budget and fair queues. Record actual first/last send time. Derive bounded RTO from smoothed RTT and variance, apply exponential backoff/Karn-style sample exclusion, and separate retry exhaustion from connection health policy.

### P1: no congestion, flow control or path-MTU policy exists

There is no congestion window, receiver credit, pacing, bandwidth estimate or in-flight byte cap. `mtu_bytes` is treated as payload bytes, so the 10/14-byte protocol header and lower transport overhead exceed the configured datagram target. A maximum accepted message can also be cloned into both the returned report and outbound queue, doubling retained payload before wire encoding.

Effective payload MTU must subtract the final authenticated wire overhead and respond to path/transport policy. Admission must stop before the send/ACK window wraps or in-flight byte/cwnd limit is exhausted. Backpressure returns a typed receipt; it must not append to an unbounded deque.

### P1: simulator and diagnostics cannot qualify the algorithm

Loss is only every-N, reorder is chunk reversal, and there is no latency, jitter, duplication, bandwidth, corruption, burst model or direction/peer identity. A deserialized `drop_every_nth_packet = 0` reaches modulo by zero even though the builder filters zero. `pending_packets()` deep-clones all queued payloads under the mutex, so diagnostics can amplify memory bandwidth and lock time.

Network emulation belongs at the real per-link transport boundary with a validated effective profile. Observation should expose bounded counters/high-water marks and revisioned snapshots, never clone all queued packets.

## 5. Reference-engine evidence and adopted policy

Unreal is the primary architecture reference:

- `NetPacketNotify.h:36-60` separates packet sequence, acknowledged sequence and bounded delivery history; its sequence number is explicitly wrap-aware.
- `NetPacketNotify.h:103-140,188-191` rejects updates outside the valid half-range and prevents sending when the sequence-history window is full.
- `NetPacketNotify.h:143-180,214-310` retains ACK records, ACK-of-ACK state and delivery/loss history, including history overflow re-synchronization and synthesized NAKs.
- `NetConnection.cpp:88-105` makes order correction, missing/cached packet bounds and congestion control explicit policies rather than consequences of an unbounded ordered map.
- `DataBunch.h:38-45,136-146` distinguishes channel sequence, packet ID and reliable state. Zircon's one sequence field cannot safely own all three meanings.

Godot's vendored ENet is secondary algorithm evidence and a candidate maturity baseline, not code to copy blindly:

- `protocol.c:574-622,691-740` validates fragment count/index/total length and matches fragments against an existing bounded command.
- `protocol.c:853-910` updates smoothed RTT and variance; `1366-1401` applies bounded timeout/backoff; `1456-1517` gates reliable sends by channel windows, packet throttle, peer window and in-flight reliable bytes.
- `peer.c:125-179` derives fragment payload from MTU and rejects excessive fragment counts; `474-496` exposes explicit timeout limits.

The canonical Runtime08E decision stands: select a mature reliable datagram protocol/library as the core, or first provide an equivalent protocol specification, interoperability fixtures and qualification matrix. Do not keep growing this disconnected simulator into production by local patches.

## 6. Required optimization sequence

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Truth and regression freeze | Keep feature unavailable as transport; preserve shared local optimizations and add failing model tests for the listed bugs. | Tests expose double-wrap false ACK, lost-ACK duplicate, oversized-budget starvation, zero cadence, cross-channel HOL and incomplete-fragment retention. |
| M1 Protocol selection/spec | Choose mature core or freeze versioned wire/sequence/channel/fragment/ACK/security rules. | Golden encode/decode/interoperability vectors and malformed/property/fuzz corpus pass across wrap boundaries. |
| M2 Canonical integration | Per-World/session/peer protocol instances on the existing UDP I/O owner with generation lifecycle. | Real two-process send/receive/cancel/close; no private manager/thread/socket and zero stale-generation delivery. |
| M3 Bounded windows/reassembly | Circular ACK history, per-channel ordering, packet/message identity and byte/age-bounded reassembly. | Insert bookkeeping is `O(fragments)`; all retained state obeys configured byte/time/peer limits under hostile input. |
| M4 RTO/congestion/pacing | Smoothed RTT/variance, backoff, timer scheduling, cwnd/credit, fair packet-level wire-byte pacing and effective MTU. | No sequence is starved by a tick budget; in-flight bytes never exceed policy; one slow/lossy peer does not stall others. |
| M5 Observation/emulation | Real link fault injection and bounded revisioned per-peer/channel metrics. | Hidden Editor polling is zero; observation clones no payload; loss/latency/jitter/duplication/bandwidth profiles are reproducible. |
| M6 Dynamic qualification | Current-source loopback, dual-process, scale/fault/soak/security and WPR/ETW captures. | Publish BuildSet-bound P50/P95/P99 latency, goodput/wire bytes, retransmit ratio, CPU, RSS, wakeups and joules/useful byte on identical workloads. |

Static current-source review is complete. Dynamic/product acceptance is pending, shared source work is unowned by this audit, and no Git milestone commit or quantified WeCom notification is warranted.
