---
title: Plugin Net Reliable UDP Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-plugin-net-reliable-udp-current-source-algorithm-performance-review.md
---

# Plugin Net Reliable UDP Protected Plan Routing

## Review ledger status

Reliable UDP **22/22** Rust files completed E3 current-source static review. Four shared-modified files were preserved; the audit made no source change. Protected `review.md` and `pending.md` remain unchanged because Cargo tests, real UDP integration, scale/fault/soak and power evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Feature ignores NetManager dependency and has zero production caller/socket/peer integration | Runtime08E P1-12/M2/M7 and Plugins10 provider graph | Replace the private simulator manager with per-connection protocol instances on the canonical World/session UDP owner. |
| Logical `u64` and wire `u16` sequences can false-ACK across wraps | Runtime08E M7 and Net07 M5 | Freeze wrap-aware packet/channel/message identities and a bounded half-range-safe send/ACK history. |
| Lost final ACK is not repeated for a completed duplicate | Runtime08E M7 | Retain bounded receive/ACK history and re-emit ACK state without redelivery. |
| Reassembly is `O(F^2)` and all incomplete/ordered/outbound state is unbounded | Runtime08E M7, Runtime54 and Plugins10 | Use count+bitset reassembly plus peer/channel entry+byte+age limits, expiration and typed overflow policy. |
| Large fragmented sequence can exceed tick budget forever without attempts advancing | Runtime08E M7 | Schedule and pace packet-level wire bytes fairly; prove every admitted packet progresses or terminates. |
| Fixed RTO ignores RTT; no congestion/window/pacing/path-MTU owner exists | Runtime08E M7 and Net07 M5 | Adopt mature protocol scheduling with smoothed RTT/variance, backoff, cwnd/credit, in-flight byte cap and effective MTU. |
| Simulator lacks real link identity/fault dimensions and diagnostics clone queued payloads | Editor26, Editor25 and Runtime08E observation milestone | Move emulation to real per-link transport and publish bounded revisioned metrics without payload snapshots. |
| Shared patch removes fragment clones, exact-preallocates simulation vectors and groups resend scans | Plugins10 current implementation record | Preserve only where compatible with final protocol; execute the ignored benchmarks and add product-level evidence before claiming gains. |

## Acceptance routing

The valid next work is protocol selection/specification and failing regression models, followed by canonical UDP integration and bounded per-peer/channel state. Local deque/hash optimizations cannot qualify a disconnected simulator. Dynamic acceptance must use the same BuildSet and declared multi-process workload for tail latency, goodput, wire bytes, CPU, memory, wakeups and energy comparison.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
