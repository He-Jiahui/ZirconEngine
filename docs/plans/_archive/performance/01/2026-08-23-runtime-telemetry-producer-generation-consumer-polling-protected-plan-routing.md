---
title: Runtime Telemetry Producer Generation and Consumer Polling Protected Plan Routing
date: 2026-08-23
status: routing_only
related_report:
  - docs/plans/performance/01/2026-08-23-runtime-telemetry-producer-generation-consumer-polling-currentness-adoption.md
protected_targets:
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
---

# Runtime Telemetry Producer Generation and Consumer Polling Protected Plan Routing

This record supplies merge input only. No protected file was modified.

## `review.md` suggestion

Do not promote `zircon_runtime/src/diagnostic_log` or `zircon_runtime/src/runtime_diagnostics`. Their 35/35 Rust files have composite static coverage and current M0 work-count reductions, but consumer queries still write producer history, process log ownership/sink isolation/durability are incomplete, and current-source Cargo/product/WPR evidence is absent.

## `pending.md` suggestion

`zircon_runtime/src/{diagnostic_log,runtime_diagnostics}`: 35/35 current-source static composite review complete. Preserve bounded/lazy queue, batch timestamp, scalar availability, current-only snapshot and visibility/coalescing M0. Pending App-owned process router, structured byte-bounded admission, isolated sink/fence/crash lifecycle, producer-owned domain generations, query-if-newer/delta APIs, and current-source Cargo/WPR/product evidence. Consumer query rate must not change history sample count or EMA.

## Canonical routing

| Issue | Target plan |
|---|---|
| process router, filter/record/queue/sink, rotation, durability, crash, multi-session ownership | Runtime44 |
| producer domain generation, DiagnosticStore commit-once, query-if-newer/delta, profiling capture isolation | Runtime03 / Runtime07 |
| App-owned runtime-library log lease and current product lifecycle | App01 / Runtime07 |
| plugin/runtime/editor host ABI for structured telemetry, budget and terminal receipts | Interface01/03 |
| visible diagnostics/timeline demand, bounded rows and generation consumption | Editor11 |
| render summary/detail generation, counters and trace qualification | Render17 |

## Promotion gates

1. Same producer generation commits at most once regardless of consumer count or `0/30/60/120 Hz` query rate; a query cannot append history.
2. Hidden domains resolve/query/write/clone zero work; changed summary/detail data is bounded by explicit row/byte/time budgets.
3. Main/frame log callers perform zero file I/O, wait zero on best-effort admission, and have a separately stated critical policy; slow sinks do not age unrelated sinks.
4. Process multi-session, plugin failure, flush/shutdown/crash and dynamic unload return terminal generation/fence receipts without busy-wait or lifecycle-lock join.
5. Managed Windows Cargo and WPR/ETW run on the same current-source runtime/editor executable and report caller p99/max, wait/lock, thread/I/O, RSS and energy. RenderDoc only accepts visible generation parity.
