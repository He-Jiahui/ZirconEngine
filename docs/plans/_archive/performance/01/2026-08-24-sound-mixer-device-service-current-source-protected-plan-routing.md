---
title: Sound Mixer and Device Service Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-sound-mixer-device-service-current-source-algorithm-performance-review.md
---

# Sound Mixer and Device Service Current-Source Protected Plan Routing

## Review ledger status

Sound mixer/device service **22/22 Rust files** completed E3 current-worktree static review at `7fe97290fd3b0350c2c0f404fd00ad2d18f1335d`; fingerprint `831db93ea61e5f333e1ef95f5ff8b4c27ff73088858afed51109401f634dd234`. All files pass standalone rustfmt and scoped diff check; no source changed. Protected ledgers remain unchanged because Cargo, product Sound activation, real CPAL/device transition, WPR/ETW and power evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Kira graph/source apply and rollback execute under global state mutex | Plugins11 P1-04/P1-10/P1-15 + Runtime08b/Runtime59 | Add immutable off-lock prepare and bounded audio-command publish with last-good acknowledgement. |
| Device configure destroys old output before replacement is proven | Plugins11 P1-03/P1-13 + Runtime48 | Add supervisor prepare/swap/recovery/backoff and desired/applied/last-good generations. |
| Inactive graph accepts effects/controls later rejected by Kira M1 | Plugins11 P1-04 + Editor17 | Make capability/backend profile explicit and reject unsupported authoring before persistence/commit. |
| Manual render/callback methods remain public but always Unsupported | Runtime08b + Plugins11 | Remove from Ready capability or provide a real non-realtime/headless owner; forwarding stubs do not count. |
| Deep snapshot and preset graph reconstruction have no cadence budget | Plugins11 P1-15 + Editor17/Editor25 | Publish immutable graph/status pages and cheap preset metadata; explicit budgeted full capture only. |
| Backend status spans config/state generations and reports stopped as Ready | Runtime03 + Editor25 | Publish one supervisor page with requested/applied generation and distinct transition states. |
| Device enumeration synchronously probes every call | Plugins11 P1-13 + Editor17 | Supervisor publishes async cached catalog generation with stale/refresh/error state. |
| Settings/parameters lack persistence and applied-generation ownership | Plugins11 P1-03 + Runtime48 | Route typed live/graph/device/restart settings through one transaction and reject orphan parameters. |

## Acceptance routing

Implementation order is product harness -> capability truth -> graph transaction -> source migration -> device supervisor -> observation/config -> dynamic qualification. Do not close this scope by caching a DTO while device/graph work still executes synchronously under shared state.

Dynamic acceptance records source/build/config/device/workload identity, graph scale/churn, audio/control/main CPU, state-lock and command P50/P95/P99, allocations/RSS, snapshot bytes, enumeration/UI latency, wakeups, callback/xrun/latency, device recovery time, power and audible parity.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
