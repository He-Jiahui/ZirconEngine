---
title: Plugin Net HTTP and WebSocket Providers Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-plugin-net-http-websocket-providers-current-source-performance-review.md
---

# Plugin Net HTTP and WebSocket Providers Protected Plan Routing

## Review ledger status

HTTP **15/15** and WebSocket **19/19** Rust files completed E3 current-source static review. Two owned redundant payload copies were removed and pass standalone formatting/diff gates. Protected `review.md` and `pending.md` remain unchanged because managed Cargo, real TLS/WSS, product, scale, soak and power evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| HTTP/WS feature factories ignore the declared base manager dependency and create private managers/executors | Plugins10, Runtime08E and Runtime42 | Install provider extensions into one World/session network instance and emit one activation/generation receipt. |
| HTTP request and WS connect/listen/accept/handshake synchronously `block_on` callers | Runtime08E and Runtime59 | Use cancellable operation handles with connect/handshake/activity/total deadlines and bounded dispatch completion. |
| WSS pinning checks only that a configured string exists and never validates the peer certificate | Plugins10 security gate and Runtime08E | Fail closed until a custom TLS connector validates peer chain/hostname/pin; add real positive/negative fixtures. |
| WebSocket inbound/events are unbounded by bytes/age; egress is only entry-bounded; readiness event emitted per frame | Runtime08E, Runtime54 and Plugins10 | Add entry+byte+age+owner budgets and coalesced per-connection readiness with overflow counters/policy. |
| HTTP client/TLS pool rebuilt per attempt; full response collected before immediate retry | Plugins10 HTTP milestone and Runtime08E | Pool by effective config/authority; enforce streaming limits and idempotency/total-deadline/backoff/jitter/`Retry-After` retry policy. |
| Route scan/mutex and manual URL parsing duplicate policy and allow ambiguous local dispatch | Runtime08E and Plugins10 | Publish immutable exact/trie route generations and use structured URI plus explicit local-listener authority. |
| Detached HTTP connection tasks and WS reader/writer tasks have no close/join receipt | Runtime59, Runtime08E and Plugins10 | Own cancellation/task handles and prove terminal close/shutdown with zero orphan work. |
| M0 removed one HTTP request-body copy and one WS inbound frame-payload clone | Plugins10 provider implementation record | Preserve move-based handoffs and verify exact copy/allocation reduction in managed current-source tests. |

## Acceptance routing

Static review does not promote either optional provider. Acceptance requires canonical provider composition, asynchronous lifecycle, actual WSS pin verification, pooled/streaming HTTP policy, dual-dimension WebSocket backpressure, explicit task harvest and current-source dynamic receipts.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
