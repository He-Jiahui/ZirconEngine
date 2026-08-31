---
title: Sound Mixer and Device Service Current-Source Algorithm Performance Review
date: 2026-08-24
status: static_complete_dynamic_pending_no_source_change
scope:
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph
  - zircon_plugins/sound/runtime/src/service_types/output_device
  - zircon_plugins/sound/runtime/src/service_types/mixer_presets.rs
  - zircon_plugins/sound/runtime/src/service_types/output_render.rs
  - zircon_plugins/sound/runtime/src/service_types/parameters.rs
  - zircon_plugins/sound/runtime/src/service_types/runtime_settings.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/{backend,mixer_graph,output_device,render,runtime_settings}.rs
canonical_owners:
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/48-runtime-wide-state-next-state-transition-hook-history-schedule-scope-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Public/AudioMixerDevice.h
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerDevice.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSubmix.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Public/AudioDeviceNotificationSubsystem.h
---

# Sound Mixer and Device Service Current-Source Algorithm Performance Review

## 1. Status and frozen scope

The Sound mixer/device service slice completed E3 current-worktree static review over **22/22 Rust files** at revision `7fe97290fd3b0350c2c0f404fd00ad2d18f1335d`:

| Module slice | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| mixer/device public service and forwarding | 22/22 | 1,095 / 987 | 35,711 | 0 / 0 | `831db93ea61e5f333e1ef95f5ff8b4c27ff73088858afed51109401f634dd234` |

All files pass standalone `rustfmt --check --edition 2021 --config skip_children=true`; the scope is clean and scoped diff check passes. No production source changed. Managed Windows Cargo, a current-source executable, real device/graph workloads, WPR/ETW and power measurement remain unavailable, so this is a static service-boundary result only.

## 2. Per-folder review ledger

| Module | Reviewed files | Static result |
|---|---|---|
| `mixer_graph` | all 7 files | Public mutations clone/compile optimistically, retry eight times, then apply Kira work under the global state mutex. Snapshot deeply clones state. Effects can be accepted while inactive even though Kira M1 later rejects them. |
| `output_device` | all 6 files | Enumeration is synchronous; configure destroys current Kira state before publish; start creates backend and restores graph/sources under the state lock; status spans separate config/state observations. |
| supporting services | `mixer_presets.rs`, `output_render.rs`, `parameters.rs`, `runtime_settings.rs` | Preset application replaces graph/state, parameters are storage-only, settings are in-memory, and retired manual render/callback APIs remain publicly forwarded as Unsupported. |
| public forwarding | 5 selected `manager_trait` files | Synchronous broad APIs expose device enumeration/start/reconfigure and graph transactions without ticket, affinity, deadline, cancellation or observation generation. |

The output/Kira bridge and state/playback/source reports own adjacent implementation findings; this report does not double-count those files.

## 3. Structural performance findings

### P0: graph mutation prepares outside the lock but performs the expensive commit inside it

`mutate_graph`/`replace_graph` snapshot an `Arc` graph, clone and mutate it, compile a Kira plan outside the lock, then reacquire the global state mutex. Once revision/active state matches, `apply_graph_update` creates or mutates Kira tracks/sends while the mutex is held, replaces the graph and runs commit callbacks. `configure_mixer_impl` additionally prepares sources, applies graph, stops prior bindings, installs/restarts new sources and may perform a full rollback/restart under the same state lock.

Eight optimistic retries bound livelock but multiply full graph clone/validation/compile work under contention. The checked-in benchmark allows a 1,000-track public lock P95 up to 255 ms; that is a regression alarm, not an interactive or audio-control budget.

Required design: graph and source configuration becomes a typed prepare ticket owned by an audio control actor. It builds immutable graph/source generations and resource deltas off the caller/UI lock. A bounded command publishes at an audio frame boundary; acknowledgement updates last-good/applied generation. Callers wait only when the API explicitly requests synchronous completion.

### P0: device configure destroys last-good before a replacement is proven

`configure_output_device_impl` takes config and state mutexes, calls `deactivate_kira`, configures the descriptor, mutates graph format, clears HRTF state and writes config. It does not prepare or start a replacement stream. `start_output_device_impl` later holds state while creating the backend, syncing graph and replaying sources. Failure records unavailable and deactivates again; prior device/voices are not retained for rollback.

Device changes must be one supervisor transition: validate and negotiate supported format, prepare stream/graph/source recovery outside shared state, then switch/crossfade and retire old state. Failure leaves last-good active or enters an explicit silent/headless policy. A descriptor/config generation cannot become desired/applied merely because old output was stopped.

### P0: the public mixer surface accepts configurations the active backend cannot execute

Inactive graph mutation calls generic graph validation but not `validate_m1_surface`. It can accept effects, advanced controls and pre-effect sends into state. When output later starts, Kira compilation rejects them. A project/editor can pay authoring, serialization, snapshot and startup work only to fail at activation. Conversely, manual render, manual output block and callback pull remain in public traits but always return Unsupported.

Capability admission must be backend/profile/generation-specific. Unsupported controls/effects/render modes fail before persistence/authoring commit, or are stored only as an explicitly unavailable authored layer with diagnostics. Public Ready capability cannot include permanently unsupported operations.

### P0: graph/source rollback is stop-and-rebuild rather than last-good continuity

Full mixer configuration applies the new graph before stopping/replacing sources. If source stop or resync fails, rollback stops current and previous bindings, syncs the previous graph, then restarts sources. A rollback failure deactivates all Kira output. This is deterministic error handling, but it is not an atomic realtime transition and can introduce silence, repeated voice allocation and cursor discontinuity.

Prepare shadow bindings with immutable old/new generations. At commit, migrate compatible voice slots and crossfade or apply declared tail policy for incompatible nodes. Until acknowledgement, old graph and voices remain authoritative. Rollback drops only uncommitted resources.

### P1: snapshots and presets rebuild more state than the caller usually needs

`mixer_snapshot` deep-clones graph/sources/bindings/events/meters under the state lock. Built-in preset enumeration constructs all three preset graphs for each call; applying one constructs the catalog, linearly finds it, then replaces graph and scans every playback/source to repair routes. These are acceptable for rare explicit commands only, but the API has no cadence/budget contract and Editor polling could make them hot.

Publish immutable graph/status generations and cheap preset metadata. Build the selected preset graph lazily or cache immutable definitions by config generation. Editor reads deltas/cursors and requests explicit full capture only for export/debug.

### P1: device/backend status is torn across locks and conflates ready with stopped

Backend status clones config, separately observes unavailable state, then locks state again to test Kira activity. A concurrent configure/start can produce fields from different generations. Inactive output is returned as `SoundBackendState::Ready` with a “stopped” detail. Output status also reports callback/xrun zeros without a producer, as documented in the output report.

One immutable supervisor status page must contain requested/applied generation, transition state, active backend/device/format, last-good, failure stage and metric availability. Stopped, starting, ready-active, degraded, unavailable and headless are distinct states.

### P1: device catalog work is synchronous and uncached

Every `available_output_devices` call clones config and enumerates CPAL devices. The service gives no asynchronous ticket, cached generation, hotplug invalidation or deadline. An Editor combo refresh can block UI repeatedly and still receive requested rather than negotiated device formats.

The supervisor owns asynchronous enumeration/probing and publishes an immutable catalog generation. UI reads the last page immediately and observes refreshing/stale/error state.

### P1: settings/parameters have storage semantics but no applied-generation contract

Global gain updates Kira and config under two locks; default spatial scale only changes config and has no current render consumer. Generic parameters are inserted into an unbounded map, but this service slice provides no compiled binding or lifecycle. Settings are not persisted and do not report desired/applied generation.

Route each setting through typed ownership: live Tween command, next graph generation, device restart or process restart. Reject orphan parameters at compiled binding admission. Persist desired state only through a transaction that reports applied/last-good outcome.

## 4. Product call-site truth

Repository-wide production scan outside this implementation found no mixer/runtime/device consumers in app, runtime or editor roots. The Sound Editor live-output controller calls enumerate/configure/start, but its checked-in construction sites are inline tests; no product extension creates it. The low current workload therefore cannot close performance gates.

M0 must activate a current-source Sound Editor preview and runtime scene through the real manager, exercise graph parameter/structural edits and device transitions, and record the exact source/build/config/device/workload identity.

## 5. Unreal source evidence and adopted policy

- `AudioMixerDevice.h:359-362,420-421,534-537` owns audio-render and game-thread command queues rather than executing arbitrary caller mutations directly.
- `AudioMixerDevice.cpp:2505-2533` enqueues and pumps render/game commands; lines 2537-2556 define explicit flush behavior for realtime, non-realtime and stopped render paths.
- `AudioMixerDevice.cpp:1196,1392,1441-1489` registers device-change listeners, detects hardware stalls/change and drives a multi-step device swap.
- `AudioDeviceNotificationSubsystem.h:51-110` exposes default-device, state, add/remove and switched notifications.
- `AudioMixerSourceManager.h:54-69` names render phases including command pumps, pending release and game-thread copy update; `AudioMixerSourceManager.cpp:4002-4094` performs those phases in render update.
- `AudioMixerSubmix.cpp` keeps submix/effect processing in the audio mixer ownership domain; `AudioMixerDevice.cpp:1883-2489` routes submix mutations through audio-render commands.

Zircon should adopt command ownership, last-good device transition and immutable observations. It must measure its own graph/device latency and cannot inherit Unreal budgets by analogy.

## 6. Required optimization plan

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Product harness | Real runtime and Editor preview activate manager/device and execute graph workloads. | Source/build/config/device/workload receipt; audible output or explicit headless result. |
| M1 Capability truth | Backend/profile-specific supported graph/control/render surface. | Unsupported authoring fails before commit; Ready operations execute in product. |
| M2 Graph transaction | Off-lock immutable prepare, bounded audio command publish, last-good acknowledgement. | Commit lock/queue time and allocations bounded at 16/64/256/1,000 tracks; failure preserves old graph/audio. |
| M3 Source migration | Stable binding slots, cursor-preserving compatible migration and crossfade/tail policy. | Live edits have bounded command latency and no unintended restart/click/silence. |
| M4 Device supervisor | Async catalog/probe, negotiated format, prepare/swap/recovery/backoff. | Default swap/unplug/stall/unsupported format preserve last-good or explicit fallback; UI never blocks on enumeration. |
| M5 Observation/config | One immutable status/config generation with truthful transition/metric availability. | No torn status; stopped is not ready-active; unsupported metrics are not zero. |
| M6 Dynamic qualification | Graph churn, snapshot polling, device transitions and overload on MVP runtime/Editor. | Publish audio/control/main CPU, lock/command P50/P95/P99, allocations/RSS, wakeups, xrun, latency, recovery time, power and output parity. |

## 7. Direct-fix decision

No production edit is made. Caching presets, removing a clone or changing a status label would not repair graph/device ownership, and behavior changes cannot be verified without Cargo and a current-source Kira/CPAL product artifact. The next safe implementation milestone begins with the supervisor/immutable-generation boundary and tests its last-good semantics before optimizing inner loops.

Static review is complete only for these 22 files. No Git milestone commit or quantified WeCom notification is warranted.
