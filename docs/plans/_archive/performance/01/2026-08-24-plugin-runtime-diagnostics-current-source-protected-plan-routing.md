---
title: Plugin Runtime Diagnostics Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-plugin-runtime-diagnostics-current-source-product-performance-review.md
---

# Plugin Runtime Diagnostics Current-Source Protected Plan Routing

## Review ledger status

Runtime Diagnostics plugin **6/6 Rust files** completed E3 current-source static review at `0a5f22c944d802b0677ebeee5fc3168361bbac5c`; fingerprint `5c294fc43ba181c7061beb491aa54aea265a4d419214b2d868f6aa2698acda9c`. All six files pass standalone rustfmt and the package passes diff check. No source changed. Protected `review.md` and `pending.md` remain unchanged because Cargo, real provider activation, WPR/ETW and power evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Plugin and builtin both own `editor.runtime_diagnostics` | Editor25 G02 + Plugins08 EAP-P1-061 + Editor50 | Keep Editor25 as the unique view owner; plugin registers only a provider lease or is removed from selectable profiles. |
| `plugins://runtime_diagnostics/editor/authoring.zui` is absent in all declared forms | Editor25 P1-57/G03 + Plugins08 M3 | Fail capability admission now; after authority selection, remove the view resource reference or package a real provider-private resource with source/static/native hash parity. |
| Native dist has zero bridge/command/event/ready/unload behavior | Plugins01 native ABI + Plugins08 EAP-P1-028/029 | Export executable typed provider callbacks and lifecycle receipts; descriptor success must not imply provider readiness. |
| No source/generation/clock/freshness/cadence/budget/backpressure/privacy contract | Runtime03 + Editor25 M1-M5 | Establish source-qualified observation session and typed provider leases; bounded immutable snapshots/deltas feed the single Editor pane. |
| Package tests use an empty registry and never resolve resources or data | Plugins08 M1-M3 + Editor50 | Add combined builtin/plugin conflict, resource closure, catalog/profile, provider flow and reload/unload tests across all packaging forms. |
| Package has no recurring hot path; real overhead lives in collection/query/presentation | Runtime03 + Editor25 | Attribute observer CPU/allocation/IPC/wakeups by provider and keep collection/FFI out of UI layout/paint. |

## Acceptance routing

Implementation order is product truth/unique authority -> provider lease/lifecycle -> packaging/catalog parity -> observation session/query cache -> dynamic qualification. Do not close the missing-resource finding with a placeholder ZUI and do not evade collision by inventing another product view ID.

Dynamic acceptance requires a launchable current-source Editor and Runtime, real catalog/profile activation, provider reload/unload/disconnect/flood workloads, and BuildSet-bound WPR/ETW metrics for snapshot/query/presentation latency, observer CPU, RSS/allocation, IPC bytes, wakeups and energy. RenderDoc is relevant only when a real diagnostics view enters a rendered frame and cannot replace CPU/provider evidence.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
