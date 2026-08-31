---
title: Runtime Telemetry Producer Generation and Consumer Polling Currentness Adoption
date: 2026-08-23
scope:
  - zircon_runtime/src/diagnostic_log
  - zircon_runtime/src/runtime_diagnostics
status: static_complete_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_runtime/44-process-diagnostic-log-router-filter-record-queue-sink-durability-rotation-crash-multi-session-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
related_reports:
  - docs/plans/performance/01/2026-08-23-runtime-diagnostic-log-bounded-sink-current-architecture-review.md
  - docs/plans/performance/01/2026-08-23-runtime-diagnostics-domain-generation-availability-current-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/OutputDeviceRedirector.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/OutputDeviceFile.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/ProfilingDebugging/CountersTrace.h
---

# Runtime Telemetry Producer Generation and Consumer Polling Currentness Adoption

## 1. Composite coverage

The current `diagnostic_log` surface is 31 Rust files, 4,121 physical / 3,688 non-empty lines, 134,682 bytes, and 45 test markers. The current `runtime_diagnostics` facade is 4 files, 389 physical / 348 non-empty lines, 13,506 bytes, and 6 test markers. Runtime44, Runtime03, both related current-source reports, all current facade files, the sink/filter/settings deltas, and the editor/dynamic-session consumers were reconciled. Test-marker counts are inventory only; no Rust test or product executable was run by this pass.

Existing foreign edits in `diagnostic_log/level/compiled.rs`, `settings.rs`, sink worker/tests and `runtime_diagnostics/collect.rs` are preserved. The related reports own their M0 implementation evidence. This adoption record does not duplicate or claim those source edits; it closes the missing producer/consumer semantics and dynamic acceptance plan.

## 2. Accepted current reductions

The present tree contains useful deterministic reductions that should remain:

- a known-full best-effort queue can reject before evaluating the lazy message closure;
- the critical caller wait is bounded by a default 2 ms instead of an unbounded blocking send;
- control admission uses deadline-bounded `send_timeout` rather than a `try_send`/`yield_now` loop;
- a worker batch computes one wall-clock display timestamp rather than one per record;
- an empty compiled scope rule set returns its minimum without a trie hash probe;
- virtual-geometry availability uses a scalar query instead of cloning the debug payload;
- store-only diagnostics omit the profiling timeline clone, and periodic log output snapshots current values without cloning retained history;
- domain values are recorded into the authoritative Core store rather than a temporary clone, so retained history is no longer discarded after each collection;
- the editor schedules diagnostics payloads only for visible Runtime Diagnostics or Performance Timeline content and coalesces repeated render-publication refreshes.

These are static work-count or ownership improvements, not latency, CPU, RSS or power measurements. In particular, one timestamp per batch is output-time formatting; it does not provide producer event time or an ordering sequence.

## 3. Remaining structural bottleneck

All three public collection variants call `collect_runtime_diagnostic_domains`; both store variants then call `record_diagnostic_domains`. Therefore a consumer query is also a producer sample. The Dev dynamic session calls the current-store collector every second for periodic logging. The editor calls the full collector after successful visible diagnostic publication. Dynamic diagnostics APIs and tests call the full collector independently. Multiple viewers or a 120 Hz consumer can append repeated render/physics/animation values to history even when the domain producer generation did not change, altering sample count, EMA and min/max semantics. Visibility gating reduces hidden work, but it cannot make consumer-owned sampling correct.

The correct boundary is producer-owned publication. Render, physics and animation owners publish a sealed summary/detail artifact once for their own generation. Core commits each generation to `DiagnosticStore` at most once. Consumers request a domain mask and `if_newer(generation)` or a bounded delta/page; queries never append samples. Periodic logging consumes only changed current rows since its last generation and applies the log filter before formatting. Full devtools/profile capture remains an explicit, budgeted operation.

The logging side also remains a process architecture issue rather than a formatting micro-optimization: authority is still per loaded image, records own scope and message strings, queue capacity is record-count only, Warn/Error share the same lane and may spend up to 2 ms per item on a frame caller, console and file are serialized by one worker, and rotation/crash artifact/byte admission/sink isolation are incomplete. One slow sink can age all records; K critical calls have a caller-wait ceiling of `K * timeout`.

## 4. Unreal constraints

Unreal `CountersTrace.h` stores counter state at the declaration/producer and `Set` suppresses an unchanged value unless the counter is explicitly unchecked; `SetIfDifferent` makes that contract explicit. A viewer does not manufacture another producer sample merely by reading it. This supports generation-owned publication and changed-value emission, not query-owned history mutation.

Unreal `OutputDeviceRedirector.cpp` routes buffered records through an MPSC queue, wakes a dedicated primary logging thread only on the empty-to-nonempty transition, uses explicit idle/fence synchronization for flush, and distinguishes device/thread/panic behavior. `OutputDeviceFile.cpp` separately owns buffered file output and periodic/forced flush. Zircon should adopt the ownership, wake and fence constraints while retaining bounded byte admission; it should not copy Unreal's unbounded buffering or global C++ lifetime model.

## 5. Optimization plan

### M0: freeze semantics and counters

Retain the current deterministic M0 changes. Add source-independent behavior tests proving a consumer query never changes authoritative history, same producer generation commits at most once, hidden domains perform zero manager resolve/query/write, and critical admission reports caller wait/drop/overflow. Do not accept batch display time as producer event time.

### M1: single process log router

App owns one `ProcessLogRouter` generation and exposes it to runtime/editor/plugins through the host ABI. Runtime libraries acquire leases; they do not initialize independent global controllers. Add stable category/scope IDs, producer monotonic sequence/event time, structured fields, per-record and queue-byte budgets, and severity-reserved admission with a coalesced overflow receipt.

### M2: isolated sink execution and durability

Route console, rotating file, editor console and crash/emergency sinks through independently budgeted consumers or nonblocking fan-out so one slow sink cannot stall the others. Reuse worker scratch buffers, make steady-state batch allocation zero, use event/completion waits instead of shutdown `yield_now`, and keep joins outside lifecycle locks. Flush/shutdown/crash return explicit fence receipts.

### M3: producer-owned telemetry generations

Each runtime domain publishes immutable `DiagnosticDomainSnapshot { generation, summary, detail_ref }`. A Core telemetry coordinator deduplicates generation commits and maintains bounded history once. Consumer APIs accept domain mask, last generation, row/byte/time budget and summary/detail choice; they return current/delta/page without recording. Profiling capture has its own generation and never rides an ordinary pane refresh.

### M4: product qualification

Editor diagnostics, Performance Timeline, dynamic API, periodic log and plugin telemetry consume the same generations. Exercise process multi-session/load-unload, plugin failure, slow/blocked sink, crash, hidden/visible editor surfaces and current-source runtime/editor products. Only measured receipts may promote the module from pending.

## 6. Dynamic acceptance matrix

1. Logging: `1/1k/100k` records/s, `1/64` callers, `0/10/1k` scopes, `16 B/4 KiB/1 MiB` messages, `0/10/100 ms` sink delay and multiple queue byte/count budgets. Record caller p50/p95/p99/max, lazy evaluations, allocated/copied bytes, critical wait sum, queue depth/bytes/age, drops/overflow receipts, worker wake/CPU, fence latency, sink isolation, RSS and energy.
2. Diagnostics: `1/541/10k` series, `1/3` domains, `0/30/60/120 Hz` and multiple consumers, same/changed producer generation, hidden/summary/detail views. Require producer sample count to be independent of consumer count/rate, same-generation store writes and domain builds `<= 1`, hidden resolve/query/write/clone `= 0`, and bounded delta bytes/rows.
3. Lifecycle: concurrent flush/shutdown, queue saturation, worker/sink panic or hang, dynamic library unload and two runtime sessions. Every accepted record/fence reaches one terminal result; main/frame callers perform zero file I/O and no lifecycle-lock join.
4. WPR/ETW on one current-source Windows executable records CPU samples, locks/waits, context switches, thread lifetime, file I/O, working set and energy for idle, diagnostics-visible and log-storm cases. RenderDoc is used only to verify that a visible diagnostic/debug overlay corresponds to the same render generation; it cannot prove logging/telemetry CPU performance.

## 7. Current result

- Static current-source composite coverage is complete for 35/35 Rust files across the two folders.
- Existing M0 reductions are adopted without claiming their foreign source edits.
- Consumer polling still mutates producer history and process logging remains unconverged; both folders remain dynamically pending.
- No current-source Cargo, product, WPR/ETW or RenderDoc evidence was produced by this pass.
