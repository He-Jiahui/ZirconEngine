---
title: Sound Dynamic Events ABI Current-Source Algorithm Performance Review
date: 2026-08-24
status: static_complete_dynamic_pending_shared_index_fix_preserved
scope:
  - zircon_plugins/sound/runtime/src/dynamic_events
  - zircon_plugins/sound/runtime/src/dynamic_event_abi
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/engine/state/dynamic_events.rs
canonical_owners:
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/58-runtime-plugin-interface-bridge-slot-generation-strong-weak-native-vm-lifecycle-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/AudioModulation/Source/AudioModulation/Private/AudioModulationSystem.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/AudioModulation/Source/AudioModulation/Private/AudioModulationSystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
---

# Sound Dynamic Events ABI Current-Source Algorithm Performance Review

## 1. Status and frozen scope

The complete Sound dynamic-event dispatch/executor/ABI slice completed E3 current-worktree static review over **22/22 Rust files** at revision `2a1299f8bf8e5a3012860ff07a6fcf528e4721d8`:

| Module slice | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| event catalog, dispatch, executors, ABI and service surface | 22/22 | 1,101 / 990 | 38,864 | 4 / 1 | `dbeb352940c144e0233b64b601125b11cb73fa57686ee19953c454d72ad64b93` |

All files pass standalone `rustfmt --check --edition 2021 --config skip_children=true`; scoped diff check has line-ending warnings only. Two shared import-order edits in `dynamic_events/{dispatch,handlers}.rs` are preserved. Managed Windows Cargo, a current-source product executable, plugin load/unload harness, ETW and power evidence remain unavailable; tests were inspected but not executed. No production source changed.

The adjacent `mixer_configuration/dynamic_events.rs` was fully reviewed in the automation/timeline scope and is used as cross-slice evidence without double-counting it here.

## 2. Per-module review ledger

| Module | Files | Static result |
|---|---:|---|
| `dynamic_events` | 5/5 | Preordered per-event handler indices are a good control-path optimization. Submission queue and fan-out are unbounded; each delivery deep-clones descriptors/payload. |
| `dynamic_event_abi` | 6/6 | Request slices borrow delivery storage during a synchronous callback and diagnostics reads are capped. Callback has no module lease, deadline, cancellation or async completion. |
| service registries/executors | 9/9 | Callbacks execute outside the state lock, but all executors are cloned, all deliveries materialized, and arbitrary handlers run serially on the caller thread. |
| trait/state boundary | 2/2 | Executor keys own strings and callbacks are `Arc` closures. Public methods expose synchronous drain/dispatch/execute without queue limits, receipts or lifecycle generations. |

## 3. Product and configuration truth

Repository-wide scan outside Sound implementation/tests found no product submit/dispatch/execute or ABI callback registration caller. `SoundConfig::dynamic_events_enabled` is populated from package options but is never read by registration, submission, dispatch or execution. The package capability advertises the dynamic-event contribution independently of that unused switch.

Thus the disabled state is ineffective, while the enabled state has no current product workload. Neither can be accepted dynamically.

## 4. Structural performance and safety findings

### P0: event fan-out is unbounded in queue length, handler count and payload bytes

Submission validates by linearly scanning the catalog and pushes an owned invocation into an unbounded `Vec`. Dispatch first computes total deliveries, allocates that full capacity, then drains every pending event. For all but the last matching handler it clones the complete invocation, including payload and strings, and clones every handler descriptor. Memory/time is `O(sum(events * matching handlers * payload bytes))` with no admission cap.

Use a bounded queue with per-source/plugin/global quotas and payload/schema byte limits. Resolve catalog IDs to compact slots at admission. Delivery batches share immutable invocation payload storage and compact handler handles; ABI borrowing happens directly from that stable batch. Overflow policy must be explicit per event class: coalesce, drop-oldest/drop-newest, defer, or fail submission.

### P0: arbitrary plugin code executes serially on the caller thread

`execute_dynamic_events_impl` correctly releases the global state mutex before callbacks, but then maps the entire delivery vector synchronously. One slow/hung executor delays all later handlers and the caller, which may be the main/world/editor thread. There is no affinity, concurrency class, time budget, timeout, cancellation or in-flight observation. Rust executor panic containment is also absent.

Dispatch belongs to the runtime task/plugin supervisor. Handlers declare affinity and concurrency policy; work receives deadlines/cancellation and bounded parallelism where safe. Audio-thread handlers must be compact realtime commands only, never arbitrary ABI calls. Per-handler circuit breaker/backoff isolates repeated failure without blocking unrelated plugins.

### P0: raw ABI callback lifetime is not tied to plugin module lifetime

`register_dynamic_event_abi_callback` wraps a raw `ZrPluginEventCallbackFnV1` in an `Arc` closure. The executor snapshot is cloned before invocation, so unregistering the map cannot revoke an in-flight closure. No module/library lease, plugin generation or quiescent unload protocol accompanies the function pointer. If native code unloads first, a retained callback can target invalid code.

Registration must require a runtime plugin lease and generation. Unload transitions the plugin to draining, rejects new events, cancels/dequeues queued work, waits for acknowledged in-flight callbacks, removes executors and only then releases code. Stale generations are rejected before invocation. Unreal's module manager exposes explicit load-state and unload/shutdown callbacks (`ModuleManager.h:187-192,296-325,531`; `ModuleManager.cpp:1316-1344`) rather than treating function pointers as independent data.

### P0: the advertised enable/disable option has no enforcement point

`dynamic_events_enabled` is dead configuration. Disabling cannot prevent handler registration, payload retention or execution; status cannot distinguish disabled/unavailable/ready. Capability and config must compile into the manager/supervisor generation and gate every admission path consistently. Disabling drains/cancels according to policy and publishes applied status.

### P1: executor and delivery snapshots clone more state than execution needs

Execution clones the complete `HashMap<SoundDynamicEventExecutorKey, SoundDynamicEventExecutor>` on every call, including two owned strings per key, even if only a subset receives events. It then creates a new string key from each delivery to look up that snapshot. Public dispatch separately returns a fully owned delivery vector, duplicating the same expansion without execution.

Compile handler slots to executor handles in an immutable registry generation. Snapshot one `Arc` registry page, then stream or chunk bounded deliveries by slot. No per-delivery string key construction or full executor-map clone is required.

### P1: failure and missing executors consume events without retry/dead-letter policy

Dispatch drains pending events before execution. Failed and missing executors are reported but not retried, quarantined or retained. Events with no handlers silently disappear. Conversely, retrying externally can duplicate handlers that already succeeded because no delivery/event idempotency token exists.

Assign invocation and delivery IDs, attempt count and terminal policy. Publish succeeded/failed/skipped/dropped/deferred counters and bounded dead-letter diagnostics. At-least-once or at-most-once semantics must be explicit per event; audio marker/impact events usually need bounded at-most-once degradation rather than an unbounded retry queue.

### P1: catalog/handler mutation rebuilds and clones global structures under the state lock

Handler registration linearly searches, sorts the full vector and rebuilds all event buckets. Unregistering an event clones the remaining handlers, then linearly checks them for every executor. Submission clones the full catalog only to validate one invocation. These are control-path costs, but plugin churn/import can block the same global Sound mutex used by playback/device work.

Prepare immutable catalog/handler/executor generations off-lock and atomically publish them. Index event IDs and handler keys once. Registration/unload is a supervisor transaction, not direct shared-state mutation.

### P1: the current benchmark isolates matching improvements only

The ignored benchmark usefully shows the intended removal of per-event matching-vector allocation and sorting through a preordered index. It still constructs owned deliveries and clones payload per handler; it does not include executor map cloning, ABI calls, slow/failing handlers, queue overflow, plugin unload or state-lock contention. It is unexecuted.

Keep it as local regression evidence after managed execution, but product qualification must measure the full bounded pipeline and lifecycle.

## 5. Unreal-primary policy adopted

- `AudioModulationSystem.h:178-181,258-262` owns a processing-thread command API and MPSC queue.
- `AudioModulationSystem.cpp:41,529-542,637,744` traces modulation processing, drains commands at its owner boundary and publishes processed-command count.
- `AudioMixerSourceManager.cpp:2999-3003,4007-4023,4227-4299` observes command-queue fill and pumps commands in explicit render phases.
- `ModuleManager.h:187-192,296-325,531` defines explicit module loaded state, unload/abandon callbacks and module-change notification; `ModuleManager.cpp:1316-1344` runs shutdown before releasing module code.

Zircon adopts owned command/task execution, queue observability and unload quiescence. It must measure its own capacities and deadlines; Unreal is structural evidence, not a source of inherited numbers.

## 6. Required optimization plan

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Capability/config truth | Applied config generation gates catalog, registration, submit and execute. | Disabled performs no queue/callback work and reports disabled; Ready has a product consumer. |
| M1 Compiled registry | Immutable event/handler/executor slots with shared payload pages and generations. | No catalog scan, string key construction, full executor-map clone or payload clone per delivery. |
| M2 Bounded admission | Queue, payload, fan-out and per-plugin quotas plus explicit overflow policy. | Worst-case memory/work derived from configured caps; overload never grows without bound. |
| M3 Task execution | Affinity, bounded concurrency, deadlines/cancellation and per-handler isolation. | Slow/failing plugin cannot stall unrelated handlers or main/audio thread; queue age is bounded/observable. |
| M4 Plugin lifecycle | Module lease, draining state, in-flight acknowledgements and stale-generation rejection. | Repeated register/submit/unregister/unload cannot call released code or leak queued payloads. |
| M5 Delivery semantics | IDs, attempt/terminal policy, dead-letter diagnostics and compact receipts. | Missing/failure/drop/retry behavior deterministic with no accidental duplicate success. |
| M6 Dynamic qualification | Current-source product drives burst, steady, slow, failing and unload workloads. | Record submit/queue/dispatch/callback P50/P95/P99/max, throughput, queue age/depth, copies/bytes, drops, failures, CPU, main stalls, RSS, wakeups and power. |

## 7. Direct-fix decision

The shared preordered-index implementation and outside-lock callback execution are preserved. No additional source edit is made: adding a `Vec` capacity or avoiding one catalog clone would not bound fan-out, make callbacks schedulable or secure native module lifetime. The correct change crosses runtime task and plugin-supervisor contracts and requires unload/overload tests with managed Cargo.

Static review is complete only for these 22 files. Dynamic acceptance, a Git milestone commit and quantified WeCom notification are not warranted.
